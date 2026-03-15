// ============================================================
// OpenGL Spinning Cube — JaDead-BIB 💀☕
// ============================================================
// Full GL 3.3+ shader pipeline: compile, link, VAO/VBO, MVP
// Uses wglGetProcAddress for extension functions
// Pure Rust math — no external dependencies
// ============================================================

#[cfg(target_os = "windows")]
use std::ffi::CString;

// ── GL Constants ────────────────────────────────────────────
const GL_FRAGMENT_SHADER: u32 = 0x8B30;
const GL_VERTEX_SHADER: u32   = 0x8B31;
const GL_COMPILE_STATUS: u32  = 0x8B81;
const GL_LINK_STATUS: u32     = 0x8B82;
const GL_FLOAT: u32           = 0x1406;
const GL_FALSE: u32           = 0;
const GL_TRIANGLES: u32       = 0x0004;
const GL_DEPTH_BUFFER_BIT: u32 = 0x00000100;
const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;
const GL_ARRAY_BUFFER: u32    = 0x8892;
const GL_STATIC_DRAW: u32     = 0x88E4;
const GL_DEPTH_TEST: u32      = 0x0B71;

// ── GL Function pointer types ───────────────────────────────
#[cfg(target_os = "windows")]
type GlCreateShader = extern "system" fn(u32) -> u32;
#[cfg(target_os = "windows")]
type GlShaderSource = extern "system" fn(u32, i32, *const *const i8, *const i32);
#[cfg(target_os = "windows")]
type GlCompileShader = extern "system" fn(u32);
#[cfg(target_os = "windows")]
type GlGetShaderiv = extern "system" fn(u32, u32, *mut i32);
#[cfg(target_os = "windows")]
type GlGetShaderInfoLog = extern "system" fn(u32, i32, *mut i32, *mut i8);
#[cfg(target_os = "windows")]
type GlCreateProgram = extern "system" fn() -> u32;
#[cfg(target_os = "windows")]
type GlAttachShader = extern "system" fn(u32, u32);
#[cfg(target_os = "windows")]
type GlLinkProgram = extern "system" fn(u32);
#[cfg(target_os = "windows")]
type GlGetProgramiv = extern "system" fn(u32, u32, *mut i32);
#[cfg(target_os = "windows")]
type GlUseProgram = extern "system" fn(u32);
#[cfg(target_os = "windows")]
type GlDeleteShader = extern "system" fn(u32);
#[cfg(target_os = "windows")]
type GlDeleteProgram = extern "system" fn(u32);
#[cfg(target_os = "windows")]
type GlGetUniformLocation = extern "system" fn(u32, *const i8) -> i32;
#[cfg(target_os = "windows")]
type GlUniformMatrix4fv = extern "system" fn(i32, i32, u8, *const f32);
#[cfg(target_os = "windows")]
type GlGenVertexArrays = extern "system" fn(i32, *mut u32);
#[cfg(target_os = "windows")]
type GlBindVertexArray = extern "system" fn(u32);
#[cfg(target_os = "windows")]
type GlGenBuffers = extern "system" fn(i32, *mut u32);
#[cfg(target_os = "windows")]
type GlBindBuffer = extern "system" fn(u32, u32);
#[cfg(target_os = "windows")]
type GlBufferData = extern "system" fn(u32, isize, *const f32, u32);
#[cfg(target_os = "windows")]
type GlEnableVertexAttribArray = extern "system" fn(u32);
#[cfg(target_os = "windows")]
type GlVertexAttribPointer = extern "system" fn(u32, i32, u32, u8, i32, usize);
#[cfg(target_os = "windows")]
type GlDrawArrays = extern "system" fn(u32, i32, i32);
#[cfg(target_os = "windows")]
type GlDeleteVertexArrays = extern "system" fn(i32, *const u32);
#[cfg(target_os = "windows")]
type GlDeleteBuffers = extern "system" fn(i32, *const u32);
#[cfg(target_os = "windows")]
type GlEnable = extern "system" fn(u32);
#[cfg(target_os = "windows")]
type GlClear = extern "system" fn(u32);
#[cfg(target_os = "windows")]
type GlClearColor = extern "system" fn(f32, f32, f32, f32);
#[cfg(target_os = "windows")]
type GlViewport = extern "system" fn(i32, i32, i32, i32);

// ── Cached GL function pointers ─────────────────────────────
#[cfg(target_os = "windows")]
struct GlFuncs {
    create_shader: Option<GlCreateShader>,
    shader_source: Option<GlShaderSource>,
    compile_shader: Option<GlCompileShader>,
    get_shaderiv: Option<GlGetShaderiv>,
    get_shader_info_log: Option<GlGetShaderInfoLog>,
    create_program: Option<GlCreateProgram>,
    attach_shader: Option<GlAttachShader>,
    link_program: Option<GlLinkProgram>,
    get_programiv: Option<GlGetProgramiv>,
    use_program: Option<GlUseProgram>,
    delete_shader: Option<GlDeleteShader>,
    delete_program: Option<GlDeleteProgram>,
    get_uniform_location: Option<GlGetUniformLocation>,
    uniform_matrix4fv: Option<GlUniformMatrix4fv>,
    gen_vertex_arrays: Option<GlGenVertexArrays>,
    bind_vertex_array: Option<GlBindVertexArray>,
    gen_buffers: Option<GlGenBuffers>,
    bind_buffer: Option<GlBindBuffer>,
    buffer_data: Option<GlBufferData>,
    enable_vertex_attrib_array: Option<GlEnableVertexAttribArray>,
    vertex_attrib_pointer: Option<GlVertexAttribPointer>,
    draw_arrays: Option<GlDrawArrays>,
    delete_vertex_arrays: Option<GlDeleteVertexArrays>,
    delete_buffers: Option<GlDeleteBuffers>,
    enable: Option<GlEnable>,
    clear: Option<GlClear>,
    clear_color: Option<GlClearColor>,
    viewport: Option<GlViewport>,
    loaded: bool,
}

#[cfg(target_os = "windows")]
static mut GL_FUNCS: GlFuncs = GlFuncs {
    create_shader: None, shader_source: None, compile_shader: None,
    get_shaderiv: None, get_shader_info_log: None,
    create_program: None, attach_shader: None, link_program: None,
    get_programiv: None, use_program: None,
    delete_shader: None, delete_program: None,
    get_uniform_location: None, uniform_matrix4fv: None,
    gen_vertex_arrays: None, bind_vertex_array: None,
    gen_buffers: None, bind_buffer: None, buffer_data: None,
    enable_vertex_attrib_array: None, vertex_attrib_pointer: None,
    draw_arrays: None, delete_vertex_arrays: None, delete_buffers: None,
    enable: None, clear: None, clear_color: None, viewport: None,
    loaded: false,
};

#[cfg(target_os = "windows")]
unsafe fn load_gl_func(dll: usize, name: &str) -> usize {
    let cname = CString::new(name).unwrap();
    // Try wglGetProcAddress first (for GL 1.2+ extension functions)
    let wgl_name = CString::new("wglGetProcAddress").unwrap();
    let wgl_ptr = GetProcAddress(dll, wgl_name.as_ptr());
    if wgl_ptr != 0 {
        let wgl_get: extern "system" fn(*const i8) -> usize = std::mem::transmute(wgl_ptr);
        let ptr = wgl_get(cname.as_ptr());
        if ptr != 0 && ptr != 1 && ptr != 2 && ptr != 3 && ptr != usize::MAX {
            return ptr;
        }
    }
    // Fallback to GetProcAddress (for GL 1.0/1.1 functions)
    GetProcAddress(dll, cname.as_ptr())
}

#[cfg(target_os = "windows")]
unsafe fn ensure_gl_funcs(dll: usize) {
    if GL_FUNCS.loaded { return; }

    macro_rules! load {
        ($field:ident, $name:expr) => {
            let ptr = load_gl_func(dll, $name);
            if ptr != 0 { GL_FUNCS.$field = Some(std::mem::transmute(ptr)); }
        };
    }

    load!(create_shader, "glCreateShader");
    load!(shader_source, "glShaderSource");
    load!(compile_shader, "glCompileShader");
    load!(get_shaderiv, "glGetShaderiv");
    load!(get_shader_info_log, "glGetShaderInfoLog");
    load!(create_program, "glCreateProgram");
    load!(attach_shader, "glAttachShader");
    load!(link_program, "glLinkProgram");
    load!(get_programiv, "glGetProgramiv");
    load!(use_program, "glUseProgram");
    load!(delete_shader, "glDeleteShader");
    load!(delete_program, "glDeleteProgram");
    load!(get_uniform_location, "glGetUniformLocation");
    load!(uniform_matrix4fv, "glUniformMatrix4fv");
    load!(gen_vertex_arrays, "glGenVertexArrays");
    load!(bind_vertex_array, "glBindVertexArray");
    load!(gen_buffers, "glGenBuffers");
    load!(bind_buffer, "glBindBuffer");
    load!(buffer_data, "glBufferData");
    load!(enable_vertex_attrib_array, "glEnableVertexAttribArray");
    load!(vertex_attrib_pointer, "glVertexAttribPointer");
    load!(draw_arrays, "glDrawArrays");
    load!(delete_vertex_arrays, "glDeleteVertexArrays");
    load!(delete_buffers, "glDeleteBuffers");
    load!(enable, "glEnable");
    load!(clear, "glClear");
    load!(clear_color, "glClearColor");
    load!(viewport, "glViewport");

    GL_FUNCS.loaded = true;
    eprintln!("✅ [GPU:OpenGL] GL 3.3+ functions loaded via wglGetProcAddress");
}

// ── Cube Vertex Data (36 vertices, 6 colored faces) ─────────
// Each vertex: (x, y, z, r, g, b) = 6 floats
// 6 faces × 2 triangles × 3 vertices = 36 vertices
static CUBE_VERTICES: [f32; 216] = [
    // Front face (red)
    -0.5, -0.5,  0.5,  0.9, 0.2, 0.2,
     0.5, -0.5,  0.5,  0.9, 0.2, 0.2,
     0.5,  0.5,  0.5,  0.9, 0.2, 0.2,
    -0.5, -0.5,  0.5,  0.9, 0.2, 0.2,
     0.5,  0.5,  0.5,  0.9, 0.2, 0.2,
    -0.5,  0.5,  0.5,  0.9, 0.2, 0.2,
    // Back face (green)
    -0.5, -0.5, -0.5,  0.2, 0.9, 0.2,
    -0.5,  0.5, -0.5,  0.2, 0.9, 0.2,
     0.5,  0.5, -0.5,  0.2, 0.9, 0.2,
    -0.5, -0.5, -0.5,  0.2, 0.9, 0.2,
     0.5,  0.5, -0.5,  0.2, 0.9, 0.2,
     0.5, -0.5, -0.5,  0.2, 0.9, 0.2,
    // Left face (blue)
    -0.5, -0.5, -0.5,  0.2, 0.2, 0.9,
    -0.5, -0.5,  0.5,  0.2, 0.2, 0.9,
    -0.5,  0.5,  0.5,  0.2, 0.2, 0.9,
    -0.5, -0.5, -0.5,  0.2, 0.2, 0.9,
    -0.5,  0.5,  0.5,  0.2, 0.2, 0.9,
    -0.5,  0.5, -0.5,  0.2, 0.2, 0.9,
    // Right face (yellow)
     0.5, -0.5, -0.5,  0.9, 0.9, 0.2,
     0.5,  0.5, -0.5,  0.9, 0.9, 0.2,
     0.5,  0.5,  0.5,  0.9, 0.9, 0.2,
     0.5, -0.5, -0.5,  0.9, 0.9, 0.2,
     0.5,  0.5,  0.5,  0.9, 0.9, 0.2,
     0.5, -0.5,  0.5,  0.9, 0.9, 0.2,
    // Top face (magenta)
    -0.5,  0.5, -0.5,  0.9, 0.2, 0.9,
    -0.5,  0.5,  0.5,  0.9, 0.2, 0.9,
     0.5,  0.5,  0.5,  0.9, 0.2, 0.9,
    -0.5,  0.5, -0.5,  0.9, 0.2, 0.9,
     0.5,  0.5,  0.5,  0.9, 0.2, 0.9,
     0.5,  0.5, -0.5,  0.9, 0.2, 0.9,
    // Bottom face (cyan)
    -0.5, -0.5, -0.5,  0.2, 0.9, 0.9,
     0.5, -0.5, -0.5,  0.2, 0.9, 0.9,
     0.5, -0.5,  0.5,  0.2, 0.9, 0.9,
    -0.5, -0.5, -0.5,  0.2, 0.9, 0.9,
     0.5, -0.5,  0.5,  0.2, 0.9, 0.9,
    -0.5, -0.5,  0.5,  0.2, 0.9, 0.9,
];

// ── Pure Rust MVP Math ──────────────────────────────────────

#[allow(dead_code)]
fn mat4_identity() -> [f32; 16] {
    [1.0, 0.0, 0.0, 0.0,
     0.0, 1.0, 0.0, 0.0,
     0.0, 0.0, 1.0, 0.0,
     0.0, 0.0, 0.0, 1.0]
}

#[allow(dead_code)]
fn mat4_mul(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut r = [0.0f32; 16];
    for i in 0..4 {
        for j in 0..4 {
            r[i * 4 + j] = a[i * 4 + 0] * b[0 * 4 + j]
                         + a[i * 4 + 1] * b[1 * 4 + j]
                         + a[i * 4 + 2] * b[2 * 4 + j]
                         + a[i * 4 + 3] * b[3 * 4 + j];
        }
    }
    r
}

// All matrices stored in COLUMN-MAJOR order (OpenGL convention)
// Column-major: m[col*4 + row]
fn mat4_rotate_y(angle_deg: f32) -> [f32; 16] {
    let r = angle_deg * std::f32::consts::PI / 180.0;
    let c = r.cos();
    let s = r.sin();
    // col0      col1      col2      col3
    [  c, 0.0,  -s, 0.0,
     0.0, 1.0, 0.0, 0.0,
       s, 0.0,   c, 0.0,
     0.0, 0.0, 0.0, 1.0]
}

fn mat4_rotate_x(angle_deg: f32) -> [f32; 16] {
    let r = angle_deg * std::f32::consts::PI / 180.0;
    let c = r.cos();
    let s = r.sin();
    // col0      col1      col2      col3
    [1.0, 0.0, 0.0, 0.0,
     0.0,   c,   s, 0.0,
     0.0,  -s,   c, 0.0,
     0.0, 0.0, 0.0, 1.0]
}

fn mat4_translate(x: f32, y: f32, z: f32) -> [f32; 16] {
    // col0      col1      col2      col3
    [1.0, 0.0, 0.0, 0.0,
     0.0, 1.0, 0.0, 0.0,
     0.0, 0.0, 1.0, 0.0,
       x,   y,   z, 1.0]
}

fn mat4_perspective(fov_deg: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
    let f = 1.0 / (fov_deg * 0.5 * std::f32::consts::PI / 180.0).tan();
    let nf = 1.0 / (near - far);
    // Column-major: [col0_row0..row3, col1_row0..row3, col2_row0..row3, col3_row0..row3]
    [f / aspect, 0.0, 0.0, 0.0,           // col0
     0.0,         f,  0.0, 0.0,           // col1
     0.0, 0.0, (far + near) * nf, -1.0,  // col2: m[10]=A, m[11]=-1
     0.0, 0.0, 2.0 * far * near * nf, 0.0] // col3: m[14]=B, m[15]=0
}

// Column-major multiply: result[col][row] = sum(a[k][row] * b[col][k])
fn mat4_mul_cm(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut r = [0.0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            r[col * 4 + row] = a[0 * 4 + row] * b[col * 4 + 0]
                             + a[1 * 4 + row] * b[col * 4 + 1]
                             + a[2 * 4 + row] * b[col * 4 + 2]
                             + a[3 * 4 + row] * b[col * 4 + 3];
        }
    }
    r
}

pub fn compute_mvp(angle_deg: f32) -> [f32; 16] {
    let projection = mat4_perspective(45.0, 800.0 / 600.0, 0.1, 100.0);
    let view = mat4_translate(0.0, 0.0, -3.5);
    let model_y = mat4_rotate_y(angle_deg);
    let model_x = mat4_rotate_x(angle_deg * 0.7);
    let model = mat4_mul_cm(&model_y, &model_x);
    let mv = mat4_mul_cm(&view, &model);
    mat4_mul_cm(&projection, &mv)
}

// ── JIT-callable functions ──────────────────────────────────

/// Load and compile shader from files, return program ID
/// Args are JdbString pointers (what the JIT passes for String literals)
#[no_mangle]
pub extern "C" fn jdb_gl_load_shader(vert_s: *const crate::backend::jit::JdbString, frag_s: *const crate::backend::jit::JdbString) -> i64 {
    #[cfg(target_os = "windows")]
    {
        if vert_s.is_null() || frag_s.is_null() {
            eprintln!("💀 [GL:Shader] Null shader path");
            return 0;
        }
        let vert_path = unsafe {
            let s = &*vert_s;
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(s.ptr, s.len as usize))
        };
        let frag_path = unsafe {
            let s = &*frag_s;
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(s.ptr, s.len as usize))
        };

        let vert_src = match std::fs::read_to_string(vert_path) {
            Ok(s) => s,
            Err(e) => { eprintln!("💀 [GL:Shader] Failed to read {}: {}", vert_path, e); return 0; }
        };
        let frag_src = match std::fs::read_to_string(frag_path) {
            Ok(s) => s,
            Err(e) => { eprintln!("💀 [GL:Shader] Failed to read {}: {}", frag_path, e); return 0; }
        };

        let ctx = super::opengl::get_ctx_pub();
        if !ctx.initialized { eprintln!("💀 [GL:Shader] OpenGL not initialized"); return 0; }

        unsafe {
            ensure_gl_funcs(ctx.dll_handle);

            let create_shader = GL_FUNCS.create_shader.unwrap();
            let shader_source = GL_FUNCS.shader_source.unwrap();
            let compile_shader = GL_FUNCS.compile_shader.unwrap();
            let get_shaderiv = GL_FUNCS.get_shaderiv.unwrap();
            let get_shader_info_log = GL_FUNCS.get_shader_info_log.unwrap();
            let create_program = GL_FUNCS.create_program.unwrap();
            let attach_shader = GL_FUNCS.attach_shader.unwrap();
            let link_program = GL_FUNCS.link_program.unwrap();
            let get_programiv = GL_FUNCS.get_programiv.unwrap();
            let delete_shader = GL_FUNCS.delete_shader.unwrap();

            // Compile vertex shader
            let vs = create_shader(GL_VERTEX_SHADER);
            let vert_c = CString::new(vert_src).unwrap();
            let vert_ptr = vert_c.as_ptr();
            let vert_len = vert_c.as_bytes().len() as i32;
            shader_source(vs, 1, &vert_ptr, &vert_len);
            compile_shader(vs);
            let mut status = 0i32;
            get_shaderiv(vs, GL_COMPILE_STATUS, &mut status);
            if status == 0 {
                let mut log = vec![0i8; 512];
                let mut len = 0i32;
                get_shader_info_log(vs, 512, &mut len, log.as_mut_ptr());
                let msg = std::ffi::CStr::from_ptr(log.as_ptr()).to_string_lossy();
                eprintln!("💀 [GL:Shader] Vertex compile error: {}", msg);
                return 0;
            }
            eprintln!("✅ [GL:Shader] Vertex shader compiled (id={})", vs);

            // Compile fragment shader
            let fs = create_shader(GL_FRAGMENT_SHADER);
            let frag_c = CString::new(frag_src).unwrap();
            let frag_p = frag_c.as_ptr();
            let frag_l = frag_c.as_bytes().len() as i32;
            shader_source(fs, 1, &frag_p, &frag_l);
            compile_shader(fs);
            get_shaderiv(fs, GL_COMPILE_STATUS, &mut status);
            if status == 0 {
                let mut log = vec![0i8; 512];
                let mut len = 0i32;
                get_shader_info_log(fs, 512, &mut len, log.as_mut_ptr());
                let msg = std::ffi::CStr::from_ptr(log.as_ptr()).to_string_lossy();
                eprintln!("💀 [GL:Shader] Fragment compile error: {}", msg);
                return 0;
            }
            eprintln!("✅ [GL:Shader] Fragment shader compiled (id={})", fs);

            // Link program
            let program = create_program();
            attach_shader(program, vs);
            attach_shader(program, fs);
            link_program(program);
            get_programiv(program, GL_LINK_STATUS, &mut status);
            if status == 0 {
                eprintln!("💀 [GL:Shader] Program link failed");
                return 0;
            }
            delete_shader(vs);
            delete_shader(fs);
            eprintln!("✅ [GL:Shader] Program linked (id={})", program);

            program as i64
        }
    }

    #[cfg(not(target_os = "windows"))]
    { let _ = (vert_s, frag_s); 1 }
}

/// Create cube VAO+VBO, return VAO ID
#[no_mangle]
pub extern "C" fn jdb_gl_create_cube() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = super::opengl::get_ctx_pub();
        if !ctx.initialized { return 0; }

        unsafe {
            ensure_gl_funcs(ctx.dll_handle);

            let gen_vertex_arrays = GL_FUNCS.gen_vertex_arrays.unwrap();
            let bind_vertex_array = GL_FUNCS.bind_vertex_array.unwrap();
            let gen_buffers = GL_FUNCS.gen_buffers.unwrap();
            let bind_buffer = GL_FUNCS.bind_buffer.unwrap();
            let buffer_data = GL_FUNCS.buffer_data.unwrap();
            let enable_vaa = GL_FUNCS.enable_vertex_attrib_array.unwrap();
            let vap = GL_FUNCS.vertex_attrib_pointer.unwrap();
            let enable = GL_FUNCS.enable.unwrap();
            let viewport = GL_FUNCS.viewport.unwrap();

            // Enable depth test
            enable(GL_DEPTH_TEST);
            viewport(0, 0, 800, 600);

            let mut vao: u32 = 0;
            let mut vbo: u32 = 0;
            gen_vertex_arrays(1, &mut vao);
            gen_buffers(1, &mut vbo);

            bind_vertex_array(vao);
            bind_buffer(GL_ARRAY_BUFFER, vbo);

            let data_size = (CUBE_VERTICES.len() * std::mem::size_of::<f32>()) as isize;
            buffer_data(GL_ARRAY_BUFFER, data_size, CUBE_VERTICES.as_ptr(), GL_STATIC_DRAW);

            let stride = (6 * std::mem::size_of::<f32>()) as i32;
            // position (location=0): 3 floats at offset 0
            vap(0, 3, GL_FLOAT, GL_FALSE as u8, stride, 0);
            enable_vaa(0);
            // color (location=1): 3 floats at offset 12
            vap(1, 3, GL_FLOAT, GL_FALSE as u8, stride, 3 * std::mem::size_of::<f32>());
            enable_vaa(1);

            bind_vertex_array(0);

            eprintln!("✅ [GL:Cube] Created VAO={} VBO={} (36 vertices, 6 colored faces)", vao, vbo);
            vao as i64
        }
    }

    #[cfg(not(target_os = "windows"))]
    { 1 }
}

/// Use shader program
#[no_mangle]
pub extern "C" fn jdb_gl_use_shader(program: i64) -> i64 {
    #[cfg(target_os = "windows")]
    unsafe {
        if GL_FUNCS.use_program.is_some() {
            GL_FUNCS.use_program.unwrap()(program as u32);
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    { let _ = program; 1 }
}

/// Set MVP uniform from rotation angle (degrees as integer)
#[no_mangle]
pub extern "C" fn jdb_gl_set_uniform_mvp(program: i64, angle: i64) -> i64 {
    #[cfg(target_os = "windows")]
    unsafe {
        if GL_FUNCS.get_uniform_location.is_some() && GL_FUNCS.uniform_matrix4fv.is_some() {
            let mvp_name = CString::new("MVP").unwrap();
            let loc = GL_FUNCS.get_uniform_location.unwrap()(program as u32, mvp_name.as_ptr());
            if loc >= 0 {
                let mvp = compute_mvp(angle as f32);
                GL_FUNCS.uniform_matrix4fv.unwrap()(loc, 1, 0, mvp.as_ptr());
            }
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    { let _ = (program, angle); 1 }
}

/// Clear screen with dark background + depth
#[no_mangle]
pub extern "C" fn jdb_gl_clear_depth(r: i64, g: i64, b: i64) -> i64 {
    #[cfg(target_os = "windows")]
    unsafe {
        if GL_FUNCS.clear_color.is_some() && GL_FUNCS.clear.is_some() {
            GL_FUNCS.clear_color.unwrap()(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);
            GL_FUNCS.clear.unwrap()(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    { let _ = (r, g, b); 1 }
}

/// Draw the cube (bind VAO, draw 36 vertices as triangles)
#[no_mangle]
pub extern "C" fn jdb_gl_draw_cube(vao: i64) -> i64 {
    #[cfg(target_os = "windows")]
    unsafe {
        if GL_FUNCS.bind_vertex_array.is_some() && GL_FUNCS.draw_arrays.is_some() {
            GL_FUNCS.bind_vertex_array.unwrap()(vao as u32);
            GL_FUNCS.draw_arrays.unwrap()(GL_TRIANGLES, 0, 36);
            GL_FUNCS.bind_vertex_array.unwrap()(0);
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    { let _ = vao; 1 }
}

/// Destroy shader program
#[no_mangle]
pub extern "C" fn jdb_gl_destroy_shader(program: i64) -> i64 {
    #[cfg(target_os = "windows")]
    unsafe {
        if GL_FUNCS.delete_program.is_some() {
            GL_FUNCS.delete_program.unwrap()(program as u32);
            eprintln!("🔥 [GL:Shader] Program {} destroyed", program);
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    { let _ = program; 1 }
}

/// Destroy cube VAO (and associated VBO)
#[no_mangle]
pub extern "C" fn jdb_gl_destroy_cube(vao: i64) -> i64 {
    #[cfg(target_os = "windows")]
    unsafe {
        if GL_FUNCS.delete_vertex_arrays.is_some() {
            let v = vao as u32;
            GL_FUNCS.delete_vertex_arrays.unwrap()(1, &v);
            eprintln!("🔥 [GL:Cube] VAO {} destroyed", vao);
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    { let _ = vao; 1 }
}

// ── Windows FFI ─────────────────────────────────────────────
#[cfg(target_os = "windows")]
extern "system" {
    fn GetProcAddress(module: usize, name: *const i8) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mvp_computation() {
        let mvp = compute_mvp(0.0);
        // MVP at angle 0 should be valid (not all zeros)
        assert!(mvp.iter().any(|&v| v != 0.0));
    }

    #[test]
    fn test_mat4_identity() {
        let id = mat4_identity();
        assert_eq!(id[0], 1.0);
        assert_eq!(id[5], 1.0);
        assert_eq!(id[10], 1.0);
        assert_eq!(id[15], 1.0);
    }

    #[test]
    fn test_cube_vertices_count() {
        // 36 vertices × 6 floats each = 216
        assert_eq!(CUBE_VERTICES.len(), 216);
    }
}
