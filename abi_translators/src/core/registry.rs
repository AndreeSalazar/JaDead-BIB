// ============================================================
// Translator Registry — Central dispatcher
// ============================================================
// Registers all available translators and auto-selects the
// correct one based on binary format detection.
// ============================================================

use super::translator::{ABIBTranslator, BinaryView};
use super::ir::ABIB_Module;

pub struct TranslatorRegistry {
    translators: Vec<Box<dyn ABIBTranslator>>,
}

impl TranslatorRegistry {
    pub fn new() -> Self {
        TranslatorRegistry {
            translators: Vec::new(),
        }
    }

    /// Register a translator
    pub fn register(&mut self, translator: Box<dyn ABIBTranslator>) {
        self.translators.push(translator);
    }

    /// Find the appropriate translator for a binary
    pub fn find(&self, view: &BinaryView) -> Option<&dyn ABIBTranslator> {
        for t in &self.translators {
            if t.can_handle(view) {
                return Some(t.as_ref());
            }
        }
        None
    }

    /// Auto-detect format and translate
    pub fn translate(&self, view: &BinaryView) -> Result<ABIB_Module, String> {
        let translator = self.find(view)
            .ok_or_else(|| {
                let fmt = view.detect_format();
                format!("No translator found for format {:?} (file: {})", fmt, view.filename)
            })?;

        println!("  Translator: {}", translator.name());
        translator.translate(view)
    }

    /// List all registered translators
    pub fn list(&self) -> Vec<&str> {
        self.translators.iter().map(|t| t.name()).collect()
    }

    /// Number of registered translators
    pub fn count(&self) -> usize {
        self.translators.len()
    }
}

/// Build a registry with all built-in translators
pub fn build_default_registry() -> TranslatorRegistry {
    let mut reg = TranslatorRegistry::new();

    // CPU translators
    reg.register(Box::new(crate::cpu::pe::PeTranslator::new()));
    reg.register(Box::new(crate::cpu::elf::ElfTranslator::new()));

    // GPU translators
    reg.register(Box::new(crate::gpu::spirv::SpirvTranslator::new()));
    reg.register(Box::new(crate::gpu::dxil::DxilTranslator::new()));

    reg
}
