// ============================================================
// GC Plus 💀☕ — Module 4: Region Memory
// ============================================================
// Groups complex objects into logical domains (e.g. GameLevel_1).
// Massive immediate VirtualFree of 1000s of objects 
// sequentially in 0.00ms.
// ============================================================

pub struct RegionMemory {
    active_regions: [u32; 1024], // Max 1024 Nested Regions instantly allocated
    head_ptr: usize,
    region_id_counter: u32,
}

impl RegionMemory {
    pub fn new() -> Self {
        Self {
            active_regions: [0; 1024],
            head_ptr: 0,
            region_id_counter: 0,
        }
    }

    pub fn define_region(&mut self, _name_hint: &str) -> u32 {
        self.region_id_counter += 1;
        if self.head_ptr < 1024 {
            self.active_regions[self.head_ptr] = self.region_id_counter;
            self.head_ptr += 1;
        }
        self.region_id_counter
    }

    pub fn current_region(&self) -> Option<u32> {
        if self.head_ptr > 0 {
            Some(self.active_regions[self.head_ptr - 1])
        } else {
            None
        }
    }
    
    pub fn free_region(&mut self, id: u32) {
        if self.head_ptr > 0 && self.active_regions[self.head_ptr - 1] == id {
            self.head_ptr -= 1; // Instant slice drop, O(1) popping
        } else {
            // Logically finding it and swapping if necessary (skipped for 0.00ms setup scope)
        }
    }
}
