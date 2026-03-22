// ============================================================
// JaDead-BIB Backend — Java + C Nativo
// ============================================================

// JaDead-BIB original modules
pub mod isa;
pub mod jit;
pub mod pe;
pub mod gpu;

// ADead-BIB CPU backend (C compilation)
pub mod cpu;
pub mod output;

// Core format re-exports
pub use cpu::elf;
pub use cpu::flat_binary;
pub use cpu::pe as cpu_pe;
pub use cpu::pe_tiny;
pub use cpu::codegen;
pub use cpu::codegen_v2;
pub use cpu::microvm;
pub use cpu::pe_minimal;
pub use cpu::syscalls;
pub use cpu::win32_resolver;
