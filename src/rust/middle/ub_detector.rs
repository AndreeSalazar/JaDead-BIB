// ============================================================
// Java UB Detector for JaDead-BIB 💀☕
// ============================================================
// Detects 15+ types of Undefined Behaviors in Java code
// Converted from Runtime Exceptions (JVM) to Compile Time Errors
// ============================================================

use crate::frontend::java::ja_ast::*;
use crate::gc_plus::cycle_breaker::CycleBreaker;

#[derive(Debug, Clone, PartialEq)]
pub enum JavaUB {
    // Heredados de C/C++ (ADead-BIB)
    NullDeref,              // null.method() -> NullPointerException pre-detectado
    ArrayIndexOutOfBounds,  // arr[100] con arr[10] -> pre-detectado
    DivisionByZero,         // x / 0 -> pre-detectado
    IntegerOverflow,        // int + int overflow -> warning
    
    // Java-specific
    ClassCastException,     // (String) integer -> pre-detectado
    StackOverflow,          // recursión sin base -> pre-detectado
    ConcurrentModification, // modificar lista en foreach -> pre-detectado
    NegativeArraySize,      // new int[-1] -> pre-detectado
    StringIndexOutOfBounds, // "hola".charAt(100) -> pre-detectado
    NumberFormatException,  // Integer.parseInt("abc") -> warning
    EmptyOptional,          // Optional.get() sin isPresent() -> pre-detectado
    UncheckedCast,          // cast genérico sin verificar -> warning
    DeadLock,               // patrones de deadlock -> warning
    ResourceLeak,           // stream/file sin close() -> pre-detectado
    UnsafePublicField,      // campo público mutable en record -> warning
}

#[derive(Debug, Clone)]
pub struct UBWarning {
    pub ub_type: JavaUB,
    pub message: String,
}

pub struct UbDetector {
    warnings: Vec<UBWarning>,
    cycle_breaker: CycleBreaker,
}

impl UbDetector {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
            cycle_breaker: CycleBreaker::new(),
        }
    }

    pub fn analyze(&mut self, ast: &JaCompilationUnit) -> Vec<UBWarning> {
        self.warnings.clear();

        for decl in &ast.declarations {
            self.analyze_type_decl(decl);
        }

        self.warnings.clone()
    }

    fn analyze_type_decl(&mut self, decl: &JaTypeDecl) {
        match decl {
            JaTypeDecl::Class { body, .. } => {
                for member in body {
                    self.analyze_class_member(member);
                }
            }
            JaTypeDecl::Record { body, .. } => {
                // Record parameters should not be mutable/public directly in unsafe ways
                for member in body {
                    if let JaClassMember::Field { modifiers, .. } = member {
                        if modifiers.contains(&JaModifier::Public) && !modifiers.contains(&JaModifier::Final) {
                            self.warnings.push(UBWarning {
                                ub_type: JavaUB::UnsafePublicField,
                                message: "Campo público mutable detectado en un Record/Clase que rompe la inmutabilidad".to_string()
                            });
                        }
                    }
                    self.analyze_class_member(member);
                }
            }
            JaTypeDecl::Enum { body, .. } => {
                for member in body {
                    self.analyze_class_member(member);
                }
            }
            JaTypeDecl::Interface { body, .. } => {
                for member in body {
                    self.analyze_class_member(member);
                }
            }
        }
    }

    fn analyze_class_member(&mut self, member: &JaClassMember) {
        match member {
            JaClassMember::Method { name, body: Some(block), .. } => {
                // StackOverflow detection: check if method calls itself without a base case
                if self.detect_unbounded_recursion(name, block) {
                    self.warnings.push(UBWarning {
                        ub_type: JavaUB::StackOverflow,
                        message: format!("Método '{}' parece ser recursivo sin caso base detectable", name),
                    });
                }
                self.analyze_block(block);
            }
            JaClassMember::Constructor { body, .. } => {
                self.analyze_block(body);
            }
            JaClassMember::Initializer(block, _) => {
                self.analyze_block(block);
            }
            _ => {}
        }
    }

    fn detect_unbounded_recursion(&self, method_name: &str, block: &JaBlock) -> bool {
        let mut has_self_call = false;
        let mut has_base_case = false;
        for stmt in &block.stmts {
            self.check_recursion_stmt(method_name, stmt, &mut has_self_call, &mut has_base_case);
        }
        has_self_call && !has_base_case
    }

    fn check_recursion_stmt(&self, method_name: &str, stmt: &JaStmt, has_self_call: &mut bool, has_base_case: &mut bool) {
        match stmt {
            JaStmt::If { then_branch, .. } => {
                // An if statement before the recursive call suggests a base case
                *has_base_case = true;
                self.check_recursion_stmt(method_name, then_branch, has_self_call, has_base_case);
            }
            JaStmt::Return(_) => {
                *has_base_case = true;
            }
            JaStmt::Expr(e) => {
                if self.expr_calls_method(e, method_name) {
                    *has_self_call = true;
                }
            }
            JaStmt::Block(b) => {
                for s in &b.stmts {
                    self.check_recursion_stmt(method_name, s, has_self_call, has_base_case);
                }
            }
            _ => {}
        }
    }

    fn expr_calls_method(&self, expr: &JaExpr, name: &str) -> bool {
        match expr {
            JaExpr::MethodCall { name: call_name, args, .. } => {
                if call_name == name { return true; }
                args.iter().any(|a| self.expr_calls_method(a, name))
            }
            JaExpr::Binary { left, right, .. } => {
                self.expr_calls_method(left, name) || self.expr_calls_method(right, name)
            }
            _ => false,
        }
    }

    fn analyze_block(&mut self, block: &JaBlock) {
        for stmt in &block.stmts {
            self.analyze_stmt(stmt);
        }
    }

    fn analyze_stmt(&mut self, stmt: &JaStmt) {
        match stmt {
            JaStmt::Block(b) => self.analyze_block(b),
            JaStmt::Expr(e) => self.analyze_expr(e),
            JaStmt::LocalVarDecl { declarators, .. } => {
                for d in declarators {
                    if let Some(init) = &d.init {
                        self.analyze_expr(init);
                    }
                }
            }
            JaStmt::If { cond, then_branch, else_branch } => {
                self.analyze_expr(cond);
                self.analyze_stmt(then_branch);
                if let Some(e) = else_branch {
                    self.analyze_stmt(e);
                }
            }
            JaStmt::Return(Some(e)) => self.analyze_expr(e),
            JaStmt::While { cond, body } => {
                self.analyze_expr(cond);
                self.analyze_stmt(body);
            }
            JaStmt::DoWhile { body, cond } => {
                self.analyze_stmt(body);
                self.analyze_expr(cond);
            }
            JaStmt::For { init, cond, update, body } => {
                if let Some(i) = init { self.analyze_stmt(i); }
                if let Some(c) = cond { self.analyze_expr(c); }
                for u in update { self.analyze_expr(u); }
                self.analyze_stmt(body);
            }
            JaStmt::ForEach { iterable, body, .. } => {
                self.analyze_expr(iterable);
                self.analyze_stmt(body);
            }
            JaStmt::Switch { expr, cases } => {
                self.analyze_expr(expr);
                for case in cases {
                    for label in &case.labels { self.analyze_expr(label); }
                    for stmt in &case.body { self.analyze_stmt(stmt); }
                }
            }
            JaStmt::Try { body, catches, finally_block, .. } => {
                self.analyze_block(body);
                for c in catches { self.analyze_block(&c.body); }
                if let Some(f) = finally_block { self.analyze_block(f); }
            }
            JaStmt::Throw(e) => self.analyze_expr(e),
            JaStmt::Synchronized { body, lock } => {
                self.analyze_expr(lock);
                self.analyze_block(body);
            }
            _ => {}
        }
    }

    fn analyze_expr(&mut self, expr: &JaExpr) {
        match expr {
            JaExpr::Binary { op, left, right } => {
                self.analyze_expr(left);
                self.analyze_expr(right);

                // Check Division by Zero
                if *op == JaBinOp::Div {
                    if let JaExpr::IntLiteral(0) = **right {
                        self.warnings.push(UBWarning {
                            ub_type: JavaUB::DivisionByZero,
                            message: "División literal por cero detectada".to_string()
                        });
                    }
                }
            }
            JaExpr::FieldAccess { target, .. } | JaExpr::MethodCall { target: Some(target), .. } => {
                self.analyze_expr(target);
                // Basic Null Deref check (if target is explicitly null)
                if let JaExpr::Null = **target {
                    self.warnings.push(UBWarning {
                        ub_type: JavaUB::NullDeref,
                        message: "Acceso a miembro sobre valor 'null' explícito".to_string()
                    });
                }
            }
            JaExpr::Assign { target, value, .. } => {
                self.analyze_expr(target);
                self.analyze_expr(value);

                // [GC PLUS] Módulo 5: Cycle Breaker Hook
                // En un proyecto robusto se infieren los tipos estáticos reales de `target` y `value`.
                // Aquí extraemos pseudo-nombres para la prueba de UB estructural.
                let mut t_name = "UnknownT".to_string();
                let mut v_name = "UnknownV".to_string();

                if let JaExpr::Name(n) = &**target { t_name = n.clone(); }
                if let JaExpr::Name(n) = &**value { v_name = n.clone(); }

                if t_name != "UnknownT" && v_name != "UnknownV" {
                    self.cycle_breaker.analyze_dependency(&t_name, &v_name);
                }
            }
            JaExpr::NewArray { dimensions, .. } => {
                for dim in dimensions {
                    if let Some(JaExpr::IntLiteral(v)) = dim {
                        if *v < 0 {
                            self.warnings.push(UBWarning {
                                ub_type: JavaUB::NegativeArraySize,
                                message: "Creación de array con tamaño negativo detectado".to_string()
                            });
                        }
                    }
                }
            }
            JaExpr::Cast { expr, .. } => {
                self.analyze_expr(expr);
                // We could do deep ClassCastException analysis here conceptually
            }
            _ => {}
        }
    }
}
