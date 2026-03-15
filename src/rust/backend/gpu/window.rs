// ============================================================
// Native Window System — JaDead-BIB 💀☕
// ============================================================
// Win32 CreateWindowExA + message pump
// Java crea ventanas nativas sin JVM, sin Swing, sin JavaFX
// Directo a user32.dll + gdi32.dll
// ============================================================

#[cfg(target_os = "windows")]
use std::ffi::CString;

/// Native window handle
pub struct NativeWindow {
    #[cfg(target_os = "windows")]
    pub hwnd: usize,
    #[cfg(target_os = "windows")]
    pub hdc: usize,
    pub width: i32,
    pub height: i32,
    pub title: String,
    pub should_close: bool,
    pub initialized: bool,
}

impl NativeWindow {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "windows")]
            hwnd: 0,
            #[cfg(target_os = "windows")]
            hdc: 0,
            width: 800,
            height: 600,
            title: String::new(),
            should_close: false,
            initialized: false,
        }
    }
}

static mut WINDOW: Option<NativeWindow> = None;

fn get_win() -> &'static mut NativeWindow {
    unsafe {
        if WINDOW.is_none() {
            WINDOW = Some(NativeWindow::new());
        }
        WINDOW.as_mut().unwrap()
    }
}

// ── Win32 constants ─────────────────────────────────────────
#[cfg(target_os = "windows")]
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
#[cfg(target_os = "windows")]
const WS_VISIBLE: u32 = 0x10000000;
#[cfg(target_os = "windows")]
const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;
#[cfg(target_os = "windows")]
const WM_DESTROY: u32 = 0x0002;
#[cfg(target_os = "windows")]
const WM_CLOSE: u32 = 0x0010;
#[cfg(target_os = "windows")]
const WM_PAINT: u32 = 0x000F;
#[cfg(target_os = "windows")]
const WM_KEYDOWN: u32 = 0x0100;
#[cfg(target_os = "windows")]
const VK_ESCAPE: usize = 0x1B;
#[cfg(target_os = "windows")]
const CS_HREDRAW: u32 = 0x0002;
#[cfg(target_os = "windows")]
const CS_VREDRAW: u32 = 0x0001;
#[cfg(target_os = "windows")]
const CS_OWNDC: u32 = 0x0020;
#[cfg(target_os = "windows")]
const COLOR_WINDOW: u32 = 5;
#[cfg(target_os = "windows")]
const IDC_ARROW: *const i8 = 32512 as *const i8;
#[cfg(target_os = "windows")]
const PM_REMOVE: u32 = 0x0001;
#[cfg(target_os = "windows")]
const SW_SHOW: i32 = 5;

// ── MSG struct ──────────────────────────────────────────────
#[cfg(target_os = "windows")]
#[repr(C)]
struct MSG {
    hwnd: usize,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt_x: i32,
    pt_y: i32,
}

#[cfg(target_os = "windows")]
impl MSG {
    fn zeroed() -> Self {
        Self { hwnd: 0, message: 0, wparam: 0, lparam: 0, time: 0, pt_x: 0, pt_y: 0 }
    }
}

// ── WNDCLASSEXA struct ──────────────────────────────────────
#[cfg(target_os = "windows")]
#[repr(C)]
struct WNDCLASSEXA {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: unsafe extern "system" fn(usize, u32, usize, isize) -> isize,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: usize,
    h_icon: usize,
    h_cursor: usize,
    hbr_background: usize,
    lpsz_menu_name: *const i8,
    lpsz_class_name: *const i8,
    h_icon_sm: usize,
}

// ── PAINTSTRUCT ─────────────────────────────────────────────
#[cfg(target_os = "windows")]
#[repr(C)]
struct PAINTSTRUCT {
    hdc: usize,
    f_erase: i32,
    rc_paint: [i32; 4],
    f_restore: i32,
    f_inc_update: i32,
    rgb_reserved: [u8; 32],
}

#[cfg(target_os = "windows")]
impl PAINTSTRUCT {
    fn zeroed() -> Self {
        Self {
            hdc: 0, f_erase: 0, rc_paint: [0; 4],
            f_restore: 0, f_inc_update: 0, rgb_reserved: [0; 32],
        }
    }
}

// ── WndProc ─────────────────────────────────────────────────
#[cfg(target_os = "windows")]
unsafe extern "system" fn wnd_proc(hwnd: usize, msg: u32, wparam: usize, lparam: isize) -> isize {
    match msg {
        WM_CLOSE => {
            let w = get_win();
            w.should_close = true;
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        WM_KEYDOWN => {
            if wparam == VK_ESCAPE {
                let w = get_win();
                w.should_close = true;
                DestroyWindow(hwnd);
            }
            DefWindowProcA(hwnd, msg, wparam, lparam)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            // Fill with dark background
            let brush = CreateSolidBrush(0x00201010); // dark teal-ish
            let rect = [0i32, 0, 800, 600];
            FillRect(hdc, rect.as_ptr(), brush);
            DeleteObject(brush);

            // Draw centered text
            SetBkMode(hdc, 1); // TRANSPARENT
            SetTextColor(hdc, 0x0000FF00); // Green
            let text = CString::new("JaDead-BIB GPU Native").unwrap();
            let text_rect = [200i32, 250, 600, 350];
            DrawTextA(hdc, text.as_ptr(), -1, text_rect.as_ptr(), 0x25); // DT_CENTER | DT_VCENTER | DT_SINGLELINE
            EndPaint(hwnd, &ps);
            0
        }
        _ => DefWindowProcA(hwnd, msg, wparam, lparam),
    }
}

// ── Public API ──────────────────────────────────────────────

/// Create a native Win32 window
#[no_mangle]
pub extern "C" fn jdb_window_create(width: i64, height: i64, title_ptr: *const crate::backend::jit::JdbString) -> i64 {
    #[cfg(target_os = "windows")]
    {
        let w = get_win();
        if w.initialized { return 1; }

        let title = if title_ptr.is_null() {
            "JaDead-BIB GPU".to_string()
        } else {
            unsafe {
                let jdb = &*title_ptr;
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(jdb.ptr, jdb.len as usize)).to_string()
            }
        };

        w.width = width as i32;
        w.height = height as i32;
        w.title = title.clone();

        unsafe {
            let h_instance = GetModuleHandleA(std::ptr::null());
            let class_name = CString::new("JaDeadBIB_GPU").unwrap();
            let window_title = CString::new(title.as_str()).unwrap();

            let wc = WNDCLASSEXA {
                cb_size: std::mem::size_of::<WNDCLASSEXA>() as u32,
                style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
                lpfn_wnd_proc: wnd_proc,
                cb_cls_extra: 0,
                cb_wnd_extra: 0,
                h_instance,
                h_icon: 0,
                h_cursor: LoadCursorA(0, IDC_ARROW),
                hbr_background: (COLOR_WINDOW + 1) as usize,
                lpsz_menu_name: std::ptr::null(),
                lpsz_class_name: class_name.as_ptr(),
                h_icon_sm: 0,
            };

            if RegisterClassExA(&wc) == 0 {
                eprintln!("💀 [Window] RegisterClassExA failed");
                return 0;
            }

            let hwnd = CreateWindowExA(
                0,
                class_name.as_ptr(),
                window_title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT, CW_USEDEFAULT,
                w.width, w.height,
                0, 0, h_instance, 0,
            );

            if hwnd == 0 {
                eprintln!("💀 [Window] CreateWindowExA failed");
                return 0;
            }

            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);

            w.hwnd = hwnd;
            w.hdc = GetDC(hwnd);
            w.initialized = true;

            eprintln!("✅ [Window] Created {}x{} '{}'", w.width, w.height, w.title);
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (width, height, title_ptr);
        let w = get_win();
        w.initialized = true;
        eprintln!("✅ [Window] Stub mode (non-Windows)");
        1
    }
}

/// Poll window events, return 1 if window should stay open
#[no_mangle]
pub extern "C" fn jdb_window_poll_events() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let w = get_win();
        if !w.initialized || w.should_close { return 0; }

        unsafe {
            let mut msg = MSG::zeroed();
            while PeekMessageA(&mut msg, 0, 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }

        if w.should_close { 0 } else { 1 }
    }

    #[cfg(not(target_os = "windows"))]
    { 0 }
}

/// Check if window should close
#[no_mangle]
pub extern "C" fn jdb_window_should_close() -> i64 {
    let w = get_win();
    if w.should_close { 1 } else { 0 }
}

/// Get window HDC (for OpenGL)
#[no_mangle]
pub extern "C" fn jdb_window_get_hdc() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let w = get_win();
        w.hdc as i64
    }

    #[cfg(not(target_os = "windows"))]
    { 0 }
}

/// Get window HWND
#[no_mangle]
pub extern "C" fn jdb_window_get_hwnd() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let w = get_win();
        w.hwnd as i64
    }

    #[cfg(not(target_os = "windows"))]
    { 0 }
}

/// Swap buffers (for double-buffered rendering)
#[no_mangle]
pub extern "C" fn jdb_window_swap_buffers() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let w = get_win();
        if !w.initialized { return 0; }
        unsafe { SwapBuffers(w.hdc); }
        1
    }

    #[cfg(not(target_os = "windows"))]
    { 1 }
}

/// Set window title
#[no_mangle]
pub extern "C" fn jdb_window_set_title(title_ptr: *const crate::backend::jit::JdbString) -> i64 {
    #[cfg(target_os = "windows")]
    {
        let w = get_win();
        if !w.initialized || title_ptr.is_null() { return 0; }
        unsafe {
            let jdb = &*title_ptr;
            let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(jdb.ptr, jdb.len as usize));
            let c_title = CString::new(s).unwrap();
            SetWindowTextA(w.hwnd, c_title.as_ptr());
            w.title = s.to_string();
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    { let _ = title_ptr; 1 }
}

/// Destroy the window
#[no_mangle]
pub extern "C" fn jdb_window_destroy() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let w = get_win();
        if !w.initialized { return 0; }
        unsafe {
            if w.hdc != 0 && w.hwnd != 0 {
                ReleaseDC(w.hwnd, w.hdc);
            }
            if w.hwnd != 0 {
                DestroyWindow(w.hwnd);
            }
        }
        w.hwnd = 0;
        w.hdc = 0;
        w.initialized = false;
        w.should_close = true;
        eprintln!("🔥 [Window] Destroyed");
        1
    }

    #[cfg(not(target_os = "windows"))]
    {
        let w = get_win();
        w.initialized = false;
        1
    }
}

/// Sleep for N milliseconds (for frame timing)
#[no_mangle]
pub extern "C" fn jdb_sleep_ms(ms: i64) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

/// Get elapsed time in milliseconds since program start
#[no_mangle]
pub extern "C" fn jdb_time_ms() -> i64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

// ── Win32 FFI ───────────────────────────────────────────────
#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn GetModuleHandleA(name: *const i8) -> usize;
    fn RegisterClassExA(wc: *const WNDCLASSEXA) -> u16;
    fn CreateWindowExA(
        ex_style: u32, class_name: *const i8, window_name: *const i8,
        style: u32, x: i32, y: i32, width: i32, height: i32,
        parent: usize, menu: usize, instance: usize, param: usize,
    ) -> usize;
    fn ShowWindow(hwnd: usize, cmd: i32) -> i32;
    fn UpdateWindow(hwnd: usize) -> i32;
    fn DestroyWindow(hwnd: usize) -> i32;
    fn PostQuitMessage(exit_code: i32);
    fn DefWindowProcA(hwnd: usize, msg: u32, wparam: usize, lparam: isize) -> isize;
    fn PeekMessageA(msg: *mut MSG, hwnd: usize, min: u32, max: u32, remove: u32) -> i32;
    fn TranslateMessage(msg: *const MSG) -> i32;
    fn DispatchMessageA(msg: *const MSG) -> isize;
    fn GetDC(hwnd: usize) -> usize;
    fn ReleaseDC(hwnd: usize, hdc: usize) -> i32;
    fn LoadCursorA(instance: usize, cursor: *const i8) -> usize;
    fn SetWindowTextA(hwnd: usize, title: *const i8) -> i32;
}

#[cfg(target_os = "windows")]
#[link(name = "gdi32")]
extern "system" {
    fn SwapBuffers(hdc: usize) -> i32;
    fn BeginPaint(hwnd: usize, ps: *mut PAINTSTRUCT) -> usize;
    fn EndPaint(hwnd: usize, ps: *const PAINTSTRUCT) -> i32;
    fn CreateSolidBrush(color: u32) -> usize;
    fn DeleteObject(obj: usize) -> i32;
    fn SetBkMode(hdc: usize, mode: i32) -> i32;
    fn SetTextColor(hdc: usize, color: u32) -> u32;
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn FillRect(hdc: usize, rect: *const i32, brush: usize) -> i32;
    fn DrawTextA(hdc: usize, text: *const i8, count: i32, rect: *const i32, format: u32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation_struct() {
        let w = NativeWindow::new();
        assert!(!w.initialized);
        assert!(!w.should_close);
        assert_eq!(w.width, 800);
        assert_eq!(w.height, 600);
    }

    #[test]
    fn test_time_ms() {
        let t = jdb_time_ms();
        assert!(t > 0);
    }
}
