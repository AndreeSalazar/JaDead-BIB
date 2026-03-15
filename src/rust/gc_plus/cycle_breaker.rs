// ============================================================
// GC Plus 💀☕ — Module 5: Cycle Breaker
// ============================================================
// Detects structural circular references statically (e.g., A <-> B).
// Prevents memory leaks common in ARC/Reference Counting engines.
// ============================================================

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct CycleBreaker {
    pub strict_mode: bool,
    bloom_filter: [u64; 16], // 1024-bit compact graph filter - 0 Heap Usage
}

impl CycleBreaker {
    pub fn new() -> Self {
        Self {
            strict_mode: true,
            bloom_filter: [0; 16],
        }
    }

    fn calculate_edge_hash(type_a: &str, type_b: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        type_b.hash(&mut hasher); // Reverse edge to check cycles -> B->A
        type_a.hash(&mut hasher); 
        (hasher.finish() % 1024) as usize
    }

    pub fn analyze_dependency(&mut self, type_a: &str, type_b: &str) -> bool {
        let bit_index = Self::calculate_edge_hash(type_b, type_a); // Check if reverse exists
        let word_idx = bit_index / 64;
        let bit_pos = bit_index % 64;

        if (self.bloom_filter[word_idx] & (1 << bit_pos)) != 0 {
            // Potential cycle caught instantly via bitwise operation!
            if self.strict_mode {
                eprintln!("🛑 [GC+ CYCLE DETECTED] Unsafe circular reference between `{}` and `{}`.", type_a, type_b);
                eprintln!("    Recommendation: Use Weak references or redesign the dependency flow to avoid immortal memory leaks.");
            }
            return false;
        }

        // Add correct forward edge (A->B)
        let fwd_index = Self::calculate_edge_hash(type_a, type_b);
        self.bloom_filter[fwd_index / 64] |= 1 << (fwd_index % 64);
        true
    }
}
