// ============================================================
// JaDead-BIB v1.0 — Main Library
// ============================================================
// JaDead = Java Dead | BIB = Binary Is Binary
//
// Filosofía:
// - Grace Hopper: 'la máquina sirve al humano'
// - Dennis Ritchie: 'small is beautiful'
// - Ken Thompson: 'trust only code you created'
// - James Gosling: 'write once, run anywhere — pero NATIVO'
//
// SIN JVM — SIN GC TRADICIONAL — SIN RUNTIME
// GC+ Determinístico — Zero-Pause — Compile-Time Memory
//
// Pipeline Java: Source → Preprocessor → Lexer → Parser → AST →
//                ImportResolver → UBDetector → IR → ISA → Binary
//
// Pipeline C:    Source → CPreprocessor → CLexer → CParser → AST →
//                CToIR → UBDetector → Optimizer → ISA → Binary
//
// Targets: windows | linux | fastos64 | fastos256
// ============================================================

// ── Core modules ─────────────────────────────────────────────
pub mod frontend;
pub mod middle;
pub mod backend;
pub mod gc_plus;

// ── Modules from ADead-BIB (C support) ──────────────────────
pub mod bg;
pub mod builder;
pub mod cache;
pub mod cli;
pub mod isa;
pub mod optimizer;
pub mod output;
pub mod preprocessor;
pub mod runtime;
pub mod stdlib;
pub mod toolchain;

// ── Frontend re-exports ──────────────────────────────────────
pub use frontend::c;

// ── Security module ──────────────────────────────────────────
pub use bg::{BinaryGuardian, SecurityLevel, SecurityPolicy, Verdict};

// ── ISA layer re-exports ─────────────────────────────────────
pub use isa::codegen;
pub use isa::isa_compiler::IsaCompiler;
pub use isa::bit_resolver::{BitResolver, BitTarget};
pub use isa::soa_optimizer::SoaOptimizer;
pub use isa::vex_emitter::VexEmitter;
pub use isa::ymm_allocator::YmmAllocator;

// ── Runtime re-exports ───────────────────────────────────────
pub use runtime::{CPUFeatures, ComputeBackend};

// ── Middle-end re-exports ────────────────────────────────────
pub use middle::ir::{Function, Module, Type};
pub use middle::lowering::lower_to_ir;
pub use middle::passes::PassManager;

// ── Preprocessor re-exports ─────────────────────────────────
pub use preprocessor::{HeaderResolver, MacroExpander, SymbolDedup};

// ── Cache re-exports ────────────────────────────────────────
pub use cache::ADeadCache;

// ── Output re-exports ───────────────────────────────────────
pub use output::OutputFormat;

// ── Toolchain Heritage re-exports ───────────────────────────
pub use toolchain::llvm_attrs::{LlvmAttribute, LlvmCallingConv, LlvmIntrinsic};
pub use toolchain::gcc_builtins::{GccAttribute, GccBuiltin};
pub use toolchain::gcc_compat::{parse_gcc_flag, GccFlagResult, GccOptLevel};
pub use toolchain::clang_compat::{parse_clang_flag, ClangFlagResult};
pub use toolchain::msvc_compat::{MsvcCallingConv, MsvcDeclspec, MsvcExtension, MsvcPragma};
pub use toolchain::calling_conventions::{
    detect_convention, shadow_space, CallFrame, CallingConvention,
};
