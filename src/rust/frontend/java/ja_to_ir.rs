// ============================================================
// Java AST to ADeadOp IR Generator for JaDead-BIB 💀☕
// ============================================================
// Takes JaCompilationUnit -> IRModule (ADeadOp SSA-form)
// Eliminates JVM overhead completely by outputting native ops
// ============================================================

use super::ja_ast::*;
use crate::middle::ir::*;
use super::ja_types::JaTypeChecker;
use crate::gc_plus::scope_tracker::ScopeTracker;
use crate::gc_plus::loop_anticipator::LoopAnticipator;
use crate::gc_plus::escape_detector::EscapeDetector;
use crate::gc_plus::region_memory::RegionMemory;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ClassLayout {
    pub name: String,
    pub size: u32,
    pub fields_offset: HashMap<String, u32>,
    pub fields_type: HashMap<String, IRType>,
}

pub struct JaToIrGenerator {
    type_checker: JaTypeChecker,
    current_functions: Vec<IRFunction>,
    local_types: HashMap<String, JaType>,
    scope_tracker: ScopeTracker,
    loop_anticipator: LoopAnticipator,
    escape_detector: EscapeDetector,
    region_memory: RegionMemory,
    
    // OOP Memory Mapping 
    class_layouts: HashMap<String, ClassLayout>,
    current_class: Option<String>,
    
    // Label tracking for control flow
    label_count: usize,
}

impl JaToIrGenerator {
    pub fn new() -> Self {
        Self {
            type_checker: JaTypeChecker::new(),
            current_functions: Vec::new(),
            local_types: HashMap::new(),
            scope_tracker: ScopeTracker::new(),
            loop_anticipator: LoopAnticipator::new(),
            escape_detector: EscapeDetector::new(),
            region_memory: RegionMemory::new(),
            class_layouts: HashMap::new(),
            current_class: None,
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

        // [GC PLUS] Módulo 4: Default Codebase Region
        let root_region_id = self.region_memory.define_region("App_Root_Region");

        // Pass 1: Compute Native Structural OOP Byte Layouts (Classes, Fields)
        for decl in &ast.declarations {
            if let JaTypeDecl::Class { name, body, .. } = decl {
                let mut offset = 0;
                let mut fields_offset = HashMap::new();
                let mut fields_type = HashMap::new();
                
                for member in body {
                    if let JaClassMember::Field { name: f_name, ty, .. } = member {
                        let ir_type = self.type_checker.resolve_type(ty)?;
                        let mut size = ir_type.byte_size() as u32;
                        if size == 0 { size = 8; } // Pointer fallback
                        
                        fields_offset.insert(f_name.clone(), offset);
                        fields_type.insert(f_name.clone(), ir_type);
                        offset += size;
                    }
                }
                
                if offset == 0 { offset = 8; } // Safe empty object size
                
                self.class_layouts.insert(name.clone(), ClassLayout {
                    name: name.clone(),
                    size: offset,
                    fields_offset,
                    fields_type,
                });
            }
        }

        // Pass 2: Method and body Native generation
        for decl in &ast.declarations {
            self.generate_type_decl(decl)?;
        }

        self.region_memory.free_region(root_region_id);

        module.functions = std::mem::take(&mut self.current_functions);

        // Return the collected module and functions
        // For simplicity in this structure, we just wrap it in the IRModule
        Ok(module)
    }

    // ── Declarations ─────────────────────────────────────────

    fn generate_type_decl(&mut self, decl: &JaTypeDecl) -> Result<(), String> {
        match decl {
            JaTypeDecl::Class { name, body, .. } => {
                self.current_class = Some(name.clone());
                for member in body {
                    self.generate_class_member(name, member)?;
                }
                self.current_class = None;
                Ok(())
            }
            JaTypeDecl::Record { name, body, .. } => {
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
            JaClassMember::Constructor { name: _name, params, body, .. } => {
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
        // [GC PLUS] Módulo 1: Enter Scope
        let sid = self.scope_tracker.current_scope().unwrap_or(0) as u32;
        self.scope_tracker.enter_scope();
        func.body.push(IRInstruction::GCPlusScopeEnter { scope_id: sid });

        for stmt in &block.stmts {
            self.generate_stmt(stmt, func)?;
        }

        // [GC PLUS] Módulo 1: Exit Scope
        let popped_sid = self.scope_tracker.current_scope().unwrap_or(0) as u32;
        let _freed_vars = self.scope_tracker.exit_scope()?;
        func.body.push(IRInstruction::GCPlusScopeExit { scope_id: popped_sid });
        
        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &JaStmt, func: &mut IRFunction) -> Result<(), String> {
        match stmt {
            JaStmt::Block(b) => self.generate_block(b, func),
            JaStmt::Expr(e) => {
                let ir_expr = self.generate_expr(e, func)?;
                // Emit side-effecting expressions
                match ir_expr {
                    IRInstruction::Call { .. } | IRInstruction::PrintStr(_) | IRInstruction::PrintInt 
                    | IRInstruction::Store(_) | IRInstruction::PropertySet { .. } | IRInstruction::StoreElement { .. } => {
                        func.body.push(ir_expr);
                    }
                    _ => {}
                }
                Ok(())
            }
            JaStmt::LocalVarDecl { ty, declarators } => {
                let ir_type = self.type_checker.resolve_type(ty)?;
                for decl in declarators {
                    self.local_types.insert(decl.name.clone(), ty.clone());
                    func.body.push(IRInstruction::VarDecl { 
                        name: decl.name.clone(), 
                        ir_type: ir_type.clone() 
                    });
                    
                    if let Some(init_expr) = &decl.init {
                        let val = self.generate_expr(init_expr, func)?;
                        func.body.push(val); 
                        func.body.push(IRInstruction::Store(decl.name.clone()));
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
        
                // [GC PLUS] Módulo 2: Pre-Alloc Loop Object Pool
                let pool_name = self.loop_anticipator.enter_loop();
                func.body.push(IRInstruction::GCPlusLoopAlloc { 
                    type_id: 0, // Placeholder for specific type inference 
                    pool_size: 1024 
                });

                let start_label = self.next_label("while_start");
                let end_label = self.next_label("while_end");

                func.body.push(IRInstruction::Label(start_label.clone()));
                
                let cond_val = self.generate_expr(cond, func)?;
                func.body.push(cond_val);
                func.body.push(IRInstruction::BranchIfFalse(end_label.clone()));

                // [GC PLUS] Inside loop body: Reuse memory instead of Alloc
                func.body.push(IRInstruction::GCPlusLoopReuse { pool_ptr: pool_name.clone() });
                self.generate_stmt(body, func)?;
                
                func.body.push(IRInstruction::Jump(start_label));
                
                // [GC PLUS] Loop End: Free Object Pool
                func.body.push(IRInstruction::Label(end_label));

                Ok(())
            }
            JaStmt::For { init, cond, update, body } => {
                if let Some(init_stmt) = init {
                    self.generate_stmt(init_stmt, func)?;
                }
                
                let start_label = self.next_label("for_start");
                let end_label = self.next_label("for_end");
                
                func.body.push(IRInstruction::Label(start_label.clone()));
                
                if let Some(cond_expr) = cond {
                    let cond_val = self.generate_expr(cond_expr, func)?;
                    func.body.push(cond_val);
                    func.body.push(IRInstruction::BranchIfFalse(end_label.clone()));
                }
                
                self.generate_stmt(body, func)?;
                
                for u in update {
                    let u_ir = self.generate_expr(u, func)?;
                    match u_ir {
                        IRInstruction::Call { .. } | IRInstruction::Store(_) | IRInstruction::PropertySet { .. } => {
                            func.body.push(u_ir);
                        }
                        _ => {}
                    }
                }
                
                func.body.push(IRInstruction::Jump(start_label));
                func.body.push(IRInstruction::Label(end_label));
                
                Ok(())
            }
            JaStmt::DoWhile { body, cond } => {
                let start_label = self.next_label("dowhile_start");
                let end_label = self.next_label("dowhile_end");

                func.body.push(IRInstruction::Label(start_label.clone()));
                self.generate_stmt(body, func)?;

                let cond_val = self.generate_expr(cond, func)?;
                func.body.push(cond_val);
                // If true, jump back
                func.body.push(IRInstruction::BranchIfFalse(end_label.clone()));
                func.body.push(IRInstruction::Jump(start_label));
                func.body.push(IRInstruction::Label(end_label));
                Ok(())
            }
            JaStmt::ForEach { ty: _, name, iterable, body } => {
                // Simplified: assume iterable is an array, iterate via index
                let iter_var = self.next_label("foreach_idx");
                let start_label = self.next_label("foreach_start");
                let end_label = self.next_label("foreach_end");

                // int __idx = 0
                func.body.push(IRInstruction::VarDecl { name: iter_var.clone(), ir_type: IRType::I64 });
                func.body.push(IRInstruction::LoadConst(IRConstValue::Int(0)));
                func.body.push(IRInstruction::Store(iter_var.clone()));

                // Declare the element variable
                func.body.push(IRInstruction::VarDecl { name: name.clone(), ir_type: IRType::I64 });

                func.body.push(IRInstruction::Label(start_label.clone()));

                // if __idx >= array.length, break
                let arr_ir = self.generate_expr(iterable, func)?;
                let arr_len = IRInstruction::ArrayLength { array: Box::new(arr_ir.clone()) };
                func.body.push(IRInstruction::Compare {
                    op: IRCmpOp::Lt,
                    left: Box::new(IRInstruction::Load(iter_var.clone())),
                    right: Box::new(arr_len),
                });
                func.body.push(IRInstruction::BranchIfFalse(end_label.clone()));

                // name = array[__idx]
                let elem = IRInstruction::LoadElement {
                    array: Box::new(arr_ir),
                    index: Box::new(IRInstruction::Load(iter_var.clone())),
                };
                func.body.push(elem);
                func.body.push(IRInstruction::Store(name.clone()));

                self.generate_stmt(body, func)?;

                // __idx++
                func.body.push(IRInstruction::BinOp {
                    op: IROp::Add,
                    left: Box::new(IRInstruction::Load(iter_var.clone())),
                    right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(1))),
                });
                func.body.push(IRInstruction::Store(iter_var));

                func.body.push(IRInstruction::Jump(start_label));
                func.body.push(IRInstruction::Label(end_label));
                Ok(())
            }
            JaStmt::Break(_) => {
                func.body.push(IRInstruction::Break);
                Ok(())
            }
            JaStmt::Continue(_) => {
                func.body.push(IRInstruction::Continue);
                Ok(())
            }
            JaStmt::Throw(expr) => {
                let val = self.generate_expr(expr, func)?;
                func.body.push(IRInstruction::Raise { exc_type: "Exception".to_string(), message: Some(Box::new(val)) });
                Ok(())
            }
            JaStmt::Switch { expr, cases } => {
                let switch_val = self.generate_expr(expr, func)?;
                let switch_var = self.next_label("switch_val");
                func.body.push(switch_val);
                func.body.push(IRInstruction::Store(switch_var.clone()));

                let end_label = self.next_label("switch_end");
                let mut case_labels = Vec::new();

                for (i, case) in cases.iter().enumerate() {
                    let case_label = self.next_label(&format!("case_{}", i));
                    case_labels.push(case_label.clone());

                    if case.labels.is_empty() {
                        // Default case — will be last fallthrough
                    } else {
                        for label_expr in &case.labels {
                            let label_val = self.generate_expr(label_expr, func)?;
                            func.body.push(IRInstruction::Compare {
                                op: IRCmpOp::Eq,
                                left: Box::new(IRInstruction::Load(switch_var.clone())),
                                right: Box::new(label_val),
                            });
                            func.body.push(IRInstruction::BranchIfFalse(if i + 1 < cases.len() {
                                self.next_label(&format!("case_next_{}", i))
                            } else {
                                end_label.clone()
                            }));
                        }
                    }

                    func.body.push(IRInstruction::Label(case_label));
                    for stmt in &case.body {
                        self.generate_stmt(stmt, func)?;
                    }
                    if case.is_arrow {
                        func.body.push(IRInstruction::Jump(end_label.clone()));
                    }
                }
                func.body.push(IRInstruction::Label(end_label));
                Ok(())
            }
            JaStmt::Try { body, catches, finally_block, .. } => {
                let handler_label = self.next_label("catch_handler");
                let end_label = self.next_label("try_end");

                func.body.push(IRInstruction::TryBegin(handler_label.clone()));
                self.generate_block(body, func)?;
                func.body.push(IRInstruction::TryEnd);
                func.body.push(IRInstruction::Jump(end_label.clone()));

                func.body.push(IRInstruction::Label(handler_label));
                for catch in catches {
                    self.generate_block(&catch.body, func)?;
                }
                func.body.push(IRInstruction::ClearError);

                if let Some(fin) = finally_block {
                    func.body.push(IRInstruction::FinallyBegin);
                    self.generate_block(fin, func)?;
                    func.body.push(IRInstruction::FinallyEnd);
                }

                func.body.push(IRInstruction::Label(end_label));
                Ok(())
            }
            JaStmt::Empty => Ok(()),
            _ => Ok(())
        }
    }

    // ── Expressions ──────────────────────────────────────────

    fn generate_expr(&mut self, expr: &JaExpr, func: &mut IRFunction) -> Result<IRInstruction, String> {
        match expr {
            JaExpr::IntLiteral(v) => Ok(IRInstruction::LoadConst(IRConstValue::Int(*v))),
            JaExpr::LongLiteral(v) => Ok(IRInstruction::LoadConst(IRConstValue::Int(*v))),
            JaExpr::FloatLiteral(v) => Ok(IRInstruction::LoadConst(IRConstValue::Float(*v))),
            JaExpr::DoubleLiteral(v) => Ok(IRInstruction::LoadConst(IRConstValue::Float(*v))),
            JaExpr::CharLiteral(v) => Ok(IRInstruction::LoadConst(IRConstValue::Int(*v as i64))),
            JaExpr::BooleanLiteral(v) => Ok(IRInstruction::LoadConst(IRConstValue::Bool(*v))),
            JaExpr::StringLiteral(v) => Ok(IRInstruction::LoadString(v.clone())),
            JaExpr::Null => Ok(IRInstruction::LoadConst(IRConstValue::Int(0))),
            JaExpr::Name(n) => Ok(IRInstruction::Load(n.clone())),
            JaExpr::NewArray { ty, dimensions, .. } => {
                let ir_type = self.type_checker.resolve_type(ty)?;
                if dimensions.is_empty() { return Err("Array dimension required".to_string()); }
                let count = if let Some(Some(c)) = dimensions.get(0) {
                    Box::new(self.generate_expr(c, func)?)
                } else {
                    return Err("Empty array dimensions not supported natively yet".to_string());
                };
                Ok(IRInstruction::AllocArray { ir_type, count })
            }
            JaExpr::NewObject { ty, args: _, body: _ } => {
                let name = match ty {
                    &JaType::Class(ref c) => c.clone(),
                    _ => return Err("Only Classes can be directly instantiated via new natively".to_string()),
                };
                let layout = self.class_layouts.get(&name).ok_or_else(|| format!("Class {} not found", name))?;
                Ok(IRInstruction::AllocObject { class_name: name.clone(), size: layout.size })
            }
            JaExpr::FieldAccess { target, field } => {
                let t = self.generate_expr(target, func)?;
                if field == "length" {
                    return Ok(IRInstruction::ArrayLength { array: Box::new(t.clone()) });
                }
                
                let (root_obj, class_name) = match t {
                    IRInstruction::Load(n) => {
                        if n == "System" && field == "out" {
                            return Ok(IRInstruction::Load("System.out".to_string()));
                        }

                        let cname = if n == "this" {
                            self.current_class.clone().unwrap_or_default()
                        } else if let Some(JaType::Class(c)) = self.local_types.get(&n) {
                            c.clone()
                        } else { "".to_string() };
                        (n, cname)
                    },
                    _ => ("temp_obj".to_string(), "".to_string())
                };

                // [GC PLUS] Módulo 3: Escape Detector Null Safety
                self.escape_detector.analyze_bounds(&root_obj, None, None);
                func.body.push(IRInstruction::GCPlusEscapeCheck { ptr: root_obj.clone(), bounds: (0, 0) });
                
                let layout = self.class_layouts.get(&class_name).ok_or_else(|| format!("Class {} layout not found for field {}", class_name, field))?;
                let offset = *layout.fields_offset.get(field).unwrap_or(&0);
                
                Ok(IRInstruction::PropertyGet { obj: root_obj, offset })
            }
            JaExpr::ArrayAccess { array, index } => {
                let a = self.generate_expr(array, func)?;
                let idx = self.generate_expr(index, func)?;
                Ok(IRInstruction::LoadElement { array: Box::new(a), index: Box::new(idx) })
            }
            JaExpr::Assign { target, value, .. } => {
                let val = self.generate_expr(value, func)?;
                
                // If it's assigning to a field like this.vida = 100
                if let JaExpr::FieldAccess { target: root_t, field } = &**target {
                    func.body.push(val); 
                    let rt = self.generate_expr(root_t, func)?;
                    let (root_obj, class_name) = match rt {
                        IRInstruction::Load(n) => {
                            let cname = if n == "this" {
                                self.current_class.clone().unwrap_or_default()
                            } else if let Some(JaType::Class(c)) = self.local_types.get(&n) {
                                c.clone()
                            } else { "".to_string() };
                            (n, cname)
                        },
                        _ => ("temp_obj".to_string(), "".to_string())
                    };
                    
                    let layout = self.class_layouts.get(&class_name).ok_or_else(|| format!("Class {} layout not found for field {}", class_name, field))?;
                    let offset = *layout.fields_offset.get(field).unwrap_or(&0);
                    
                    Ok(IRInstruction::PropertySet { obj: root_obj, offset })
                } else if let JaExpr::ArrayAccess { array, index } = &**target {
                    let a = self.generate_expr(array, func)?;
                    let idx = self.generate_expr(index, func)?;
                    Ok(IRInstruction::StoreElement { array: Box::new(a), index: Box::new(idx), value: Box::new(val) })
                } else if let JaExpr::Name(n) = &**target {
                    func.body.push(val);
                    Ok(IRInstruction::Store(n.clone()))
                } else {
                    Err("Unsupported assignment target".to_string())
                }
            }
            JaExpr::Binary { op, left, right } => {
                let l = Box::new(self.generate_expr(left, func)?);
                let r = Box::new(self.generate_expr(right, func)?);
                
                // If this is string concatenation, delegate to native C-hook
                if *op == JaBinOp::Add {
                    let is_l_str = match &*l {
                        IRInstruction::LoadString(_) => true,
                        IRInstruction::Load(name) => {
                            if let Some(JaType::Class(class)) = self.local_types.get(name) {
                                class == "String"
                            } else { false }
                        }
                        IRInstruction::Call { func: fn_name, .. } => fn_name == "jdb_string_concat",
                        _ => false,
                    };
                    
                    let is_r_str = match &*r {
                        IRInstruction::LoadString(_) => true,
                        IRInstruction::Load(name) => {
                            if let Some(JaType::Class(class)) = self.local_types.get(name) {
                                class == "String"
                            } else { false }
                        }
                        IRInstruction::Call { func: fn_name, .. } => fn_name == "jdb_string_concat",
                        _ => false,
                    };

                    if is_l_str || is_r_str {
                        return Ok(IRInstruction::Call { func: "jdb_string_concat".to_string(), args: vec![*l, *r] });
                    }
                }

                match op {
                    JaBinOp::Add | JaBinOp::Sub | JaBinOp::Mul | JaBinOp::Div | JaBinOp::Rem | 
                    JaBinOp::Shl | JaBinOp::Shr | JaBinOp::BitAnd | JaBinOp::BitOr | JaBinOp::BitXor => {
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
                            _ => unreachable!()
                        };
                        Ok(IRInstruction::BinOp { op: ir_op, left: l, right: r })
                    }
                    JaBinOp::Eq | JaBinOp::Neq | JaBinOp::Lt | JaBinOp::Gt | JaBinOp::Le | JaBinOp::Ge => {
                        let ir_cmp = match op {
                            JaBinOp::Eq => IRCmpOp::Eq,
                            JaBinOp::Neq => IRCmpOp::Ne,
                            JaBinOp::Lt => IRCmpOp::Lt,
                            JaBinOp::Gt => IRCmpOp::Gt,
                            JaBinOp::Le => IRCmpOp::Le,
                            JaBinOp::Ge => IRCmpOp::Ge,
                            _ => unreachable!()
                        };
                        Ok(IRInstruction::Compare { op: ir_cmp, left: l, right: r })
                    }
                    _ => Err(format!("Unsupported binary op {:?}", op))
                }
            }
            JaExpr::Unary { op, expr, is_postfix } => {
                match op {
                    JaUnaryOp::Inc => {
                        if let JaExpr::Name(n) = &**expr {
                            if *is_postfix {
                                // Return old value, then increment
                                let old_val = IRInstruction::Load(n.clone());
                                func.body.push(IRInstruction::BinOp {
                                    op: IROp::Add,
                                    left: Box::new(IRInstruction::Load(n.clone())),
                                    right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(1))),
                                });
                                func.body.push(IRInstruction::Store(n.clone()));
                                Ok(old_val)
                            } else {
                                let add = IRInstruction::BinOp {
                                    op: IROp::Add,
                                    left: Box::new(IRInstruction::Load(n.clone())),
                                    right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(1))),
                                };
                                func.body.push(add);
                                Ok(IRInstruction::Store(n.clone()))
                            }
                        } else {
                            Err("Unsupported inc target".to_string())
                        }
                    }
                    JaUnaryOp::Dec => {
                        if let JaExpr::Name(n) = &**expr {
                            if *is_postfix {
                                let old_val = IRInstruction::Load(n.clone());
                                func.body.push(IRInstruction::BinOp {
                                    op: IROp::Sub,
                                    left: Box::new(IRInstruction::Load(n.clone())),
                                    right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(1))),
                                });
                                func.body.push(IRInstruction::Store(n.clone()));
                                Ok(old_val)
                            } else {
                                let sub = IRInstruction::BinOp {
                                    op: IROp::Sub,
                                    left: Box::new(IRInstruction::Load(n.clone())),
                                    right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(1))),
                                };
                                func.body.push(sub);
                                Ok(IRInstruction::Store(n.clone()))
                            }
                        } else {
                            Err("Unsupported dec target".to_string())
                        }
                    }
                    JaUnaryOp::Minus => {
                        let e = Box::new(self.generate_expr(expr, func)?);
                        Ok(IRInstruction::BinOp { op: IROp::Sub, left: Box::new(IRInstruction::LoadConst(IRConstValue::Int(0))), right: e })
                    }
                    JaUnaryOp::Plus => {
                        self.generate_expr(expr, func)
                    }
                    JaUnaryOp::Not => {
                        let e = Box::new(self.generate_expr(expr, func)?);
                        Ok(IRInstruction::Compare {
                            op: IRCmpOp::Eq,
                            left: e,
                            right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(0))),
                        })
                    }
                    JaUnaryOp::BitNot => {
                        let e = Box::new(self.generate_expr(expr, func)?);
                        Ok(IRInstruction::BinOp { op: IROp::Xor, left: e, right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(-1))) })
                    }
                }
            }
            JaExpr::MethodCall { target, name, args, .. } => {
                let mut ir_args = Vec::new();
                
                if let Some(t) = target {
                    ir_args.push(self.generate_expr(t, func)?);
                }
                
                for a in args {
                    ir_args.push(self.generate_expr(a, func)?);
                }
                
                // Built-in mapping
                if name == "System.out.println" || name == "println" || name == "print" {
                    // Extract the true argument
                    let actual_arg = if target.is_some() && ir_args.len() > 1 {
                        &ir_args[1]
                    } else if let Some(a) = ir_args.first() {
                        a
                    } else {
                        return Ok(IRInstruction::PrintStr("".to_string()));
                    };

                    match actual_arg {
                        IRInstruction::LoadString(s) => return Ok(IRInstruction::PrintStr(s.clone())),
                        IRInstruction::Load(ref var_name) => {
                            if let Some(JaType::Class(ref class_name)) = self.local_types.get(var_name) {
                                if class_name == "String" {
                                    return Ok(IRInstruction::Call { func: "jdb_print_obj".to_string(), args: vec![actual_arg.clone()] });
                                }
                            }
                            func.body.push(actual_arg.clone());
                            return Ok(IRInstruction::PrintInt);
                        }
                        IRInstruction::Call { func: ref fn_name, args: ref _a } => {
                            if fn_name == "jdb_string_concat" || fn_name == "jdb_string_len" {
                                if fn_name == "jdb_string_concat" {
                                    return Ok(IRInstruction::Call { func: "jdb_print_obj".to_string(), args: vec![actual_arg.clone()] });
                                } else {
                                    func.body.push(actual_arg.clone());
                                    return Ok(IRInstruction::PrintInt);
                                }
                            }
                            // Default fallback
                            func.body.push(actual_arg.clone());
                            return Ok(IRInstruction::PrintInt);
                        }
                        _ => {
                            func.body.push(actual_arg.clone());
                            return Ok(IRInstruction::PrintInt);
                        }
                    }
                }
                
                // String core extensions mappings
                if name == "length" {
                    // Target is ir_args[0], since length() operates on an object
                    return Ok(IRInstruction::Call { func: "jdb_string_len".to_string(), args: vec![ir_args[0].clone()] });
                }
                if name == "equals" || name == "contentEquals" {
                    return Ok(IRInstruction::Call { func: "jdb_string_eq".to_string(), args: vec![ir_args[0].clone(), ir_args[1].clone()] });
                }

                Ok(IRInstruction::Call { func: name.clone(), args: ir_args })
            }
            JaExpr::Ternary { cond, true_expr, false_expr } => {
                let cond_val = self.generate_expr(cond, func)?;
                func.body.push(cond_val);
                let else_label = self.next_label("tern_else");
                let end_label = self.next_label("tern_end");
                let result_var = self.next_label("tern_result");
                func.body.push(IRInstruction::VarDecl { name: result_var.clone(), ir_type: IRType::I64 });
                func.body.push(IRInstruction::BranchIfFalse(else_label.clone()));
                let tv = self.generate_expr(true_expr, func)?;
                func.body.push(tv);
                func.body.push(IRInstruction::Store(result_var.clone()));
                func.body.push(IRInstruction::Jump(end_label.clone()));
                func.body.push(IRInstruction::Label(else_label));
                let fv = self.generate_expr(false_expr, func)?;
                func.body.push(fv);
                func.body.push(IRInstruction::Store(result_var.clone()));
                func.body.push(IRInstruction::Label(end_label));
                Ok(IRInstruction::Load(result_var))
            }
            JaExpr::Cast { ty, expr } => {
                // For now, just generate the inner expression (native types are all register-width)
                let _ir_type = self.type_checker.resolve_type(ty)?;
                self.generate_expr(expr, func)
            }
            JaExpr::Instanceof { expr, ty: _, pattern_name } => {
                // Simplified: always returns true for now (full vtable check in future)
                let _e = self.generate_expr(expr, func)?;
                if let Some(pname) = pattern_name {
                    func.body.push(IRInstruction::VarDecl { name: pname.clone(), ir_type: IRType::Ptr });
                    func.body.push(_e);
                    func.body.push(IRInstruction::Store(pname.clone()));
                }
                Ok(IRInstruction::LoadConst(IRConstValue::Bool(true)))
            }
            _ => Err(format!("Unimplemented IR Generation for Expr: {:?}", expr))
        }
    }
}
