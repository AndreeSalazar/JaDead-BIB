// ============================================================
// JaDead-BIB Middle-End — Java + C
// ============================================================

pub mod analysis;
pub mod ir;
pub mod lowering;
pub mod passes;
pub mod strict_type_checker;

// JaDead-BIB original modules
pub mod ja_ir;
pub mod ub_detector;

pub use ir::{BasicBlock, Function, Instruction, Module, Type as IRType, Value};
pub use lowering::lower_to_ir;
pub use passes::PassManager;
