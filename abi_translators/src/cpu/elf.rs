// ============================================================
// ELF Translator — Linux ELF64 → ADead-BIB IR
// ============================================================
// Pipeline: ELF Binary → Parse → Decode → Map → ABIB_Module
//
// Parses:
//   ELF Header, Program Headers, Section Headers,
//   Symbol Table (.symtab/.dynsym), String Table (.strtab)
//
// Decodes:
//   x86-64 instructions from executable segments
// ============================================================

use crate::core::ir::*;
use crate::core::translator::{ABIBTranslator, BinaryView};
use crate::core::context::TranslationContext;
use crate::utils::binary_reader::BinaryReader;
use crate::utils::x86_decoder;

// ELF constants
const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;
const EM_X86_64: u16 = 62;
const PT_LOAD: u32 = 1;
const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_DYNSYM: u32 = 11;
const PF_X: u32 = 1;

// ============================================================
// ELF Parsed structures
// ============================================================

#[derive(Debug)]
struct ElfHeader {
    class: u8,
    data: u8,
    machine: u16,
    entry: u64,
    phoff: u64,
    shoff: u64,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[derive(Debug, Clone)]
struct ElfPhdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_filesz: u64,
    p_memsz: u64,
}

#[derive(Debug, Clone)]
struct ElfShdr {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_link: u32,
    sh_entsize: u64,
    name: String,
}

#[derive(Debug, Clone)]
struct ElfSymbol {
    name: String,
    value: u64,
    size: u64,
    sym_type: u8,
    bind: u8,
    shndx: u16,
}

// ============================================================
// ELF Parser
// ============================================================

fn parse_elf(view: &BinaryView) -> Result<(ElfHeader, Vec<ElfPhdr>, Vec<ElfShdr>, Vec<ElfSymbol>), String> {
    let data = &view.data;
    if data.len() < 64 { return Err("File too small for ELF".into()); }

    // ELF magic
    if data[0] != 0x7F || data[1] != b'E' || data[2] != b'L' || data[3] != b'F' {
        return Err("Not an ELF file".into());
    }

    let class = data[4];
    if class != ELFCLASS64 {
        return Err("Only ELF64 is supported".into());
    }

    let elf_data = data[5];
    if elf_data != ELFDATA2LSB {
        return Err("Only little-endian ELF is supported".into());
    }

    let mut r = BinaryReader::at(data, 16);
    let _e_type = r.read_u16().ok_or("Failed to read e_type")?;
    let machine = r.read_u16().ok_or("Failed to read e_machine")?;
    r.skip(4); // e_version
    let entry = r.read_u64().ok_or("Failed to read e_entry")?;
    let phoff = r.read_u64().ok_or("Failed to read e_phoff")?;
    let shoff = r.read_u64().ok_or("Failed to read e_shoff")?;
    r.skip(4); // e_flags
    r.skip(2); // e_ehsize
    let phentsize = r.read_u16().ok_or("Failed to read e_phentsize")?;
    let phnum = r.read_u16().ok_or("Failed to read e_phnum")?;
    let shentsize = r.read_u16().ok_or("Failed to read e_shentsize")?;
    let shnum = r.read_u16().ok_or("Failed to read e_shnum")?;
    let shstrndx = r.read_u16().ok_or("Failed to read e_shstrndx")?;

    let header = ElfHeader {
        class, data: elf_data, machine, entry,
        phoff, shoff, phentsize, phnum,
        shentsize, shnum, shstrndx,
    };

    // Program headers
    let mut phdrs = Vec::new();
    for i in 0..phnum as usize {
        let off = phoff as usize + i * phentsize as usize;
        if off + 56 > data.len() { break; }
        let mut r = BinaryReader::at(data, off);
        let p_type = r.read_u32().unwrap_or(0);
        let p_flags = r.read_u32().unwrap_or(0);
        let p_offset = r.read_u64().unwrap_or(0);
        let p_vaddr = r.read_u64().unwrap_or(0);
        r.skip(8); // p_paddr
        let p_filesz = r.read_u64().unwrap_or(0);
        let p_memsz = r.read_u64().unwrap_or(0);

        phdrs.push(ElfPhdr {
            p_type, p_flags, p_offset, p_vaddr, p_filesz, p_memsz,
        });
    }

    // Section headers
    let mut shdrs = Vec::new();
    for i in 0..shnum as usize {
        let off = shoff as usize + i * shentsize as usize;
        if off + 64 > data.len() { break; }
        let mut r = BinaryReader::at(data, off);
        let sh_name = r.read_u32().unwrap_or(0);
        let sh_type = r.read_u32().unwrap_or(0);
        let sh_flags = r.read_u64().unwrap_or(0);
        let sh_addr = r.read_u64().unwrap_or(0);
        let sh_offset = r.read_u64().unwrap_or(0);
        let sh_size = r.read_u64().unwrap_or(0);
        let sh_link = r.read_u32().unwrap_or(0);
        r.skip(4); // sh_info
        r.skip(8); // sh_addralign
        let sh_entsize = r.read_u64().unwrap_or(0);

        shdrs.push(ElfShdr {
            sh_name, sh_type, sh_flags, sh_addr,
            sh_offset, sh_size, sh_link, sh_entsize,
            name: String::new(),
        });
    }

    // Resolve section names from .shstrtab
    if (shstrndx as usize) < shdrs.len() {
        let strtab_off = shdrs[shstrndx as usize].sh_offset as usize;
        let strtab_size = shdrs[shstrndx as usize].sh_size as usize;
        if strtab_off + strtab_size <= data.len() {
            let strtab = &data[strtab_off..strtab_off + strtab_size];
            for shdr in &mut shdrs {
                let name_off = shdr.sh_name as usize;
                if name_off < strtab.len() {
                    let end = strtab[name_off..].iter().position(|&b| b == 0).unwrap_or(0);
                    shdr.name = String::from_utf8_lossy(&strtab[name_off..name_off+end]).to_string();
                }
            }
        }
    }

    // Parse symbol tables
    let mut symbols = Vec::new();
    for shdr in &shdrs {
        if shdr.sh_type != SHT_SYMTAB && shdr.sh_type != SHT_DYNSYM { continue; }
        if shdr.sh_entsize == 0 { continue; }

        // Find associated string table
        let strtab_idx = shdr.sh_link as usize;
        if strtab_idx >= shdrs.len() { continue; }
        let strtab_shdr = &shdrs[strtab_idx];
        if strtab_shdr.sh_type != SHT_STRTAB { continue; }

        let strtab_off = strtab_shdr.sh_offset as usize;
        let strtab_size = strtab_shdr.sh_size as usize;
        if strtab_off + strtab_size > data.len() { continue; }
        let strtab = &data[strtab_off..strtab_off + strtab_size];

        let num_syms = shdr.sh_size / shdr.sh_entsize;
        for i in 0..num_syms as usize {
            let sym_off = shdr.sh_offset as usize + i * shdr.sh_entsize as usize;
            if sym_off + 24 > data.len() { break; }
            let mut r = BinaryReader::at(data, sym_off);
            let st_name = r.read_u32().unwrap_or(0);
            let st_info = r.read_u8().unwrap_or(0);
            r.skip(1); // st_other
            let st_shndx = r.read_u16().unwrap_or(0);
            let st_value = r.read_u64().unwrap_or(0);
            let st_size = r.read_u64().unwrap_or(0);

            let name = if (st_name as usize) < strtab.len() {
                let end = strtab[st_name as usize..].iter().position(|&b| b == 0).unwrap_or(0);
                String::from_utf8_lossy(&strtab[st_name as usize..st_name as usize + end]).to_string()
            } else {
                String::new()
            };

            if name.is_empty() { continue; }

            symbols.push(ElfSymbol {
                name,
                value: st_value,
                size: st_size,
                sym_type: st_info & 0xF,
                bind: st_info >> 4,
                shndx: st_shndx,
            });
        }
    }

    Ok((header, phdrs, shdrs, symbols))
}

// ============================================================
// ELF Translator
// ============================================================

pub struct ElfTranslator;

impl ElfTranslator {
    pub fn new() -> Self { ElfTranslator }
}

impl ABIBTranslator for ElfTranslator {
    fn name(&self) -> &str { "ELF Translator (Linux x86-64)" }

    fn source_format(&self) -> SourceFormat { SourceFormat::ELF }

    fn can_handle(&self, view: &BinaryView) -> bool {
        view.size >= 4
            && view.data[0] == 0x7F
            && view.data[1] == b'E'
            && view.data[2] == b'L'
            && view.data[3] == b'F'
    }

    fn translate(&self, view: &BinaryView) -> Result<ABIB_Module, String> {
        let (header, phdrs, shdrs, symbols) = parse_elf(view)?;

        let mut ctx = TranslationContext::new_cpu(&view.filename, SourceFormat::ELF);
        ctx.module.image_base = 0;
        ctx.module.arch = if header.machine == EM_X86_64 {
            "x86-64".to_string()
        } else {
            format!("unknown-elf-{}", header.machine)
        };

        // Add symbols as exports/functions
        for sym in &symbols {
            let is_func = sym.sym_type == 2; // STT_FUNC
            let is_global = sym.bind == 1; // STB_GLOBAL
            if is_func && is_global && sym.value != 0 {
                ctx.add_export(&sym.name, sym.value, 0);
            }
        }

        // Decode executable LOAD segments
        for phdr in &phdrs {
            if phdr.p_type != PT_LOAD { continue; }
            if phdr.p_flags & PF_X == 0 { continue; } // not executable

            let start = phdr.p_offset as usize;
            let size = phdr.p_filesz as usize;
            if start + size > view.data.len() { continue; }

            let code = &view.data[start..start + size];
            let base_va = phdr.p_vaddr;

            // Try to split by known function symbols
            let mut func_syms: Vec<&ElfSymbol> = symbols.iter()
                .filter(|s| s.sym_type == 2 && s.value >= base_va && s.value < base_va + size as u64)
                .collect();
            func_syms.sort_by_key(|s| s.value);

            if func_syms.is_empty() {
                // No symbols — decode entire segment as one function
                let func_name = if header.entry == base_va { "_start" } else { ".text" };
                if header.entry >= base_va && header.entry < base_va + size as u64 {
                    ctx.module.entry_point = Some(func_name.to_string());
                }

                ctx.begin_function(func_name, base_va);
                ctx.auto_block(base_va);
                let instructions = x86_decoder::decode_all(code, base_va);
                for inst in instructions {
                    ctx.emit(inst);
                }
                ctx.end_function(size as u64);
            } else {
                // Decode per-function
                for (i, sym) in func_syms.iter().enumerate() {
                    let func_start = (sym.value - base_va) as usize;
                    let func_end = if sym.size > 0 {
                        func_start + sym.size as usize
                    } else if i + 1 < func_syms.len() {
                        (func_syms[i + 1].value - base_va) as usize
                    } else {
                        size
                    };

                    if func_start >= size || func_end > size { continue; }
                    let func_code = &code[func_start..func_end];

                    if sym.value == header.entry {
                        ctx.module.entry_point = Some(sym.name.clone());
                    }

                    ctx.begin_function(&sym.name, sym.value);
                    ctx.auto_block(sym.value);
                    let instructions = x86_decoder::decode_all(func_code, sym.value);
                    for inst in instructions {
                        ctx.emit(inst);
                    }
                    ctx.end_function((func_end - func_start) as u64);
                }
            }

            ctx.module.code_size += size as u64;
        }

        // Add data sections as globals
        for shdr in &shdrs {
            if shdr.sh_flags & 2 != 0 && shdr.sh_flags & 4 == 0 { // ALLOC && !EXECINSTR
                let start = shdr.sh_offset as usize;
                let end = (start + shdr.sh_size as usize).min(view.data.len());
                if start < end && !shdr.name.is_empty() {
                    let readonly = shdr.sh_flags & 1 == 0; // !WRITE
                    ctx.add_global(
                        &shdr.name, shdr.sh_addr, shdr.sh_size,
                        &view.data[start..end], readonly,
                    );
                }
            }
        }

        Ok(ctx.finish())
    }
}
