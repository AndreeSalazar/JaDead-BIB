#![allow(non_camel_case_types)]

// ============================================================
// ADead-BIB ABI Translators — Binary Frontend
// ============================================================
// Converts external binary formats → ADead-BIB IR
//
// Supported inputs:
//   CPU: PE (Windows), ELF (Linux), Mach-O (macOS)
//   GPU: SPIR-V (Vulkan/OpenCL), DXIL (DirectX 12)
//
// Pipeline: Parse → Decode → Map → Emit ADead-BIB IR
//
// Does NOT execute. Does NOT optimize. Does NOT compile.
// Only: Binary → IR
// ============================================================

pub mod core;
pub mod cpu;
pub mod gpu;
pub mod utils;
