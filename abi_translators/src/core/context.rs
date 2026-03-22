// ============================================================
// Translation Context — State during translation
// ============================================================
// Holds the module being built, current function/block,
// and provides helper methods for emitting IR.
// ============================================================

use super::ir::*;

pub struct TranslationContext {
    pub module: ABIB_Module,
    current_function_idx: Option<usize>,
    current_block_idx: Option<usize>,
    next_label_id: u32,
}

impl TranslationContext {
    pub fn new_cpu(name: &str, source: SourceFormat) -> Self {
        TranslationContext {
            module: ABIB_Module::new_cpu(name, source),
            current_function_idx: None,
            current_block_idx: None,
            next_label_id: 0,
        }
    }

    pub fn new_gpu(name: &str, source: SourceFormat) -> Self {
        TranslationContext {
            module: ABIB_Module::new_gpu(name, source),
            current_function_idx: None,
            current_block_idx: None,
            next_label_id: 0,
        }
    }

    // ---- Function management ----

    pub fn begin_function(&mut self, name: &str, addr: u64) {
        let func = ABIB_Function::new(name, addr);
        self.module.functions.push(func);
        self.current_function_idx = Some(self.module.functions.len() - 1);
        self.current_block_idx = None;
    }

    pub fn end_function(&mut self, size: u64) {
        if let Some(idx) = self.current_function_idx {
            self.module.functions[idx].size = size;
        }
        self.current_function_idx = None;
        self.current_block_idx = None;
    }

    pub fn current_function(&mut self) -> Option<&mut ABIB_Function> {
        self.current_function_idx.map(|idx| &mut self.module.functions[idx])
    }

    // ---- Block management ----

    pub fn begin_block(&mut self, label: &str, addr: u64) {
        if let Some(func_idx) = self.current_function_idx {
            let block = ABIB_Block::new(label, addr);
            self.module.functions[func_idx].blocks.push(block);
            self.current_block_idx = Some(
                self.module.functions[func_idx].blocks.len() - 1
            );
        }
    }

    pub fn auto_block(&mut self, addr: u64) {
        let label = format!("L{}", self.next_label_id);
        self.next_label_id += 1;
        self.begin_block(&label, addr);
    }

    // ---- Instruction emission ----

    pub fn emit(&mut self, inst: ABIB_Instruction) {
        if let (Some(func_idx), Some(block_idx)) =
            (self.current_function_idx, self.current_block_idx)
        {
            self.module.functions[func_idx].blocks[block_idx].emit(inst);
        }
    }

    pub fn emit_op(&mut self, opcode: Opcode, operands: Vec<Operand>, addr: u64, size: u8) {
        let inst = ABIB_Instruction {
            opcode,
            operands,
            source_addr: addr,
            source_size: size,
            raw: Vec::new(),
        };
        self.emit(inst);
    }

    pub fn emit_raw(&mut self, bytes: &[u8], addr: u64) {
        let inst = ABIB_Instruction {
            opcode: Opcode::RawBytes,
            operands: Vec::new(),
            source_addr: addr,
            source_size: bytes.len() as u8,
            raw: bytes.to_vec(),
        };
        self.emit(inst);
    }

    // ---- Import/Export/Global helpers ----

    pub fn add_import(&mut self, module: &str, symbol: &str, hint: u16, iat_addr: u64) {
        self.module.imports.push(ABIB_Import {
            module: module.to_string(),
            symbol: symbol.to_string(),
            hint,
            iat_addr,
        });
    }

    pub fn add_export(&mut self, name: &str, addr: u64, ordinal: u16) {
        self.module.exports.push(ABIB_Export {
            name: name.to_string(),
            addr,
            ordinal,
        });
    }

    pub fn add_global(&mut self, name: &str, addr: u64, size: u64, data: &[u8], readonly: bool) {
        self.module.globals.push(ABIB_Global {
            name: name.to_string(),
            addr,
            size,
            data: data.to_vec(),
            is_readonly: readonly,
        });
    }

    pub fn add_relocation(&mut self, addr: u64, reloc_type: RelocType, symbol: &str, addend: i64) {
        self.module.relocations.push(ABIB_Relocation {
            addr,
            reloc_type,
            symbol: symbol.to_string(),
            addend,
        });
    }

    /// Finalize and return the module
    pub fn finish(self) -> ABIB_Module {
        self.module
    }
}
