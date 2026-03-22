// ============================================================
// Symbol Table — Shared symbol resolution for translators
// ============================================================

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub name: String,
    pub addr: u64,
    pub size: u64,
    pub is_function: bool,
    pub is_import: bool,
    pub is_export: bool,
    pub section_index: usize,
}

pub struct SymbolTable {
    by_name: HashMap<String, usize>,
    by_addr: HashMap<u64, usize>,
    entries: Vec<SymbolEntry>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            by_name: HashMap::new(),
            by_addr: HashMap::new(),
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, entry: SymbolEntry) {
        let idx = self.entries.len();
        self.by_name.insert(entry.name.clone(), idx);
        self.by_addr.insert(entry.addr, idx);
        self.entries.push(entry);
    }

    pub fn find_by_name(&self, name: &str) -> Option<&SymbolEntry> {
        self.by_name.get(name).map(|&idx| &self.entries[idx])
    }

    pub fn find_by_addr(&self, addr: u64) -> Option<&SymbolEntry> {
        self.by_addr.get(&addr).map(|&idx| &self.entries[idx])
    }

    pub fn find_function_at(&self, addr: u64) -> Option<&str> {
        self.by_addr.get(&addr)
            .map(|&idx| &self.entries[idx])
            .filter(|e| e.is_function)
            .map(|e| e.name.as_str())
    }

    pub fn all(&self) -> &[SymbolEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
