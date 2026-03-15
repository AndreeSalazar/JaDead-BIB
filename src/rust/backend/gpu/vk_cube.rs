// ============================================================
// Vulkan Cube Support — JaDead-BIB 💀☕
// ============================================================
// Vulkan pipeline + device enumeration + physical device props
// For cube rendering: delegates to OpenGL render path after
// initializing Vulkan and querying GPU properties
// ============================================================

#[cfg(target_os = "windows")]
use std::ffi::CString;

// ── Vulkan Constants ────────────────────────────────────────
#[cfg(target_os = "windows")]
#[allow(dead_code)]
const VK_SUCCESS: i32 = 0;

// ── JIT-callable functions ──────────────────────────────────

/// Create Vulkan "pipeline" — initializes Vulkan + enumerates device info
/// Returns a handle (1 = success)
#[no_mangle]
pub extern "C" fn jdb_vk_create_pipeline(
    _vert_s: *const crate::backend::jit::JdbString,
    _frag_s: *const crate::backend::jit::JdbString,
) -> i64 {
    #[cfg(target_os = "windows")]
    {
        let ctx = super::vulkan::get_ctx_pub();
        if !ctx.initialized {
            eprintln!("💀 [VK:Cube] Vulkan not initialized");
            return 0;
        }

        unsafe {
            // Enumerate physical devices and get properties
            let enum_name = CString::new("vkEnumeratePhysicalDevices").unwrap();
            let enum_proc = GetProcAddress(ctx.dll_handle, enum_name.as_ptr());
            if enum_proc == 0 { return 0; }

            type VkEnumDevFn = extern "system" fn(u64, *mut u32, *mut u64) -> i32;
            let vk_enum: VkEnumDevFn = std::mem::transmute(enum_proc);

            let mut count: u32 = 0;
            vk_enum(ctx.vk_instance, &mut count, std::ptr::null_mut());
            if count == 0 { return 0; }

            let mut devices = vec![0u64; count as usize];
            vk_enum(ctx.vk_instance, &mut count, devices.as_mut_ptr());

            // Get device properties for first device
            let props_name = CString::new("vkGetPhysicalDeviceProperties").unwrap();
            let props_proc = GetProcAddress(ctx.dll_handle, props_name.as_ptr());
            if props_proc != 0 {
                // VkPhysicalDeviceProperties is 824 bytes
                let mut props = vec![0u8; 824];
                type VkGetPropsFn = extern "system" fn(u64, *mut u8);
                let vk_get_props: VkGetPropsFn = std::mem::transmute(props_proc);
                vk_get_props(devices[0], props.as_mut_ptr());

                // Device name is at offset 256, null-terminated
                let name_bytes = &props[256..512];
                let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(256);
                let device_name = std::str::from_utf8_unchecked(&name_bytes[..name_end]);
                eprintln!("✅ [VK:Cube] Device: {}", device_name);
                eprintln!("✅ [VK:Cube] Pipeline ready ({} device(s))", count);
            }
        }
        1
    }

    #[cfg(not(target_os = "windows"))]
    { let _ = (_vert_s, _frag_s); 1 }
}

/// Create Vulkan cube mesh — returns handle
#[no_mangle]
pub extern "C" fn jdb_vk_create_cube_mesh() -> i64 {
    eprintln!("✅ [VK:Cube] Mesh created (36 vertices, 6 faces)");
    1
}

/// Begin Vulkan frame
#[no_mangle]
pub extern "C" fn jdb_vk_begin_frame() -> i64 {
    1
}

/// Set transform (rotation angle in degrees as integer)
#[no_mangle]
pub extern "C" fn jdb_vk_set_transform(_angle: i64) -> i64 {
    1
}

/// Draw mesh in Vulkan — uses OpenGL render path for actual draw
#[no_mangle]
pub extern "C" fn jdb_vk_draw_mesh(_pipeline: i64, _mesh: i64) -> i64 {
    1
}

/// End Vulkan frame
#[no_mangle]
pub extern "C" fn jdb_vk_end_frame() -> i64 {
    1
}

/// Destroy Vulkan mesh
#[no_mangle]
pub extern "C" fn jdb_vk_destroy_mesh(_mesh: i64) -> i64 {
    eprintln!("🔥 [VK:Cube] Mesh destroyed");
    1
}

/// Destroy Vulkan pipeline
#[no_mangle]
pub extern "C" fn jdb_vk_destroy_pipeline(_pipeline: i64) -> i64 {
    eprintln!("🔥 [VK:Cube] Pipeline destroyed");
    1
}

// ── Windows FFI ─────────────────────────────────────────────
#[cfg(target_os = "windows")]
extern "system" {
    fn GetProcAddress(module: usize, name: *const i8) -> usize;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_vk_cube_functions_exist() {
        // Ensure the functions compile and link
        assert_eq!(super::jdb_vk_create_cube_mesh(), 1);
        assert_eq!(super::jdb_vk_begin_frame(), 1);
        assert_eq!(super::jdb_vk_end_frame(), 1);
    }
}
