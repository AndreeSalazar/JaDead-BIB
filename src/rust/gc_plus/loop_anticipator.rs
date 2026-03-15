// ============================================================
// GC Plus 💀☕ — Module 2: Loop Anticipator
// ============================================================
// Exclusive to JaDead-BIB
// Detects iterative allocations like `for() { new X(); }`
// and transforms them into an Object Pool statically injected
// before the loop starts.
// ============================================================

pub struct LoopAnticipator {
    active_pools_ids: [u32; 128], // Inline stack Array, no `Vec<String>`
    head_ptr: usize,
    pool_counter: u32,
}

impl LoopAnticipator {
    pub fn new() -> Self {
        Self {
            active_pools_ids: [0; 128],
            head_ptr: 0,
            pool_counter: 0,
        }
    }

    /// Registers that an iteration block is beginning
    pub fn enter_loop(&mut self) -> String {
        self.pool_counter += 1;
        if self.head_ptr < 128 {
            self.active_pools_ids[self.head_ptr] = self.pool_counter;
            self.head_ptr += 1;
        }
        format!("__gc_loop_pool_{}", self.pool_counter) // String formatted only strictly required by IR emitter
    }

    /// Exits the loop boundary and schedules the complete pool destruction
    pub fn exit_loop(&mut self) -> Result<String, String> {
        if self.head_ptr > 0 {
            self.head_ptr -= 1;
            let pool_id = self.active_pools_ids[self.head_ptr];
            Ok(format!("__gc_loop_pool_{}", pool_id))
        } else {
            Err("GC Plus Panic: Loop exit called without an active loop pool.".to_string())
        }
    }

    /// Returns the currently active iteration pool to transform internal allocs
    pub fn current_pool(&self) -> Option<String> {
        if self.head_ptr > 0 {
            let pool_id = self.active_pools_ids[self.head_ptr - 1];
            Some(format!("__gc_loop_pool_{}", pool_id))
        } else {
            None
        }
    }
}
