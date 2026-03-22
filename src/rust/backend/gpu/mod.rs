#[allow(static_mut_refs)]
pub mod opengl;
#[allow(static_mut_refs)]
pub mod vulkan;
#[allow(static_mut_refs)]
pub mod dx12;
#[allow(static_mut_refs)]
pub mod cuda;
#[allow(static_mut_refs)]
pub mod window;
#[allow(static_mut_refs)]
pub mod gl_cube;
#[allow(static_mut_refs)]
pub mod vk_cube;
#[allow(static_mut_refs)]
pub mod gpu_detect;
#[allow(static_mut_refs)]
pub mod compute;
#[allow(static_mut_refs)]
pub mod memory;
#[allow(static_mut_refs)]
pub mod metrics;
#[allow(static_mut_refs)]
pub mod scheduler;
#[allow(static_mut_refs)]
pub mod unified_pipeline;
#[allow(static_mut_refs)]
pub mod vulkan_runtime;
#[allow(static_mut_refs)]
pub mod hex;
#[allow(static_mut_refs)]
pub mod hip;
#[allow(static_mut_refs)]
pub mod spirv;

// ── GPU Auto-detect ─────────────────────────────────────────

/// Detect best available GPU backend at runtime
/// Priority: DX12 (Windows) → Vulkan → CUDA → OpenGL (fallback)
pub fn detect_best_backend() -> &'static str {
    if dx12::is_available() {
        return "DirectX12";
    }
    if vulkan::is_available() {
        return "Vulkan";
    }
    if cuda::is_available() {
        return "CUDA";
    }
    if opengl::is_available() {
        return "OpenGL";
    }
    "None"
}

/// JIT-callable auto-detect
#[no_mangle]
pub extern "C" fn jdb_gpu_detect_best() -> i64 {
    let backend = detect_best_backend();
    match backend {
        "DirectX12" => { eprintln!("✅ [GPU:Auto] Best backend: DirectX12"); 4 }
        "Vulkan"    => { eprintln!("✅ [GPU:Auto] Best backend: Vulkan"); 3 }
        "CUDA"      => { eprintln!("✅ [GPU:Auto] Best backend: CUDA"); 2 }
        "OpenGL"    => { eprintln!("✅ [GPU:Auto] Best backend: OpenGL"); 1 }
        _           => { eprintln!("💀 [GPU:Auto] No GPU backend available"); 0 }
    }
}

/// Get backend name as string for Java
#[no_mangle]
pub extern "C" fn jdb_gpu_detect_best_name() -> *const crate::backend::jit::JdbString {
    let name = detect_best_backend();
    let boxed = Box::leak(name.to_string().into_boxed_str());
    let jdb = Box::new(crate::backend::jit::JdbString {
        ptr: boxed.as_ptr(),
        len: boxed.len() as u32,
    });
    Box::leak(jdb) as *const crate::backend::jit::JdbString
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_best_returns_string() {
        let backend = detect_best_backend();
        assert!(["DirectX12", "Vulkan", "CUDA", "OpenGL", "None"].contains(&backend));
    }

    #[test]
    fn test_gpu_backend_priority() {
        // On a Windows machine with RTX 3060, DX12 should be available
        // This test just verifies the function doesn't crash
        let result = jdb_gpu_detect_best();
        assert!(result >= 0 && result <= 4);
    }
}
