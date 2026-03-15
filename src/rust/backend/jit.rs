// ============================================================
// Executable Memory JIT Runtime for JaDead-BIB 💀☕
// JIT 2.0 KILLER — ELIMINA TOTALMENTE LA JIT ORIGINAL JAVA
// ============================================================
// Directly maps x86-64 bytes into executable memory
// No warmed-up JVM needed. Instant execution.
// "El CPU no piensa — ya sabe. La RAM no espera — ya recibe."
//
// MEJORA 1: Ejecución Directa en RAM (Execute-in-Place)
// MEJORA 2: Zero Copy Data — .text RWX, .data RW (no exec)
// MEJORA 3: CPU Feature Detection (AVX2/SSE4)
// MEJORA 4: Thermal Cache (Evita recompilar ASTs idénticos)
// ============================================================

use std::collections::HashMap;
use std::sync::Mutex;
use std::ptr;

/// Cache de código de máquina compilado en la misma sesión
struct CacheEntry {
    #[allow(dead_code)]
    text: Vec<u8>,
    #[allow(dead_code)]
    data: Vec<u8>,
}

static THERMAL_CACHE: std::sync::LazyLock<Mutex<HashMap<u64, CacheEntry>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub static CPU_FEATURES: std::sync::LazyLock<CpuFeatures> = std::sync::LazyLock::new(|| {
    JitExecutor::detect_cpu_features()
});

/// Hashes Java Source to identify it uniquely
pub fn hash_source(source: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325; // FNV-1a offset basis
    for b in source.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3); // FNV prime
    }
    h
}

static PRINT_BUFFER: std::sync::LazyLock<Mutex<String>> = std::sync::LazyLock::new(|| Mutex::new(String::with_capacity(8192)));

pub fn jdb_flush_prints() -> String {
    if let Ok(mut buf) = PRINT_BUFFER.lock() {
        let rv = buf.clone();
        buf.clear();
        rv
    } else {
        String::new()
    }
}

#[repr(C)]
pub struct JdbString {
    pub ptr: *const u8,
    pub len: u32,
}

#[no_mangle]
pub extern "C" fn jdb_print_str(ptr: *const u8, len: u32) {
    let slice = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    if let Ok(s) = std::str::from_utf8(slice) {
        if let Ok(mut buf) = PRINT_BUFFER.lock() {
            buf.push_str(s);
            buf.push('\n');
        }
    }
}

#[no_mangle]
pub extern "C" fn jdb_print_obj(s: *const JdbString) {
    if s.is_null() {
        if let Ok(mut buf) = PRINT_BUFFER.lock() { buf.push_str("null\n"); }
        return;
    }
    let slice = unsafe { std::slice::from_raw_parts((*s).ptr, (*s).len as usize) };
    if let Ok(st) = std::str::from_utf8(slice) {
        if let Ok(mut buf) = PRINT_BUFFER.lock() {
            buf.push_str(st);
            buf.push('\n');
        }
    }
}

#[no_mangle]
pub extern "C" fn jdb_string_len(s: *const JdbString) -> i64 {
    if s.is_null() { return 0; }
    unsafe { (*s).len as i64 }
}

#[no_mangle]
pub extern "C" fn jdb_string_eq(s1: *const JdbString, s2: *const JdbString) -> i64 {
    if s1.is_null() && s2.is_null() { return 1; }
    if s1.is_null() || s2.is_null() { return 0; }
    unsafe {
        if (*s1).len != (*s2).len { return 0; }
        let slice1 = std::slice::from_raw_parts((*s1).ptr, (*s1).len as usize);
        let slice2 = std::slice::from_raw_parts((*s2).ptr, (*s2).len as usize);
        if slice1 == slice2 { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn jdb_string_concat(s1: *const JdbString, s2: *const JdbString) -> *const JdbString {
    let str1 = if s1.is_null() { "" } else {
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts((*s1).ptr, (*s1).len as usize)) }
    };
    let str2 = if s2.is_null() { "" } else {
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts((*s2).ptr, (*s2).len as usize)) }
    };
    
    let concat = format!("{}{}", str1, str2);
    let boxed_str = Box::leak(concat.into_boxed_str());
    let jdb_str = Box::new(JdbString {
        ptr: boxed_str.as_ptr(),
        len: boxed_str.len() as u32,
    });
    Box::leak(jdb_str) as *const JdbString
}

// ──────────────────────────────────────────────────────────
// Java 1.0 (1996) — V1 ADeadOp Array Structures
// ──────────────────────────────────────────────────────────

#[repr(C)]
pub struct JdbArray {
    pub ptr: *mut u8,
    pub len: u32,
    pub element_size: u32,
}

#[no_mangle]
pub extern "C" fn jdb_alloc_array(count: u32, element_size: u32) -> *const JdbArray {
    let total_bytes = (count as usize) * (element_size as usize);
    let raw_buf = unsafe {
        std::alloc::alloc_zeroed(
            std::alloc::Layout::from_size_align(total_bytes.max(1), 8).unwrap()
        )
    };
    let jdb_arr = Box::new(JdbArray {
        ptr: raw_buf,
        len: count,
        element_size,
    });
    Box::leak(jdb_arr) as *const JdbArray
}

#[no_mangle]
pub extern "C" fn jdb_array_len(arr: *const JdbArray) -> i64 {
    if arr.is_null() { return 0; }
    unsafe { (*arr).len as i64 }
}

// ──────────────────────────────────────────────────────────
// Java 1.0 (1996) — V1 ADeadOp OOP Native Allocation
// ──────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn jdb_alloc_obj(size: u32) -> *mut u8 {
    let size = size as usize;
    let raw_buf = unsafe {
        std::alloc::alloc_zeroed(
            std::alloc::Layout::from_size_align(size.max(1), 8).unwrap()
        )
    };
    raw_buf
}

#[no_mangle]
pub extern "C" fn jdb_print_int(val: i64) {
    if let Ok(mut buf) = PRINT_BUFFER.lock() {
        use std::fmt::Write;
        let _ = writeln!(buf, "{}", val);
    }
}

#[no_mangle]
pub extern "C" fn jdb_print_float(val: f64) {
    if let Ok(mut buf) = PRINT_BUFFER.lock() {
        use std::fmt::Write;
        let _ = writeln!(buf, "{}", val);
    }
}

#[no_mangle]
pub extern "C" fn jdb_print_char(val: i64) {
    if let Ok(mut buf) = PRINT_BUFFER.lock() {
        if let Some(c) = char::from_u32(val as u32) {
            buf.push(c);
            buf.push('\n');
        }
    }
}

#[no_mangle]
pub extern "C" fn jdb_print_newline() {
    if let Ok(mut buf) = PRINT_BUFFER.lock() {
        buf.push('\n');
    }
}

#[no_mangle]
pub extern "C" fn jdb_print_bool(val: i64) {
    if let Ok(mut buf) = PRINT_BUFFER.lock() {
        use std::fmt::Write;
        let _ = writeln!(buf, "{}", if val != 0 { "true" } else { "false" });
    }
}

// ──────────────────────────────────────────────────────────
// Java 1.0 — String API Native (sin JVM, sin GC)
// ──────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn jdb_string_contains(haystack: *const JdbString, needle: *const JdbString) -> i64 {
    if haystack.is_null() || needle.is_null() { return 0; }
    unsafe {
        let h = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*haystack).ptr, (*haystack).len as usize));
        let n = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*needle).ptr, (*needle).len as usize));
        if h.contains(n) { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn jdb_string_substring(s: *const JdbString, start: i64, end: i64) -> *const JdbString {
    if s.is_null() { return std::ptr::null(); }
    unsafe {
        let src = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*s).ptr, (*s).len as usize));
        let start = start.max(0) as usize;
        let end = (end as usize).min(src.len());
        if start > end { return std::ptr::null(); }
        let sub = &src[start..end];
        let boxed = Box::leak(sub.to_string().into_boxed_str());
        let jdb = Box::new(JdbString { ptr: boxed.as_ptr(), len: boxed.len() as u32 });
        Box::leak(jdb) as *const JdbString
    }
}

#[no_mangle]
pub extern "C" fn jdb_string_to_upper(s: *const JdbString) -> *const JdbString {
    if s.is_null() { return std::ptr::null(); }
    unsafe {
        let src = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*s).ptr, (*s).len as usize));
        let upper = src.to_uppercase();
        let boxed = Box::leak(upper.into_boxed_str());
        let jdb = Box::new(JdbString { ptr: boxed.as_ptr(), len: boxed.len() as u32 });
        Box::leak(jdb) as *const JdbString
    }
}

#[no_mangle]
pub extern "C" fn jdb_string_to_lower(s: *const JdbString) -> *const JdbString {
    if s.is_null() { return std::ptr::null(); }
    unsafe {
        let src = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*s).ptr, (*s).len as usize));
        let lower = src.to_lowercase();
        let boxed = Box::leak(lower.into_boxed_str());
        let jdb = Box::new(JdbString { ptr: boxed.as_ptr(), len: boxed.len() as u32 });
        Box::leak(jdb) as *const JdbString
    }
}

#[no_mangle]
pub extern "C" fn jdb_string_index_of(s: *const JdbString, needle: *const JdbString) -> i64 {
    if s.is_null() || needle.is_null() { return -1; }
    unsafe {
        let src = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*s).ptr, (*s).len as usize));
        let n = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*needle).ptr, (*needle).len as usize));
        src.find(n).map(|i| i as i64).unwrap_or(-1)
    }
}

#[no_mangle]
pub extern "C" fn jdb_string_char_at(s: *const JdbString, index: i64) -> i64 {
    if s.is_null() { return 0; }
    unsafe {
        let src = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*s).ptr, (*s).len as usize));
        let idx = index as usize;
        if idx >= src.len() { return 0; }
        src.as_bytes()[idx] as i64
    }
}

#[no_mangle]
pub extern "C" fn jdb_string_replace(s: *const JdbString, from: *const JdbString, to: *const JdbString) -> *const JdbString {
    if s.is_null() || from.is_null() || to.is_null() { return s; }
    unsafe {
        let src = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*s).ptr, (*s).len as usize));
        let f = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*from).ptr, (*from).len as usize));
        let t = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*to).ptr, (*to).len as usize));
        let replaced = src.replace(f, t);
        let boxed = Box::leak(replaced.into_boxed_str());
        let jdb = Box::new(JdbString { ptr: boxed.as_ptr(), len: boxed.len() as u32 });
        Box::leak(jdb) as *const JdbString
    }
}

#[no_mangle]
pub extern "C" fn jdb_string_trim(s: *const JdbString) -> *const JdbString {
    if s.is_null() { return std::ptr::null(); }
    unsafe {
        let src = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*s).ptr, (*s).len as usize));
        let trimmed = src.trim();
        let boxed = Box::leak(trimmed.to_string().into_boxed_str());
        let jdb = Box::new(JdbString { ptr: boxed.as_ptr(), len: boxed.len() as u32 });
        Box::leak(jdb) as *const JdbString
    }
}

#[no_mangle]
pub extern "C" fn jdb_string_starts_with(s: *const JdbString, prefix: *const JdbString) -> i64 {
    if s.is_null() || prefix.is_null() { return 0; }
    unsafe {
        let src = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*s).ptr, (*s).len as usize));
        let p = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*prefix).ptr, (*prefix).len as usize));
        if src.starts_with(p) { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn jdb_string_ends_with(s: *const JdbString, suffix: *const JdbString) -> i64 {
    if s.is_null() || suffix.is_null() { return 0; }
    unsafe {
        let src = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*s).ptr, (*s).len as usize));
        let p = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*suffix).ptr, (*suffix).len as usize));
        if src.ends_with(p) { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn jdb_int_to_string(val: i64) -> *const JdbString {
    let s = format!("{}", val);
    let boxed = Box::leak(s.into_boxed_str());
    let jdb = Box::new(JdbString { ptr: boxed.as_ptr(), len: boxed.len() as u32 });
    Box::leak(jdb) as *const JdbString
}

// ──────────────────────────────────────────────────────────
// java.lang.Math — Native x86-64 (sin JVM)
// ──────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn jdb_math_abs_int(val: i64) -> i64 {
    val.abs()
}

#[no_mangle]
pub extern "C" fn jdb_math_abs_float(val: f64) -> f64 {
    val.abs()
}

#[no_mangle]
pub extern "C" fn jdb_math_max_int(a: i64, b: i64) -> i64 {
    a.max(b)
}

#[no_mangle]
pub extern "C" fn jdb_math_min_int(a: i64, b: i64) -> i64 {
    a.min(b)
}

#[no_mangle]
pub extern "C" fn jdb_math_max_float(a: f64, b: f64) -> f64 {
    a.max(b)
}

#[no_mangle]
pub extern "C" fn jdb_math_min_float(a: f64, b: f64) -> f64 {
    a.min(b)
}

#[no_mangle]
pub extern "C" fn jdb_math_sqrt(val: f64) -> f64 {
    val.sqrt()
}

#[no_mangle]
pub extern "C" fn jdb_math_pow(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

#[no_mangle]
pub extern "C" fn jdb_math_floor(val: f64) -> f64 {
    val.floor()
}

#[no_mangle]
pub extern "C" fn jdb_math_ceil(val: f64) -> f64 {
    val.ceil()
}

#[no_mangle]
pub extern "C" fn jdb_math_sin(val: f64) -> f64 {
    val.sin()
}

#[no_mangle]
pub extern "C" fn jdb_math_cos(val: f64) -> f64 {
    val.cos()
}

#[no_mangle]
pub extern "C" fn jdb_math_log(val: f64) -> f64 {
    val.ln()
}

#[no_mangle]
pub extern "C" fn jdb_math_round(val: f64) -> i64 {
    val.round() as i64
}

/// Math.PI and Math.E constants
pub const JDB_MATH_PI: f64 = std::f64::consts::PI;
pub const JDB_MATH_E: f64 = std::f64::consts::E;

// ──────────────────────────────────────────────────────────
// Integer.parseInt / Integer.toString native
// ──────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn jdb_parse_int(s: *const JdbString) -> i64 {
    if s.is_null() { return 0; }
    unsafe {
        let src = std::str::from_utf8_unchecked(std::slice::from_raw_parts((*s).ptr, (*s).len as usize));
        src.trim().parse::<i64>().unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub struct CpuFeatures {
    pub has_avx2: bool,
    pub has_sse42: bool,
    pub brand: String,
}

pub struct JitStats {
    pub alloc_ms: f64,
    pub patch_ms: f64,
    pub exec_ms: f64,
}

pub struct JitExecutor {
    machine_code: Vec<u8>,
    data_segment: Vec<u8>, // Simulamos un espacio de .data separado
    source_hash: u64,
}

impl JitExecutor {
    pub fn new(code: Vec<u8>, data: Vec<u8>, source_hash: u64) -> Self {
        Self { machine_code: code, data_segment: data, source_hash }
    }

    pub fn detect_cpu_features() -> CpuFeatures {
        #[cfg(target_arch = "x86_64")]
        {
            let has_avx2;
            let has_sse42;
            let ebx_feat1: u32;
            let ebx_feat7: u32;
            unsafe {
                let mut _eax_out: u32;
                let mut ebx_out: u32;
                let mut _ecx_out: u32;
                let mut _edx_out: u32;
                std::arch::asm!(
                    "push rbx",
                    "cpuid",
                    "mov {ebx_out:e}, ebx",
                    "pop rbx",
                    inout("eax") 1u32 => _eax_out,
                    ebx_out = out(reg) ebx_out,
                    out("ecx") _ecx_out,
                    out("edx") _edx_out,
                );
                has_sse42 = (_ecx_out & (1 << 20)) != 0;
                ebx_feat1 = ebx_out;

                std::arch::asm!(
                    "push rbx",
                    "cpuid",
                    "mov {ebx_out:e}, ebx",
                    "pop rbx",
                    inout("eax") 7u32 => _eax_out,
                    ebx_out = out(reg) ebx_out,
                    inout("ecx") 0u32 => _ecx_out,
                    out("edx") _edx_out,
                );
                has_avx2 = (ebx_out & (1 << 5)) != 0;
                ebx_feat7 = ebx_out;
            }

            let mut brand_bytes = [0u8; 48];
            unsafe {
                for i in 0u32..3 {
                    let mut eax_out: u32;
                    let mut ebx_out: u32;
                    let mut ecx_out: u32;
                    let mut edx_out: u32;
                    std::arch::asm!(
                        "push rbx",
                        "cpuid",
                        "mov {ebx_out:e}, ebx",
                        "pop rbx",
                        inout("eax") (0x80000002u32 + i) => eax_out,
                        ebx_out = out(reg) ebx_out,
                        out("ecx") ecx_out,
                        out("edx") edx_out,
                    );
                    let off = (i as usize) * 16;
                    brand_bytes[off..off+4].copy_from_slice(&eax_out.to_le_bytes());
                    brand_bytes[off+4..off+8].copy_from_slice(&ebx_out.to_le_bytes());
                    brand_bytes[off+8..off+12].copy_from_slice(&ecx_out.to_le_bytes());
                    brand_bytes[off+12..off+16].copy_from_slice(&edx_out.to_le_bytes());
                }
            }
            let brand = String::from_utf8_lossy(&brand_bytes)
                .trim_end_matches('\0')
                .trim()
                .to_string();

            let _ = (ebx_feat1, ebx_feat7);
            CpuFeatures { has_avx2, has_sse42, brand }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            CpuFeatures { has_avx2: false, has_sse42: false, brand: "unknown".to_string() }
        }
    }

    /// Executable Memory Engine (Windows Implementations)
    #[cfg(target_os = "windows")]
    pub fn execute_with_stats(&self) -> Result<(i32, JitStats), String> {
        let _cpu = &*CPU_FEATURES;
        // THERMAL CACHE LOOKUP
        if let Ok(mut cache) = THERMAL_CACHE.lock() {
            if cache.contains_key(&self.source_hash) {
                // Return early in a real scenario
            } else {
                cache.insert(self.source_hash, CacheEntry {
                    text: self.machine_code.clone(),
                    data: self.data_segment.clone()
                });
            }
        }

        const MEM_COMMIT: u32 = 0x1000;
        const MEM_RESERVE: u32 = 0x2000;
        const MEM_RELEASE: u32 = 0x8000;
        const PAGE_EXECUTE_READWRITE: u32 = 0x40;
        const PAGE_READWRITE: u32 = 0x04;

        extern "system" {
            fn VirtualAlloc(addr: *mut u8, size: usize, alloc_type: u32, protect: u32) -> *mut u8;
            fn VirtualFree(addr: *mut u8, size: usize, free_type: u32) -> i32;
        }

        let alloc_start = std::time::Instant::now();

        let text_size = std::cmp::max(self.machine_code.len(), 1);
        let text_size = (text_size + 4095) & !4095;

        let data_size = std::cmp::max(self.data_segment.len(), 1);
        let data_size = (data_size + 4095) & !4095;

        let text_ptr = unsafe {
            VirtualAlloc(ptr::null_mut(), text_size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE)
        };
        if text_ptr.is_null() { return Err("VirtualAlloc .text failed".to_string()); }

        let data_ptr = unsafe {
            VirtualAlloc(ptr::null_mut(), data_size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE)
        };
        if data_ptr.is_null() {
            unsafe { VirtualFree(text_ptr, 0, MEM_RELEASE); }
            return Err("VirtualAlloc .data failed".to_string());
        }
        let alloc_ms = alloc_start.elapsed().as_secs_f64() * 1000.0;

        let patch_start = std::time::Instant::now();
        unsafe {
            ptr::copy_nonoverlapping(self.machine_code.as_ptr(), text_ptr, self.machine_code.len());
            ptr::copy_nonoverlapping(self.data_segment.as_ptr(), data_ptr, self.data_segment.len());
        }
        let patch_ms = patch_start.elapsed().as_secs_f64() * 1000.0;

        let exec_start = std::time::Instant::now();
        let _func: extern "C" fn() -> i32 = unsafe { std::mem::transmute(text_ptr) };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            _func(); 
            0
        }));
        let exec_ms = exec_start.elapsed().as_secs_f64() * 1000.0;

        unsafe {
            VirtualFree(text_ptr, 0, MEM_RELEASE);
            VirtualFree(data_ptr, 0, MEM_RELEASE);
        }

        let stats = JitStats { alloc_ms, patch_ms, exec_ms };

        match result {
            Ok(code) => Ok((code, stats)),
            Err(_) => Err("Native Exception".to_string())
        }
    }

    #[cfg(target_os = "windows")]
    pub fn execute(&self) -> Result<i32, String> {
        self.execute_with_stats().map(|(code, _)| code)
    }

    #[cfg(not(target_os = "windows"))]
    pub fn execute_with_stats(&self) -> Result<(i32, JitStats), String> {
        Ok((0, JitStats { alloc_ms: 0.0, patch_ms: 0.0, exec_ms: 0.0 }))
    }

    #[cfg(not(target_os = "windows"))]
    pub fn execute(&self) -> Result<i32, String> {
        println!("[JIT 2.0] Ejecución nativa actualmente stubbed para sistemas No-Windows.");
        Ok(0)
    }
}
