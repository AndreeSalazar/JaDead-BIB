// ============================================================
// Translator Interface — Base trait for all ABI translators
// ============================================================
// Every translator (PE, ELF, SPIR-V, DXIL) implements this.
// ============================================================

use super::ir::{ABIB_Module, SourceFormat};

/// Binary view — raw bytes + metadata for a binary file
#[derive(Debug)]
pub struct BinaryView {
    pub data: Vec<u8>,
    pub filename: String,
    pub size: usize,
}

impl BinaryView {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let data = std::fs::read(path)
            .map_err(|e| format!("Failed to read '{}': {}", path, e))?;
        let size = data.len();
        Ok(BinaryView {
            data,
            filename: path.to_string(),
            size,
        })
    }

    pub fn from_bytes(data: Vec<u8>, filename: &str) -> Self {
        let size = data.len();
        BinaryView {
            data,
            filename: filename.to_string(),
            size,
        }
    }

    /// Read u16 little-endian at offset
    pub fn read_u16(&self, offset: usize) -> Option<u16> {
        if offset + 2 > self.size { return None; }
        Some(u16::from_le_bytes([self.data[offset], self.data[offset + 1]]))
    }

    /// Read u32 little-endian at offset
    pub fn read_u32(&self, offset: usize) -> Option<u32> {
        if offset + 4 > self.size { return None; }
        Some(u32::from_le_bytes([
            self.data[offset], self.data[offset+1],
            self.data[offset+2], self.data[offset+3],
        ]))
    }

    /// Read u64 little-endian at offset
    pub fn read_u64(&self, offset: usize) -> Option<u64> {
        if offset + 8 > self.size { return None; }
        Some(u64::from_le_bytes(self.data[offset..offset+8].try_into().unwrap()))
    }

    /// Read a null-terminated string at offset
    pub fn read_cstring(&self, offset: usize) -> Option<String> {
        if offset >= self.size { return None; }
        let end = self.data[offset..].iter().position(|&b| b == 0)?;
        String::from_utf8(self.data[offset..offset + end].to_vec()).ok()
    }

    /// Read a slice of bytes
    pub fn read_bytes(&self, offset: usize, len: usize) -> Option<&[u8]> {
        if offset + len > self.size { return None; }
        Some(&self.data[offset..offset + len])
    }

    /// Detect the format of this binary
    pub fn detect_format(&self) -> SourceFormat {
        if self.size < 4 { return SourceFormat::Unknown; }

        // PE: starts with "MZ"
        if self.data[0] == 0x4D && self.data[1] == 0x5A {
            return SourceFormat::PE;
        }

        // ELF: starts with 0x7F "ELF"
        if self.data[0] == 0x7F && self.data[1] == b'E'
            && self.data[2] == b'L' && self.data[3] == b'F' {
            return SourceFormat::ELF;
        }

        // Mach-O: 0xFEEDFACE or 0xFEEDFACF
        if self.size >= 4 {
            let magic = u32::from_le_bytes([self.data[0], self.data[1], self.data[2], self.data[3]]);
            if magic == 0xFEEDFACE || magic == 0xFEEDFACF
                || magic == 0xCEFAEDFE || magic == 0xCFFAEDFE {
                return SourceFormat::MachO;
            }
        }

        // SPIR-V: magic 0x07230203
        if self.size >= 4 {
            let magic = u32::from_le_bytes([self.data[0], self.data[1], self.data[2], self.data[3]]);
            if magic == 0x07230203 {
                return SourceFormat::SPIRV;
            }
        }

        // DXIL: starts with "DXBC"
        if self.size >= 4 && &self.data[0..4] == b"DXBC" {
            return SourceFormat::DXIL;
        }

        SourceFormat::Unknown
    }
}

/// The universal translator trait
pub trait ABIBTranslator {
    /// Name of this translator (e.g. "PE Translator", "SPIR-V Translator")
    fn name(&self) -> &str;

    /// Source format this translator handles
    fn source_format(&self) -> SourceFormat;

    /// Check if this translator can handle the given binary
    fn can_handle(&self, view: &BinaryView) -> bool;

    /// Translate the binary into an ABIB_Module
    fn translate(&self, view: &BinaryView) -> Result<ABIB_Module, String>;
}
