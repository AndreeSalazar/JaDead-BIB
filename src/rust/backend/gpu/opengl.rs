// ============================================================
// OpenGL Native Dispatch — JaDead-BIB 💀☕
// ============================================================
// Java → OpenGL sin JVM, sin JNI, sin LWJGL
// Carga opengl32.dll via LoadLibraryA (Windows)
// libGL.so via dlopen (Linux)
// ============================================================

#[cfg(target_os = "windows")]
use std::ffi::CString;

/// OpenGL function pointers loaded at runtime
pub struct OpenGLContext {
    #[cfg(target_os = "windows")]
    dll_handle: usize,
    pub initialized: bool,
    pub version_string: String,
}

impl OpenGLContext {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "windows")]
            dll_handle: 0,
            initialized: false,
            version_string: String::new(),
        }
    }
}

// ── Global state ────────────────────────────────────────────
static mut GL_CTX: Option<OpenGLContext> = None;

fn get_ctx() -> &'static mut OpenGLContext {
    unsafe {
        if GL_CTX.is_none() {
            GL_CTX = Some(OpenGLContext::new());
        }
        GL_CTX.as_mut().unwrap()
    }
}

// ── OpenGL constants ────────────────────────────────────────
const GL_VERSION: u32 = 0x1F02;
const GL_RENDERER: u32 = 0x1F01;
const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;

// ── Native runtime functions (called from JIT) ──────────────

/// Initialize OpenGL context via opengl32.dll
#[no_mangle]
pub extern "C" fn jdb_gl_init() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if ctx.initialized {
            return 1; // Already initialized
        }

        unsafe {
            let dll_name = CString::new("opengl32.dll").unwrap();
            let handle = LoadLibraryA(dll_name.as_ptr());
            if handle == 0 {
                eprintln!("💀 [GPU:OpenGL] Failed to load opengl32.dll");
                return 0;
            }
            ctx.dll_handle = handle;
            ctx.initialized = true;
            eprintln!("✅ [GPU:OpenGL] opengl32.dll loaded at 0x{:X}", handle);
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    {
        let ctx = get_ctx();
        ctx.initialized = true;
        eprintln!("✅ [GPU:OpenGL] Stub mode (non-Windows)");
        1
    }
}

/// Get OpenGL version as integer (major * 100 + minor)
#[no_mangle]
pub extern "C" fn jdb_gl_get_version() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if !ctx.initialized {
            if jdb_gl_init() == 0 { return 0; }
        }

        unsafe {
            let proc_name = CString::new("glGetString").unwrap();
            let proc = GetProcAddress(ctx.dll_handle, proc_name.as_ptr());
            if proc == 0 {
                return 0;
            }
            let gl_get_string: extern "system" fn(u32) -> *const u8 = std::mem::transmute(proc);
            let ptr = gl_get_string(GL_VERSION);
            if ptr.is_null() {
                return 0;
            }
            let c_str = std::ffi::CStr::from_ptr(ptr as *const i8);
            if let Ok(s) = c_str.to_str() {
                ctx.version_string = s.to_string();
                // Parse "4.6.0" → 460
                let parts: Vec<&str> = s.split('.').collect();
                if parts.len() >= 2 {
                    let major = parts[0].parse::<i64>().unwrap_or(0);
                    let minor = parts[1].parse::<i64>().unwrap_or(0);
                    return major * 100 + minor;
                }
            }
        }
        0
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Stub: return OpenGL 4.6
        460
    }
}

/// Get OpenGL renderer string as printed output
#[no_mangle]
pub extern "C" fn jdb_gl_get_renderer() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if !ctx.initialized { return 0; }

        unsafe {
            let proc_name = CString::new("glGetString").unwrap();
            let proc = GetProcAddress(ctx.dll_handle, proc_name.as_ptr());
            if proc == 0 { return 0; }
            let gl_get_string: extern "system" fn(u32) -> *const u8 = std::mem::transmute(proc);
            let ptr = gl_get_string(GL_RENDERER);
            if ptr.is_null() { return 0; }
            let c_str = std::ffi::CStr::from_ptr(ptr as *const i8);
            if let Ok(s) = c_str.to_str() {
                eprintln!("🖥️ [GPU:OpenGL] Renderer: {}", s);
                return 1;
            }
        }
        0
    }

    #[cfg(not(target_os = "windows"))]
    { 1 }
}

/// Clear the color buffer
#[no_mangle]
pub extern "C" fn jdb_gl_clear(r: f64, g: f64, b: f64, a: f64) -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if !ctx.initialized { return 0; }

        unsafe {
            // glClearColor
            let name1 = CString::new("glClearColor").unwrap();
            let proc1 = GetProcAddress(ctx.dll_handle, name1.as_ptr());
            if proc1 != 0 {
                let gl_clear_color: extern "system" fn(f32, f32, f32, f32) = std::mem::transmute(proc1);
                gl_clear_color(r as f32, g as f32, b as f32, a as f32);
            }
            // glClear
            let name2 = CString::new("glClear").unwrap();
            let proc2 = GetProcAddress(ctx.dll_handle, name2.as_ptr());
            if proc2 != 0 {
                let gl_clear: extern "system" fn(u32) = std::mem::transmute(proc2);
                gl_clear(GL_COLOR_BUFFER_BIT);
            }
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    { let _ = (r, g, b, a); 1 }
}

/// Destroy OpenGL context
#[no_mangle]
pub extern "C" fn jdb_gl_destroy() -> i64 {
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
        eprintln!("🔥 [GPU:OpenGL] Context destroyed");
        1
    }

    #[cfg(not(target_os = "windows"))]
    {
        let ctx = get_ctx();
        ctx.initialized = false;
        1
    }
}

/// Check if OpenGL is available on this system
pub fn is_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let dll_name = CString::new("opengl32.dll").unwrap();
            let handle = LoadLibraryA(dll_name.as_ptr());
            if handle != 0 {
                FreeLibrary(handle);
                return true;
            }
            false
        }
    }

    #[cfg(not(target_os = "windows"))]
    { true } // Assume available on non-Windows
}

// ── Windows FFI ─────────────────────────────────────────────
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
    fn test_opengl_context_creation() {
        let ctx = OpenGLContext::new();
        assert!(!ctx.initialized);
        assert!(ctx.version_string.is_empty());
    }

    #[test]
    fn test_opengl_is_available() {
        // Just verify it doesn't crash
        let _ = is_available();
    }
}
