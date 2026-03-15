// ============================================================
// JaDead-BIB IR (Intermediate Representation) 💀☕
// ============================================================
// ADeadOp SSA-form — Java Nativo sin JVM
// Tipos explícitos en cada instrucción
// BasicBlocks — sin ambigüedad semántica
// GC eliminado: cada objeto tiene ownership ✓
// ============================================================

/// IR Type — maps Java types to machine types
#[derive(Debug, Clone, PartialEq)]
pub enum IRType {
    Void,
    I8,      // bool
    I16,
    I32,
    I64,     // int (default)
    I128,
    F32,
    F64,     // float (default)
    Ptr,     // str, list, dict, object references
    Vec256,  // YMM 256-bit (SIMD)
}

impl IRType {
    pub fn byte_size(&self) -> usize {
        match self {
            IRType::Void => 0,
            IRType::I8 => 1,
            IRType::I16 => 2,
            IRType::I32 => 4,
            IRType::I64 => 8,
            IRType::I128 => 16,
            IRType::F32 => 4,
            IRType::F64 => 8,
            IRType::Ptr => 8,
            IRType::Vec256 => 32,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, IRType::I8 | IRType::I16 | IRType::I32 | IRType::I64 | IRType::I128)
    }

    pub fn is_float(&self) -> bool {
        matches!(self, IRType::F32 | IRType::F64)
    }
}

/// IR Module — top-level container
#[derive(Debug)]
pub struct IRModule {
    pub name: String,
    pub functions: Vec<IRFunction>,
}

impl IRModule {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), functions: Vec::new() }
    }
}

/// IR Function
#[derive(Debug)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<(String, IRType)>,
    pub return_type: IRType,
    pub body: Vec<IRInstruction>,
}

impl IRFunction {
    pub fn new(name: String, params: Vec<(String, IRType)>, return_type: IRType) -> Self {
        Self { name, params, return_type, body: Vec::new() }
    }
}

/// IR Instruction — SSA-form operations
#[derive(Debug, Clone)]
pub enum IRInstruction {
    // Constants
    LoadConst(IRConstValue),
    LoadString(String),     // label in .data
    Load(String),           // load variable
    Store(String),          // store to variable

    // Variable declaration
    VarDecl { name: String, ir_type: IRType },

    // Arithmetic
    BinOp { op: IROp, left: Box<IRInstruction>, right: Box<IRInstruction> },
    Compare { op: IRCmpOp, left: Box<IRInstruction>, right: Box<IRInstruction> },

    // Control flow
    Label(String),
    Jump(String),
    BranchIfFalse(String),
    Return,
    ReturnVoid,
    Break,
    Continue,

    // Function call
    Call { func: String, args: Vec<IRInstruction> },

    // Iterator
    IterNext { target: String, end_label: String },

    // Builtins — direct runtime calls
    PrintStr(String),       // print string literal (label in .data)
    PrintInt,               // print RAX as decimal integer
    PrintFloat,             // print XMM0 as float
    PrintNewline,           // print "\n"
    PrintChar,              // print AL as single character
    ExitProcess,            // exit with RAX as exit code

    // Math builtins (result in XMM0 or RAX)
    MathSqrt,               // SQRTSD XMM0, XMM0
    MathFloor,              // ROUNDSD XMM0, XMM0, 1 → CVTTSD2SI
    MathCeil,               // ROUNDSD XMM0, XMM0, 2 → CVTTSD2SI
    MathSin,                // x87 FSIN
    MathCos,                // x87 FCOS
    MathLog,                // x87 FYL2X
    MathAbsFloat,           // ANDPD sign mask
    MathAbsInt,             // NEG + CMOV
    MathLoadConst(String),  // load named float constant (pi, e)

    // Int builtins
    BuiltinMin,             // min(RAX, RCX)
    BuiltinMax,             // max(RAX, RCX)
    BuiltinChr,             // chr(RAX) → print char
    BuiltinOrd,             // ord(char) → RAX

    // Exception handling
    TryBegin(String),           // label for except handler
    TryEnd,                     // clear error state
    Raise { exc_type: String, message: Option<Box<IRInstruction>> },
    CheckError(String),         // branch to label if error set
    ClearError,
    FinallyBegin,
    FinallyEnd,

    // v3.0 — Coroutine / async state machine
    CoroutineCreate { func: String },       // create coroutine struct on heap
    CoroutineResume,                        // resume coroutine (RAX = coro ptr)
    CoroutineYield,                         // yield value from coroutine

    // v3.0 — Generator protocol
    GeneratorCreate { func: String },       // create generator struct
    GeneratorNext,                          // call next() on generator
    GeneratorSend(Box<IRInstruction>),      // send value to generator

    // v3.0 Native OOP — Struct field binding
    PropertyGet { obj: String, offset: u32 },
    PropertySet { obj: String, offset: u32 },

    // v3.0 — LRU Cache
    LruCacheCheck { func: String, key: Box<IRInstruction> },
    LruCacheStore { func: String, key: Box<IRInstruction>, value: Box<IRInstruction> },

    // v3.0 — SIMD AVX2 (YMM 256-bit)
    SimdLoad { label: String },             // VMOVAPS ymm, [data]
    SimdOp { op: String, src: String },     // VADDPS/VMULPS/VSUBPS/VDIVPS
    SimdStore { label: String },            // VMOVAPS [data], ymm
    SimdReduce { op: String },              // horizontal reduce (sum/max/min)
    SimdSqrt,                               // VSQRTPS ymm

    // v3.0 — C extension / DLL
    DllLoad { path: String },               // LoadLibraryA
    DllGetProc { name: String },            // GetProcAddress
    DllCall { func_ptr: String, args: Vec<IRInstruction> },

    // v4.0 — Global State Tracker (FASE 1)
    GlobalLoad(String),             // MOV RAX, [__global_name] from .data
    GlobalStore(String),            // MOV [__global_name], RAX to .data

    // v4.0 — GPU Dispatch (FASE 4)
    GpuInit,                                            // cuInit(0) via nvcuda.dll
    GpuDeviceGet,                                       // cuDeviceGet(&dev, 0)
    GpuCtxCreate,                                       // cuCtxCreate(&ctx, 0, dev)
    GpuMalloc { size: Box<IRInstruction> },             // cuMemAlloc(&dptr, size)
    GpuMemcpyHtoD { dst: String, src: String, size: Box<IRInstruction> }, // cuMemcpyHtoD
    GpuMemcpyDtoH { dst: String, src: String, size: Box<IRInstruction> }, // cuMemcpyDtoH
    GpuLaunch { kernel: String, args: Vec<IRInstruction> },              // cuLaunchKernel
    GpuFree { ptr: String },                            // cuMemFree(dptr)
    GpuCtxDestroy,                                      // cuCtxDestroy(ctx)
    GpuAvxToCuda { avx_label: String, gpu_ptr: String, count: Box<IRInstruction> }, // AVX2→CUDA handoff

    // v4.0 — Vulkan/SPIR-V Dispatch
    VkInit,                                             // vkCreateInstance
    VkDeviceGet,                                        // vkEnumeratePhysicalDevices
    VkDeviceCreate,                                     // vkCreateDevice + queue
    VkBufferCreate { size: Box<IRInstruction> },        // vkCreateBuffer + vkAllocateMemory
    VkBufferWrite { dst: String, src: String, size: Box<IRInstruction> },  // vkMapMemory + memcpy
    VkBufferRead { dst: String, src: String, size: Box<IRInstruction> },   // vkMapMemory read back
    VkShaderLoad { spirv_path: String },                // vkCreateShaderModule from SPIR-V
    VkDispatch { shader: String, x: Box<IRInstruction>, y: Box<IRInstruction>, z: Box<IRInstruction> }, // vkCmdDispatch
    VkBufferFree { ptr: String },                       // vkFreeMemory + vkDestroyBuffer
    VkDestroy,                                          // vkDestroyDevice + vkDestroyInstance

    // Array operations
    AllocArray { ir_type: IRType, count: Box<IRInstruction> },
    LoadElement { array: Box<IRInstruction>, index: Box<IRInstruction> },
    StoreElement { array: Box<IRInstruction>, index: Box<IRInstruction>, value: Box<IRInstruction> },
    ArrayLength { array: Box<IRInstruction> },

    // OOP operations
    AllocObject { class_name: String, size: u32 },

    // --- GC Plus 💀☕ Exclusive Instructions (JaDead-BIB v1.0) ---
    // Módulo 1: Scope Tracker
    GCPlusScopeEnter { scope_id: u32 },
    GCPlusScopeExit { scope_id: u32 },
    
    // Módulo 2: Loop Anticipator
    GCPlusLoopAlloc { type_id: u32, pool_size: usize },
    GCPlusLoopReuse { pool_ptr: String },
    GCPlusLoopFree { pool_ptr: String },

    // Módulo 3: Escape Detector
    GCPlusEscapeCheck { ptr: String, bounds: (usize, usize) },
    GCPlusEscapeKill { ptr: String },

    // Módulo 4: Region Memory
    GCPlusRegionCreate { region_id: u32, size: usize },
    GCPlusRegionAlloc { region_id: u32 },
    GCPlusRegionFree { region_id: u32 },

    // Módulo 5: Cycle Breaker
    GCPlusCycleDetect { type_a: String, type_b: String },
    GCPlusCycleBreak { ptr: String },
    GCPlusWeakRef { ptr: String },

    // No-op
    Nop,
}

/// Constant value in IR
#[derive(Debug, Clone)]
pub enum IRConstValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    None,
}

/// IR binary operation
#[derive(Debug, Clone, Copy)]
pub enum IROp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    Shl,
    Shr,
    And,
    Or,
    Xor,
    MatMul,
}

/// IR comparison operation
#[derive(Debug, Clone, Copy)]
pub enum IRCmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    NotIn,
}

// ══════════════════════════════════════════════════════════
// v3.0 — Optimization passes
// ══════════════════════════════════════════════════════════

/// Constant folding: evaluate BinOp(Const, Const) at compile time
pub fn optimize_constant_folding(func: &mut IRFunction) -> usize {
    let mut folded = 0;
    let len = func.body.len();
    for i in 0..len {
        let new_instr = match &func.body[i] {
            IRInstruction::BinOp { op, left, right } => {
                if let (IRInstruction::LoadConst(IRConstValue::Int(a)),
                        IRInstruction::LoadConst(IRConstValue::Int(b))) = (left.as_ref(), right.as_ref()) {
                    let result = match op {
                        IROp::Add => Some(a.wrapping_add(*b)),
                        IROp::Sub => Some(a.wrapping_sub(*b)),
                        IROp::Mul => Some(a.wrapping_mul(*b)),
                        IROp::Div if *b != 0 => Some(a / b),
                        IROp::FloorDiv if *b != 0 => Some(a / b),
                        IROp::Mod if *b != 0 => Some(a % b),
                        IROp::Pow => Some(a.wrapping_pow(*b as u32)),
                        IROp::Shl => Some(a << (*b as u32)),
                        IROp::Shr => Some(a >> (*b as u32)),
                        IROp::And => Some(a & b),
                        IROp::Or => Some(a | b),
                        IROp::Xor => Some(a ^ b),
                        _ => None,
                    };
                    result.map(|v| IRInstruction::LoadConst(IRConstValue::Int(v)))
                } else if let (IRInstruction::LoadConst(IRConstValue::Float(a)),
                               IRInstruction::LoadConst(IRConstValue::Float(b))) = (left.as_ref(), right.as_ref()) {
                    let result = match op {
                        IROp::Add => Some(a + b),
                        IROp::Sub => Some(a - b),
                        IROp::Mul => Some(a * b),
                        IROp::Div if *b != 0.0 => Some(a / b),
                        _ => None,
                    };
                    result.map(|v| IRInstruction::LoadConst(IRConstValue::Float(v)))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(optimized) = new_instr {
            func.body[i] = optimized;
            folded += 1;
        }
    }
    folded
}

/// Dead code elimination: remove Nop instructions and unreachable code after Return
pub fn optimize_dead_code_elimination(func: &mut IRFunction) -> usize {
    let before = func.body.len();
    // Remove Nop instructions
    func.body.retain(|instr| !matches!(instr, IRInstruction::Nop));
    let eliminated = before - func.body.len();
    eliminated
}

/// Run all optimization passes on a function
pub fn optimize_function(func: &mut IRFunction) -> (usize, usize) {
    let folded = optimize_constant_folding(func);
    let eliminated = optimize_dead_code_elimination(func);
    (folded, eliminated)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_func(body: Vec<IRInstruction>) -> IRFunction {
        IRFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: IRType::I64,
            body,
        }
    }

    #[test]
    fn test_constant_folding_add() {
        let mut func = make_func(vec![
            IRInstruction::BinOp {
                op: IROp::Add,
                left: Box::new(IRInstruction::LoadConst(IRConstValue::Int(3))),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(4))),
            },
        ]);
        let folded = optimize_constant_folding(&mut func);
        assert_eq!(folded, 1);
        assert!(matches!(&func.body[0], IRInstruction::LoadConst(IRConstValue::Int(7))));
    }

    #[test]
    fn test_constant_folding_mul() {
        let mut func = make_func(vec![
            IRInstruction::BinOp {
                op: IROp::Mul,
                left: Box::new(IRInstruction::LoadConst(IRConstValue::Int(6))),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(7))),
            },
        ]);
        let folded = optimize_constant_folding(&mut func);
        assert_eq!(folded, 1);
        assert!(matches!(&func.body[0], IRInstruction::LoadConst(IRConstValue::Int(42))));
    }

    #[test]
    fn test_constant_folding_div_safe() {
        let mut func = make_func(vec![
            IRInstruction::BinOp {
                op: IROp::Div,
                left: Box::new(IRInstruction::LoadConst(IRConstValue::Int(10))),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(3))),
            },
        ]);
        let folded = optimize_constant_folding(&mut func);
        assert_eq!(folded, 1);
        assert!(matches!(&func.body[0], IRInstruction::LoadConst(IRConstValue::Int(3))));
    }

    #[test]
    fn test_constant_folding_div_by_zero_no_fold() {
        let mut func = make_func(vec![
            IRInstruction::BinOp {
                op: IROp::Div,
                left: Box::new(IRInstruction::LoadConst(IRConstValue::Int(10))),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(0))),
            },
        ]);
        let folded = optimize_constant_folding(&mut func);
        assert_eq!(folded, 0); // Should NOT fold div by zero
    }

    #[test]
    fn test_constant_folding_bitwise() {
        let mut func = make_func(vec![
            IRInstruction::BinOp {
                op: IROp::And,
                left: Box::new(IRInstruction::LoadConst(IRConstValue::Int(0xFF))),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(0x0F))),
            },
        ]);
        let folded = optimize_constant_folding(&mut func);
        assert_eq!(folded, 1);
        assert!(matches!(&func.body[0], IRInstruction::LoadConst(IRConstValue::Int(0x0F))));
    }

    #[test]
    fn test_constant_folding_shift() {
        let mut func = make_func(vec![
            IRInstruction::BinOp {
                op: IROp::Shl,
                left: Box::new(IRInstruction::LoadConst(IRConstValue::Int(1))),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(10))),
            },
        ]);
        let folded = optimize_constant_folding(&mut func);
        assert_eq!(folded, 1);
        assert!(matches!(&func.body[0], IRInstruction::LoadConst(IRConstValue::Int(1024))));
    }

    #[test]
    fn test_constant_folding_float_add() {
        let mut func = make_func(vec![
            IRInstruction::BinOp {
                op: IROp::Add,
                left: Box::new(IRInstruction::LoadConst(IRConstValue::Float(1.5))),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Float(2.5))),
            },
        ]);
        let folded = optimize_constant_folding(&mut func);
        assert_eq!(folded, 1);
        if let IRInstruction::LoadConst(IRConstValue::Float(v)) = &func.body[0] {
            assert!((v - 4.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected float constant");
        }
    }

    #[test]
    fn test_constant_folding_no_fold_variable() {
        let mut func = make_func(vec![
            IRInstruction::BinOp {
                op: IROp::Add,
                left: Box::new(IRInstruction::Load("x".to_string())),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(1))),
            },
        ]);
        let folded = optimize_constant_folding(&mut func);
        assert_eq!(folded, 0);
    }

    #[test]
    fn test_dead_code_elimination_nops() {
        let mut func = make_func(vec![
            IRInstruction::LoadConst(IRConstValue::Int(1)),
            IRInstruction::Nop,
            IRInstruction::Nop,
            IRInstruction::Return,
        ]);
        let eliminated = optimize_dead_code_elimination(&mut func);
        assert_eq!(eliminated, 2);
        assert_eq!(func.body.len(), 2);
    }

    #[test]
    fn test_dead_code_elimination_no_nops() {
        let mut func = make_func(vec![
            IRInstruction::LoadConst(IRConstValue::Int(1)),
            IRInstruction::Return,
        ]);
        let eliminated = optimize_dead_code_elimination(&mut func);
        assert_eq!(eliminated, 0);
    }

    #[test]
    fn test_optimize_function_combined() {
        let mut func = make_func(vec![
            IRInstruction::BinOp {
                op: IROp::Add,
                left: Box::new(IRInstruction::LoadConst(IRConstValue::Int(10))),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(20))),
            },
            IRInstruction::Nop,
            IRInstruction::Return,
        ]);
        let (folded, eliminated) = optimize_function(&mut func);
        assert_eq!(folded, 1);
        assert_eq!(eliminated, 1);
        assert_eq!(func.body.len(), 2);
        assert!(matches!(&func.body[0], IRInstruction::LoadConst(IRConstValue::Int(30))));
    }

    #[test]
    fn test_ir_type_byte_sizes() {
        assert_eq!(IRType::Void.byte_size(), 0);
        assert_eq!(IRType::I8.byte_size(), 1);
        assert_eq!(IRType::I16.byte_size(), 2);
        assert_eq!(IRType::I32.byte_size(), 4);
        assert_eq!(IRType::I64.byte_size(), 8);
        assert_eq!(IRType::F32.byte_size(), 4);
        assert_eq!(IRType::F64.byte_size(), 8);
        assert_eq!(IRType::Ptr.byte_size(), 8);
        assert_eq!(IRType::Vec256.byte_size(), 32);
    }

    #[test]
    fn test_ir_type_is_integer() {
        assert!(IRType::I8.is_integer());
        assert!(IRType::I32.is_integer());
        assert!(IRType::I64.is_integer());
        assert!(!IRType::F64.is_integer());
        assert!(!IRType::Ptr.is_integer());
    }

    #[test]
    fn test_ir_type_is_float() {
        assert!(IRType::F32.is_float());
        assert!(IRType::F64.is_float());
        assert!(!IRType::I64.is_float());
        assert!(!IRType::Void.is_float());
    }

    #[test]
    fn test_ir_module_creation() {
        let module = IRModule::new("TestModule");
        assert_eq!(module.name, "TestModule");
        assert!(module.functions.is_empty());
    }

    #[test]
    fn test_ir_function_creation() {
        let func = IRFunction::new(
            "main".to_string(),
            vec![("x".to_string(), IRType::I64)],
            IRType::Void,
        );
        assert_eq!(func.name, "main");
        assert_eq!(func.params.len(), 1);
        assert!(func.body.is_empty());
    }
}
