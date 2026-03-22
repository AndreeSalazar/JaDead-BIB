// ============================================================
// ADead-BIB IR — Intermediate Representation
// ============================================================
// This is the universal IR that all translators emit.
// PE, ELF, SPIR-V, DXIL → all produce ABIB_Module.
//
// Hierarchy:
//   ABIB_Module
//     ├── functions[]     (ABIB_Function)
//     │     └── blocks[]  (ABIB_Block)
//     │           └── instructions[] (ABIB_Instruction)
//     ├── globals[]       (ABIB_Global)
//     ├── imports[]       (ABIB_Import)
//     ├── exports[]       (ABIB_Export)
//     ├── gpu_kernels[]   (ABIB_GpuKernel)
//     └── relocations[]   (ABIB_Relocation)
// ============================================================

use std::fmt;

// ============================================================
// Registers (x86-64 + GPU virtual)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Register {
    // x86-64 general purpose
    RAX = 0, RBX = 1, RCX = 2, RDX = 3,
    RSI = 4, RDI = 5, RBP = 6, RSP = 7,
    R8  = 8, R9  = 9, R10 = 10, R11 = 11,
    R12 = 12, R13 = 13, R14 = 14, R15 = 15,
    // Instruction pointer
    RIP = 16,
    // Flags
    RFLAGS = 17,
    // XMM (SSE/AVX)
    XMM0 = 32, XMM1 = 33, XMM2 = 34, XMM3 = 35,
    XMM4 = 36, XMM5 = 37, XMM6 = 38, XMM7 = 39,
    XMM8 = 40, XMM9 = 41, XMM10 = 42, XMM11 = 43,
    XMM12 = 44, XMM13 = 45, XMM14 = 46, XMM15 = 47,
    // GPU virtual registers (SPIR-V / DXIL)
    VREG0 = 64, VREG1 = 65, VREG2 = 66, VREG3 = 67,
    VREG4 = 68, VREG5 = 69, VREG6 = 70, VREG7 = 71,
    // Virtual / unresolved
    Virtual = 255,
}

impl Register {
    pub fn from_x86_reg(reg: u8) -> Self {
        match reg {
            0  => Register::RAX, 1  => Register::RCX,
            2  => Register::RDX, 3  => Register::RBX,
            4  => Register::RSP, 5  => Register::RBP,
            6  => Register::RSI, 7  => Register::RDI,
            8  => Register::R8,  9  => Register::R9,
            10 => Register::R10, 11 => Register::R11,
            12 => Register::R12, 13 => Register::R13,
            14 => Register::R14, 15 => Register::R15,
            _  => Register::Virtual,
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ============================================================
// Operand
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Reg(Register),
    Imm64(i64),
    Imm32(i32),
    /// Memory: [base + index*scale + disp]
    Mem {
        base: Option<Register>,
        index: Option<Register>,
        scale: u8,
        disp: i64,
        size: u8, // 1, 2, 4, 8 bytes
    },
    /// Symbol reference (import/function name)
    Symbol(String),
    /// Label reference (block name)
    Label(String),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Reg(r) => write!(f, "{}", r),
            Operand::Imm64(v) => write!(f, "0x{:X}", v),
            Operand::Imm32(v) => write!(f, "0x{:X}", v),
            Operand::Mem { base, index, scale, disp, size } => {
                write!(f, "{}[", match size { 1 => "byte", 2 => "word", 4 => "dword", 8 => "qword", _ => "?" })?;
                if let Some(b) = base { write!(f, "{}", b)?; }
                if let Some(i) = index {
                    write!(f, "+{}*{}", i, scale)?;
                }
                if *disp != 0 { write!(f, "{:+}", disp)?; }
                write!(f, "]")
            }
            Operand::Symbol(s) => write!(f, "@{}", s),
            Operand::Label(l) => write!(f, ".{}", l),
        }
    }
}

// ============================================================
// Opcodes — ADead-BIB IR operations
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // Data movement
    Mov, Push, Pop, Lea, Xchg,
    // Arithmetic
    Add, Sub, Mul, Imul, Div, Idiv, Neg, Inc, Dec,
    // Bitwise
    And, Or, Xor, Not, Shl, Shr, Sar, Rol, Ror,
    // Comparison
    Cmp, Test,
    // Control flow
    Jmp, Je, Jne, Jg, Jge, Jl, Jle, Ja, Jae, Jb, Jbe,
    Call, Ret, Syscall, Int,
    // Stack frame
    Enter, Leave,
    // No-op / padding
    Nop,
    // SSE/AVX
    Movss, Movsd, Addss, Addsd, Subss, Subsd,
    Mulss, Mulsd, Divss, Divsd,
    Movaps, Movups, Addps, Mulps,
    // Conversion
    Cdq, Cqo, Movzx, Movsx, Cvtsi2sd, Cvtsd2si,
    // String ops
    Rep, Movsb, Stosb,
    // GPU-specific (SPIR-V / DXIL mapped)
    GpuLoad, GpuStore, GpuAdd, GpuMul, GpuFma,
    GpuMatMul, GpuDot, GpuSync, GpuBarrier,
    GpuThreadId, GpuGroupId, GpuLocalId,
    // Special
    Hlt, Ud2,
    // Unknown / raw bytes
    RawBytes,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ============================================================
// Instruction
// ============================================================

#[derive(Debug, Clone)]
pub struct ABIB_Instruction {
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
    /// Original address in source binary (for debug/reloc)
    pub source_addr: u64,
    /// Size of original instruction in bytes
    pub source_size: u8,
    /// Raw bytes (for RawBytes opcode or debug)
    pub raw: Vec<u8>,
}

impl ABIB_Instruction {
    pub fn new(opcode: Opcode) -> Self {
        ABIB_Instruction {
            opcode,
            operands: Vec::new(),
            source_addr: 0,
            source_size: 0,
            raw: Vec::new(),
        }
    }

    pub fn with_ops(opcode: Opcode, operands: Vec<Operand>) -> Self {
        ABIB_Instruction {
            opcode,
            operands,
            source_addr: 0,
            source_size: 0,
            raw: Vec::new(),
        }
    }
}

impl fmt::Display for ABIB_Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  {:08X}: {} ", self.source_addr, self.opcode)?;
        for (i, op) in self.operands.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{}", op)?;
        }
        Ok(())
    }
}

// ============================================================
// Basic Block
// ============================================================

#[derive(Debug, Clone)]
pub struct ABIB_Block {
    pub label: String,
    pub instructions: Vec<ABIB_Instruction>,
    /// Start address in source binary
    pub start_addr: u64,
}

impl ABIB_Block {
    pub fn new(label: &str, start_addr: u64) -> Self {
        ABIB_Block {
            label: label.to_string(),
            instructions: Vec::new(),
            start_addr,
        }
    }

    pub fn emit(&mut self, inst: ABIB_Instruction) {
        self.instructions.push(inst);
    }
}

impl fmt::Display for ABIB_Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  .{}:", self.label)?;
        for inst in &self.instructions {
            writeln!(f, "{}", inst)?;
        }
        Ok(())
    }
}

// ============================================================
// Function
// ============================================================

#[derive(Debug, Clone)]
pub struct ABIB_Function {
    pub name: String,
    pub blocks: Vec<ABIB_Block>,
    /// Start address in source binary
    pub start_addr: u64,
    /// Size in source binary
    pub size: u64,
    /// Is this an exported function?
    pub is_export: bool,
    /// Calling convention hint
    pub calling_conv: CallingConv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConv {
    Win64,
    SysV,
    Cdecl,
    Stdcall,
    Unknown,
}

impl ABIB_Function {
    pub fn new(name: &str, start_addr: u64) -> Self {
        ABIB_Function {
            name: name.to_string(),
            blocks: Vec::new(),
            start_addr,
            size: 0,
            is_export: false,
            calling_conv: CallingConv::Unknown,
        }
    }

    pub fn add_block(&mut self, block: ABIB_Block) {
        self.blocks.push(block);
    }

    pub fn instruction_count(&self) -> usize {
        self.blocks.iter().map(|b| b.instructions.len()).sum()
    }
}

impl fmt::Display for ABIB_Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "fn {} @ 0x{:X} ({} bytes, {:?}):",
            self.name, self.start_addr, self.size, self.calling_conv)?;
        for block in &self.blocks {
            write!(f, "{}", block)?;
        }
        Ok(())
    }
}

// ============================================================
// Global variable
// ============================================================

#[derive(Debug, Clone)]
pub struct ABIB_Global {
    pub name: String,
    pub addr: u64,
    pub size: u64,
    pub data: Vec<u8>,
    pub is_readonly: bool,
}

// ============================================================
// Import / Export
// ============================================================

#[derive(Debug, Clone)]
pub struct ABIB_Import {
    pub module: String,   // "kernel32.dll", "libc.so.6"
    pub symbol: String,   // "ExitProcess", "printf"
    pub hint: u16,
    pub iat_addr: u64,    // Address in IAT (for PE) or GOT (for ELF)
}

#[derive(Debug, Clone)]
pub struct ABIB_Export {
    pub name: String,
    pub addr: u64,
    pub ordinal: u16,
}

// ============================================================
// Relocation
// ============================================================

#[derive(Debug, Clone)]
pub struct ABIB_Relocation {
    pub addr: u64,
    pub reloc_type: RelocType,
    pub symbol: String,
    pub addend: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocType {
    Abs64,
    Rel32,
    Plt32,
    GotPcRel,
    Dir64,
}

// ============================================================
// GPU Kernel (SPIR-V / DXIL)
// ============================================================

#[derive(Debug, Clone)]
pub struct ABIB_GpuKernel {
    pub name: String,
    pub entry_point: String,
    pub execution_model: GpuExecutionModel,
    pub instructions: Vec<ABIB_Instruction>,
    pub local_size: [u32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuExecutionModel {
    Vertex,
    Fragment,
    Compute,
    Geometry,
    TessControl,
    TessEval,
    Kernel, // OpenCL
}

impl ABIB_GpuKernel {
    pub fn new(name: &str, model: GpuExecutionModel) -> Self {
        ABIB_GpuKernel {
            name: name.to_string(),
            entry_point: name.to_string(),
            execution_model: model,
            instructions: Vec::new(),
            local_size: [1, 1, 1],
        }
    }
}

// ============================================================
// Module — top-level container
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
    Cpu,
    Gpu,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    PE,
    ELF,
    MachO,
    SPIRV,
    DXIL,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ABIB_Module {
    pub name: String,
    pub source_format: SourceFormat,
    pub module_type: ModuleType,
    pub arch: String,

    // CPU IR
    pub functions: Vec<ABIB_Function>,
    pub globals: Vec<ABIB_Global>,
    pub imports: Vec<ABIB_Import>,
    pub exports: Vec<ABIB_Export>,
    pub relocations: Vec<ABIB_Relocation>,

    // GPU IR
    pub gpu_kernels: Vec<ABIB_GpuKernel>,

    // Metadata
    pub entry_point: Option<String>,
    pub image_base: u64,
    pub code_size: u64,
}

impl ABIB_Module {
    pub fn new_cpu(name: &str, source: SourceFormat) -> Self {
        ABIB_Module {
            name: name.to_string(),
            source_format: source,
            module_type: ModuleType::Cpu,
            arch: "x86-64".to_string(),
            functions: Vec::new(),
            globals: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            relocations: Vec::new(),
            gpu_kernels: Vec::new(),
            entry_point: None,
            image_base: 0,
            code_size: 0,
        }
    }

    pub fn new_gpu(name: &str, source: SourceFormat) -> Self {
        ABIB_Module {
            name: name.to_string(),
            source_format: source,
            module_type: ModuleType::Gpu,
            arch: "gpu".to_string(),
            functions: Vec::new(),
            globals: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            relocations: Vec::new(),
            gpu_kernels: Vec::new(),
            entry_point: None,
            image_base: 0,
            code_size: 0,
        }
    }

    pub fn total_instructions(&self) -> usize {
        let cpu: usize = self.functions.iter().map(|f| f.instruction_count()).sum();
        let gpu: usize = self.gpu_kernels.iter().map(|k| k.instructions.len()).sum();
        cpu + gpu
    }
}

impl fmt::Display for ABIB_Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== ABIB Module: {} ===", self.name)?;
        writeln!(f, "  Source:       {:?}", self.source_format)?;
        writeln!(f, "  Type:         {:?}", self.module_type)?;
        writeln!(f, "  Arch:         {}", self.arch)?;
        writeln!(f, "  Functions:    {}", self.functions.len())?;
        writeln!(f, "  GPU Kernels:  {}", self.gpu_kernels.len())?;
        writeln!(f, "  Globals:      {}", self.globals.len())?;
        writeln!(f, "  Imports:      {}", self.imports.len())?;
        writeln!(f, "  Exports:      {}", self.exports.len())?;
        writeln!(f, "  Relocations:  {}", self.relocations.len())?;
        writeln!(f, "  Instructions: {}", self.total_instructions())?;
        if let Some(ref ep) = self.entry_point {
            writeln!(f, "  Entry Point:  {}", ep)?;
        }
        writeln!(f, "  Image Base:   0x{:X}", self.image_base)?;
        for func in &self.functions {
            writeln!(f)?;
            write!(f, "{}", func)?;
        }
        for kernel in &self.gpu_kernels {
            writeln!(f)?;
            writeln!(f, "gpu_kernel {} ({:?}, local_size={:?}):",
                kernel.name, kernel.execution_model, kernel.local_size)?;
            for inst in &kernel.instructions {
                writeln!(f, "{}", inst)?;
            }
        }
        Ok(())
    }
}
