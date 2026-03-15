// ============================================================
// Java AST to ADeadOp IR Generator for JaDead-BIB 💀☕
// ============================================================
// Takes JaCompilationUnit -> IRModule (ADeadOp SSA-form)
// Eliminates JVM overhead completely by outputting native ops
// ============================================================

use super::ja_ast::*;
use crate::middle::ir::*;
use super::ja_types::JaTypeChecker;

pub struct JaToIrGenerator {
    type_checker: JaTypeChecker,
    current_module: Option<IRModule>,
    current_functions: Vec<IRFunction>,
    
    // Label tracking for control flow
    label_count: usize,
}

impl JaToIrGenerator {
    pub fn new() -> Self {
        Self {
            type_checker: JaTypeChecker::new(),
            current_module: None,
            current_functions: Vec::new(),
            label_count: 0,
        }
    }

    fn next_label(&mut self, prefix: &str) -> String {
        self.label_count += 1;
        format!("{}_{}", prefix, self.label_count)
    }

    // ── Entry Point ──────────────────────────────────────────

    pub fn generate(&mut self, ast: &JaCompilationUnit) -> Result<IRModule, String> {
        let name = match &ast.package {
            Some(pkg) => pkg.name.clone(),
            None => "JaDead_DefaultModule".to_string(),
        };
        
        // Modules in ADeadOp usually represent the compilation unit / binary
        let mut module = IRModule::new(&name);

        for decl in &ast.declarations {
            self.generate_type_decl(decl)?;
        }

        // Return the collected module and functions
        // For simplicity in this structure, we just wrap it in the IRModule
        Ok(module)
    }

    // ── Declarations ─────────────────────────────────────────

    fn generate_type_decl(&mut self, decl: &JaTypeDecl) -> Result<(), String> {
        match decl {
            JaTypeDecl::Class { name, body, .. } => {
                for member in body {
                    self.generate_class_member(name, member)?;
                }
                Ok(())
            }
            JaTypeDecl::Record { name, components, body, .. } => {
                // Record translates natively to an IR Struct definition + auto methods
                // Here we just map its explicitly defined members
                for member in body {
                    self.generate_class_member(name, member)?;
                }
                Ok(())
            }
            JaTypeDecl::Interface { .. } => {
                // Interfaces just generate vtable signatures, no direct IR bodies
                // unless they have default methods.
                Ok(())
            }
            JaTypeDecl::Enum { name, body, .. } => {
                for member in body {
                    self.generate_class_member(name, member)?;
                }
                Ok(())
            }
        }
    }

    fn generate_class_member(&mut self, class_name: &str, member: &JaClassMember) -> Result<(), String> {
        match member {
            JaClassMember::Method { name, return_type, params, body, .. } => {
                if let Some(block) = body {
                    // Mangle name for the Native ABI: ClassName_MethodName
                    let mangled_name = format!("{}_{}", class_name, name);
                    
                    let ret_ir_type = self.type_checker.resolve_type(return_type)?;
                    let mut ir_params = Vec::new();

                    for p in params {
                        // All Java instance methods have an implicit 'this' pointer. 
                        // Except statics, but we ignore the distinction in this basic pass.
                        ir_params.push((p.name.clone(), self.type_checker.resolve_type(&p.ty)?));
                    }

                    let mut ir_func = IRFunction::new(mangled_name, ir_params, ret_ir_type);
                    self.generate_block(block, &mut ir_func)?;
                    
                    // Implicit return void if not present
                    if ir_func.return_type == IRType::Void {
                        ir_func.body.push(IRInstruction::ReturnVoid);
                    }
                    
                    self.current_functions.push(ir_func);
                }
                Ok(())
            }
            JaClassMember::Constructor { name, params, body, .. } => {
                let mangled_name = format!("{}_<init>", class_name);
                let ret_ir_type = IRType::Void; // Constructors return void internally
                let mut ir_params = Vec::new();
                
                // Implicit 'this' ptr
                ir_params.push(("this".to_string(), IRType::Ptr));

                for p in params {
                    ir_params.push((p.name.clone(), self.type_checker.resolve_type(&p.ty)?));
                }

                let mut ir_func = IRFunction::new(mangled_name, ir_params, ret_ir_type);
                self.generate_block(body, &mut ir_func)?;
                ir_func.body.push(IRInstruction::ReturnVoid);
                self.current_functions.push(ir_func);

                Ok(())
            }
            _ => { Ok(()) }
        }
    }

    // ── Statements ───────────────────────────────────────────

    fn generate_block(&mut self, block: &JaBlock, func: &mut IRFunction) -> Result<(), String> {
        for stmt in &block.stmts {
            self.generate_stmt(stmt, func)?;
        }
        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &JaStmt, func: &mut IRFunction) -> Result<(), String> {
        match stmt {
            JaStmt::Block(b) => self.generate_block(b, func),
            JaStmt::Expr(e) => {
                let ir_expr = self.generate_expr(e, func)?;
                // If it's just an expression statement, we evaluate it. 
                // ADeadOp requires assignment to registers/variables, but for Call we just emit it.
                if let IRInstruction::Call { .. } = ir_expr {
                    func.body.push(ir_expr);
                }
                Ok(())
            }
            JaStmt::LocalVarDecl { ty, declarators } => {
                let ir_type = self.type_checker.resolve_type(ty)?;
                for decl in declarators {
                    func.body.push(IRInstruction::VarDecl { 
                        name: decl.name.clone(), 
                        ir_type: ir_type.clone() 
                    });
                    
                    if let Some(init_expr) = &decl.init {
                        let val = self.generate_expr(init_expr, func)?;
                        func.body.push(IRInstruction::Store(decl.name.clone()));
                        func.body.push(val); // In actual ADeadOp, Store usually takes the val as argument, but keeping semantics simple here.
                    }
                }
                Ok(())
            }
            JaStmt::Return(expr_opt) => {
                if let Some(e) = expr_opt {
                    let val = self.generate_expr(e, func)?;
                    func.body.push(val); // Put on top of stack/RAX
                    func.body.push(IRInstruction::Return);
                } else {
                    func.body.push(IRInstruction::ReturnVoid);
                }
                Ok(())
            }
            JaStmt::If { cond, then_branch, else_branch } => {
                let cond_val = self.generate_expr(cond, func)?;
                func.body.push(cond_val);
                
                let else_label = self.next_label("else");
                let end_label = self.next_label("endif");

                func.body.push(IRInstruction::BranchIfFalse(else_label.clone()));
                
                self.generate_stmt(then_branch, func)?;
                func.body.push(IRInstruction::Jump(end_label.clone()));
                
                func.body.push(IRInstruction::Label(else_label));
                if let Some(eb) = else_branch {
                    self.generate_stmt(eb, func)?;
                }
                func.body.push(IRInstruction::Label(end_label));

                Ok(())
            }
            JaStmt::While { cond, body } => {
                let start_label = self.next_label("while_start");
                let end_label = self.next_label("while_end");

                func.body.push(IRInstruction::Label(start_label.clone()));
                
                let cond_val = self.generate_expr(cond, func)?;
                func.body.push(cond_val);
                func.body.push(IRInstruction::BranchIfFalse(end_label.clone()));

                self.generate_stmt(body, func)?;
                func.body.push(IRInstruction::Jump(start_label));
                
                func.body.push(IRInstruction::Label(end_label));

                Ok(())
            }
            _ => Ok(()) // Stub
        }
    }

    // ── Expressions ──────────────────────────────────────────

    fn generate_expr(&mut self, expr: &JaExpr, func: &mut IRFunction) -> Result<IRInstruction, String> {
        match expr {
            JaExpr::IntLiteral(v) => Ok(IRInstruction::LoadConst(IRConstValue::Int(*v))),
            JaExpr::FloatLiteral(v) => Ok(IRInstruction::LoadConst(IRConstValue::Float(*v))),
            JaExpr::BooleanLiteral(v) => Ok(IRInstruction::LoadConst(IRConstValue::Bool(*v))),
            JaExpr::Name(n) => Ok(IRInstruction::Load(n.clone())),
            JaExpr::FieldAccess { target, field } => {
                // E.g. this.vida, target=this, field=vida
                // In ADeadOp v3.0, PropertyGet is used
                let t = self.generate_expr(target, func)?;
                // Simulating extraction of root var name
                let root_obj = match t {
                    IRInstruction::Load(n) | IRInstruction::LoadString(n) => n,
                    _ => "temp_obj".to_string()
                };
                Ok(IRInstruction::PropertyGet { obj: root_obj, name: field.clone() })
            }
            JaExpr::Assign { target, value, .. } => {
                let val = self.generate_expr(value, func)?;
                func.body.push(val); // Put value on stack/rax
                
                // If it's assigning to a field like this.vida = 100
                if let JaExpr::FieldAccess { target: root_t, field } = &**target {
                    let rt = self.generate_expr(root_t, func)?;
                    let root_obj = match rt {
                        IRInstruction::Load(n) | IRInstruction::LoadString(n) => n,
                        _ => "temp_obj".to_string()
                    };
                    Ok(IRInstruction::PropertySet { obj: root_obj, name: field.clone() })
                } else if let JaExpr::Name(n) = &**target {
                    Ok(IRInstruction::Store(n.clone()))
                } else {
                    Err("Unsupported assignment target".to_string())
                }
            }
            JaExpr::Binary { op, left, right } => {
                let l = Box::new(self.generate_expr(left, func)?);
                let r = Box::new(self.generate_expr(right, func)?);
                let ir_op = match op {
                    JaBinOp::Add => IROp::Add,
                    JaBinOp::Sub => IROp::Sub,
                    JaBinOp::Mul => IROp::Mul,
                    JaBinOp::Div => IROp::Div,
                    JaBinOp::Rem => IROp::Mod,
                    JaBinOp::Shl => IROp::Shl,
                    JaBinOp::Shr => IROp::Shr,
                    JaBinOp::BitAnd => IROp::And,
                    JaBinOp::BitOr => IROp::Or,
                    JaBinOp::BitXor => IROp::Xor,
                    _ => IROp::Add // Fallback
                };
                Ok(IRInstruction::BinOp { op: ir_op, left: l, right: r })
            }
            JaExpr::MethodCall { name, args, .. } => {
                let mut ir_args = Vec::new();
                for a in args {
                    ir_args.push(self.generate_expr(a, func)?);
                }
                
                // Built-in mapping -> print maps to ADeadOp native Print
                if name == "System.out.println" || name == "println" || name == "print" {
                    // For string literals
                    if let Some(IRInstruction::LoadString(s)) = ir_args.first() {
                         return Ok(IRInstruction::PrintStr(s.clone()));
                    }
                }

                Ok(IRInstruction::Call { func: name.clone(), args: ir_args })
            }
            _ => Err(format!("Unimplemented IR Generation for Expr: {:?}", expr))
        }
    }
}
