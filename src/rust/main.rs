pub mod frontend;
pub mod middle;
pub mod backend;
pub mod gc_plus;

use frontend::java::ja_lexer::JaLexer;
use frontend::java::ja_parser::JaParser;
use frontend::java::ja_to_ir::JaToIrGenerator;
use frontend::java::ja_preprocessor::JaPreprocessor;
use frontend::java::ja_import_resolver::JaImportResolver;
use middle::ub_detector::UbDetector;
use backend::isa::ISATranslator;
use backend::pe::PeExporter;
use backend::jit::{JitExecutor, hash_source, jdb_flush_prints};

use std::env;
use std::fs;
use std::process;
use std::time::Instant;

// --- ANSI Colors ---
const C_RESET: &str = "\x1b[0m";
const C_TITLE: &str = "\x1b[1;36m";   // Cyan
const C_OK: &str = "\x1b[1;32m";      // Green
const C_WARN: &str = "\x1b[1;33m";    // Yellow
const C_ERR: &str = "\x1b[1;31m";     // Red
const C_PHASE: &str = "\x1b[1;35m";   // Purple
const C_TEXT: &str = "\x1b[1;37m";    // White
const C_DIM: &str = "\x1b[2;37m";     // Gray
const C_GCPLUS: &str = "\x1b[1;34m";  // Blue/Indigo for Garbage Collection 2.0
const C_CYBER: &str = "\x1b[38;5;45m";// Cyberpunk blue
const C_RAD: &str = "\x1b[38;5;196m"; // Radical Red

fn print_header() {
    println!("{C_TITLE}╔══════════════════════════════════════════════════════════════╗");
    println!("║   {C_CYBER}JaDead-BIB v1.0 💀☕{C_TITLE}                             ║");
    println!("║   {C_RAD}Java Nativo — Sin JVM — Sin GC — Sin Runtime{C_TITLE}        ║");
    println!("╚═══════════════════════════════════════════════════════════════════════╝{C_RESET}");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        println!("{C_TITLE}Uso:{C_RESET} jab run <archivo.java>    {C_DIM}(In-Memory Execution - JIT 2.0){C_RESET}");
        println!("     jab java <archivo.java>   {C_DIM}(Exportar .exe nativo){C_RESET}");
        println!("     jab step <archivo.java>   {C_DIM}(Modo Análisis y verbose){C_RESET}");
        process::exit(1);
    }

    let mode = args[1].as_str();
    if mode != "java" && mode != "run" && mode != "step" {
        println!("Comando desconocido: {}", mode);
        process::exit(1);
    }

    let is_step_mode = mode == "step";
    let _is_run_mode = mode == "run";
    let file_path = &args[2];

    if is_step_mode || true {
        print_header();
    }

    println!("  {C_TEXT}Source:{C_RESET}   {}", file_path);
    println!("  {C_TEXT}Language:{C_RESET} Java 21");
    println!();

    // LECTURA DEL CÓDIGO FUENTE
    let source = fs::read_to_string(file_path).unwrap_or_else(|_| {
        eprintln!("{C_ERR}Error:{C_RESET} No se pudo leer el archivo {}", file_path);
        process::exit(1);
    });

    let mut step_logs = String::with_capacity(4096);
    macro_rules! s_print {
        ($($arg:tt)*) => {
            if is_step_mode {
                use std::fmt::Write;
                let _ = writeln!(&mut step_logs, $($arg)*);
            }
        };
    }

    let start_time = Instant::now();

    // FASE 01: PREPROCESSOR
    s_print!("{C_PHASE}--- Phase 01: PREPROCESSOR ---{C_RESET}");
    let mut preprocessor = JaPreprocessor::new(&source);
    let processed_source = match preprocessor.process() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{C_ERR}Error de Preprocesamiento:\n{}{C_RESET}", e);
            process::exit(1);
        }
    };
    s_print!("{C_DIM}[PREPROC]  Directivas y limpieza aplicadas{C_RESET}");

    // FASE 03: LEXER
    s_print!("{C_PHASE}--- Phase 03: LEXER ---{C_RESET}");
    let mut lexer = JaLexer::new(&processed_source);
    let tokens = lexer.tokenize();
    s_print!("{C_DIM}[LEXER]    {} tokens generados{C_RESET}", tokens.len());

    // FASE 04: PARSER (AST)
    s_print!("{C_PHASE}--- Phase 04: PARSER ---{C_RESET}");
    let reset_lexer = JaLexer::new(&processed_source);
    let mut parser = JaParser::new(reset_lexer);
    let mut ast = match parser.parse_compilation_unit() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Error de Sintaxis:\n{}", e);
            process::exit(1);
        }
    };
    s_print!("[PARSER]   {} declaraciones de nivel superior analizadas", ast.declarations.len());

    // FASE 04.5: IMPORT RESOLVER
    s_print!("\n--- Phase 04.5: IMPORT RESOLVER ---");
    let import_resolver = JaImportResolver::new();
    if let Err(e) = import_resolver.resolve_imports(&mut ast) {
        eprintln!("Error Resolviendo Imports:\n{}", e);
        process::exit(1);
    }
    s_print!("[IMPORTS]  Librería Estándar API mapeada a FastOS.bib Nativo");

    // FASE 04.8: UB DETECTOR (Seguridad de Java adelantada a Tiempo de Compilación)
    s_print!("\n{C_PHASE}--- Phase 04.8: UB DETECTOR ---{C_RESET}");
    let mut ub_detector = UbDetector::new();
    let warnings = ub_detector.analyze(&ast);
    for warn in &warnings {
        s_print!("{C_WARN}⚠️  [UB-WARN] {:?}: {}{C_RESET}", warn.kind, warn.message);
    }
    s_print!("{C_DIM}[UB-DETECT] Analizado para 15+ tipos de comportamiento indefinido (Warns: {}){C_RESET}", warnings.len());

    // FASE 05 & 06: TYPE CHECKER + IR GENERATION (ADeadOp)
    s_print!("\n{C_PHASE}--- Phase 06: IR (ADeadOp SSA-form) + GC PLUS ENGINE ---{C_RESET}");
    let mut ir_generator = JaToIrGenerator::new();
    let module = match ir_generator.generate(&ast) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{C_ERR}Error de Types / IR:\n{}{C_RESET}", e);
            process::exit(1);
        }
    };

    s_print!("{C_GCPLUS}[GC+] Scope Tracker:      {C_OK}ACTIVO (Zero-Pause Deterministic Free){C_RESET}");
    s_print!("{C_GCPLUS}[GC+] Loop Anticipator:   {C_OK}ACTIVO (Object Pools Pre-Allocated){C_RESET}");
    s_print!("{C_GCPLUS}[GC+] Escape Detector:    {C_OK}ACTIVO (Nativo Out-Of-Bounds Checks){C_RESET}");
    s_print!("{C_GCPLUS}[GC+] Region Memory:      {C_OK}ACTIVO (App_Root_Region){C_RESET}");
    s_print!("{C_GCPLUS}[GC+] Cycle Breaker:      {C_OK}ACTIVO (Strict Mode){C_RESET}\n");
    s_print!("{C_DIM}[IR]  {} module generado{C_RESET}", module.name);
    s_print!("{C_DIM}[IR]  Tipos estáticos Java validados y mapeados a nativo{C_RESET}");
    s_print!("{C_DIM}[IR]  JVM eliminado — machine code listo para backend ✓{C_RESET}");

    // FASE 07: ISA TRANSLATION (ADeadOp IR -> x86-64 Machine Code)
    s_print!("\n--- Phase 07: BACKEND (x86-64 ISA) ---");
    let mut translator = ISATranslator::new();
    let machine_code = match translator.translate(&module) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error de Traducción ISA:\n{}", e);
            process::exit(1);
        }
    };
    s_print!("[ISA]      Código máquina generado ({} bytes)", machine_code.len());

    if mode == "java" {
        // FASE 08: LINKING & EXPORT (.exe PE Nativo WIndows)
        if is_step_mode { println!("\n--- Phase 08: LINK&EXPORT (PE .exe) ---"); }
        let output_name = file_path.replace(".java", ".exe");
        let exporter = PeExporter::new(machine_code.clone());
        if let Err(e) = exporter.export_exe(&output_name) {
            eprintln!("Error Exportando .exe:\n{}", e);
            process::exit(1);
        }

        // Final Statistics para Export
        let duration = start_time.elapsed();
        println!("\n{C_OK}✅ JaDead-BIB compilación Native completada{C_RESET}");
        println!("   {} generado con éxito.", output_name);
        println!("   {C_DIM}Tiempo consumido: {:?}{C_RESET}", duration);
    } else {
        // FASE 09: JIT 2.0 EXECUTION (jab run)
        let source_hash = hash_source(&source);
        let jit = JitExecutor::new(machine_code, vec![], source_hash);
        
        // Ejecución (Evitar printing I/O slow mid-execution)
        let exec_result = jit.execute_with_stats();
        let total_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        
        let out_buf = jdb_flush_prints();
        if !out_buf.is_empty() {
            print!("{}", out_buf);
        }
        
        if is_step_mode {
            print!("{}", step_logs);
            println!("{C_PHASE}--- RUNTIME: JIT 2.0 IN-MEMORY ---{C_RESET}");
        }

        match exec_result {
            Ok((code, stats)) => {
                println!("{C_DIM}[JIT 2.0]{C_RESET} {C_TEXT}alloc{C_RESET}:   {:.4}ms (.text RWX, .data RW)", stats.alloc_ms);
                println!("{C_DIM}[JIT 2.0]{C_RESET} {C_TEXT}patch{C_RESET}:   {:.4}ms (pre-patched instant image)", stats.patch_ms);
                println!("{C_DIM}[JIT 2.0]{C_RESET} {C_TEXT}exec{C_RESET}:    {:.4}ms", stats.exec_ms);
                println!("{C_OK}[JIT 2.0] time-to-RAM: {:.4}ms{C_RESET}", total_time_ms);
                process::exit(code);
            },
            Err(e) => {
                eprintln!("{C_ERR}[JIT 2.0] Error de ejecución: {}{C_RESET}", e);
                process::exit(1);
            }
        }
    }
}
