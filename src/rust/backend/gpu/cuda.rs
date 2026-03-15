// ============================================================
// CUDA Native Dispatch — JaDead-BIB 💀☕
// ============================================================
// Java → CUDA sin JVM, sin JNI
// Carga nvcuda.dll via LoadLibraryA (Windows)
// libcuda.so via dlopen (Linux)
// ============================================================

#[cfg(target_os = "windows")]
use std::ffi::CString;

/// CUDA context
pub struct CudaContext {
    #[cfg(target_os = "windows")]
    dll_handle: usize,
    pub initialized: bool,
    pub driver_version: i32,
}

impl CudaContext {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "windows")]
            dll_handle: 0,
            initialized: false,
            driver_version: 0,
        }
    }
}

static mut CUDA_CTX: Option<CudaContext> = None;

fn get_ctx() -> &'static mut CudaContext {
    unsafe {
        if CUDA_CTX.is_none() {
            CUDA_CTX = Some(CudaContext::new());
        }
        CUDA_CTX.as_mut().unwrap()
    }
}

/// Initialize CUDA via nvcuda.dll
#[no_mangle]
pub extern "C" fn jdb_cuda_init() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if ctx.initialized { return 1; }

        unsafe {
            let dll_name = CString::new("nvcuda.dll").unwrap();
            let handle = LoadLibraryA(dll_name.as_ptr());
            if handle == 0 {
                eprintln!("💀 [GPU:CUDA] Failed to load nvcuda.dll");
                return 0;
            }

            // cuInit(0)
            let proc_name = CString::new("cuInit").unwrap();
            let proc = GetProcAddress(handle, proc_name.as_ptr());
            if proc == 0 {
                eprintln!("💀 [GPU:CUDA] cuInit not found");
                FreeLibrary(handle);
                return 0;
            }

            type CuInitFn = extern "system" fn(u32) -> i32;
            let cu_init: CuInitFn = std::mem::transmute(proc);
            let result = cu_init(0);
            if result != 0 {
                eprintln!("💀 [GPU:CUDA] cuInit failed (result={})", result);
                FreeLibrary(handle);
                return 0;
            }

            ctx.dll_handle = handle;
            ctx.initialized = true;
            eprintln!("✅ [GPU:CUDA] nvcuda.dll loaded, cuInit OK");
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("💀 [GPU:CUDA] Stub mode (non-Windows)");
        0
    }
}

/// Get CUDA driver version
#[no_mangle]
pub extern "C" fn jdb_cuda_get_version() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if !ctx.initialized {
            if jdb_cuda_init() == 0 { return 0; }
        }

        unsafe {
            let proc_name = CString::new("cuDriverGetVersion").unwrap();
            let proc = GetProcAddress(ctx.dll_handle, proc_name.as_ptr());
            if proc == 0 { return 0; }

            type CuDriverVerFn = extern "system" fn(*mut i32) -> i32;
            let cu_ver: CuDriverVerFn = std::mem::transmute(proc);
            let mut version: i32 = 0;
            let result = cu_ver(&mut version);
            if result == 0 {
                ctx.driver_version = version;
                eprintln!("✅ [GPU:CUDA] Driver version: {}", version);
                return version as i64;
            }
        }
        0
    }

    #[cfg(not(target_os = "windows"))]
    { 0 }
}

/// Destroy CUDA context
#[no_mangle]
pub extern "C" fn jdb_cuda_destroy() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if !ctx.initialized { return 0; }

        unsafe {
            if ctx.dll_handle != 0 {
                FreeLibrary(ctx.dll_handle);
                ctx.dll_handle = 0;
            }
        }
        ctx.initialized = false;
        eprintln!("🔥 [GPU:CUDA] Context destroyed");
        1
    }

    #[cfg(not(target_os = "windows"))]
    { 0 }
}

/// Check if CUDA is available on this system
pub fn is_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let dll_name = CString::new("nvcuda.dll").unwrap();
            let handle = LoadLibraryA(dll_name.as_ptr());
            if handle != 0 {
                FreeLibrary(handle);
                return true;
            }
            false
        }
    }

    #[cfg(not(target_os = "windows"))]
    { false }
}

#[cfg(target_os = "windows")]
extern "system" {
    fn LoadLibraryA(name: *const i8) -> usize;
    fn GetProcAddress(module: usize, name: *const i8) -> usize;
    fn FreeLibrary(module: usize) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cuda_context_creation() {
        let ctx = CudaContext::new();
        assert!(!ctx.initialized);
        assert_eq!(ctx.driver_version, 0);
    }

    #[test]
    fn test_cuda_is_available() {
        let _ = is_available();
    }
}
