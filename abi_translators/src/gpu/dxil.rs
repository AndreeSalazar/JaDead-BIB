// ============================================================
// DXIL Translator — DirectX 12 DXIL/DXBC → ADead-BIB IR
// ============================================================
// Pipeline: DXBC Container → Parse → Decode DXIL → Map → ABIB_Module
//
// DXBC container format:
//   "DXBC" magic (4 bytes)
//   Hash (16 bytes)
//   Version (4 bytes)
//   Total size (4 bytes)
//   Chunk count (4 bytes)
//   Chunk offsets[]
//   Chunks: RDEF, ISGN, OSGN, SHEX/SHDR, STAT, DXIL, etc.
//
// The DXIL chunk contains LLVM bitcode (shader model 6.0+).
// Older shaders use SHEX/SHDR (shader bytecode).
//
// This translator handles the DXBC container and maps
// shader instructions to ABIB GPU IR.
// ============================================================

use crate::core::ir::*;
use crate::core::translator::{ABIBTranslator, BinaryView};
use crate::core::context::TranslationContext;
use crate::utils::binary_reader::BinaryReader;

// DXBC magic
const DXBC_MAGIC: &[u8; 4] = b"DXBC";

// Chunk FourCCs
const CHUNK_RDEF: u32 = fourcc(b"RDEF"); // Resource definitions
const CHUNK_ISGN: u32 = fourcc(b"ISGN"); // Input signature
const CHUNK_OSGN: u32 = fourcc(b"OSGN"); // Output signature
const CHUNK_SHEX: u32 = fourcc(b"SHEX"); // Shader bytecode (SM 4/5)
const CHUNK_SHDR: u32 = fourcc(b"SHDR"); // Shader bytecode (SM 4/5)
const CHUNK_DXIL: u32 = fourcc(b"DXIL"); // DXIL bitcode (SM 6+)
const CHUNK_STAT: u32 = fourcc(b"STAT"); // Statistics

const fn fourcc(s: &[u8; 4]) -> u32 {
    (s[0] as u32) | ((s[1] as u32) << 8) | ((s[2] as u32) << 16) | ((s[3] as u32) << 24)
}

// SHEX/SHDR opcode tokens (SM 4/5)
const SHEX_OP_MOV: u32 = 0x36;
const SHEX_OP_ADD: u32 = 0x00;
const SHEX_OP_MUL: u32 = 0x38;
const SHEX_OP_MAD: u32 = 0x32; // multiply-add (FMA)
const SHEX_OP_DP3: u32 = 0x10;
const SHEX_OP_DP4: u32 = 0x11;
const SHEX_OP_RET: u32 = 0x3E;
const SHEX_OP_SAMPLE: u32 = 0x45;
const SHEX_OP_STORE_UAV: u32 = 0xA4;
const SHEX_OP_LD_UAV: u32 = 0xA5;
const SHEX_OP_SYNC: u32 = 0xBE;
const SHEX_OP_DCL_THREAD_GROUP: u32 = 0x9B;

// ============================================================
// DXBC Parsed structures
// ============================================================

#[derive(Debug)]
struct DxbcHeader {
    hash: [u8; 16],
    version: u32,
    total_size: u32,
    chunk_count: u32,
}

#[derive(Debug)]
struct DxbcChunk {
    fourcc: u32,
    offset: u32,
    size: u32,
}

// ============================================================
// DXBC Parser
// ============================================================

fn parse_dxbc(view: &BinaryView) -> Result<(DxbcHeader, Vec<DxbcChunk>), String> {
    let data = &view.data;
    if data.len() < 32 { return Err("File too small for DXBC".into()); }

    if &data[0..4] != DXBC_MAGIC {
        return Err("Not a DXBC file".into());
    }

    let mut hash = [0u8; 16];
    hash.copy_from_slice(&data[4..20]);

    let mut r = BinaryReader::at(data, 20);
    let version = r.read_u32().ok_or("Failed to read version")?;
    let total_size = r.read_u32().ok_or("Failed to read total size")?;
    let chunk_count = r.read_u32().ok_or("Failed to read chunk count")?;

    let header = DxbcHeader { hash, version, total_size, chunk_count };

    // Read chunk offsets
    let mut chunks = Vec::new();
    for _ in 0..chunk_count {
        let offset = r.read_u32().ok_or("Failed to read chunk offset")?;
        chunks.push(DxbcChunk { fourcc: 0, offset, size: 0 });
    }

    // Read chunk headers (fourcc + size at each offset)
    for chunk in &mut chunks {
        let off = chunk.offset as usize;
        if off + 8 > data.len() { continue; }
        chunk.fourcc = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
        chunk.size = u32::from_le_bytes([data[off+4], data[off+5], data[off+6], data[off+7]]);
    }

    Ok((header, chunks))
}

fn fourcc_str(cc: u32) -> String {
    let bytes = cc.to_le_bytes();
    String::from_utf8_lossy(&bytes).to_string()
}

// ============================================================
// SHEX/SHDR Decoder (SM 4/5 shader bytecode)
// ============================================================

fn decode_shex(data: &[u8], offset: usize, size: usize) -> (Vec<ABIB_Instruction>, [u32; 3]) {
    let mut instructions = Vec::new();
    let mut local_size = [1u32, 1, 1];

    if size < 8 { return (instructions, local_size); }
    let chunk_data = &data[offset + 8..]; // skip fourcc + size
    if chunk_data.len() < 8 { return (instructions, local_size); }

    // First two words: version token + length token
    let mut pos = 8; // skip version + length

    while pos + 4 <= chunk_data.len() {
        let token = u32::from_le_bytes([
            chunk_data[pos], chunk_data[pos+1], chunk_data[pos+2], chunk_data[pos+3]
        ]);

        let opcode = token & 0x7FF; // bits 0-10
        let _length = ((token >> 24) & 0x1F) as usize; // bits 24-28 (instruction length in dwords)
        let inst_len = if _length == 0 { 1 } else { _length };

        let addr = (offset + 8 + pos) as u64;

        let abib_op = match opcode {
            op if op == SHEX_OP_ADD => Opcode::GpuAdd,
            op if op == SHEX_OP_MUL => Opcode::GpuMul,
            op if op == SHEX_OP_MAD => Opcode::GpuFma,
            op if op == SHEX_OP_MOV => Opcode::Mov,
            op if op == SHEX_OP_DP3 || op == SHEX_OP_DP4 => Opcode::GpuDot,
            op if op == SHEX_OP_RET => Opcode::Ret,
            op if op == SHEX_OP_SAMPLE => Opcode::GpuLoad,
            op if op == SHEX_OP_STORE_UAV => Opcode::GpuStore,
            op if op == SHEX_OP_LD_UAV => Opcode::GpuLoad,
            op if op == SHEX_OP_SYNC => Opcode::GpuSync,
            op if op == SHEX_OP_DCL_THREAD_GROUP => {
                // Read thread group dimensions
                if pos + 16 <= chunk_data.len() {
                    local_size[0] = u32::from_le_bytes(chunk_data[pos+4..pos+8].try_into().unwrap_or([1,0,0,0]));
                    local_size[1] = u32::from_le_bytes(chunk_data[pos+8..pos+12].try_into().unwrap_or([1,0,0,0]));
                    local_size[2] = u32::from_le_bytes(chunk_data[pos+12..pos+16].try_into().unwrap_or([1,0,0,0]));
                }
                pos += inst_len * 4;
                continue;
            }
            _ => Opcode::RawBytes,
        };

        let mut inst = ABIB_Instruction::new(abib_op);
        inst.source_addr = addr;
        inst.source_size = (inst_len * 4) as u8;
        instructions.push(inst);

        pos += inst_len * 4;
    }

    (instructions, local_size)
}

// ============================================================
// DXIL Translator
// ============================================================

pub struct DxilTranslator;

impl DxilTranslator {
    pub fn new() -> Self { DxilTranslator }
}

impl ABIBTranslator for DxilTranslator {
    fn name(&self) -> &str { "DXIL Translator (DirectX 12)" }

    fn source_format(&self) -> SourceFormat { SourceFormat::DXIL }

    fn can_handle(&self, view: &BinaryView) -> bool {
        view.size >= 4 && &view.data[0..4] == DXBC_MAGIC
    }

    fn translate(&self, view: &BinaryView) -> Result<ABIB_Module, String> {
        let (_header, chunks) = parse_dxbc(view)?;

        let mut ctx = TranslationContext::new_gpu(&view.filename, SourceFormat::DXIL);

        // Find shader bytecode chunk (SHEX or SHDR or DXIL)
        let mut shader_instructions = Vec::new();
        let mut local_size = [1u32, 1, 1];
        let mut found_shader = false;

        for chunk in &chunks {
            if chunk.fourcc == CHUNK_SHEX || chunk.fourcc == CHUNK_SHDR {
                let (insts, ls) = decode_shex(&view.data, chunk.offset as usize, chunk.size as usize);
                shader_instructions = insts;
                local_size = ls;
                found_shader = true;
                break;
            }
            if chunk.fourcc == CHUNK_DXIL {
                // DXIL contains LLVM bitcode — for now emit as raw
                let mut inst = ABIB_Instruction::new(Opcode::RawBytes);
                inst.source_addr = chunk.offset as u64 + 8;
                inst.source_size = 0; // large
                inst.raw = view.data[chunk.offset as usize + 8..
                    (chunk.offset as usize + 8 + chunk.size as usize).min(view.data.len())]
                    .to_vec();
                shader_instructions.push(inst);
                found_shader = true;
                break;
            }
        }

        if !found_shader {
            return Err("No shader bytecode found in DXBC container".into());
        }

        // Create GPU kernel
        let mut kernel = ABIB_GpuKernel::new("main", GpuExecutionModel::Compute);
        kernel.local_size = local_size;
        kernel.instructions = shader_instructions.clone();
        ctx.module.gpu_kernels.push(kernel);
        ctx.module.entry_point = Some("main".to_string());

        // Also create a function record
        ctx.begin_function("main", 0);
        ctx.auto_block(0);
        for inst in shader_instructions {
            ctx.emit(inst);
        }
        ctx.end_function(0);

        // Log chunk info
        for chunk in &chunks {
            let cc = fourcc_str(chunk.fourcc);
            ctx.module.globals.push(ABIB_Global {
                name: format!("dxbc_chunk_{}", cc),
                addr: chunk.offset as u64,
                size: chunk.size as u64,
                data: Vec::new(),
                is_readonly: true,
            });
        }

        Ok(ctx.finish())
    }
}
