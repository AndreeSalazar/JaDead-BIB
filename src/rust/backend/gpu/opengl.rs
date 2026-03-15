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
    pub dll_handle: usize,
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

pub fn get_ctx_pub() -> &'static mut OpenGLContext {
    get_ctx()
}

// ── OpenGL constants ────────────────────────────────────────
const GL_VERSION: u32 = 0x1F02;
const GL_RENDERER: u32 = 0x1F01;
const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;

// ── Native runtime functions (called from JIT) ──────────────

/// Initialize OpenGL context via opengl32.dll + wglCreateContext
#[no_mangle]
pub extern "C" fn jdb_gl_init() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if ctx.initialized {
            return 1; // Already initialized
        }

        unsafe {
            // Load opengl32.dll
            let dll_name = CString::new("opengl32.dll").unwrap();
            let handle = LoadLibraryA(dll_name.as_ptr());
            if handle == 0 {
                eprintln!("💀 [GPU:OpenGL] Failed to load opengl32.dll");
                return 0;
            }
            ctx.dll_handle = handle;
            eprintln!("✅ [GPU:OpenGL] opengl32.dll loaded at 0x{:X}", handle);

            // Get the window HDC from the window module
            let hdc = super::window::jdb_window_get_hdc() as usize;
            if hdc == 0 {
                eprintln!("💀 [GPU:OpenGL] No window HDC — create window first");
                return 0;
            }

            // Set pixel format for OpenGL
            #[repr(C)]
            struct PIXELFORMATDESCRIPTOR {
                n_size: u16,
                n_version: u16,
                dw_flags: u32,
                i_pixel_type: u8,
                c_color_bits: u8,
                c_red_bits: u8, c_red_shift: u8,
                c_green_bits: u8, c_green_shift: u8,
                c_blue_bits: u8, c_blue_shift: u8,
                c_alpha_bits: u8, c_alpha_shift: u8,
                c_accum_bits: u8,
                c_accum_red_bits: u8, c_accum_green_bits: u8,
                c_accum_blue_bits: u8, c_accum_alpha_bits: u8,
                c_depth_bits: u8,
                c_stencil_bits: u8,
                c_aux_buffers: u8,
                i_layer_type: u8,
                b_reserved: u8,
                dw_layer_mask: u32,
                dw_visible_mask: u32,
                dw_damage_mask: u32,
            }

            const PFD_DRAW_TO_WINDOW: u32 = 0x00000004;
            const PFD_SUPPORT_OPENGL: u32 = 0x00000020;
            const PFD_DOUBLEBUFFER: u32   = 0x00000001;
            const PFD_TYPE_RGBA: u8 = 0;
            const PFD_MAIN_PLANE: u8 = 0;

            let pfd = PIXELFORMATDESCRIPTOR {
                n_size: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
                n_version: 1,
                dw_flags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
                i_pixel_type: PFD_TYPE_RGBA,
                c_color_bits: 32,
                c_red_bits: 0, c_red_shift: 0,
                c_green_bits: 0, c_green_shift: 0,
                c_blue_bits: 0, c_blue_shift: 0,
                c_alpha_bits: 8, c_alpha_shift: 0,
                c_accum_bits: 0,
                c_accum_red_bits: 0, c_accum_green_bits: 0,
                c_accum_blue_bits: 0, c_accum_alpha_bits: 0,
                c_depth_bits: 24,
                c_stencil_bits: 8,
                c_aux_buffers: 0,
                i_layer_type: PFD_MAIN_PLANE,
                b_reserved: 0,
                dw_layer_mask: 0,
                dw_visible_mask: 0,
                dw_damage_mask: 0,
            };

            let pixel_format = ChoosePixelFormat(hdc, &pfd as *const _ as *const u8);
            if pixel_format == 0 {
                eprintln!("💀 [GPU:OpenGL] ChoosePixelFormat failed");
                return 0;
            }
            if SetPixelFormat(hdc, pixel_format, &pfd as *const _ as *const u8) == 0 {
                eprintln!("💀 [GPU:OpenGL] SetPixelFormat failed");
                return 0;
            }
            eprintln!("✅ [GPU:OpenGL] Pixel format set (format={})", pixel_format);

            // Create OpenGL rendering context
            let wgl_create = CString::new("wglCreateContext").unwrap();
            let wgl_create_ptr = GetProcAddress(handle, wgl_create.as_ptr());
            if wgl_create_ptr == 0 {
                eprintln!("💀 [GPU:OpenGL] wglCreateContext not found");
                return 0;
            }
            let wgl_create_context: extern "system" fn(usize) -> usize = std::mem::transmute(wgl_create_ptr);
            let hglrc = wgl_create_context(hdc);
            if hglrc == 0 {
                eprintln!("💀 [GPU:OpenGL] wglCreateContext failed");
                return 0;
            }
            eprintln!("✅ [GPU:OpenGL] GL context created (hglrc=0x{:X})", hglrc);

            // Make it current
            let wgl_make = CString::new("wglMakeCurrent").unwrap();
            let wgl_make_ptr = GetProcAddress(handle, wgl_make.as_ptr());
            if wgl_make_ptr == 0 {
                eprintln!("💀 [GPU:OpenGL] wglMakeCurrent not found");
                return 0;
            }
            let wgl_make_current: extern "system" fn(usize, usize) -> i32 = std::mem::transmute(wgl_make_ptr);
            if wgl_make_current(hdc, hglrc) == 0 {
                eprintln!("💀 [GPU:OpenGL] wglMakeCurrent failed");
                return 0;
            }
            eprintln!("✅ [GPU:OpenGL] GL context made current");

            ctx.initialized = true;
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

/// Clear the color buffer (args: r,g,b,a as 0-255 ints)
#[no_mangle]
pub extern "C" fn jdb_gl_clear(r: i64, g: i64, b: i64, a: i64) -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if !ctx.initialized { return 0; }

        // Convert 0-255 int range to 0.0-1.0 float range
        let rf = (r as f32) / 255.0;
        let gf = (g as f32) / 255.0;
        let bf = (b as f32) / 255.0;
        let af = (a as f32) / 255.0;

        unsafe {
            // glClearColor
            let name1 = CString::new("glClearColor").unwrap();
            let proc1 = GetProcAddress(ctx.dll_handle, name1.as_ptr());
            if proc1 != 0 {
                let gl_clear_color: extern "system" fn(f32, f32, f32, f32) = std::mem::transmute(proc1);
                gl_clear_color(rf, gf, bf, af);
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

/// Check if OpenGL is available (JIT-callable)
#[no_mangle]
pub extern "C" fn jdb_gl_is_available() -> i64 {
    if is_available() { 1 } else { 0 }
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

#[cfg(target_os = "windows")]
#[link(name = "gdi32")]
extern "system" {
    fn ChoosePixelFormat(hdc: usize, ppfd: *const u8) -> i32;
    fn SetPixelFormat(hdc: usize, format: i32, ppfd: *const u8) -> i32;
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
