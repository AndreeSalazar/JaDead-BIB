// ============================================================
// JaDead-BIB Toolchain Heritage Module
// ============================================================
// LLVM, GCC, MSVC heritage — Sin C++ name mangling
// ============================================================

pub mod calling_conventions;
pub mod clang_compat;
pub mod gcc_builtins;
pub mod gcc_compat;
pub mod llvm_attrs;
pub mod msvc_compat;

// Keep cpp_name_mangler available but not re-exported prominently
#[allow(dead_code)]
pub mod cpp_name_mangler;

pub use llvm_attrs::{LlvmAttribute, LlvmCallingConv, LlvmIntrinsic};
pub use gcc_builtins::{GccAttribute, GccBuiltin};
pub use msvc_compat::{MsvcCallingConv, MsvcDeclspec, MsvcExtension, MsvcPragma};
pub use calling_conventions::{detect_convention, shadow_space, CallFrame, CallingConvention};
