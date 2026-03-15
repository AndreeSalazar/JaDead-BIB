// ============================================================
// GC Plus 2.0 💀☕ — Unified Engine
// ============================================================
// Lazy Module Init: only activates modules the code actually needs
// Zero-Init Mode: for simple programs (no loops, no pointers, no cycles)
// Target: < 0.5ms overhead (was ~1.37ms in v1.0)
// ============================================================

use crate::gc_plus::scope_tracker::ScopeTracker;
use crate::gc_plus::loop_anticipator::LoopAnticipator;
use crate::gc_plus::escape_detector::EscapeDetector;
use crate::gc_plus::region_memory::RegionMemory;
use crate::gc_plus::cycle_breaker::CycleBreaker;

/// Feature flags detected at compile time from the Java AST
#[derive(Debug, Clone, Default)]
pub struct GCPlusFeatureFlags {
    pub has_loops: bool,
    pub has_pointers: bool,       // new Object(), arrays, etc.
    pub has_cycles_risk: bool,    // mutual field assignments
    pub has_regions: bool,        // nested scopes with allocs
    pub has_try_catch: bool,      // exception scopes need tracking
}

/// GC Plus 2.0 operating mode
#[derive(Debug, Clone, PartialEq)]
pub enum GCPlusMode {
    ZeroInit,   // No GC Plus modules needed — ~0.05ms overhead
    Minimal,    // Only ScopeTracker — ~0.1ms
    Standard,   // ScopeTracker + LoopAnticipator + EscapeDetector — ~0.3ms
    Full,       // All 5 modules — ~0.5ms
}

/// Unified GC Plus 2.0 Engine
pub struct GCPlusEngine {
    pub mode: GCPlusMode,
    pub flags: GCPlusFeatureFlags,

    // Lazy-initialized modules (Option = not allocated until needed)
    scope_tracker: Option<ScopeTracker>,
    loop_anticipator: Option<LoopAnticipator>,
    escape_detector: Option<EscapeDetector>,
    region_memory: Option<RegionMemory>,
    cycle_breaker: Option<CycleBreaker>,

    // Stats
    pub init_time_us: f64,
    pub modules_active: u8,
}

impl GCPlusEngine {
    /// Create a new GC Plus 2.0 engine based on feature flags
    pub fn new(flags: GCPlusFeatureFlags) -> Self {
        let start = std::time::Instant::now();

        let mode = Self::determine_mode(&flags);
        let mut modules_active = 0u8;

        // Lazy init: only allocate modules we actually need
        let scope_tracker = if mode != GCPlusMode::ZeroInit {
            modules_active += 1;
            Some(ScopeTracker::new())
        } else { None };

        let loop_anticipator = if flags.has_loops {
            modules_active += 1;
            Some(LoopAnticipator::new())
        } else { None };

        let escape_detector = if flags.has_pointers {
            modules_active += 1;
            Some(EscapeDetector::new())
        } else { None };

        let region_memory = if flags.has_regions || flags.has_try_catch {
            modules_active += 1;
            Some(RegionMemory::new())
        } else { None };

        let cycle_breaker = if flags.has_cycles_risk {
            modules_active += 1;
            Some(CycleBreaker::new())
        } else { None };

        let init_time_us = start.elapsed().as_secs_f64() * 1_000_000.0;

        Self {
            mode,
            flags,
            scope_tracker,
            loop_anticipator,
            escape_detector,
            region_memory,
            cycle_breaker,
            init_time_us,
            modules_active,
        }
    }

    /// Determine the operating mode from feature flags
    fn determine_mode(flags: &GCPlusFeatureFlags) -> GCPlusMode {
        if !flags.has_loops && !flags.has_pointers && !flags.has_cycles_risk
            && !flags.has_regions && !flags.has_try_catch {
            GCPlusMode::ZeroInit
        } else if !flags.has_loops && !flags.has_cycles_risk {
            GCPlusMode::Minimal
        } else if !flags.has_cycles_risk {
            GCPlusMode::Standard
        } else {
            GCPlusMode::Full
        }
    }

    // ── Scope Tracker delegates ─────────────────────────────

    pub fn enter_scope(&mut self) {
        if let Some(ref mut st) = self.scope_tracker {
            st.enter_scope();
        }
    }

    pub fn exit_scope(&mut self) -> Vec<String> {
        if let Some(ref mut st) = self.scope_tracker {
            st.exit_scope().unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub fn declare_var(&mut self, var: String) {
        if let Some(ref mut st) = self.scope_tracker {
            st.declare_var(var);
        }
    }

    // ── Loop Anticipator delegates ──────────────────────────

    pub fn enter_loop(&mut self) -> Option<String> {
        self.loop_anticipator.as_mut().map(|la| la.enter_loop())
    }

    pub fn exit_loop(&mut self) -> Option<String> {
        self.loop_anticipator.as_mut().and_then(|la| la.exit_loop().ok())
    }

    pub fn current_pool(&self) -> Option<String> {
        self.loop_anticipator.as_ref().and_then(|la| la.current_pool())
    }

    // ── Escape Detector delegates ───────────────────────────

    pub fn check_bounds(&self, var: &str, limit: Option<usize>, idx: Option<usize>) -> bool {
        if let Some(ref ed) = self.escape_detector {
            ed.analyze_bounds(var, limit, idx)
        } else {
            true // No detector = always safe
        }
    }

    // ── Region Memory delegates ─────────────────────────────

    pub fn define_region(&mut self, hint: &str) -> Option<u32> {
        self.region_memory.as_mut().map(|rm| rm.define_region(hint))
    }

    pub fn free_region(&mut self, id: u32) {
        if let Some(ref mut rm) = self.region_memory {
            rm.free_region(id);
        }
    }

    // ── Cycle Breaker delegates ─────────────────────────────

    pub fn analyze_dependency(&mut self, type_a: &str, type_b: &str) -> bool {
        if let Some(ref mut cb) = self.cycle_breaker {
            cb.analyze_dependency(type_a, type_b)
        } else {
            true // No breaker = no cycle detected
        }
    }

    // ── Stats ───────────────────────────────────────────────

    pub fn summary(&self) -> String {
        format!(
            "[GC Plus 2.0] mode={:?} | modules={}/5 | init={:.1}µs",
            self.mode, self.modules_active, self.init_time_us
        )
    }
}

/// Analyze a Java AST to determine GC Plus feature flags
pub fn detect_features_from_ast(ast: &crate::frontend::java::ja_ast::JaCompilationUnit) -> GCPlusFeatureFlags {
    let mut flags = GCPlusFeatureFlags::default();

    for decl in &ast.declarations {
        scan_type_decl(decl, &mut flags);
    }
    flags
}

fn scan_type_decl(decl: &crate::frontend::java::ja_ast::JaTypeDecl, flags: &mut GCPlusFeatureFlags) {
    use crate::frontend::java::ja_ast::*;
    let body = match decl {
        JaTypeDecl::Class { body, .. } => body,
        JaTypeDecl::Record { body, .. } => body,
        JaTypeDecl::Enum { body, .. } => body,
        JaTypeDecl::Interface { body, .. } => body,
    };
    for member in body {
        match member {
            JaClassMember::Method { body: Some(block), .. } |
            JaClassMember::Constructor { body: block, .. } |
            JaClassMember::Initializer(block, _) => {
                scan_block(block, flags);
            }
            _ => {}
        }
    }
}

fn scan_block(block: &crate::frontend::java::ja_ast::JaBlock, flags: &mut GCPlusFeatureFlags) {
    for stmt in &block.stmts {
        scan_stmt(stmt, flags);
    }
}

fn scan_stmt(stmt: &crate::frontend::java::ja_ast::JaStmt, flags: &mut GCPlusFeatureFlags) {
    use crate::frontend::java::ja_ast::*;
    match stmt {
        JaStmt::For { body, .. } | JaStmt::While { body, .. } => {
            flags.has_loops = true;
            scan_stmt(body, flags);
        }
        JaStmt::DoWhile { body, .. } => {
            flags.has_loops = true;
            scan_stmt(body, flags);
        }
        JaStmt::ForEach { body, .. } => {
            flags.has_loops = true;
            scan_stmt(body, flags);
        }
        JaStmt::Try { body, catches, finally_block, .. } => {
            flags.has_try_catch = true;
            flags.has_regions = true;
            scan_block(body, flags);
            for c in catches { scan_block(&c.body, flags); }
            if let Some(f) = finally_block { scan_block(f, flags); }
        }
        JaStmt::Block(b) => scan_block(b, flags),
        JaStmt::If { then_branch, else_branch, .. } => {
            scan_stmt(then_branch, flags);
            if let Some(e) = else_branch { scan_stmt(e, flags); }
        }
        JaStmt::Expr(e) => scan_expr(e, flags),
        JaStmt::LocalVarDecl { declarators, .. } => {
            for d in declarators {
                if let Some(init) = &d.init {
                    scan_expr(init, flags);
                }
            }
        }
        JaStmt::Switch { cases, .. } => {
            for case in cases {
                for s in &case.body { scan_stmt(s, flags); }
            }
        }
        JaStmt::Synchronized { body, .. } => {
            flags.has_regions = true;
            scan_block(body, flags);
        }
        _ => {}
    }
}

fn scan_expr(expr: &crate::frontend::java::ja_ast::JaExpr, flags: &mut GCPlusFeatureFlags) {
    use crate::frontend::java::ja_ast::*;
    match expr {
        JaExpr::NewObject { .. } => {
            flags.has_pointers = true;
        }
        JaExpr::NewArray { .. } => {
            flags.has_pointers = true;
        }
        JaExpr::Assign { target, value, .. } => {
            // Field-to-field assignments between objects may create cycles
            if matches!(**target, JaExpr::FieldAccess { .. }) &&
               matches!(**value, JaExpr::FieldAccess { .. } | JaExpr::Name(_)) {
                flags.has_cycles_risk = true;
            }
            scan_expr(target, flags);
            scan_expr(value, flags);
        }
        JaExpr::MethodCall { target: Some(t), args, .. } => {
            scan_expr(t, flags);
            for a in args { scan_expr(a, flags); }
        }
        JaExpr::MethodCall { args, .. } => {
            for a in args { scan_expr(a, flags); }
        }
        JaExpr::Binary { left, right, .. } => {
            scan_expr(left, flags);
            scan_expr(right, flags);
        }
        JaExpr::Unary { expr, .. } => scan_expr(expr, flags),
        JaExpr::Ternary { cond, true_expr, false_expr } => {
            scan_expr(cond, flags);
            scan_expr(true_expr, flags);
            scan_expr(false_expr, flags);
        }
        JaExpr::Cast { expr, .. } => scan_expr(expr, flags),
        JaExpr::ArrayAccess { array, index } => {
            scan_expr(array, flags);
            scan_expr(index, flags);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_init_mode() {
        let flags = GCPlusFeatureFlags::default();
        let engine = GCPlusEngine::new(flags);
        assert_eq!(engine.mode, GCPlusMode::ZeroInit);
        assert_eq!(engine.modules_active, 0);
    }

    #[test]
    fn test_minimal_mode() {
        let flags = GCPlusFeatureFlags {
            has_pointers: true,
            ..Default::default()
        };
        let engine = GCPlusEngine::new(flags);
        assert_eq!(engine.mode, GCPlusMode::Minimal);
        assert_eq!(engine.modules_active, 2); // scope_tracker + escape_detector
    }

    #[test]
    fn test_standard_mode() {
        let flags = GCPlusFeatureFlags {
            has_loops: true,
            has_pointers: true,
            ..Default::default()
        };
        let engine = GCPlusEngine::new(flags);
        assert_eq!(engine.mode, GCPlusMode::Standard);
        assert!(engine.modules_active >= 3);
    }

    #[test]
    fn test_full_mode() {
        let flags = GCPlusFeatureFlags {
            has_loops: true,
            has_pointers: true,
            has_cycles_risk: true,
            has_regions: true,
            has_try_catch: true,
        };
        let engine = GCPlusEngine::new(flags);
        assert_eq!(engine.mode, GCPlusMode::Full);
        assert_eq!(engine.modules_active, 5);
    }

    #[test]
    fn test_lazy_scope_operations() {
        let flags = GCPlusFeatureFlags {
            has_pointers: true,
            ..Default::default()
        };
        let mut engine = GCPlusEngine::new(flags);
        engine.enter_scope();
        engine.declare_var("x".to_string());
        engine.declare_var("y".to_string());
        let freed = engine.exit_scope();
        assert_eq!(freed.len(), 2);
        assert_eq!(freed[0], "x");
        assert_eq!(freed[1], "y");
    }

    #[test]
    fn test_lazy_loop_operations() {
        let flags = GCPlusFeatureFlags {
            has_loops: true,
            ..Default::default()
        };
        let mut engine = GCPlusEngine::new(flags);
        let pool = engine.enter_loop();
        assert!(pool.is_some());
        assert!(pool.unwrap().starts_with("__gc_loop_pool_"));
        let freed_pool = engine.exit_loop();
        assert!(freed_pool.is_some());
    }

    #[test]
    fn test_no_loop_module_when_no_loops() {
        let flags = GCPlusFeatureFlags::default();
        let mut engine = GCPlusEngine::new(flags);
        let pool = engine.enter_loop();
        assert!(pool.is_none());
    }

    #[test]
    fn test_cycle_breaker_lazy() {
        let flags = GCPlusFeatureFlags {
            has_cycles_risk: true,
            ..Default::default()
        };
        let mut engine = GCPlusEngine::new(flags);
        // First dependency A->B should be fine
        assert!(engine.analyze_dependency("A", "B"));
        // Reverse dependency B->A should detect cycle
        assert!(!engine.analyze_dependency("B", "A"));
    }

    #[test]
    fn test_engine_summary() {
        let engine = GCPlusEngine::new(GCPlusFeatureFlags::default());
        let summary = engine.summary();
        assert!(summary.contains("GC Plus 2.0"));
        assert!(summary.contains("ZeroInit"));
    }
}
