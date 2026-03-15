// ============================================================
// DirectX 12 Native Dispatch — JaDead-BIB 💀☕
// ============================================================
// HISTÓRICO: Primer Java con DX12 nativo
// Sin JVM, sin JNI, sin overhead
// Carga d3d12.dll + dxgi.dll via LoadLibraryA
// Lima, Perú 🇵🇪 — Binary Is Binary 💀🦈
// ============================================================

#[cfg(target_os = "windows")]
use std::ffi::CString;

/// DX12 context holding DLL handles and device
pub struct DX12Context {
    #[cfg(target_os = "windows")]
    d3d12_handle: usize,
    #[cfg(target_os = "windows")]
    dxgi_handle: usize,
    pub initialized: bool,
    pub feature_level: u32,
    pub adapter_name: String,
}

impl DX12Context {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "windows")]
            d3d12_handle: 0,
            #[cfg(target_os = "windows")]
            dxgi_handle: 0,
            initialized: false,
            feature_level: 0,
            adapter_name: String::new(),
        }
    }
}

// ── Global state ────────────────────────────────────────────
static mut DX12_CTX: Option<DX12Context> = None;

fn get_ctx() -> &'static mut DX12Context {
    unsafe {
        if DX12_CTX.is_none() {
            DX12_CTX = Some(DX12Context::new());
        }
        DX12_CTX.as_mut().unwrap()
    }
}

// ── DX12 constants ──────────────────────────────────────────
#[allow(dead_code)]
const D3D_FEATURE_LEVEL_11_0: u32 = 0xb000;
const D3D_FEATURE_LEVEL_12_0: u32 = 0xc000;
#[allow(dead_code)]
const D3D_FEATURE_LEVEL_12_1: u32 = 0xc100;
#[allow(dead_code)]
const D3D_FEATURE_LEVEL_12_2: u32 = 0xc200;

// ── Native runtime functions (called from JIT) ──────────────

/// Initialize DX12 — load d3d12.dll + dxgi.dll
#[no_mangle]
pub extern "C" fn jdb_dx12_init() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if ctx.initialized { return 1; }

        unsafe {
            // Load d3d12.dll
            let d3d12_name = CString::new("d3d12.dll").unwrap();
            let d3d12 = LoadLibraryA(d3d12_name.as_ptr());
            if d3d12 == 0 {
                eprintln!("💀 [GPU:DX12] Failed to load d3d12.dll");
                return 0;
            }

            // Load dxgi.dll
            let dxgi_name = CString::new("dxgi.dll").unwrap();
            let dxgi = LoadLibraryA(dxgi_name.as_ptr());
            if dxgi == 0 {
                eprintln!("💀 [GPU:DX12] Failed to load dxgi.dll");
                FreeLibrary(d3d12);
                return 0;
            }

            ctx.d3d12_handle = d3d12;
            ctx.dxgi_handle = dxgi;
            ctx.initialized = true;
            eprintln!("✅ [GPU:DX12] d3d12.dll loaded at 0x{:X}", d3d12);
            eprintln!("✅ [GPU:DX12] dxgi.dll loaded at 0x{:X}", dxgi);
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("💀 [GPU:DX12] DirectX 12 is Windows-only");
        0
    }
}

/// Get DX12 feature level (returns e.g. 120 for D3D_FEATURE_LEVEL_12_0)
#[no_mangle]
pub extern "C" fn jdb_dx12_get_feature_level() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if !ctx.initialized {
            if jdb_dx12_init() == 0 { return 0; }
        }

        unsafe {
            // Try D3D12CreateDevice with nullptr adapter to test feature level
            let proc_name = CString::new("D3D12CreateDevice").unwrap();
            let proc = GetProcAddress(ctx.d3d12_handle, proc_name.as_ptr());
            if proc == 0 {
                eprintln!("💀 [GPU:DX12] D3D12CreateDevice not found");
                return 0;
            }

            // D3D12CreateDevice(nullptr, D3D_FEATURE_LEVEL_12_0, IID_ID3D12Device, nullptr)
            // We call with ppDevice=nullptr to just test if the feature level is supported
            type D3D12CreateDeviceFn = extern "system" fn(
                adapter: usize,        // IUnknown* pAdapter (nullptr = default)
                feature_level: u32,    // D3D_FEATURE_LEVEL
                riid: *const [u8; 16], // REFIID (IID_ID3D12Device)
                pp_device: *mut usize, // void** ppDevice
            ) -> i32; // HRESULT

            let create_device: D3D12CreateDeviceFn = std::mem::transmute(proc);

            // IID_ID3D12Device = {189819f1-1db6-4b57-be54-1821339b85f7}
            let iid: [u8; 16] = [
                0xf1, 0x19, 0x98, 0x18, 0xb6, 0x1d, 0x57, 0x4b,
                0xbe, 0x54, 0x18, 0x21, 0x33, 0x9b, 0x85, 0xf7,
            ];

            let mut device: usize = 0;
            let hr = create_device(0, D3D_FEATURE_LEVEL_12_0, &iid, &mut device);

            if hr >= 0 {
                // Success! Feature level 12.0 supported
                ctx.feature_level = D3D_FEATURE_LEVEL_12_0;

                // Release the device if we got one
                if device != 0 {
                    // Call IUnknown::Release (vtable index 2)
                    let vtable = *(device as *const *const usize);
                    let release: extern "system" fn(usize) -> u32 = std::mem::transmute(*vtable.add(2));
                    release(device);
                }

                eprintln!("✅ [GPU:DX12] Feature Level: 12_0 (0x{:X})", D3D_FEATURE_LEVEL_12_0);
                return 120;
            } else {
                eprintln!("💀 [GPU:DX12] D3D12CreateDevice failed (HRESULT=0x{:X})", hr as u32);
                return 0;
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    { 0 }
}

/// Get DXGI adapter info (GPU name)
#[no_mangle]
pub extern "C" fn jdb_dx12_get_adapter() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if !ctx.initialized { return 0; }

        unsafe {
            // CreateDXGIFactory1
            let proc_name = CString::new("CreateDXGIFactory1").unwrap();
            let proc = GetProcAddress(ctx.dxgi_handle, proc_name.as_ptr());
            if proc == 0 {
                eprintln!("💀 [GPU:DX12] CreateDXGIFactory1 not found");
                return 0;
            }

            // IID_IDXGIFactory1 = {770aae78-f26f-4dba-a829-253c83d1b387}
            let iid_factory: [u8; 16] = [
                0x78, 0xae, 0x0a, 0x77, 0x6f, 0xf2, 0xba, 0x4d,
                0xa8, 0x29, 0x25, 0x3c, 0x83, 0xd1, 0xb3, 0x87,
            ];

            type CreateFactoryFn = extern "system" fn(
                riid: *const [u8; 16],
                pp_factory: *mut usize,
            ) -> i32;

            let create_factory: CreateFactoryFn = std::mem::transmute(proc);
            let mut factory: usize = 0;
            let hr = create_factory(&iid_factory, &mut factory);

            if hr < 0 || factory == 0 {
                eprintln!("💀 [GPU:DX12] CreateDXGIFactory1 failed");
                return 0;
            }

            // EnumAdapters (vtable index 7 for IDXGIFactory1)
            let vtable = *(factory as *const *const usize);
            let enum_adapters: extern "system" fn(usize, u32, *mut usize) -> i32 =
                std::mem::transmute(*vtable.add(7));

            let mut adapter: usize = 0;
            let hr2 = enum_adapters(factory, 0, &mut adapter);

            if hr2 >= 0 && adapter != 0 {
                // GetDesc (vtable index 8 for IDXGIAdapter)
                // DXGI_ADAPTER_DESC is 304 bytes, Description at offset 0 (128 wchar_t)
                let adapter_vtable = *(adapter as *const *const usize);
                let get_desc: extern "system" fn(usize, *mut [u8; 304]) -> i32 =
                    std::mem::transmute(*adapter_vtable.add(8));

                let mut desc = [0u8; 304];
                let hr3 = get_desc(adapter, &mut desc);

                if hr3 >= 0 {
                    // Description is at offset 0, 128 wchar_t (256 bytes)
                    let wide_ptr = desc.as_ptr() as *const u16;
                    let mut name = String::new();
                    for i in 0..128 {
                        let ch = *wide_ptr.add(i);
                        if ch == 0 { break; }
                        if let Some(c) = char::from_u32(ch as u32) {
                            name.push(c);
                        }
                    }
                    ctx.adapter_name = name.clone();
                    eprintln!("🖥️ [GPU:DX12] Adapter: {}", name);
                }

                // Release adapter
                let release: extern "system" fn(usize) -> u32 =
                    std::mem::transmute(*adapter_vtable.add(2));
                release(adapter);
            }

            // Release factory
            let release_factory: extern "system" fn(usize) -> u32 =
                std::mem::transmute(*vtable.add(2));
            release_factory(factory);

            return 1;
        }
    }

    #[cfg(not(target_os = "windows"))]
    { 0 }
}

/// Destroy DX12 context
#[no_mangle]
pub extern "C" fn jdb_dx12_destroy() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if !ctx.initialized { return 0; }

        unsafe {
            if ctx.d3d12_handle != 0 {
                FreeLibrary(ctx.d3d12_handle);
                ctx.d3d12_handle = 0;
            }
            if ctx.dxgi_handle != 0 {
                FreeLibrary(ctx.dxgi_handle);
                ctx.dxgi_handle = 0;
            }
        }
        ctx.initialized = false;
        ctx.feature_level = 0;
        eprintln!("🔥 [GPU:DX12] Context destroyed");
        1
    }

    #[cfg(not(target_os = "windows"))]
    { 0 }
}

/// Check if DX12 is available (JIT-callable)
#[no_mangle]
pub extern "C" fn jdb_dx12_is_available() -> i64 {
    if is_available() { 1 } else { 0 }
}

/// Check if DX12 is available on this system
pub fn is_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let d3d12_name = CString::new("d3d12.dll").unwrap();
            let handle = LoadLibraryA(d3d12_name.as_ptr());
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
    fn test_dx12_context_creation() {
        let ctx = DX12Context::new();
        assert!(!ctx.initialized);
        assert_eq!(ctx.feature_level, 0);
        assert!(ctx.adapter_name.is_empty());
    }

    #[test]
    fn test_dx12_is_available() {
        // On Windows with DX12 this should be true
        let _ = is_available();
    }
}
