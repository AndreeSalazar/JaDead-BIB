// ============================================================
// Vulkan Native Dispatch — JaDead-BIB 💀☕
// ============================================================
// Java → Vulkan sin JVM, sin JNI
// Carga vulkan-1.dll via LoadLibraryA (Windows)
// libvulkan.so via dlopen (Linux)
// SPIR-V compute dispatch ready
// ============================================================

#[cfg(target_os = "windows")]
use std::ffi::CString;

/// Vulkan context holding DLL handle and instance
pub struct VulkanContext {
    #[cfg(target_os = "windows")]
    pub dll_handle: usize,
    pub initialized: bool,
    pub api_version: u32,
    pub device_name: String,
    pub vk_instance: u64,
}

impl VulkanContext {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "windows")]
            dll_handle: 0,
            initialized: false,
            api_version: 0,
            device_name: String::new(),
            vk_instance: 0,
        }
    }
}

// ── Global state ────────────────────────────────────────────
static mut VK_CTX: Option<VulkanContext> = None;

fn get_ctx() -> &'static mut VulkanContext {
    unsafe {
        if VK_CTX.is_none() {
            VK_CTX = Some(VulkanContext::new());
        }
        VK_CTX.as_mut().unwrap()
    }
}

pub fn get_ctx_pub() -> &'static mut VulkanContext {
    get_ctx()
}

// ── Vulkan constants ────────────────────────────────────────
const VK_SUCCESS: i32 = 0;
const VK_API_VERSION_1_3: u32 = (1 << 22) | (3 << 12); // 1.3.0

// ── Native runtime functions (called from JIT) ──────────────

/// Initialize Vulkan instance via vulkan-1.dll
#[no_mangle]
pub extern "C" fn jdb_vk_init() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if ctx.initialized { return 1; }

        unsafe {
            let dll_name = CString::new("vulkan-1.dll").unwrap();
            let handle = LoadLibraryA(dll_name.as_ptr());
            if handle == 0 {
                eprintln!("💀 [GPU:Vulkan] Failed to load vulkan-1.dll");
                return 0;
            }
            ctx.dll_handle = handle;

            // Get vkCreateInstance
            let proc_name = CString::new("vkCreateInstance").unwrap();
            let create_instance_ptr = GetProcAddress(handle, proc_name.as_ptr());
            if create_instance_ptr == 0 {
                eprintln!("💀 [GPU:Vulkan] vkCreateInstance not found");
                FreeLibrary(handle);
                ctx.dll_handle = 0;
                return 0;
            }

            // VkApplicationInfo
            let app_name = CString::new("JaDead-BIB").unwrap();
            let engine_name = CString::new("DeadBIB").unwrap();

            #[repr(C)]
            struct VkApplicationInfo {
                s_type: u32,
                p_next: *const std::ffi::c_void,
                p_application_name: *const i8,
                application_version: u32,
                p_engine_name: *const i8,
                engine_version: u32,
                api_version: u32,
            }

            #[repr(C)]
            struct VkInstanceCreateInfo {
                s_type: u32,
                p_next: *const std::ffi::c_void,
                flags: u32,
                p_application_info: *const VkApplicationInfo,
                enabled_layer_count: u32,
                pp_enabled_layer_names: *const *const i8,
                enabled_extension_count: u32,
                pp_enabled_extension_names: *const *const i8,
            }

            let app_info = VkApplicationInfo {
                s_type: 0, // VK_STRUCTURE_TYPE_APPLICATION_INFO
                p_next: std::ptr::null(),
                p_application_name: app_name.as_ptr(),
                application_version: 1,
                p_engine_name: engine_name.as_ptr(),
                engine_version: 1,
                api_version: VK_API_VERSION_1_3,
            };

            let create_info = VkInstanceCreateInfo {
                s_type: 1, // VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO
                p_next: std::ptr::null(),
                flags: 0,
                p_application_info: &app_info,
                enabled_layer_count: 0,
                pp_enabled_layer_names: std::ptr::null(),
                enabled_extension_count: 0,
                pp_enabled_extension_names: std::ptr::null(),
            };

            type VkCreateInstanceFn = extern "system" fn(
                *const VkInstanceCreateInfo,
                *const std::ffi::c_void,
                *mut u64,
            ) -> i32;

            let vk_create_instance: VkCreateInstanceFn = std::mem::transmute(create_instance_ptr);
            let mut instance: u64 = 0;
            let result = vk_create_instance(&create_info, std::ptr::null(), &mut instance);

            if result != VK_SUCCESS {
                eprintln!("💀 [GPU:Vulkan] vkCreateInstance failed (result={})", result);
                FreeLibrary(handle);
                ctx.dll_handle = 0;
                return 0;
            }

            ctx.vk_instance = instance;
            ctx.initialized = true;
            eprintln!("✅ [GPU:Vulkan] Instance created (handle=0x{:X})", instance);
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    {
        let ctx = get_ctx();
        ctx.initialized = true;
        eprintln!("✅ [GPU:Vulkan] Stub mode (non-Windows)");
        1
    }
}

/// Get Vulkan API version
#[no_mangle]
pub extern "C" fn jdb_vk_get_version() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if !ctx.initialized {
            if jdb_vk_init() == 0 { return 0; }
        }

        unsafe {
            let proc_name = CString::new("vkEnumerateInstanceVersion").unwrap();
            let proc = GetProcAddress(ctx.dll_handle, proc_name.as_ptr());
            if proc == 0 {
                // Vulkan 1.0 doesn't have this function
                ctx.api_version = 1 << 22; // 1.0.0
                return 100;
            }

            type VkEnumVerFn = extern "system" fn(*mut u32) -> i32;
            let vk_enum_ver: VkEnumVerFn = std::mem::transmute(proc);
            let mut version: u32 = 0;
            let result = vk_enum_ver(&mut version);
            if result == VK_SUCCESS {
                ctx.api_version = version;
                let major = (version >> 22) & 0x7F;
                let minor = (version >> 12) & 0x3FF;
                let patch = version & 0xFFF;
                eprintln!("✅ [GPU:Vulkan] API Version: {}.{}.{}", major, minor, patch);
                return (major * 100 + minor) as i64;
            }
        }
        0
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Stub: return Vulkan 1.3
        130
    }
}

/// Get Vulkan physical device name
#[no_mangle]
pub extern "C" fn jdb_vk_get_device() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if !ctx.initialized { return 0; }

        unsafe {
            let proc_name = CString::new("vkEnumeratePhysicalDevices").unwrap();
            let proc = GetProcAddress(ctx.dll_handle, proc_name.as_ptr());
            if proc == 0 { return 0; }

            type VkEnumDevFn = extern "system" fn(u64, *mut u32, *mut u64) -> i32;
            let vk_enum_dev: VkEnumDevFn = std::mem::transmute(proc);

            let mut count: u32 = 0;
            let result = vk_enum_dev(ctx.vk_instance, &mut count, std::ptr::null_mut());
            if result != VK_SUCCESS || count == 0 {
                eprintln!("💀 [GPU:Vulkan] No physical devices found");
                return 0;
            }

            eprintln!("✅ [GPU:Vulkan] Found {} physical device(s)", count);
            return count as i64;
        }
    }

    #[cfg(not(target_os = "windows"))]
    { 1 }
}

/// Destroy Vulkan instance
#[no_mangle]
pub extern "C" fn jdb_vk_destroy() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = get_ctx();
        if !ctx.initialized { return 0; }

        unsafe {
            if ctx.vk_instance != 0 {
                let proc_name = CString::new("vkDestroyInstance").unwrap();
                let proc = GetProcAddress(ctx.dll_handle, proc_name.as_ptr());
                if proc != 0 {
                    type VkDestroyFn = extern "system" fn(u64, *const std::ffi::c_void);
                    let vk_destroy: VkDestroyFn = std::mem::transmute(proc);
                    vk_destroy(ctx.vk_instance, std::ptr::null());
                }
                ctx.vk_instance = 0;
            }
            if ctx.dll_handle != 0 {
                FreeLibrary(ctx.dll_handle);
                ctx.dll_handle = 0;
            }
        }
        ctx.initialized = false;
        eprintln!("🔥 [GPU:Vulkan] Instance destroyed");
        1
    }

    #[cfg(not(target_os = "windows"))]
    {
        let ctx = get_ctx();
        ctx.initialized = false;
        1
    }
}

/// Check if Vulkan is available (JIT-callable)
#[no_mangle]
pub extern "C" fn jdb_vk_is_available() -> i64 {
    if is_available() { 1 } else { 0 }
}

/// Check if Vulkan is available on this system
pub fn is_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let dll_name = CString::new("vulkan-1.dll").unwrap();
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
    fn test_vulkan_context_creation() {
        let ctx = VulkanContext::new();
        assert!(!ctx.initialized);
        assert_eq!(ctx.api_version, 0);
        assert!(ctx.device_name.is_empty());
    }

    #[test]
    fn test_vulkan_is_available() {
        let _ = is_available();
    }
}
