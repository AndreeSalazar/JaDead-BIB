// ============================================================
// GC Plus 💀☕ — Module 1: Scope Tracker
// ============================================================
// Exclusive to JaDead-BIB
// Deterministic 0-pause memory region release.
// ============================================================

pub struct ScopeTracker {
    arena_vars: Vec<String>, // Flatter arena to avoid nested Vecs and ptr chasing
    scope_pointers: Vec<usize>, 
}

impl ScopeTracker {
    pub fn new() -> Self {
        Self {
            arena_vars: Vec::with_capacity(1024), // Reserve huge block once
            scope_pointers: Vec::with_capacity(128), // Max 128 nested scopes
        }
    }

    /// Enters a new deterministic lexical block
    pub fn enter_scope(&mut self) {
        self.scope_pointers.push(self.arena_vars.len());
    }

    pub fn declare_var(&mut self, var: String) {
        self.arena_vars.push(var);
    }

    /// Exits the current active lexical block
    pub fn exit_scope(&mut self) -> Result<Vec<String>, String> {
        if let Some(start_idx) = self.scope_pointers.pop() {
            // Drain instantly from the flat arena tail
            let vars: Vec<String> = self.arena_vars.drain(start_idx..).collect();
            Ok(vars)
        } else {
            Err("GC Plus Panic: Scope Exit called but no active scopes left.".to_string())
        }
    }

    /// Returns the active scope Pointer Size for allocations to register against
    pub fn current_scope(&self) -> Option<usize> {
        self.scope_pointers.last().copied()
    }
}
