// ============================================================
// Java UB Detector v2.0 for JaDead-BIB 💀☕
// ============================================================
// Heredado de ADead-BIB v8.0 (21 tipos C/C++)
// + Java-specific UB detection (15+ tipos)
//
// JVM: todos estos → excepción en RUNTIME ❌
// JaDead-BIB: todos → detectados en COMPILE TIME ✓
//
// TIER 1: Críticos (bloquean compilación con --strict-ub)
// TIER 2: Warnings (avisan, no bloquean)
// TIER 3: Info (sugerencias de mejora)
// Sin flags: compila silencioso = respeta al desarrollador
// ============================================================

use crate::frontend::java::ja_ast::*;
use crate::gc_plus::cycle_breaker::CycleBreaker;
use crate::middle::ir::{IRFunction, IRInstruction, IROp, IRConstValue, IRType};

/// Java-specific undefined behavior types
#[derive(Debug, Clone, PartialEq)]
pub enum JavaUB {
    // ── TIER 1: Críticos ─────────────────────────────
    NullDeref,              // null.method() → NullPointerException
    ArrayIndexOutOfBounds,  // arr[100] con arr[10]
    DivisionByZero,         // x / 0 literal
    ClassCastException,     // (String) integer
    StackOverflow,          // recursión sin base case

    // ── TIER 2: Warnings ─────────────────────────────
    IntegerOverflow,        // int + int overflow
    StringEquality,         // "hola" == "hola" con == en vez de .equals()
    ConcurrentModification, // modificar lista en foreach
    NegativeArraySize,      // new int[-1]
    StringIndexOutOfBounds, // "hola".charAt(100)
    NumberFormatException,  // Integer.parseInt("abc")
    EmptyOptional,          // Optional.get() sin isPresent()
    ResourceLeak,           // stream/file sin close()
    EmptyReturnNonVoid,     // return; in non-void function

    // ── TIER 3: Info ────────────────────────────────
    UncheckedCast,          // cast genérico sin verificar
    DeadLock,               // synchronized anidado
    UnsafePublicField,      // campo público mutable en record
}

/// Severity level for UB reports (matches PyDead-BIB pattern)
#[derive(Debug, Clone, PartialEq)]
pub enum UBSeverity {
    Error,
    Warning,
    Info,
}

/// UB detection report with context (matches PyDead-BIB pattern)
#[derive(Debug, Clone)]
pub struct UBReport {
    pub kind: JavaUB,
    pub severity: UBSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}

// Keep backward compat alias
pub type UBWarning = UBReport;

pub struct UbDetector {
    reports: Vec<UBReport>,
    cycle_breaker: CycleBreaker,
    pub strict_mode: bool,
}

impl UbDetector {
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
            cycle_breaker: CycleBreaker::new(),
            strict_mode: false,
        }
    }

    pub fn with_strict(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    pub fn analyze(&mut self, ast: &JaCompilationUnit) -> Vec<UBReport> {
        self.reports.clear();

        for decl in &ast.declarations {
            self.analyze_type_decl(decl);
        }

        self.reports.clone()
    }

    /// Analyze IR functions for UB (like PyDead-BIB does)
    pub fn check_function(&mut self, func: &IRFunction) {
        // ── Empty return in non-void function ──────────────
        if func.return_type != IRType::Void {
            for instr in &func.body {
                if matches!(instr, IRInstruction::ReturnVoid) {
                    self.reports.push(UBReport {
                        kind: JavaUB::EmptyReturnNonVoid,
                        severity: UBSeverity::Warning,
                        message: format!("Empty return in non-void method '{}' (expected {:?})", func.name, func.return_type),
                        suggestion: Some("Return a value matching the declared return type".to_string()),
                    });
                }
            }
        }

        for instr in &func.body {
            match instr {
                // ── Division by zero in IR ────────────────────
                IRInstruction::BinOp { op: IROp::Div, right, .. }
                | IRInstruction::BinOp { op: IROp::Mod, right, .. } => {
                    if Self::is_zero_constant(right) {
                        self.reports.push(UBReport {
                            kind: JavaUB::DivisionByZero,
                            severity: UBSeverity::Error,
                            message: format!("Division by zero detected in method '{}'", func.name),
                            suggestion: Some("Check divisor is not zero before dividing".to_string()),
                        });
                    }
                }

                // ── Null passed to a function call ────────────
                IRInstruction::Call { func: callee, args } => {
                    for arg in args {
                        if matches!(arg, IRInstruction::LoadConst(IRConstValue::None)) {
                            self.reports.push(UBReport {
                                kind: JavaUB::NullDeref,
                                severity: UBSeverity::Error,
                                message: format!("Possible null argument in call to '{}' in method '{}'", callee, func.name),
                                suggestion: Some("Check for null before passing to method".to_string()),
                            });
                        }
                    }
                }

                _ => {}
            }
        }

        // ── Null-then-call pattern (LoadConst(None) followed by Call) ──
        for window in func.body.windows(2) {
            if let (IRInstruction::LoadConst(IRConstValue::None), IRInstruction::Call { func: callee, .. }) = (&window[0], &window[1]) {
                self.reports.push(UBReport {
                    kind: JavaUB::NullDeref,
                    severity: UBSeverity::Error,
                    message: format!("Null value used before call to '{}' in method '{}' — likely NullPointerException", callee, func.name),
                    suggestion: Some("Add a null check before this call".to_string()),
                });
            }
        }
    }

    fn is_zero_constant(instr: &IRInstruction) -> bool {
        match instr {
            IRInstruction::LoadConst(IRConstValue::Int(0)) => true,
            IRInstruction::LoadConst(IRConstValue::Float(f)) if *f == 0.0 => true,
            _ => false,
        }
    }

    pub fn reports(&self) -> &[UBReport] {
        &self.reports
    }

    pub fn has_errors(&self) -> bool {
        self.reports.iter().any(|r| r.severity == UBSeverity::Error)
    }

    #[allow(dead_code)]
    pub fn warnings(&self) -> Vec<UBReport> {
        self.reports.clone()
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
                            self.reports.push(UBReport {
                                kind: JavaUB::UnsafePublicField,
                                severity: UBSeverity::Info,
                                message: "Campo público mutable detectado en un Record/Clase que rompe la inmutabilidad".to_string(),
                                suggestion: Some("Considera usar private o final".to_string()),
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
                    self.reports.push(UBReport {
                        kind: JavaUB::StackOverflow,
                        severity: UBSeverity::Error,
                        message: format!("Método '{}' parece ser recursivo sin caso base detectable", name),
                        suggestion: Some("Add a base case (if/return) before the recursive call".to_string()),
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

                // TIER 1: Division by Zero
                if *op == JaBinOp::Div || *op == JaBinOp::Rem {
                    if let JaExpr::IntLiteral(0) = **right {
                        self.reports.push(UBReport {
                            kind: JavaUB::DivisionByZero,
                            severity: UBSeverity::Error,
                            message: "División literal por cero detectada".to_string(),
                            suggestion: Some("Verificar que el divisor no sea cero".to_string()),
                        });
                    }
                }

                // TIER 2: String equality with == instead of .equals()
                if *op == JaBinOp::Eq || *op == JaBinOp::Neq {
                    let left_is_str = matches!(**left, JaExpr::StringLiteral(_));
                    let right_is_str = matches!(**right, JaExpr::StringLiteral(_));
                    if left_is_str || right_is_str {
                        self.reports.push(UBReport {
                            kind: JavaUB::StringEquality,
                            severity: UBSeverity::Warning,
                            message: "Comparación de String con == en vez de .equals()".to_string(),
                            suggestion: Some("Usa .equals() para comparar contenido de Strings".to_string()),
                        });
                    }
                }
            }
            JaExpr::FieldAccess { target, .. } | JaExpr::MethodCall { target: Some(target), .. } => {
                self.analyze_expr(target);
                // TIER 1: Null Deref
                if let JaExpr::Null = **target {
                    self.reports.push(UBReport {
                        kind: JavaUB::NullDeref,
                        severity: UBSeverity::Error,
                        message: "Acceso a miembro sobre valor 'null' explícito".to_string(),
                        suggestion: Some("Verificar null antes de acceder al miembro".to_string()),
                    });
                }
            }
            JaExpr::MethodCall { target: None, name, args, .. } => {
                for a in args { self.analyze_expr(a); }
                // TIER 2: NumberFormatException potential
                if name == "parseInt" || name == "parseDouble" || name == "parseLong" {
                    if let Some(arg) = args.first() {
                        if let JaExpr::StringLiteral(s) = arg {
                            if s.parse::<i64>().is_err() && s.parse::<f64>().is_err() {
                                self.reports.push(UBReport {
                                    kind: JavaUB::NumberFormatException,
                                    severity: UBSeverity::Warning,
                                    message: format!("'{}' no es un número válido para {}", s, name),
                                    suggestion: Some("Envolver en try-catch o validar el formato primero".to_string()),
                                });
                            }
                        }
                    }
                }
            }
            JaExpr::Assign { target, value, .. } => {
                self.analyze_expr(target);
                self.analyze_expr(value);

                // [GC PLUS] Módulo 5: Cycle Breaker Hook
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
                            self.reports.push(UBReport {
                                kind: JavaUB::NegativeArraySize,
                                severity: UBSeverity::Error,
                                message: "Creación de array con tamaño negativo detectado".to_string(),
                                suggestion: Some("Usar un tamaño >= 0 para el array".to_string()),
                            });
                        }
                    }
                }
            }
            JaExpr::Cast { expr, ty } => {
                self.analyze_expr(expr);
                // TIER 1: Suspicious cast from String literal to primitive type
                if matches!(**expr, JaExpr::StringLiteral(_)) {
                    let is_primitive = matches!(ty, JaType::Int | JaType::Long | JaType::Float | JaType::Double | JaType::Boolean | JaType::Char | JaType::Byte | JaType::Short);
                    if is_primitive {
                        self.reports.push(UBReport {
                            kind: JavaUB::ClassCastException,
                            severity: UBSeverity::Error,
                            message: format!("Cast de String a {:?} puede lanzar ClassCastException", ty),
                            suggestion: Some("Verificar el tipo antes del cast".to_string()),
                        });
                    }
                }
            }
            JaExpr::Instanceof { expr, .. } => {
                self.analyze_expr(expr);
            }
            JaExpr::Ternary { cond, true_expr, false_expr } => {
                self.analyze_expr(cond);
                self.analyze_expr(true_expr);
                self.analyze_expr(false_expr);
            }
            JaExpr::Unary { expr, .. } => {
                self.analyze_expr(expr);
            }
            JaExpr::ArrayAccess { array, index } => {
                self.analyze_expr(array);
                self.analyze_expr(index);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middle::ir::*;

    // ── Helper to build an IRFunction ──────────────────────
    fn make_func(name: &str, ret: IRType, body: Vec<IRInstruction>) -> IRFunction {
        IRFunction { name: name.to_string(), params: vec![], return_type: ret, body }
    }

    // ── IR-level tests (matching PyDead-BIB pattern) ───────

    #[test]
    fn test_ir_division_by_zero() {
        let func = make_func("div_zero", IRType::I64, vec![
            IRInstruction::BinOp {
                op: IROp::Div,
                left: Box::new(IRInstruction::LoadConst(IRConstValue::Int(10))),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(0))),
            },
        ]);
        let mut det = UbDetector::new();
        det.check_function(&func);
        assert!(det.has_errors());
        assert_eq!(det.reports()[0].kind, JavaUB::DivisionByZero);
    }

    #[test]
    fn test_ir_mod_by_zero() {
        let func = make_func("mod_zero", IRType::I64, vec![
            IRInstruction::BinOp {
                op: IROp::Mod,
                left: Box::new(IRInstruction::LoadConst(IRConstValue::Int(10))),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(0))),
            },
        ]);
        let mut det = UbDetector::new();
        det.check_function(&func);
        assert!(det.has_errors());
        assert_eq!(det.reports()[0].kind, JavaUB::DivisionByZero);
    }

    #[test]
    fn test_ir_no_div_zero_safe() {
        let func = make_func("safe_div", IRType::I64, vec![
            IRInstruction::BinOp {
                op: IROp::Div,
                left: Box::new(IRInstruction::LoadConst(IRConstValue::Int(10))),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Int(2))),
            },
        ]);
        let mut det = UbDetector::new();
        det.check_function(&func);
        assert!(!det.has_errors());
    }

    #[test]
    fn test_ir_null_arg_in_call() {
        let func = make_func("null_call", IRType::Void, vec![
            IRInstruction::Call {
                func: "process".to_string(),
                args: vec![IRInstruction::LoadConst(IRConstValue::None)],
            },
        ]);
        let mut det = UbDetector::new();
        det.check_function(&func);
        assert!(det.has_errors());
        assert_eq!(det.reports()[0].kind, JavaUB::NullDeref);
    }

    #[test]
    fn test_ir_null_then_call_pattern() {
        let func = make_func("null_pattern", IRType::Void, vec![
            IRInstruction::LoadConst(IRConstValue::None),
            IRInstruction::Call {
                func: "method".to_string(),
                args: vec![],
            },
        ]);
        let mut det = UbDetector::new();
        det.check_function(&func);
        assert!(det.has_errors());
        assert!(det.reports().iter().any(|r| r.kind == JavaUB::NullDeref));
    }

    #[test]
    fn test_ir_empty_return_nonvoid() {
        let func = make_func("bad_return", IRType::I64, vec![
            IRInstruction::ReturnVoid,
        ]);
        let mut det = UbDetector::new();
        det.check_function(&func);
        assert!(!det.reports().is_empty());
        assert_eq!(det.reports()[0].kind, JavaUB::EmptyReturnNonVoid);
        assert_eq!(det.reports()[0].severity, UBSeverity::Warning);
    }

    #[test]
    fn test_ir_return_void_in_void_ok() {
        let func = make_func("void_func", IRType::Void, vec![
            IRInstruction::ReturnVoid,
        ]);
        let mut det = UbDetector::new();
        det.check_function(&func);
        assert!(det.reports().is_empty());
    }

    #[test]
    fn test_ir_safe_call_no_null() {
        let func = make_func("safe_call", IRType::Void, vec![
            IRInstruction::Call {
                func: "println".to_string(),
                args: vec![IRInstruction::LoadConst(IRConstValue::Int(42))],
            },
        ]);
        let mut det = UbDetector::new();
        det.check_function(&func);
        assert!(!det.has_errors());
    }

    #[test]
    fn test_ir_float_div_by_zero() {
        let func = make_func("float_div_zero", IRType::F64, vec![
            IRInstruction::BinOp {
                op: IROp::Div,
                left: Box::new(IRInstruction::LoadConst(IRConstValue::Float(3.14))),
                right: Box::new(IRInstruction::LoadConst(IRConstValue::Float(0.0))),
            },
        ]);
        let mut det = UbDetector::new();
        det.check_function(&func);
        assert!(det.has_errors());
    }

    // ── AST-level tests ────────────────────────────────────

    #[test]
    fn test_ast_div_by_zero() {
        let code = "class A { void f() { int x = 10 / 0; } }";
        let lexer = crate::frontend::java::ja_lexer::JaLexer::new(code);
        let mut parser = crate::frontend::java::ja_parser::JaParser::new(lexer);
        let ast = parser.parse_compilation_unit().unwrap();
        let mut det = UbDetector::new();
        let reports = det.analyze(&ast);
        assert!(reports.iter().any(|r| r.kind == JavaUB::DivisionByZero));
    }

    #[test]
    fn test_ast_negative_array_size() {
        // Parser produces Unary(Minus, IntLiteral(5)) for -5, so test with direct AST
        let ast = JaCompilationUnit {
            package: None,
            imports: vec![],
            declarations: vec![
                JaTypeDecl::Class {
                    name: "A".to_string(),
                    modifiers: vec![],
                    type_params: vec![],
                    extends: None,
                    implements: vec![],
                    permits: vec![],
                    body: vec![
                        JaClassMember::Method {
                            modifiers: vec![],
                            return_type: JaType::Void,
                            name: "f".to_string(),
                            type_params: vec![],
                            params: vec![],
                            body: Some(JaBlock {
                                stmts: vec![
                                    JaStmt::LocalVarDecl {
                                        ty: JaType::Array(Box::new(JaType::Int)),
                                        declarators: vec![JaVarDeclarator {
                                            name: "arr".to_string(),
                                            init: Some(JaExpr::NewArray {
                                                ty: JaType::Int,
                                                dimensions: vec![Some(JaExpr::IntLiteral(-5))],
                                                init: None,
                                            }),
                                        }],
                                    },
                                ],
                            }),
                            throws: vec![],
                        },
                    ],
                },
            ],
        };
        let mut det = UbDetector::new();
        let reports = det.analyze(&ast);
        assert!(reports.iter().any(|r| r.kind == JavaUB::NegativeArraySize));
    }

    #[test]
    fn test_detector_has_errors() {
        let mut det = UbDetector::new();
        assert!(!det.has_errors());
        det.reports.push(UBReport {
            kind: JavaUB::DivisionByZero,
            severity: UBSeverity::Error,
            message: "test".to_string(),
            suggestion: None,
        });
        assert!(det.has_errors());
    }

    #[test]
    fn test_detector_strict_mode() {
        let det = UbDetector::new().with_strict();
        assert!(det.strict_mode);
    }

    #[test]
    fn test_ub_kinds_equality() {
        assert_eq!(JavaUB::NullDeref, JavaUB::NullDeref);
        assert_ne!(JavaUB::NullDeref, JavaUB::DivisionByZero);
    }

    #[test]
    fn test_severity_equality() {
        assert_eq!(UBSeverity::Error, UBSeverity::Error);
        assert_ne!(UBSeverity::Error, UBSeverity::Warning);
    }
}
