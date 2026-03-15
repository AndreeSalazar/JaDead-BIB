// ============================================================
// GC Plus 💀☕ — Module 3: Escape Detector v2.0
// ============================================================
// Detects out-of-bounds indices, unsafe null dereferences,
// and object escape from scope at compile time.
// Two modes: --warn-escape (report) / --strict-escape (block)
// ============================================================

/// Escape detection mode
#[derive(Debug, Clone, PartialEq)]
pub enum EscapeMode {
    Warn,    // Report but allow compilation
    Strict,  // Block compilation on escape
}

/// Report from escape analysis
#[derive(Debug, Clone)]
pub struct EscapeReport {
    pub var_name: String,
    pub message: String,
    pub is_error: bool,
}

pub struct EscapeDetector {
    pub mode: EscapeMode,
    reports: Vec<EscapeReport>,
}

impl EscapeDetector {
    pub fn new() -> Self {
        Self {
            mode: EscapeMode::Strict,
            reports: Vec::new(),
        }
    }

    pub fn with_mode(mut self, mode: EscapeMode) -> Self {
        self.mode = mode;
        self
    }

    /// Evaluates if an array index is statically safe
    pub fn analyze_bounds(&self, base_var: &str, static_limit: Option<usize>, requested_idx: Option<usize>) -> bool {
        if let (Some(limit), Some(idx)) = (static_limit, requested_idx) {
            if idx >= limit {
                match self.mode {
                    EscapeMode::Strict => {
                        eprintln!("🔥 [GC+ ESCAPE v2] Index {} out of bounds for '{}' (limit: {}) — compilation blocked", idx, base_var, limit);
                        return false;
                    }
                    EscapeMode::Warn => {
                        eprintln!("⚠️ [GC+ ESCAPE v2] Index {} out of bounds for '{}' (limit: {}) — warning", idx, base_var, limit);
                        return true; // Allow but warn
                    }
                }
            }
        }
        true
    }

    /// Detect if a pointer/object might be null at a given access point
    pub fn analyze_null_access(&mut self, var_name: &str, is_known_null: bool) -> bool {
        if is_known_null {
            let is_error = self.mode == EscapeMode::Strict;
            self.reports.push(EscapeReport {
                var_name: var_name.to_string(),
                message: format!("Null dereference on '{}'", var_name),
                is_error,
            });
            if is_error {
                eprintln!("🔥 [GC+ ESCAPE v2] Null dereference on '{}' — compilation blocked", var_name);
            } else {
                eprintln!("⚠️ [GC+ ESCAPE v2] Possible null dereference on '{}'", var_name);
            }
            return false;
        }
        true
    }

    /// Detect if an object escapes its declaring scope
    pub fn analyze_scope_escape(&mut self, var_name: &str, declared_scope: u32, used_scope: u32) -> bool {
        if used_scope < declared_scope {
            // Object used in outer scope = escape
            let is_error = self.mode == EscapeMode::Strict;
            self.reports.push(EscapeReport {
                var_name: var_name.to_string(),
                message: format!("Object '{}' escapes scope {} into scope {}", var_name, declared_scope, used_scope),
                is_error,
            });
            return false;
        }
        true
    }

    pub fn reports(&self) -> &[EscapeReport] {
        &self.reports
    }

    pub fn has_errors(&self) -> bool {
        self.reports.iter().any(|r| r.is_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_safe() {
        let ed = EscapeDetector::new();
        assert!(ed.analyze_bounds("arr", Some(10), Some(5)));
    }

    #[test]
    fn test_bounds_oob_strict() {
        let ed = EscapeDetector::new();
        assert!(!ed.analyze_bounds("arr", Some(10), Some(15)));
    }

    #[test]
    fn test_bounds_oob_warn() {
        let ed = EscapeDetector::new().with_mode(EscapeMode::Warn);
        // Warn mode allows compilation
        assert!(ed.analyze_bounds("arr", Some(10), Some(15)));
    }

    #[test]
    fn test_bounds_unknown() {
        let ed = EscapeDetector::new();
        assert!(ed.analyze_bounds("arr", None, Some(5)));
        assert!(ed.analyze_bounds("arr", Some(10), None));
    }

    #[test]
    fn test_null_access_strict() {
        let mut ed = EscapeDetector::new();
        assert!(!ed.analyze_null_access("obj", true));
        assert!(ed.has_errors());
    }

    #[test]
    fn test_null_access_safe() {
        let mut ed = EscapeDetector::new();
        assert!(ed.analyze_null_access("obj", false));
        assert!(!ed.has_errors());
    }

    #[test]
    fn test_null_access_warn_mode() {
        let mut ed = EscapeDetector::new().with_mode(EscapeMode::Warn);
        assert!(!ed.analyze_null_access("obj", true));
        assert!(!ed.has_errors()); // Warn mode = no errors
    }

    #[test]
    fn test_scope_escape_detected() {
        let mut ed = EscapeDetector::new();
        // Object declared in scope 3, used in scope 1 = escape
        assert!(!ed.analyze_scope_escape("local", 3, 1));
        assert_eq!(ed.reports().len(), 1);
    }

    #[test]
    fn test_scope_no_escape() {
        let mut ed = EscapeDetector::new();
        // Object declared in scope 1, used in scope 3 = fine
        assert!(ed.analyze_scope_escape("local", 1, 3));
        assert!(ed.reports().is_empty());
    }

    #[test]
    fn test_mode_default_strict() {
        let ed = EscapeDetector::new();
        assert_eq!(ed.mode, EscapeMode::Strict);
    }
}
