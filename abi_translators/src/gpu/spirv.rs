// ============================================================
// SPIR-V Translator — Vulkan/OpenCL SPIR-V → ADead-BIB IR
// ============================================================
// Pipeline: SPIR-V Binary → Parse → Decode → Map → ABIB_Module
//
// SPIR-V is a structured binary format (word-based, not byte-based).
// Each instruction is: [word_count:16 | opcode:16] [operands...]
//
// Parses:
//   Header (magic, version, generator, bound)
//   Instructions (OpEntryPoint, OpFunction, OpLabel, etc.)
//
// Maps:
//   SPIR-V ops → ABIB GPU IR instructions
// ============================================================

use crate::core::ir::*;
use crate::core::translator::{ABIBTranslator, BinaryView};
use crate::core::context::TranslationContext;

// SPIR-V magic
const SPIRV_MAGIC: u32 = 0x07230203;

// SPIR-V opcodes we care about
const OP_ENTRY_POINT: u16 = 15;
const OP_EXECUTION_MODE: u16 = 16;
const OP_TYPE_VOID: u16 = 19;
const OP_TYPE_BOOL: u16 = 20;
const OP_TYPE_INT: u16 = 21;
const OP_TYPE_FLOAT: u16 = 22;
const OP_TYPE_VECTOR: u16 = 23;
const OP_TYPE_FUNCTION: u16 = 33;
const OP_TYPE_POINTER: u16 = 32;
const OP_FUNCTION: u16 = 54;
const OP_FUNCTION_END: u16 = 56;
const OP_LABEL: u16 = 248;
const OP_RETURN: u16 = 253;
const OP_RETURN_VALUE: u16 = 254;
const OP_LOAD: u16 = 61;
const OP_STORE: u16 = 62;
const OP_ACCESS_CHAIN: u16 = 65;
const OP_VARIABLE: u16 = 59;
const OP_CONSTANT: u16 = 43;
const OP_IADD: u16 = 128;
const OP_ISUB: u16 = 130;
const OP_IMUL: u16 = 132;
const OP_FADD: u16 = 129;
const OP_FSUB: u16 = 131;
const OP_FMUL: u16 = 133;
const OP_FDIV: u16 = 136;
const OP_DOT: u16 = 148;
const OP_BRANCH: u16 = 249;
const OP_BRANCH_CONDITIONAL: u16 = 250;
const OP_CONTROL_BARRIER: u16 = 224;
const OP_NAME: u16 = 5;
const OP_MEMBER_NAME: u16 = 6;
const OP_DECORATE: u16 = 71;

// Execution models
const EXEC_MODEL_VERTEX: u32 = 0;
const EXEC_MODEL_FRAGMENT: u32 = 4;
const EXEC_MODEL_GLCOMPUTE: u32 = 5;
const EXEC_MODEL_KERNEL: u32 = 6;

// ============================================================
// SPIR-V Parsed structures
// ============================================================

#[derive(Debug)]
struct SpirvHeader {
    magic: u32,
    version: u32,
    generator: u32,
    bound: u32,
}

#[derive(Debug, Clone)]
struct SpirvInstruction {
    opcode: u16,
    word_count: u16,
    words: Vec<u32>,
    offset: usize, // byte offset in file
}

#[derive(Debug, Clone)]
struct SpirvEntryPoint {
    execution_model: u32,
    function_id: u32,
    name: String,
}

// ============================================================
// SPIR-V Parser
// ============================================================

fn parse_spirv(view: &BinaryView) -> Result<(SpirvHeader, Vec<SpirvInstruction>), String> {
    let data = &view.data;
    if data.len() < 20 { return Err("File too small for SPIR-V".into()); }

    // Read header (5 words)
    let magic = read_word(data, 0);
    if magic != SPIRV_MAGIC {
        return Err(format!("Invalid SPIR-V magic: 0x{:08X}", magic));
    }

    let header = SpirvHeader {
        magic,
        version: read_word(data, 4),
        generator: read_word(data, 8),
        bound: read_word(data, 12),
    };

    // Parse instructions (starting at word 5 = byte 20)
    let mut instructions = Vec::new();
    let mut pos = 20;

    while pos + 4 <= data.len() {
        let first_word = read_word(data, pos);
        let word_count = (first_word >> 16) as u16;
        let opcode = (first_word & 0xFFFF) as u16;

        if word_count == 0 { break; } // invalid

        let byte_len = word_count as usize * 4;
        if pos + byte_len > data.len() { break; }

        let mut words = Vec::with_capacity(word_count as usize);
        for i in 0..word_count as usize {
            words.push(read_word(data, pos + i * 4));
        }

        instructions.push(SpirvInstruction {
            opcode,
            word_count,
            words,
            offset: pos,
        });

        pos += byte_len;
    }

    Ok((header, instructions))
}

fn read_word(data: &[u8], offset: usize) -> u32 {
    if offset + 4 > data.len() { return 0; }
    u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]])
}

/// Extract a null-terminated string from SPIR-V words starting at word index
fn extract_string(words: &[u32], start_word: usize) -> String {
    let mut bytes = Vec::new();
    for &w in &words[start_word..] {
        let wb = w.to_le_bytes();
        for &b in &wb {
            if b == 0 { return String::from_utf8_lossy(&bytes).to_string(); }
            bytes.push(b);
        }
    }
    String::from_utf8_lossy(&bytes).to_string()
}

// ============================================================
// SPIR-V Translator
// ============================================================

pub struct SpirvTranslator;

impl SpirvTranslator {
    pub fn new() -> Self { SpirvTranslator }
}

impl ABIBTranslator for SpirvTranslator {
    fn name(&self) -> &str { "SPIR-V Translator (Vulkan/OpenCL)" }

    fn source_format(&self) -> SourceFormat { SourceFormat::SPIRV }

    fn can_handle(&self, view: &BinaryView) -> bool {
        if view.size < 4 { return false; }
        let magic = u32::from_le_bytes([view.data[0], view.data[1], view.data[2], view.data[3]]);
        magic == SPIRV_MAGIC
    }

    fn translate(&self, view: &BinaryView) -> Result<ABIB_Module, String> {
        let (_header, instructions) = parse_spirv(view)?;

        let mut ctx = TranslationContext::new_gpu(&view.filename, SourceFormat::SPIRV);

        // First pass: collect names and entry points
        let mut names: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
        let mut entry_points: Vec<SpirvEntryPoint> = Vec::new();
        let mut local_sizes: std::collections::HashMap<u32, [u32; 3]> = std::collections::HashMap::new();

        for inst in &instructions {
            match inst.opcode {
                OP_NAME => {
                    if inst.words.len() >= 3 {
                        let id = inst.words[1];
                        let name = extract_string(&inst.words, 2);
                        names.insert(id, name);
                    }
                }
                OP_ENTRY_POINT => {
                    if inst.words.len() >= 4 {
                        let exec_model = inst.words[1];
                        let func_id = inst.words[2];
                        let name = extract_string(&inst.words, 3);
                        entry_points.push(SpirvEntryPoint {
                            execution_model: exec_model,
                            function_id: func_id,
                            name,
                        });
                    }
                }
                OP_EXECUTION_MODE => {
                    if inst.words.len() >= 3 {
                        let func_id = inst.words[1];
                        let mode = inst.words[2];
                        // LocalSize = 17
                        if mode == 17 && inst.words.len() >= 6 {
                            local_sizes.insert(func_id, [
                                inst.words[3], inst.words[4], inst.words[5]
                            ]);
                        }
                    }
                }
                _ => {}
            }
        }

        // Second pass: translate functions and instructions
        let mut in_function = false;
        let mut current_func_id: u32 = 0;
        let mut block_count: u32 = 0;

        for inst in &instructions {
            match inst.opcode {
                OP_FUNCTION => {
                    if inst.words.len() >= 3 {
                        current_func_id = inst.words[2];
                        let func_name = names.get(&current_func_id)
                            .cloned()
                            .unwrap_or_else(|| format!("func_{}", current_func_id));

                        // Check if this is an entry point
                        let ep = entry_points.iter().find(|e| e.function_id == current_func_id);
                        if let Some(ep) = ep {
                            let model = match ep.execution_model {
                                EXEC_MODEL_VERTEX => GpuExecutionModel::Vertex,
                                EXEC_MODEL_FRAGMENT => GpuExecutionModel::Fragment,
                                EXEC_MODEL_GLCOMPUTE => GpuExecutionModel::Compute,
                                EXEC_MODEL_KERNEL => GpuExecutionModel::Kernel,
                                _ => GpuExecutionModel::Compute,
                            };
                            let mut kernel = ABIB_GpuKernel::new(&ep.name, model);
                            if let Some(ls) = local_sizes.get(&current_func_id) {
                                kernel.local_size = *ls;
                            }
                            ctx.module.gpu_kernels.push(kernel);
                            ctx.module.entry_point = Some(ep.name.clone());
                        }

                        // Also create a CPU-side function record
                        ctx.begin_function(&func_name, inst.offset as u64);
                        in_function = true;
                        block_count = 0;
                    }
                }

                OP_FUNCTION_END => {
                    if in_function {
                        ctx.end_function(0);
                        in_function = false;
                    }
                }

                OP_LABEL => {
                    if in_function {
                        let label = if inst.words.len() >= 2 {
                            names.get(&inst.words[1])
                                .cloned()
                                .unwrap_or_else(|| format!("bb{}", block_count))
                        } else {
                            format!("bb{}", block_count)
                        };
                        ctx.begin_block(&label, inst.offset as u64);
                        block_count += 1;
                    }
                }

                // Map SPIR-V ops to ABIB GPU ops
                OP_LOAD => {
                    let abib_inst = ABIB_Instruction::with_ops(
                        Opcode::GpuLoad,
                        map_spirv_operands(&inst.words, &names),
                    );
                    emit_to_ctx_and_kernel(&mut ctx, abib_inst, inst.offset as u64);
                }

                OP_STORE => {
                    let abib_inst = ABIB_Instruction::with_ops(
                        Opcode::GpuStore,
                        map_spirv_operands(&inst.words, &names),
                    );
                    emit_to_ctx_and_kernel(&mut ctx, abib_inst, inst.offset as u64);
                }

                OP_IADD | OP_FADD => {
                    let abib_inst = ABIB_Instruction::with_ops(
                        Opcode::GpuAdd,
                        map_spirv_operands(&inst.words, &names),
                    );
                    emit_to_ctx_and_kernel(&mut ctx, abib_inst, inst.offset as u64);
                }

                OP_IMUL | OP_FMUL => {
                    let abib_inst = ABIB_Instruction::with_ops(
                        Opcode::GpuMul,
                        map_spirv_operands(&inst.words, &names),
                    );
                    emit_to_ctx_and_kernel(&mut ctx, abib_inst, inst.offset as u64);
                }

                OP_ISUB | OP_FSUB => {
                    let abib_inst = ABIB_Instruction::with_ops(
                        Opcode::Sub,
                        map_spirv_operands(&inst.words, &names),
                    );
                    emit_to_ctx_and_kernel(&mut ctx, abib_inst, inst.offset as u64);
                }

                OP_FDIV => {
                    let abib_inst = ABIB_Instruction::with_ops(
                        Opcode::Div,
                        map_spirv_operands(&inst.words, &names),
                    );
                    emit_to_ctx_and_kernel(&mut ctx, abib_inst, inst.offset as u64);
                }

                OP_DOT => {
                    let abib_inst = ABIB_Instruction::with_ops(
                        Opcode::GpuDot,
                        map_spirv_operands(&inst.words, &names),
                    );
                    emit_to_ctx_and_kernel(&mut ctx, abib_inst, inst.offset as u64);
                }

                OP_CONTROL_BARRIER => {
                    let abib_inst = ABIB_Instruction::new(Opcode::GpuBarrier);
                    emit_to_ctx_and_kernel(&mut ctx, abib_inst, inst.offset as u64);
                }

                OP_BRANCH => {
                    if inst.words.len() >= 2 {
                        let target = names.get(&inst.words[1])
                            .cloned()
                            .unwrap_or_else(|| format!("label_{}", inst.words[1]));
                        let abib_inst = ABIB_Instruction::with_ops(
                            Opcode::Jmp,
                            vec![Operand::Label(target)],
                        );
                        ctx.emit(abib_inst);
                    }
                }

                OP_BRANCH_CONDITIONAL => {
                    if inst.words.len() >= 4 {
                        let true_label = names.get(&inst.words[2])
                            .cloned()
                            .unwrap_or_else(|| format!("label_{}", inst.words[2]));
                        let false_label = names.get(&inst.words[3])
                            .cloned()
                            .unwrap_or_else(|| format!("label_{}", inst.words[3]));
                        let abib_inst = ABIB_Instruction::with_ops(
                            Opcode::Jne,
                            vec![Operand::Label(true_label), Operand::Label(false_label)],
                        );
                        ctx.emit(abib_inst);
                    }
                }

                OP_RETURN | OP_RETURN_VALUE => {
                    let abib_inst = ABIB_Instruction::new(Opcode::Ret);
                    ctx.emit(abib_inst);
                }

                // Skip type/decoration/variable declarations (metadata)
                OP_TYPE_VOID | OP_TYPE_BOOL | OP_TYPE_INT | OP_TYPE_FLOAT
                | OP_TYPE_VECTOR | OP_TYPE_FUNCTION | OP_TYPE_POINTER
                | OP_VARIABLE | OP_CONSTANT | OP_DECORATE | OP_MEMBER_NAME
                | OP_ACCESS_CHAIN => {
                    // These are type system / metadata — not emitted as instructions
                }

                _ => {
                    // Unknown op — emit as raw
                    if in_function {
                        let mut raw_inst = ABIB_Instruction::new(Opcode::RawBytes);
                        raw_inst.source_addr = inst.offset as u64;
                        raw_inst.source_size = (inst.word_count * 4) as u8;
                        ctx.emit(raw_inst);
                    }
                }
            }
        }

        Ok(ctx.finish())
    }
}

// ============================================================
// Helpers
// ============================================================

fn map_spirv_operands(words: &[u32], names: &std::collections::HashMap<u32, String>) -> Vec<Operand> {
    let mut ops = Vec::new();
    // Skip first word (opcode+count), map remaining as symbol references
    for &w in words.iter().skip(1) {
        if let Some(name) = names.get(&w) {
            ops.push(Operand::Symbol(name.clone()));
        } else {
            ops.push(Operand::Imm32(w as i32));
        }
    }
    ops
}

fn emit_to_ctx_and_kernel(ctx: &mut TranslationContext, mut inst: ABIB_Instruction, offset: u64) {
    inst.source_addr = offset;
    // Emit to current function block
    ctx.emit(inst.clone());
    // Also emit to last GPU kernel if present
    if let Some(kernel) = ctx.module.gpu_kernels.last_mut() {
        kernel.instructions.push(inst);
    }
}
