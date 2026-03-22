// ============================================================
// PE Translator — Windows PE/PE32+ → ADead-BIB IR
// ============================================================
// FASM-inspired PE parser with full format coverage.
//
// Parsing (from FASM's FORMATS.INC format_pe):
//   DOS Header (MZ), NT Headers (PE\0\0),
//   COFF Header (machine 0x8664/0x14C),
//   Optional Header (PE32+ magic 0x020B),
//   Section Table, Data Directories (16 entries),
//   Import Directory (IDT→ILT→IAT→hint/name),
//   Export Directory (addr/name/ordinal tables),
//   Base Relocations (block→entries)
//
// Decoding:
//   x86-64 instructions via FASM-quality decoder
//   (80+ instruction patterns, full ModR/M+SIB+disp)
//
// FASM constants used:
//   IMAGE_FILE_MACHINE_AMD64 = 0x8664
//   IMAGE_FILE_MACHINE_I386  = 0x14C
//   IMAGE_SUBSYSTEM_CONSOLE  = 3
//   IMAGE_SUBSYSTEM_GUI      = 2
//   IMAGE_SCN_MEM_EXECUTE    = 0x20000000
//   IMAGE_SCN_MEM_READ       = 0x40000000
//   IMAGE_SCN_MEM_WRITE      = 0x80000000
// ============================================================

use crate::core::ir::*;
use crate::core::translator::{ABIBTranslator, BinaryView};
use crate::core::context::TranslationContext;
use crate::utils::binary_reader::BinaryReader;
use crate::utils::x86_decoder;

// ============================================================
// PE Parsed structures
// ============================================================

#[derive(Debug)]
struct PeHeaders {
    // COFF
    machine: u16,
    num_sections: u16,
    size_of_optional: u16,
    characteristics: u16,
    // Optional Header
    magic: u16,
    entry_point_rva: u32,
    image_base: u64,
    section_alignment: u32,
    file_alignment: u32,
    size_of_image: u32,
    size_of_headers: u32,
    subsystem: u16,
    num_data_dirs: u32,
    // Data directories (RVA, Size)
    import_dir: (u32, u32),
    export_dir: (u32, u32),
    reloc_dir: (u32, u32),
    iat_dir: (u32, u32),
}

#[derive(Debug, Clone)]
struct PeSection {
    name: String,
    virtual_size: u32,
    virtual_address: u32,
    raw_size: u32,
    raw_offset: u32,
    characteristics: u32,
}

impl PeSection {
    fn contains_rva(&self, rva: u32) -> bool {
        rva >= self.virtual_address && rva < self.virtual_address + self.virtual_size.max(self.raw_size)
    }

    fn rva_to_offset(&self, rva: u32) -> usize {
        (rva - self.virtual_address + self.raw_offset) as usize
    }

    fn is_code(&self) -> bool {
        self.characteristics & 0x20000000 != 0 // IMAGE_SCN_MEM_EXECUTE
    }

    fn is_data(&self) -> bool {
        self.characteristics & 0x40000000 != 0 && !self.is_code() // IMAGE_SCN_MEM_READ
    }
}

#[derive(Debug, Clone)]
struct PeImportEntry {
    dll_name: String,
    symbol_name: String,
    hint: u16,
    iat_rva: u32,
}

#[derive(Debug, Clone)]
struct PeExportEntry {
    name: String,
    rva: u32,
    ordinal: u16,
}

// ============================================================
// PE Parser
// ============================================================

fn parse_pe(view: &BinaryView) -> Result<(PeHeaders, Vec<PeSection>, Vec<PeImportEntry>, Vec<PeExportEntry>), String> {
    let data = &view.data;
    if data.len() < 64 { return Err("File too small for PE".into()); }

    // DOS Header
    if data[0] != 0x4D || data[1] != 0x5A {
        return Err("Not a PE file (missing MZ)".into());
    }
    let e_lfanew = u32::from_le_bytes([data[0x3C], data[0x3D], data[0x3E], data[0x3F]]) as usize;

    // PE Signature
    if e_lfanew + 4 > data.len() { return Err("Invalid e_lfanew".into()); }
    if &data[e_lfanew..e_lfanew+4] != b"PE\0\0" {
        return Err("Invalid PE signature".into());
    }

    // COFF Header
    let coff_off = e_lfanew + 4;
    let mut r = BinaryReader::at(data, coff_off);
    let machine = r.read_u16().ok_or("Failed to read machine")?;
    let num_sections = r.read_u16().ok_or("Failed to read num_sections")?;
    r.skip(12); // timestamp, symbol table, num symbols
    let size_of_optional = r.read_u16().ok_or("Failed to read size_of_optional")?;
    let characteristics = r.read_u16().ok_or("Failed to read characteristics")?;

    // Optional Header
    let opt_off = coff_off + 20;
    let mut r = BinaryReader::at(data, opt_off);
    let magic = r.read_u16().ok_or("Failed to read magic")?;
    let is_pe32plus = magic == 0x020B;

    r.skip(2); // linker version
    r.skip(4); // size of code
    r.skip(4); // size of init data
    r.skip(4); // size of uninit data
    let entry_point_rva = r.read_u32().ok_or("Failed to read entry point")?;
    r.skip(4); // base of code

    let image_base = if is_pe32plus {
        r.read_u64().ok_or("Failed to read image base")?
    } else {
        r.read_u32().ok_or("Failed to read image base")? as u64
    };

    let section_alignment = r.read_u32().ok_or("Failed to read section alignment")?;
    let file_alignment = r.read_u32().ok_or("Failed to read file alignment")?;
    r.skip(16); // OS/image/subsystem versions
    let size_of_image = r.read_u32().ok_or("Failed to read size of image")?;
    let size_of_headers = r.read_u32().ok_or("Failed to read size of headers")?;
    r.skip(4); // checksum
    let subsystem = r.read_u16().ok_or("Failed to read subsystem")?;
    r.skip(2); // dll characteristics

    // Stack/heap sizes
    if is_pe32plus { r.skip(32); } else { r.skip(16); }

    r.skip(4); // loader flags
    let num_data_dirs = r.read_u32().ok_or("Failed to read num data dirs")?;

    // Data directories
    let mut import_dir = (0u32, 0u32);
    let mut export_dir = (0u32, 0u32);
    let mut reloc_dir = (0u32, 0u32);
    let mut iat_dir = (0u32, 0u32);

    for i in 0..num_data_dirs.min(16) {
        let rva = r.read_u32().unwrap_or(0);
        let size = r.read_u32().unwrap_or(0);
        match i {
            0 => export_dir = (rva, size),
            1 => import_dir = (rva, size),
            5 => reloc_dir = (rva, size),
            12 => iat_dir = (rva, size),
            _ => {}
        }
    }

    let headers = PeHeaders {
        machine, num_sections, size_of_optional, characteristics,
        magic, entry_point_rva, image_base, section_alignment,
        file_alignment, size_of_image, size_of_headers, subsystem,
        num_data_dirs, import_dir, export_dir, reloc_dir, iat_dir,
    };

    // Section Table
    let sec_off = opt_off + size_of_optional as usize;
    let mut sections = Vec::new();
    for i in 0..num_sections as usize {
        let off = sec_off + i * 40;
        if off + 40 > data.len() { break; }
        let name_bytes = &data[off..off+8];
        let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(8);
        let name = String::from_utf8_lossy(&name_bytes[..name_end]).to_string();

        let mut sr = BinaryReader::at(data, off + 8);
        let virtual_size = sr.read_u32().unwrap_or(0);
        let virtual_address = sr.read_u32().unwrap_or(0);
        let raw_size = sr.read_u32().unwrap_or(0);
        let raw_offset = sr.read_u32().unwrap_or(0);
        sr.skip(12); // relocs, linenums
        let characteristics = sr.read_u32().unwrap_or(0);

        sections.push(PeSection {
            name, virtual_size, virtual_address,
            raw_size, raw_offset, characteristics,
        });
    }

    // Parse imports
    let imports = parse_imports(data, &sections, import_dir);

    // Parse exports
    let exports = parse_exports(data, &sections, export_dir);

    Ok((headers, sections, imports, exports))
}

fn rva_to_file_offset(sections: &[PeSection], rva: u32) -> Option<usize> {
    for sec in sections {
        if sec.contains_rva(rva) {
            return Some(sec.rva_to_offset(rva));
        }
    }
    None
}

fn parse_imports(data: &[u8], sections: &[PeSection], import_dir: (u32, u32)) -> Vec<PeImportEntry> {
    let mut imports = Vec::new();
    if import_dir.0 == 0 || import_dir.1 == 0 { return imports; }

    let idt_offset = match rva_to_file_offset(sections, import_dir.0) {
        Some(o) => o,
        None => return imports,
    };

    // Each IDT entry is 20 bytes, null-terminated
    let mut entry_off = idt_offset;
    loop {
        if entry_off + 20 > data.len() { break; }
        let mut r = BinaryReader::at(data, entry_off);
        let ilt_rva = r.read_u32().unwrap_or(0);
        r.skip(4); // timestamp
        r.skip(4); // forwarder chain
        let name_rva = r.read_u32().unwrap_or(0);
        let iat_rva = r.read_u32().unwrap_or(0);

        // Null terminator
        if ilt_rva == 0 && name_rva == 0 { break; }

        // DLL name
        let dll_name = match rva_to_file_offset(sections, name_rva) {
            Some(off) => {
                let end = data[off..].iter().position(|&b| b == 0).unwrap_or(0);
                String::from_utf8_lossy(&data[off..off+end]).to_string()
            }
            None => "unknown.dll".to_string(),
        };

        // Parse ILT/IAT entries (64-bit thunks)
        let thunk_rva = if ilt_rva != 0 { ilt_rva } else { iat_rva };
        if let Some(thunk_off) = rva_to_file_offset(sections, thunk_rva) {
            let mut t_off = thunk_off;
            let mut iat_entry_rva = iat_rva;
            loop {
                if t_off + 8 > data.len() { break; }
                let thunk = u64::from_le_bytes(data[t_off..t_off+8].try_into().unwrap());
                if thunk == 0 { break; }

                // Check if import by ordinal (bit 63 set)
                if thunk & (1u64 << 63) != 0 {
                    let ordinal = (thunk & 0xFFFF) as u16;
                    imports.push(PeImportEntry {
                        dll_name: dll_name.clone(),
                        symbol_name: format!("ordinal_{}", ordinal),
                        hint: ordinal,
                        iat_rva: iat_entry_rva,
                    });
                } else {
                    // Import by name
                    let hint_rva = (thunk & 0x7FFFFFFF) as u32;
                    if let Some(hint_off) = rva_to_file_offset(sections, hint_rva) {
                        if hint_off + 2 < data.len() {
                            let hint = u16::from_le_bytes([data[hint_off], data[hint_off+1]]);
                            let name_start = hint_off + 2;
                            let name_end = data[name_start..].iter()
                                .position(|&b| b == 0).unwrap_or(0);
                            let sym_name = String::from_utf8_lossy(
                                &data[name_start..name_start+name_end]
                            ).to_string();

                            imports.push(PeImportEntry {
                                dll_name: dll_name.clone(),
                                symbol_name: sym_name,
                                hint,
                                iat_rva: iat_entry_rva,
                            });
                        }
                    }
                }

                t_off += 8;
                iat_entry_rva += 8;
            }
        }

        entry_off += 20;
    }

    imports
}

fn parse_exports(data: &[u8], sections: &[PeSection], export_dir: (u32, u32)) -> Vec<PeExportEntry> {
    let mut exports = Vec::new();
    if export_dir.0 == 0 || export_dir.1 == 0 { return exports; }

    let exp_offset = match rva_to_file_offset(sections, export_dir.0) {
        Some(o) => o,
        None => return exports,
    };

    if exp_offset + 40 > data.len() { return exports; }
    let mut r = BinaryReader::at(data, exp_offset);
    r.skip(4); // characteristics
    r.skip(4); // timestamp
    r.skip(4); // version
    r.skip(4); // name RVA
    let ordinal_base = r.read_u32().unwrap_or(1);
    let num_functions = r.read_u32().unwrap_or(0);
    let num_names = r.read_u32().unwrap_or(0);
    let addr_table_rva = r.read_u32().unwrap_or(0);
    let name_table_rva = r.read_u32().unwrap_or(0);
    let ordinal_table_rva = r.read_u32().unwrap_or(0);

    let addr_off = rva_to_file_offset(sections, addr_table_rva);
    let name_off = rva_to_file_offset(sections, name_table_rva);
    let ord_off = rva_to_file_offset(sections, ordinal_table_rva);

    if let (Some(addr_off), Some(name_off), Some(ord_off)) = (addr_off, name_off, ord_off) {
        for i in 0..num_names as usize {
            // Name pointer
            let np_off = name_off + i * 4;
            if np_off + 4 > data.len() { break; }
            let name_rva = u32::from_le_bytes(data[np_off..np_off+4].try_into().unwrap());

            // Ordinal
            let o_off = ord_off + i * 2;
            if o_off + 2 > data.len() { break; }
            let ordinal_idx = u16::from_le_bytes([data[o_off], data[o_off+1]]);

            // Function RVA
            let f_off = addr_off + ordinal_idx as usize * 4;
            if f_off + 4 > data.len() { break; }
            let func_rva = u32::from_le_bytes(data[f_off..f_off+4].try_into().unwrap());

            // Name string
            if let Some(str_off) = rva_to_file_offset(sections, name_rva) {
                let end = data[str_off..].iter().position(|&b| b == 0).unwrap_or(0);
                let name = String::from_utf8_lossy(&data[str_off..str_off+end]).to_string();

                exports.push(PeExportEntry {
                    name,
                    rva: func_rva,
                    ordinal: ordinal_base as u16 + ordinal_idx,
                });
            }
        }
    }

    exports
}

// ============================================================
// FASM-inspired: Base Relocation Parser
// ============================================================
// FASM's FORMATS.INC handles relocations in close_pe_section/
// generate_pe_data. The .reloc section contains blocks of:
//   VirtualAddress (4 bytes) + SizeOfBlock (4 bytes)
//   followed by (SizeOfBlock-8)/2 entries of type+offset (2 bytes each)
// Types: 0=ABSOLUTE(pad), 3=HIGHLOW(32-bit), 10=DIR64(64-bit)
// ============================================================

#[derive(Debug, Clone)]
struct PeRelocation {
    rva: u32,
    reloc_type: u8, // IMAGE_REL_BASED_*
}

fn parse_relocations(data: &[u8], sections: &[PeSection], reloc_dir: (u32, u32)) -> Vec<PeRelocation> {
    let mut relocs = Vec::new();
    if reloc_dir.0 == 0 || reloc_dir.1 == 0 { return relocs; }

    let base_off = match rva_to_file_offset(sections, reloc_dir.0) {
        Some(o) => o,
        None => return relocs,
    };

    let mut offset = base_off;
    let end = base_off + reloc_dir.1 as usize;

    while offset + 8 <= end && offset + 8 <= data.len() {
        let block_rva = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
        let block_size = u32::from_le_bytes(data[offset+4..offset+8].try_into().unwrap());

        if block_size < 8 { break; } // Invalid block

        let num_entries = (block_size - 8) / 2;
        let entries_off = offset + 8;

        for i in 0..num_entries as usize {
            let entry_off = entries_off + i * 2;
            if entry_off + 2 > data.len() { break; }
            let entry = u16::from_le_bytes([data[entry_off], data[entry_off + 1]]);
            let reloc_type = (entry >> 12) as u8;
            let reloc_offset = entry & 0x0FFF;

            // Skip padding entries (type 0 = IMAGE_REL_BASED_ABSOLUTE)
            if reloc_type != 0 {
                relocs.push(PeRelocation {
                    rva: block_rva + reloc_offset as u32,
                    reloc_type,
                });
            }
        }

        offset += block_size as usize;
    }

    relocs
}

// ============================================================
// PE Translator
// ============================================================

pub struct PeTranslator;

impl PeTranslator {
    pub fn new() -> Self { PeTranslator }
}

impl ABIBTranslator for PeTranslator {
    fn name(&self) -> &str { "PE Translator (Windows x86-64)" }

    fn source_format(&self) -> SourceFormat { SourceFormat::PE }

    fn can_handle(&self, view: &BinaryView) -> bool {
        view.size >= 2 && view.data[0] == 0x4D && view.data[1] == 0x5A
    }

    fn translate(&self, view: &BinaryView) -> Result<ABIB_Module, String> {
        let (headers, sections, imports, exports) = parse_pe(view)?;

        let mut ctx = TranslationContext::new_cpu(&view.filename, SourceFormat::PE);
        ctx.module.image_base = headers.image_base;

        // FASM-inspired: machine type detection (from format_pe)
        // FASM sets [machine] to 14Ch (i386) or 8664h (AMD64)
        ctx.module.arch = match headers.machine {
            0x8664 => "x86-64".to_string(),
            0x014C => "x86".to_string(),
            0x01C0 => "arm".to_string(),
            0xAA64 => "arm64".to_string(),
            _ => format!("unknown-0x{:04X}", headers.machine),
        };

        // Add imports
        for imp in &imports {
            ctx.add_import(&imp.dll_name, &imp.symbol_name, imp.hint,
                headers.image_base + imp.iat_rva as u64);
        }

        // Add exports
        for exp in &exports {
            ctx.add_export(&exp.name, headers.image_base + exp.rva as u64, exp.ordinal);
        }

        // FASM-inspired: Parse relocations (from generate_pe_data)
        let relocations = parse_relocations(&view.data, &sections, headers.reloc_dir);
        for reloc in &relocations {
            // Type 3 = IMAGE_REL_BASED_HIGHLOW (32-bit), 10 = IMAGE_REL_BASED_DIR64
            let reloc_ir_type = match reloc.reloc_type {
                3 => RelocType::Rel32,   // HIGHLOW → 32-bit
                10 => RelocType::Dir64,  // DIR64 → 64-bit absolute
                _ => RelocType::Abs64,
            };
            ctx.module.relocations.push(ABIB_Relocation {
                addr: headers.image_base + reloc.rva as u64,
                reloc_type: reloc_ir_type,
                symbol: String::new(),
                addend: 0,
            });
        }

        // Decode code sections
        for sec in &sections {
            if !sec.is_code() { continue; }
            let start = sec.raw_offset as usize;
            let end = start + sec.raw_size as usize;
            if end > view.data.len() { continue; }

            let code = &view.data[start..end];
            let base_va = headers.image_base + sec.virtual_address as u64;

            // Create a function for the entry point or the whole section
            let func_name = if headers.entry_point_rva >= sec.virtual_address
                && headers.entry_point_rva < sec.virtual_address + sec.virtual_size
            {
                ctx.module.entry_point = Some("_entry".to_string());
                "_entry"
            } else {
                &sec.name
            };

            ctx.begin_function(func_name, base_va);
            ctx.auto_block(base_va);

            // Decode x86-64 instructions using FASM-quality decoder
            let instructions = x86_decoder::decode_all(code, base_va);
            for inst in instructions {
                ctx.emit(inst);
            }

            ctx.end_function(sec.virtual_size as u64);
            ctx.module.code_size += sec.virtual_size as u64;
        }

        // Add data sections as globals
        for sec in &sections {
            if sec.is_data() && !sec.is_code() {
                let start = sec.raw_offset as usize;
                let end = (start + sec.raw_size as usize).min(view.data.len());
                if start < end {
                    let data = &view.data[start..end];
                    // FASM-inspired: check IMAGE_SCN_MEM_WRITE for mutability
                    let readonly = sec.characteristics & 0x80000000 == 0;
                    ctx.add_global(
                        &sec.name,
                        headers.image_base + sec.virtual_address as u64,
                        sec.virtual_size as u64,
                        data,
                        readonly,
                    );
                }
            }
        }

        Ok(ctx.finish())
    }
}
