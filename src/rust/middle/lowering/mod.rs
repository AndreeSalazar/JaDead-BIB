// ============================================================
// JaDead-BIB AST → IR Lowering
// ============================================================

mod c_lower;

pub use c_lower::lower_c_to_ir;

use crate::frontend::ast::Program;
use crate::middle::ir::Module;

/// Lower a Program AST to IR Module
pub fn lower_to_ir(program: &Program, name: &str) -> Module {
    let mut module = Module::new(name);

    #[cfg(target_os = "windows")]
    module.set_target("x86_64-pc-windows-msvc");
    #[cfg(target_os = "linux")]
    module.set_target("x86_64-unknown-linux-gnu");

    for func in &program.functions {
        let ir_func = c_lower::lower_function(func);
        module.add_function(ir_func);
    }

    module
}
