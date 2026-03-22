// ============================================================
// ADead-BIB ABI Translators — CLI
// ============================================================
// Usage:
//   abi-translate <input> [options]
//
// Options:
//   -o <output.abib>   Output ABIB IR dump file
//   --info              Show module info only
//   --dump              Dump all instructions
//   --demo              Run built-in demo (PE + SPIR-V)
//   --list              List registered translators
//
// Auto-detects: PE, ELF, SPIR-V, DXBC/DXIL
//
// Examples:
//   abi-translate program.exe
//   abi-translate shader.spv --dump
//   abi-translate program.elf -o output.abib
//   abi-translate --demo
// ============================================================

use abi_translators::core::registry;
use abi_translators::core::translator::BinaryView;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let mut input_path: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut show_info = false;
    let mut dump_all = false;
    let mut demo_mode = false;
    let mut list_mode = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                if i < args.len() { output_path = Some(args[i].clone()); }
            }
            "--info" => { show_info = true; }
            "--dump" => { dump_all = true; }
            "--demo" => { demo_mode = true; }
            "--list" => { list_mode = true; }
            "-h" | "--help" => {
                print_usage();
                return;
            }
            other => {
                if input_path.is_none() && !other.starts_with('-') {
                    input_path = Some(other.to_string());
                } else {
                    eprintln!("Unknown argument: {}", other);
                    std::process::exit(1);
                }
            }
        }
        i += 1;
    }

    // Build registry
    let reg = registry::build_default_registry();

    if list_mode {
        println!("=== Registered Translators ({}) ===", reg.count());
        for name in reg.list() {
            println!("  - {}", name);
        }
        return;
    }

    if demo_mode {
        run_demo(&reg);
        return;
    }

    // Normal mode
    let input = match input_path {
        Some(p) => p,
        None => {
            eprintln!("Error: No input file. Use --demo for a demo or --help for usage.");
            std::process::exit(1);
        }
    };

    println!("=== ADead-BIB ABI Translator v1.0 ===");
    println!("  Input: {}", input);

    let view = match BinaryView::from_file(&input) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let format = view.detect_format();
    println!("  Format: {:?}", format);
    println!("  Size: {} bytes", view.size);

    let module = match reg.translate(&view) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Show info
    if show_info || !dump_all {
        println!();
        print!("{}", module);
    }

    // Dump all instructions
    if dump_all {
        println!();
        println!("=== Full IR Dump ===");
        for func in &module.functions {
            println!();
            print!("{}", func);
        }
        for kernel in &module.gpu_kernels {
            println!();
            println!("gpu_kernel {} ({:?}, local_size={:?}):",
                kernel.name, kernel.execution_model, kernel.local_size);
            for inst in &kernel.instructions {
                println!("{}", inst);
            }
        }
    }

    // Save output
    if let Some(ref out_path) = output_path {
        let dump = format!("{}", module);
        match std::fs::write(out_path, dump) {
            Ok(()) => println!("\n  Output saved: {}", out_path),
            Err(e) => eprintln!("\n  Error saving: {}", e),
        }
    }

    println!("\n  Translation complete!");
}

// ============================================================
// Demo — translate a synthetic PE binary
// ============================================================

fn run_demo(reg: &registry::TranslatorRegistry) {
    println!("=== ADead-BIB ABI Translator — Demo ===");
    println!();

    // Demo 1: Synthetic PE
    println!("--- Demo 1: Synthetic PE Binary ---");
    let pe_binary = build_demo_pe();
    let pe_view = BinaryView::from_bytes(pe_binary, "demo.exe");
    println!("  Format: {:?}", pe_view.detect_format());
    println!("  Size: {} bytes", pe_view.size);

    match reg.translate(&pe_view) {
        Ok(module) => {
            print!("{}", module);
        }
        Err(e) => {
            eprintln!("  PE translation error: {}", e);
        }
    }

    println!();

    // Demo 2: Synthetic SPIR-V
    println!("--- Demo 2: Synthetic SPIR-V Binary ---");
    let spirv_binary = build_demo_spirv();
    let spirv_view = BinaryView::from_bytes(spirv_binary, "demo.spv");
    println!("  Format: {:?}", spirv_view.detect_format());
    println!("  Size: {} bytes", spirv_view.size);

    match reg.translate(&spirv_view) {
        Ok(module) => {
            print!("{}", module);
        }
        Err(e) => {
            eprintln!("  SPIR-V translation error: {}", e);
        }
    }

    println!();
    println!("  Demo complete!");
}

/// Build a minimal valid PE binary for demo
fn build_demo_pe() -> Vec<u8> {
    let file_align = 0x200usize;
    let section_align = 0x1000usize;

    // Code: sub rsp,0x28 / mov eax,42 / add rsp,0x28 / ret
    let code: Vec<u8> = vec![
        0x48, 0x83, 0xEC, 0x28,             // sub rsp, 0x28
        0xB8, 0x2A, 0x00, 0x00, 0x00,       // mov eax, 42
        0x48, 0x83, 0xC4, 0x28,             // add rsp, 0x28
        0xC3,                                 // ret
    ];

    let code_raw_size = ((code.len() + file_align - 1) / file_align) * file_align;
    let headers_size = 64 + 4 + 20 + 240 + 40; // DOS + PE sig + COFF + Optional + 1 section
    let headers_aligned = ((headers_size + file_align - 1) / file_align) * file_align;
    let total_size = headers_aligned + code_raw_size;

    let mut pe = vec![0u8; total_size];

    // DOS Header
    pe[0] = 0x4D; pe[1] = 0x5A; // MZ
    pe[0x3C] = 0x40; // e_lfanew = 64

    // PE Signature
    pe[64..68].copy_from_slice(b"PE\0\0");

    // COFF Header
    let coff = 68;
    pe[coff..coff+2].copy_from_slice(&0x8664u16.to_le_bytes()); // x64
    pe[coff+2..coff+4].copy_from_slice(&1u16.to_le_bytes()); // 1 section
    pe[coff+16..coff+18].copy_from_slice(&240u16.to_le_bytes()); // optional header size
    pe[coff+18..coff+20].copy_from_slice(&0x0022u16.to_le_bytes()); // characteristics

    // Optional Header
    let opt = coff + 20;
    pe[opt..opt+2].copy_from_slice(&0x020Bu16.to_le_bytes()); // PE32+
    pe[opt+2] = 14; // linker version
    pe[opt+4..opt+8].copy_from_slice(&(code_raw_size as u32).to_le_bytes()); // SizeOfCode
    pe[opt+16..opt+20].copy_from_slice(&(section_align as u32).to_le_bytes()); // EntryPoint RVA
    pe[opt+20..opt+24].copy_from_slice(&(section_align as u32).to_le_bytes()); // BaseOfCode
    pe[opt+24..opt+32].copy_from_slice(&0x140000000u64.to_le_bytes()); // ImageBase
    pe[opt+32..opt+36].copy_from_slice(&(section_align as u32).to_le_bytes()); // SectionAlignment
    pe[opt+36..opt+40].copy_from_slice(&(file_align as u32).to_le_bytes()); // FileAlignment
    pe[opt+40..opt+42].copy_from_slice(&6u16.to_le_bytes()); // MajorOSVersion
    pe[opt+48..opt+50].copy_from_slice(&6u16.to_le_bytes()); // MajorSubsystemVersion
    pe[opt+56..opt+60].copy_from_slice(&0x2000u32.to_le_bytes()); // SizeOfImage
    pe[opt+60..opt+64].copy_from_slice(&(headers_aligned as u32).to_le_bytes()); // SizeOfHeaders
    pe[opt+68..opt+70].copy_from_slice(&3u16.to_le_bytes()); // Subsystem CUI
    pe[opt+72..opt+80].copy_from_slice(&0x100000u64.to_le_bytes()); // StackReserve
    pe[opt+80..opt+88].copy_from_slice(&0x1000u64.to_le_bytes()); // StackCommit
    pe[opt+88..opt+96].copy_from_slice(&0x100000u64.to_le_bytes()); // HeapReserve
    pe[opt+96..opt+104].copy_from_slice(&0x1000u64.to_le_bytes()); // HeapCommit
    pe[opt+108..opt+112].copy_from_slice(&16u32.to_le_bytes()); // NumberOfRvaAndSizes

    // Section Header (.text)
    let sec = opt + 240;
    pe[sec..sec+5].copy_from_slice(b".text");
    pe[sec+8..sec+12].copy_from_slice(&(code.len() as u32).to_le_bytes()); // VirtualSize
    pe[sec+12..sec+16].copy_from_slice(&(section_align as u32).to_le_bytes()); // VirtualAddress
    pe[sec+16..sec+20].copy_from_slice(&(code_raw_size as u32).to_le_bytes()); // SizeOfRawData
    pe[sec+20..sec+24].copy_from_slice(&(headers_aligned as u32).to_le_bytes()); // PointerToRawData
    pe[sec+36..sec+40].copy_from_slice(&0x60000020u32.to_le_bytes()); // Characteristics

    // Write code
    pe[headers_aligned..headers_aligned + code.len()].copy_from_slice(&code);

    pe
}

/// Build a minimal valid SPIR-V binary for demo
fn build_demo_spirv() -> Vec<u8> {
    let mut words: Vec<u32> = Vec::new();

    // Header
    words.push(0x07230203); // magic
    words.push(0x00010500); // version 1.5
    words.push(0);          // generator
    words.push(20);         // bound (max ID + 1)
    words.push(0);          // reserved

    // OpCapability Shader (word_count=2, opcode=17)
    words.push((2 << 16) | 17);
    words.push(1); // Shader

    // OpMemoryModel Logical GLSL450 (word_count=3, opcode=14)
    words.push((3 << 16) | 14);
    words.push(0); // Logical
    words.push(1); // GLSL450

    // OpEntryPoint GLCompute %main "main" (word_count=4, opcode=15)
    words.push((4 << 16) | 15);
    words.push(5); // GLCompute
    words.push(1); // %main = ID 1
    // "main" as words
    let main_bytes = b"main";
    let mut name_word = 0u32;
    for (i, &b) in main_bytes.iter().enumerate() {
        name_word |= (b as u32) << (i * 8);
    }
    words.push(name_word);

    // OpExecutionMode %main LocalSize 64 1 1 (word_count=6, opcode=16)
    words.push((6 << 16) | 16);
    words.push(1); // %main
    words.push(17); // LocalSize
    words.push(64); // x
    words.push(1);  // y
    words.push(1);  // z

    // OpName %main "main" (word_count=3, opcode=5)
    words.push((3 << 16) | 5);
    words.push(1); // %main
    words.push(name_word);

    // OpTypeVoid %2 (word_count=2, opcode=19)
    words.push((2 << 16) | 19);
    words.push(2);

    // OpTypeFunction %3 %2 (word_count=3, opcode=33)
    words.push((3 << 16) | 33);
    words.push(3); // result
    words.push(2); // return type (void)

    // OpFunction %2 %main None %3 (word_count=5, opcode=54)
    words.push((5 << 16) | 54);
    words.push(2); // result type
    words.push(1); // %main
    words.push(0); // function control: None
    words.push(3); // function type

    // OpLabel %4 (word_count=2, opcode=248)
    words.push((2 << 16) | 248);
    words.push(4);

    // OpReturn (word_count=1, opcode=253)
    words.push((1 << 16) | 253);

    // OpFunctionEnd (word_count=1, opcode=56)
    words.push((1 << 16) | 56);

    // Convert to bytes
    let mut bytes = Vec::with_capacity(words.len() * 4);
    for w in &words {
        bytes.extend_from_slice(&w.to_le_bytes());
    }
    bytes
}

fn print_usage() {
    println!("=== ADead-BIB ABI Translator v1.0 ===");
    println!("Binary Frontend: PE/ELF/SPIR-V/DXIL → ADead-BIB IR");
    println!();
    println!("Usage:");
    println!("  abi-translate <input> [options]");
    println!();
    println!("Options:");
    println!("  -o <output>     Save IR dump to file");
    println!("  --info          Show module summary");
    println!("  --dump          Dump all instructions");
    println!("  --demo          Run built-in demo");
    println!("  --list          List registered translators");
    println!("  -h, --help      Show this help");
    println!();
    println!("Supported formats:");
    println!("  PE   (.exe/.dll)  — Windows x86-64");
    println!("  ELF  (.elf/.so)   — Linux x86-64");
    println!("  SPIR-V (.spv)     — Vulkan/OpenCL shaders");
    println!("  DXBC/DXIL (.dxil) — DirectX 12 shaders");
    println!();
    println!("Pipeline:");
    println!("  Binary → Parse → Decode → Map → ADead-BIB IR");
}
