// ============================================================
// DirectX 12 PURE Cube Rendering — JaDead-BIB 💀☕
// ============================================================
// HISTÓRICO: Primer cubo Java renderizado con DX12 PURO
// Sin OpenGL — Sin JVM — Sin GC — Nativo x86-64
// Real: Device, SwapChain, CommandList, PSO, HLSL, VB, CBV
// Lima, Perú 🇵🇪 — Binary Is Binary 💀🦈
// ============================================================

#[cfg(target_os = "windows")]
use std::ffi::CString;

// ── DX12/DXGI Constants ────────────────────────────────────
#[cfg(target_os = "windows")]
const BUFFER_COUNT: u32 = 2;
#[cfg(target_os = "windows")]
const DXGI_FORMAT_R8G8B8A8_UNORM: u32 = 28;
#[cfg(target_os = "windows")]
const DXGI_USAGE_RENDER_TARGET_OUTPUT: u32 = 0x00000020;
#[cfg(target_os = "windows")]
const DXGI_SWAP_EFFECT_FLIP_DISCARD: u32 = 4;
#[cfg(target_os = "windows")]
const D3D12_COMMAND_LIST_TYPE_DIRECT: u32 = 0;
#[cfg(target_os = "windows")]
const D3D12_COMMAND_QUEUE_FLAG_NONE: u32 = 0;
#[cfg(target_os = "windows")]
const D3D12_DESCRIPTOR_HEAP_TYPE_RTV: u32 = 0;
#[cfg(target_os = "windows")]
const D3D12_DESCRIPTOR_HEAP_FLAG_NONE: u32 = 0;
#[cfg(target_os = "windows")]
const D3D12_RESOURCE_STATE_PRESENT: u32 = 0;
#[cfg(target_os = "windows")]
const D3D12_RESOURCE_STATE_RENDER_TARGET: u32 = 4;
#[cfg(target_os = "windows")]
const D3D12_RESOURCE_BARRIER_TYPE_TRANSITION: u32 = 0;
#[cfg(target_os = "windows")]
const D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES: u32 = 0xFFFFFFFF;
#[cfg(target_os = "windows")]
const D3D12_HEAP_TYPE_UPLOAD: u32 = 2;
#[cfg(target_os = "windows")]
const D3D12_RESOURCE_STATE_GENERIC_READ: u32 = 0x1 | 0x2 | 0x40 | 0x80 | 0x200 | 0x800;
#[cfg(target_os = "windows")]
const D3D12_RESOURCE_DIMENSION_BUFFER: u32 = 1;
#[cfg(target_os = "windows")]
const D3D12_TEXTURE_LAYOUT_ROW_MAJOR: u32 = 1;
#[cfg(target_os = "windows")]
const D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER: u32 = 0x1;
#[cfg(target_os = "windows")]
const D3D12_ROOT_PARAMETER_TYPE_CBV: u32 = 2;
#[cfg(target_os = "windows")]
const D3D12_SHADER_VISIBILITY_ALL: u32 = 0;
#[cfg(target_os = "windows")]
const D3D12_INPUT_CLASSIFICATION_PER_VERTEX: u32 = 0;
#[cfg(target_os = "windows")]
const D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE: u32 = 3;
#[cfg(target_os = "windows")]
const D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST: u32 = 4;
#[cfg(target_os = "windows")]
const D3D12_FILL_MODE_SOLID: u32 = 3;
#[cfg(target_os = "windows")]
const D3D12_CULL_MODE_BACK: u32 = 3;
#[cfg(target_os = "windows")]
const DXGI_SAMPLE_DESC_COUNT: u32 = 1;
#[cfg(target_os = "windows")]
const D3D12_FENCE_FLAG_NONE: u32 = 0;
#[cfg(target_os = "windows")]
const D3D12_COMPARISON_FUNC_LESS: u32 = 2;
#[cfg(target_os = "windows")]
const D3D12_DEPTH_WRITE_MASK_ALL: u32 = 1;
#[cfg(target_os = "windows")]
const DXGI_FORMAT_D32_FLOAT: u32 = 40;
#[cfg(target_os = "windows")]
const D3D12_DESCRIPTOR_HEAP_TYPE_DSV: u32 = 3;
#[cfg(target_os = "windows")]
const D3D12_RESOURCE_STATE_DEPTH_WRITE: u32 = 0x10;
#[cfg(target_os = "windows")]
const D3D12_HEAP_TYPE_DEFAULT: u32 = 1;
#[cfg(target_os = "windows")]
const D3D12_RESOURCE_DIMENSION_TEXTURE2D: u32 = 3;
#[cfg(target_os = "windows")]
const D3D12_CLEAR_FLAG_DEPTH: u32 = 1;

// ── IID constants ──────────────────────────────────────────
#[cfg(target_os = "windows")]
const IID_ID3D12DEVICE: [u8; 16] = [
    0xf1, 0x19, 0x98, 0x18, 0xb6, 0x1d, 0x57, 0x4b,
    0xbe, 0x54, 0x18, 0x21, 0x33, 0x9b, 0x85, 0xf7,
];
#[cfg(target_os = "windows")]
const IID_ID3D12COMMAND_QUEUE: [u8; 16] = [
    // {0EC870A6-5D7E-4C22-8CFC-5BAAE07616ED}
    0xa6, 0x70, 0xc8, 0x0e, 0x7e, 0x5d, 0x22, 0x4c,
    0x8c, 0xfc, 0x5b, 0xaa, 0xe0, 0x76, 0x16, 0xed,
];
#[cfg(target_os = "windows")]
const IID_IDXGI_FACTORY4: [u8; 16] = [
    // {50c83a1c-e072-4c48-87b0-3630fa36a6d0} = IDXGIFactory2
    0x1c, 0x3a, 0xc8, 0x50, 0x72, 0xe0, 0x48, 0x4c,
    0x87, 0xb0, 0x36, 0x30, 0xfa, 0x36, 0xa6, 0xd0,
];
#[cfg(target_os = "windows")]
#[allow(dead_code)]
const IID_IDXGI_SWAPCHAIN3: [u8; 16] = [
    0x81, 0x06, 0x08, 0x94, 0x0b, 0xf9, 0x34, 0x4d,
    0xa5, 0xc2, 0x2e, 0x9a, 0x49, 0xc2, 0xbe, 0xce, // IDXGISwapChain3 fake — we use base vtable
];
#[cfg(target_os = "windows")]
const IID_ID3D12_ROOT_SIGNATURE: [u8; 16] = [
    // {C54A6B66-72DF-4EE8-8BE5-A946A1429214}
    0x66, 0x6b, 0x4a, 0xc5, 0xdf, 0x72, 0xe8, 0x4e,
    0x8b, 0xe5, 0xa9, 0x46, 0xa1, 0x42, 0x92, 0x14,
];

// ── Cube vertex data (pos xyz + color rgb) ──────────────────
#[cfg(target_os = "windows")]
#[rustfmt::skip]
const CUBE_VERTICES: [f32; 216] = [
    // Front face (red)
    -0.5, -0.5,  0.5,  1.0, 0.2, 0.2,
     0.5, -0.5,  0.5,  1.0, 0.2, 0.2,
     0.5,  0.5,  0.5,  1.0, 0.2, 0.2,
    -0.5, -0.5,  0.5,  1.0, 0.2, 0.2,
     0.5,  0.5,  0.5,  1.0, 0.2, 0.2,
    -0.5,  0.5,  0.5,  1.0, 0.2, 0.2,
    // Back face (green)
    -0.5, -0.5, -0.5,  0.2, 1.0, 0.2,
    -0.5,  0.5, -0.5,  0.2, 1.0, 0.2,
     0.5,  0.5, -0.5,  0.2, 1.0, 0.2,
    -0.5, -0.5, -0.5,  0.2, 1.0, 0.2,
     0.5,  0.5, -0.5,  0.2, 1.0, 0.2,
     0.5, -0.5, -0.5,  0.2, 1.0, 0.2,
    // Top face (blue)
    -0.5,  0.5, -0.5,  0.2, 0.2, 1.0,
    -0.5,  0.5,  0.5,  0.2, 0.2, 1.0,
     0.5,  0.5,  0.5,  0.2, 0.2, 1.0,
    -0.5,  0.5, -0.5,  0.2, 0.2, 1.0,
     0.5,  0.5,  0.5,  0.2, 0.2, 1.0,
     0.5,  0.5, -0.5,  0.2, 0.2, 1.0,
    // Bottom face (yellow)
    -0.5, -0.5, -0.5,  1.0, 1.0, 0.2,
     0.5, -0.5, -0.5,  1.0, 1.0, 0.2,
     0.5, -0.5,  0.5,  1.0, 1.0, 0.2,
    -0.5, -0.5, -0.5,  1.0, 1.0, 0.2,
     0.5, -0.5,  0.5,  1.0, 1.0, 0.2,
    -0.5, -0.5,  0.5,  1.0, 1.0, 0.2,
    // Right face (magenta)
     0.5, -0.5, -0.5,  1.0, 0.2, 1.0,
     0.5,  0.5, -0.5,  1.0, 0.2, 1.0,
     0.5,  0.5,  0.5,  1.0, 0.2, 1.0,
     0.5, -0.5, -0.5,  1.0, 0.2, 1.0,
     0.5,  0.5,  0.5,  1.0, 0.2, 1.0,
     0.5, -0.5,  0.5,  1.0, 0.2, 1.0,
    // Left face (cyan)
    -0.5, -0.5, -0.5,  0.2, 1.0, 1.0,
    -0.5, -0.5,  0.5,  0.2, 1.0, 1.0,
    -0.5,  0.5,  0.5,  0.2, 1.0, 1.0,
    -0.5, -0.5, -0.5,  0.2, 1.0, 1.0,
    -0.5,  0.5,  0.5,  0.2, 1.0, 1.0,
    -0.5,  0.5, -0.5,  0.2, 1.0, 1.0,
];

// ── DX12 rendering state ────────────────────────────────────
#[cfg(target_os = "windows")]
struct DX12RenderState {
    device: usize,
    command_queue: usize,
    swap_chain: usize,
    rtv_heap: usize,
    dsv_heap: usize,
    render_targets: [usize; 2],
    depth_buffer: usize,
    cmd_allocator: usize,
    cmd_list: usize,
    fence: usize,
    fence_value: u64,
    fence_event: usize,
    rtv_descriptor_size: u32,
    frame_index: u32,
    root_signature: usize,
    pso: usize,
    vertex_buffer: usize,
    vb_view_buffer: usize,
    vb_view_size: u32,
    vb_view_stride: u32,
    cbv_buffer: usize,
    cbv_mapped: *mut u8,
    cbv_gpu_va: u64,
    initialized: bool,
}

#[cfg(target_os = "windows")]
impl DX12RenderState {
    fn new() -> Self {
        Self {
            device: 0, command_queue: 0, swap_chain: 0,
            rtv_heap: 0, dsv_heap: 0, render_targets: [0; 2], depth_buffer: 0,
            cmd_allocator: 0, cmd_list: 0,
            fence: 0, fence_value: 0, fence_event: 0,
            rtv_descriptor_size: 0, frame_index: 0,
            root_signature: 0, pso: 0,
            vertex_buffer: 0, vb_view_buffer: 0, vb_view_size: 0, vb_view_stride: 0,
            cbv_buffer: 0, cbv_mapped: std::ptr::null_mut(), cbv_gpu_va: 0,
            initialized: false,
        }
    }
}

#[cfg(target_os = "windows")]
static mut DX12_RENDER: Option<DX12RenderState> = None;

#[cfg(target_os = "windows")]
#[allow(static_mut_refs)]
fn get_render() -> &'static mut DX12RenderState {
    unsafe {
        if DX12_RENDER.is_none() {
            DX12_RENDER = Some(DX12RenderState::new());
        }
        DX12_RENDER.as_mut().unwrap()
    }
}

// ── COM helper: call Release (vtable index 2) ──────────────
#[cfg(target_os = "windows")]
unsafe fn com_release(obj: usize) {
    if obj != 0 {
        let vtable = *(obj as *const *const usize);
        let release: extern "system" fn(usize) -> u32 = std::mem::transmute(*vtable.add(2));
        release(obj);
    }
}

// ── COM helper: call a vtable method returning HRESULT ──────
#[cfg(target_os = "windows")]
macro_rules! vtable_call {
    ($obj:expr, $idx:expr, $fn_type:ty $(, $arg:expr)*) => {{
        let vt = *($obj as *const *const usize);
        let f: $fn_type = std::mem::transmute(*vt.add($idx));
        f($obj $(, $arg)*)
    }};
}

// ── MVP math (row-major for HLSL) ──────────────────────────
#[cfg(target_os = "windows")]
fn compute_mvp_rowmajor(angle_deg: f32) -> [f32; 16] {
    // Get column-major MVP from gl_cube
    let cm = super::gl_cube::compute_mvp(angle_deg);
    // Transpose to row-major for HLSL mul(MVP, pos)
    let mut rm = [0.0f32; 16];
    for r in 0..4 {
        for c in 0..4 {
            rm[r * 4 + c] = cm[c * 4 + r];
        }
    }
    rm
}

// ── Repr-C structs for DX12 API ─────────────────────────────

#[cfg(target_os = "windows")]
#[repr(C)]
struct D3D12CommandQueueDesc {
    list_type: u32,
    priority: i32,
    flags: u32,
    node_mask: u32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct DxgiSwapChainDesc1 {
    width: u32,
    height: u32,
    format: u32,
    stereo: i32,
    sample_count: u32,
    sample_quality: u32,
    buffer_usage: u32,
    buffer_count: u32,
    scaling: u32,
    swap_effect: u32,
    alpha_mode: u32,
    flags: u32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct D3D12DescriptorHeapDesc {
    heap_type: u32,
    num_descriptors: u32,
    flags: u32,
    node_mask: u32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct D3D12HeapProperties {
    heap_type: u32,
    cpu_page_property: u32,
    memory_pool_preference: u32,
    creation_node_mask: u32,
    visible_node_mask: u32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct D3D12ResourceDesc {
    dimension: u32,
    alignment: u64,
    width: u64,
    height: u32,
    depth_or_array_size: u16,
    mip_levels: u16,
    format: u32,
    sample_count: u32,
    sample_quality: u32,
    layout: u32,
    flags: u32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct D3D12ResourceBarrier {
    barrier_type: u32,
    flags: u32,
    transition: D3D12ResourceTransitionBarrier,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct D3D12ResourceTransitionBarrier {
    resource: usize,
    subresource: u32,
    state_before: u32,
    state_after: u32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct D3D12VertexBufferView {
    buffer_location: u64,
    size_in_bytes: u32,
    stride_in_bytes: u32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct D3D12Viewport {
    top_left_x: f32,
    top_left_y: f32,
    width: f32,
    height: f32,
    min_depth: f32,
    max_depth: f32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct D3D12Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

// CPU descriptor handle
#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct D3D12CpuDescriptorHandle {
    ptr: u64,
}

// GPU descriptor handle
#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct D3D12GpuDescriptorHandle {
    ptr: u64,
}

// Root parameter for root signature
// D3D12_ROOT_PARAMETER has a union that's 16 bytes (largest member is DescriptorTable with 2 pointers)
// Layout: ParameterType(4) + pad(4) + Union(16) + ShaderVisibility(4) + pad(4) = 32 bytes
#[cfg(target_os = "windows")]
#[repr(C)]
struct D3D12RootParameter {
    parameter_type: u32,
    _pad0: u32,
    // Union: for CBV/SRV/UAV descriptor this is { ShaderRegister: u32, RegisterSpace: u32 }
    shader_register: u32,
    register_space: u32,
    _union_pad: [u64; 1], // Pad to 16 bytes for union
    shader_visibility: u32,
    _pad1: u32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct D3D12RootSignatureDesc {
    num_parameters: u32,
    _pad0: u32,
    p_parameters: *const D3D12RootParameter,
    num_static_samplers: u32,
    _pad1: u32,
    p_static_samplers: *const u8,
    flags: u32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct D3D12InputElementDesc {
    semantic_name: *const i8,
    semantic_index: u32,
    format: u32,
    input_slot: u32,
    aligned_byte_offset: u32,
    input_slot_class: u32,
    instance_data_step_rate: u32,
}

// Simplified GRAPHICS_PIPELINE_STATE_DESC — 600+ bytes, zeroed then filled
#[cfg(target_os = "windows")]
const GRAPHICS_PSO_DESC_SIZE: usize = 656;

// ── Windows FFI ─────────────────────────────────────────────
#[cfg(target_os = "windows")]
extern "system" {
    fn LoadLibraryA(name: *const i8) -> usize;
    fn GetProcAddress(module: usize, name: *const i8) -> usize;
    fn FreeLibrary(module: usize) -> i32;
    fn CreateEventA(attrs: usize, manual: i32, initial: i32, name: *const i8) -> usize;
    fn WaitForSingleObject(handle: usize, ms: u32) -> u32;
    fn CloseHandle(handle: usize) -> i32;
}

// ════════════════════════════════════════════════════════════
// JIT-CALLABLE FUNCTIONS — DX12 PURE RENDERING
// ════════════════════════════════════════════════════════════

/// Create the full DX12 rendering pipeline: Device + Queue + SwapChain + RTV + CmdAlloc + CmdList + Fence
#[no_mangle]
pub extern "C" fn jdb_dx12_create_pso(
    vert_s: *const crate::backend::jit::JdbString,
    pixel_s: *const crate::backend::jit::JdbString,
) -> i64 {
    #[cfg(target_os = "windows")]
    {
        let dx_ctx = super::dx12::get_ctx_pub();
        if !dx_ctx.initialized {
            eprintln!("💀 [DX12:Cube] DX12 not initialized");
            return 0;
        }

        unsafe {
            let rs = get_render();
            if rs.initialized { return 1; }

            let d3d12_handle = dx_ctx.d3d12_handle;
            let dxgi_handle = dx_ctx.dxgi_handle;

            // ── 1. Reuse D3D12 Device from dx12.rs context ─────
            let device = dx_ctx.device;
            if device == 0 {
                // Create fresh if not available
                let proc_name = CString::new("D3D12CreateDevice").unwrap();
                let proc = GetProcAddress(d3d12_handle, proc_name.as_ptr());
                if proc == 0 { eprintln!("💀 [DX12] D3D12CreateDevice not found"); return 0; }

                type CreateDeviceFn = extern "system" fn(usize, u32, *const [u8; 16], *mut usize) -> i32;
                let create_device: CreateDeviceFn = std::mem::transmute(proc);
                let mut new_device: usize = 0;
                let hr = create_device(0, 0xc000, &IID_ID3D12DEVICE, &mut new_device);
                if hr < 0 || new_device == 0 {
                    eprintln!("💀 [DX12] CreateDevice failed hr=0x{:08X}", hr as u32);
                    return 0;
                }
                rs.device = new_device;
            } else {
                rs.device = device;
            }
            let device = rs.device;
            eprintln!("✅ [DX12:Pure] Device ready (0x{:X})", device);

            // ── 2. Create Command Queue ─────────────────────
            let queue_desc = D3D12CommandQueueDesc {
                list_type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                priority: 0,
                flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
                node_mask: 0,
            };
            let mut cmd_queue: usize = 0;
            // ID3D12Device::CreateCommandQueue = vtable index 8
            let hr = vtable_call!(device, 8,
                extern "system" fn(usize, *const D3D12CommandQueueDesc, *const [u8; 16], *mut usize) -> i32,
                &queue_desc, &IID_ID3D12COMMAND_QUEUE, &mut cmd_queue);
            if hr < 0 || cmd_queue == 0 {
                eprintln!("💀 [DX12] CreateCommandQueue failed hr=0x{:08X}", hr as u32);
                return 0;
            }
            rs.command_queue = cmd_queue;
            eprintln!("✅ [DX12:Pure] Command Queue created");

            // ── 3. Create DXGI Factory + SwapChain ──────────
            let factory_fn_name = CString::new("CreateDXGIFactory1").unwrap();
            let factory_proc = GetProcAddress(dxgi_handle, factory_fn_name.as_ptr());
            if factory_proc == 0 { eprintln!("💀 [DX12] CreateDXGIFactory1 not found"); return 0; }

            type CreateFactoryFn = extern "system" fn(*const [u8; 16], *mut usize) -> i32;
            let create_factory: CreateFactoryFn = std::mem::transmute(factory_proc);
            let mut factory: usize = 0;
            let hr = create_factory(&IID_IDXGI_FACTORY4, &mut factory);
            if hr < 0 || factory == 0 {
                eprintln!("💀 [DX12] CreateDXGIFactory1 failed hr=0x{:08X}", hr as u32);
                return 0;
            }

            let win = super::window::get_win_pub();
            let hwnd = win.hwnd;

            let sc_desc = DxgiSwapChainDesc1 {
                width: 800,
                height: 600,
                format: DXGI_FORMAT_R8G8B8A8_UNORM,
                stereo: 0,
                sample_count: 1,
                sample_quality: 0,
                buffer_usage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                buffer_count: BUFFER_COUNT,
                scaling: 0, // DXGI_SCALING_STRETCH
                swap_effect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                alpha_mode: 0,
                flags: 0,
            };

            // IDXGIFactory2::CreateSwapChainForHwnd = vtable index 15
            let mut swap_chain1: usize = 0;
            let hr = vtable_call!(factory, 15,
                extern "system" fn(usize, usize, usize, *const DxgiSwapChainDesc1, usize, usize, *mut usize) -> i32,
                cmd_queue, hwnd, &sc_desc, 0, 0, &mut swap_chain1);
            if hr < 0 || swap_chain1 == 0 {
                eprintln!("💀 [DX12] CreateSwapChainForHwnd failed hr=0x{:08X}", hr as u32);
                com_release(factory);
                return 0;
            }
            rs.swap_chain = swap_chain1;
            com_release(factory);
            eprintln!("✅ [DX12:Pure] SwapChain created (2 buffers, FLIP_DISCARD)");

            // Get current back buffer index
            // IDXGISwapChain3::GetCurrentBackBufferIndex = vtable index 36
            // But for IDXGISwapChain1 we start at frame 0
            rs.frame_index = 0;

            // ── 4. Create RTV Descriptor Heap ───────────────
            eprintln!("  [DX12:Debug] Creating RTV heap...");
            let rtv_heap_desc = D3D12DescriptorHeapDesc {
                heap_type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                num_descriptors: BUFFER_COUNT,
                flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
                node_mask: 0,
            };
            let mut rtv_heap: usize = 0;
            // IID for ID3D12DescriptorHeap
            // IID_ID3D12DescriptorHeap = {8EFB471D-616C-4F49-90F7-127BB763FA51}
            let iid_desc_heap: [u8; 16] = [
                0x1d, 0x47, 0xfb, 0x8e, 0x6c, 0x61, 0x49, 0x4f,
                0x90, 0xf7, 0x12, 0x7b, 0xb7, 0x63, 0xfa, 0x51,
            ];
            // ID3D12Device::CreateDescriptorHeap = vtable index 14
            let hr = vtable_call!(device, 14,
                extern "system" fn(usize, *const D3D12DescriptorHeapDesc, *const [u8; 16], *mut usize) -> i32,
                &rtv_heap_desc, &iid_desc_heap, &mut rtv_heap);
            if hr < 0 || rtv_heap == 0 {
                eprintln!("💀 [DX12] CreateDescriptorHeap(RTV) failed hr=0x{:08X}", hr as u32);
                return 0;
            }
            rs.rtv_heap = rtv_heap;
            eprintln!("  [DX12:Debug] RTV heap created 0x{:X}", rtv_heap);

            // ID3D12Device::GetDescriptorHandleIncrementSize = vtable index 15
            rs.rtv_descriptor_size = vtable_call!(device, 15,
                extern "system" fn(usize, u32) -> u32,
                D3D12_DESCRIPTOR_HEAP_TYPE_RTV);

            // ── GetCPUDescriptorHandleForHeapStart ───────────────
            // COM vtable index 9 on ID3D12DescriptorHeap
            // COM uses C-style hidden return pointer: fn(this, *mut RetVal)
            let mut rtv_handle_start: usize = 0;
            {
                let heap_vt = *(rtv_heap as *const *const usize);
                let get_handle_fn: extern "system" fn(usize, *mut usize) -> usize =
                    std::mem::transmute(*heap_vt.add(9));
                get_handle_fn(rtv_heap, &mut rtv_handle_start);
            }
            eprintln!("  [DX12:Lib] HeapStart = 0x{:X}", rtv_handle_start);
            if rtv_handle_start == 0 {
                eprintln!("💀 [DX12] GetCPUDescriptorHandleForHeapStart returned 0!");
                return 0;
            }

            // Create RTVs for each buffer
            let iid_resource: [u8; 16] = [
                0xbe, 0x42, 0x64, 0x69, 0x2e, 0xa7, 0x59, 0x40,
                0xbc, 0x79, 0x5b, 0x5c, 0x98, 0x04, 0x0f, 0xad,
            ];
            eprintln!("  [DX12:Lib] RTV desc size={}", rs.rtv_descriptor_size);

            // Debug: dump device vtable entries around CreateRenderTargetView
            {
                let dev_vt = *(device as *const *const usize);
                for idx in 18..=22 {
                    eprintln!("  [DX12:Lib] Device vtable[{}] = 0x{:X}", idx, *dev_vt.add(idx));
                }
            }

            for i in 0..BUFFER_COUNT {
                let mut resource: usize = 0;
                let hr = vtable_call!(swap_chain1, 9,
                    extern "system" fn(usize, u32, *const [u8; 16], *mut usize) -> i32,
                    i, &iid_resource, &mut resource);
                if hr < 0 || resource == 0 {
                    eprintln!("💀 [DX12] GetBuffer({}) failed hr=0x{:08X}", i, hr as u32);
                    return 0;
                }
                rs.render_targets[i as usize] = resource;

                let handle_value: usize = rtv_handle_start + (i as usize) * (rs.rtv_descriptor_size as usize);
                eprintln!("  [DX12:Lib] Buffer[{}]=0x{:X} handle=0x{:X}", i, resource, handle_value);

                // Get the function pointer from vtable index 20
                let dev_vt = *(device as *const *const usize);
                let crtv_fn_ptr = *dev_vt.add(20);
                eprintln!("  [DX12:Lib] CreateRTV fn=0x{:X} calling...", crtv_fn_ptr);

                // Call CreateRenderTargetView via vtable index 20
                // Signature: void(this, pResource, pDesc, DestDescriptor)
                // Try calling as if it returns HRESULT (i32) to capture debug layer errors
                let crtv_result = vtable_call!(device, 20,
                    extern "system" fn(usize, usize, usize, usize) -> i32,
                    resource, 0usize, handle_value);
                if crtv_result < 0 {
                    eprintln!("💀 [DX12:Lib] CreateRTV FAILED hr=0x{:08X}", crtv_result as u32);
                    return 0;
                }
                eprintln!("  [DX12:Lib] CreateRTV ✅ buffer {} (hr=0x{:X})", i, crtv_result as u32);
            }
            eprintln!("✅ [DX12:Pure] RTV Heap + {} Render Targets created", BUFFER_COUNT);

            // Skip depth buffer for now - render without depth test
            rs.dsv_heap = 0;
            rs.depth_buffer = 0;

            // ── 5. Create Command Allocator ─────────────────
            let mut cmd_alloc: usize = 0;
            // IID_ID3D12CommandAllocator = {6102DEE4-AF59-4B09-B999-B44D73F09B24}
            let iid_cmd_alloc: [u8; 16] = [
                0xe4, 0xde, 0x02, 0x61, 0x59, 0xaf, 0x09, 0x4b,
                0xb9, 0x99, 0xb4, 0x4d, 0x73, 0xf0, 0x9b, 0x24,
            ];
            // ID3D12Device::CreateCommandAllocator = vtable index 9
            let hr = vtable_call!(device, 9,
                extern "system" fn(usize, u32, *const [u8; 16], *mut usize) -> i32,
                D3D12_COMMAND_LIST_TYPE_DIRECT, &iid_cmd_alloc, &mut cmd_alloc);
            if hr < 0 || cmd_alloc == 0 {
                eprintln!("💀 [DX12] CreateCommandAllocator failed hr=0x{:08X}", hr as u32);
                return 0;
            }
            rs.cmd_allocator = cmd_alloc;
            eprintln!("✅ [DX12:Pure] Command Allocator created");

            // ── 6. Compile HLSL shaders ─────────────────────
            let d3dcompiler_name = CString::new("d3dcompiler_47.dll").unwrap();
            let compiler_dll = LoadLibraryA(d3dcompiler_name.as_ptr());
            if compiler_dll == 0 {
                eprintln!("💀 [DX12] d3dcompiler_47.dll not found");
                return 0;
            }

            let compile_fn_name = CString::new("D3DCompile").unwrap();
            let compile_proc = GetProcAddress(compiler_dll, compile_fn_name.as_ptr());
            if compile_proc == 0 {
                eprintln!("💀 [DX12] D3DCompile not found");
                FreeLibrary(compiler_dll);
                return 0;
            }
            // D3DCompile(pSrcData, SrcDataSize, pSourceName, pDefines, pInclude,
            //            pEntrypoint, pTarget, Flags1, Flags2, ppCode, ppErrorMsgs)
            type D3DCompileFn = extern "system" fn(
                *const u8, usize, *const i8, usize, usize,
                *const i8, *const i8, u32, u32, *mut usize, *mut usize,
            ) -> i32;
            let d3d_compile: D3DCompileFn = std::mem::transmute(compile_proc);

            // Read shader files
            let vert_jdb = &*vert_s;
            let pixel_jdb = &*pixel_s;
            let vert_path = std::str::from_utf8_unchecked(
                std::slice::from_raw_parts(vert_jdb.ptr, vert_jdb.len as usize));
            let pixel_path = std::str::from_utf8_unchecked(
                std::slice::from_raw_parts(pixel_jdb.ptr, pixel_jdb.len as usize));

            let vert_src = match std::fs::read_to_string(vert_path) {
                Ok(s) => s,
                Err(e) => { eprintln!("💀 [DX12] Cannot read {}: {}", vert_path, e); return 0; }
            };
            let pixel_src = match std::fs::read_to_string(pixel_path) {
                Ok(s) => s,
                Err(e) => { eprintln!("💀 [DX12] Cannot read {}: {}", pixel_path, e); return 0; }
            };

            let vs_entry = CString::new("VSMain").unwrap();
            let ps_entry = CString::new("PSMain").unwrap();
            let vs_target = CString::new("vs_5_0").unwrap();
            let ps_target = CString::new("ps_5_0").unwrap();

            let mut vs_blob: usize = 0;
            let mut vs_errors: usize = 0;
            let hr = d3d_compile(
                vert_src.as_ptr(), vert_src.len(), std::ptr::null(), 0, 0,
                vs_entry.as_ptr(), vs_target.as_ptr(), 0, 0, &mut vs_blob, &mut vs_errors);
            if hr < 0 || vs_blob == 0 {
                if vs_errors != 0 {
                    // ID3DBlob::GetBufferPointer = vtable index 3
                    let err_ptr = vtable_call!(vs_errors, 3, extern "system" fn(usize) -> *const i8);
                    let err_msg = std::ffi::CStr::from_ptr(err_ptr).to_string_lossy();
                    eprintln!("💀 [DX12] VS compile error: {}", err_msg);
                    com_release(vs_errors);
                }
                FreeLibrary(compiler_dll);
                return 0;
            }
            eprintln!("✅ [DX12:Pure] Vertex shader compiled (vs_5_0)");

            let mut ps_blob: usize = 0;
            let mut ps_errors: usize = 0;
            let hr = d3d_compile(
                pixel_src.as_ptr(), pixel_src.len(), std::ptr::null(), 0, 0,
                ps_entry.as_ptr(), ps_target.as_ptr(), 0, 0, &mut ps_blob, &mut ps_errors);
            if hr < 0 || ps_blob == 0 {
                if ps_errors != 0 {
                    let err_ptr = vtable_call!(ps_errors, 3, extern "system" fn(usize) -> *const i8);
                    let err_msg = std::ffi::CStr::from_ptr(err_ptr).to_string_lossy();
                    eprintln!("💀 [DX12] PS compile error: {}", err_msg);
                    com_release(ps_errors);
                }
                com_release(vs_blob);
                FreeLibrary(compiler_dll);
                return 0;
            }
            eprintln!("✅ [DX12:Pure] Pixel shader compiled (ps_5_0)");
            FreeLibrary(compiler_dll);

            // ── 7. Create Root Signature ────────────────────
            let root_param = D3D12RootParameter {
                parameter_type: D3D12_ROOT_PARAMETER_TYPE_CBV,
                _pad0: 0,
                shader_register: 0,
                register_space: 0,
                _union_pad: [0],
                shader_visibility: D3D12_SHADER_VISIBILITY_ALL,
                _pad1: 0,
            };

            let root_sig_desc = D3D12RootSignatureDesc {
                num_parameters: 1,
                _pad0: 0,
                p_parameters: &root_param,
                num_static_samplers: 0,
                _pad1: 0,
                p_static_samplers: std::ptr::null(),
                flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER,
            };

            // D3D12SerializeRootSignature
            let serialize_name = CString::new("D3D12SerializeRootSignature").unwrap();
            let serialize_proc = GetProcAddress(d3d12_handle, serialize_name.as_ptr());
            if serialize_proc == 0 {
                eprintln!("💀 [DX12] D3D12SerializeRootSignature not found");
                return 0;
            }
            type SerializeRootSigFn = extern "system" fn(
                *const D3D12RootSignatureDesc, u32, *mut usize, *mut usize) -> i32;
            let serialize_root_sig: SerializeRootSigFn = std::mem::transmute(serialize_proc);

            let mut sig_blob: usize = 0;
            let mut sig_error: usize = 0;
            let hr = serialize_root_sig(&root_sig_desc, 1, &mut sig_blob, &mut sig_error); // version 1
            if hr < 0 || sig_blob == 0 {
                eprintln!("💀 [DX12] SerializeRootSignature failed hr=0x{:08X}", hr as u32);
                if sig_error != 0 { com_release(sig_error); }
                return 0;
            }

            let sig_data = vtable_call!(sig_blob, 3, extern "system" fn(usize) -> *const u8);
            let sig_size = vtable_call!(sig_blob, 4, extern "system" fn(usize) -> usize);
            eprintln!("  [DX12:Debug] RootSig blob size={} data={:?}", sig_size, sig_data);

            let mut root_sig: usize = 0;
            // ID3D12Device::CreateRootSignature = vtable index 16
            // Signature: CreateRootSignature(nodeMask, pBlobWithRootSignature, blobLengthInBytes, riid, ppvRootSignature)
            let hr = vtable_call!(device, 16,
                extern "system" fn(usize, u32, *const u8, usize, *const [u8; 16], *mut usize) -> i32,
                0, sig_data, sig_size, &IID_ID3D12_ROOT_SIGNATURE, &mut root_sig);
            com_release(sig_blob);
            if hr < 0 || root_sig == 0 {
                eprintln!("💀 [DX12] CreateRootSignature failed hr=0x{:08X}", hr as u32);
                return 0;
            }
            rs.root_signature = root_sig;
            eprintln!("✅ [DX12:Pure] Root Signature created (1 CBV)");

            // ── 8. Create Fence ──────────────────────────────
            // IID_ID3D12Fence = {0A753DCF-C4D8-4B91-ADF6-BE5A60D95A76}
            let iid_fence: [u8; 16] = [
                0xcf, 0x3d, 0x75, 0x0a, 0xd8, 0xc4, 0x91, 0x4b,
                0xad, 0xf6, 0xbe, 0x5a, 0x60, 0xd9, 0x5a, 0x76,
            ];
            let mut fence: usize = 0;
            // ID3D12Device::CreateFence = vtable index 36
            let hr = vtable_call!(device, 36,
                extern "system" fn(usize, u64, u32, *const [u8; 16], *mut usize) -> i32,
                0u64, D3D12_FENCE_FLAG_NONE, &iid_fence, &mut fence);
            if hr < 0 || fence == 0 {
                eprintln!("💀 [DX12] CreateFence failed hr=0x{:08X}", hr as u32);
                return 0;
            }
            rs.fence = fence;
            rs.fence_value = 1;
            rs.fence_event = CreateEventA(0, 0, 0, std::ptr::null());
            if rs.fence_event == 0 {
                eprintln!("💀 [DX12] CreateEvent for fence failed");
                return 0;
            }
            eprintln!("✅ [DX12:Pure] Fence created");

            // ── 9. Create Command List ──────────────────────
            let iid_cmd_list: [u8; 16] = [
                0x0f, 0x0d, 0x16, 0x5b, 0x1b, 0xac, 0x85, 0x41,
                0x8b, 0xa8, 0xb3, 0xae, 0x42, 0xa5, 0xa4, 0x55,
            ];
            let mut cmd_list: usize = 0;
            // ID3D12Device::CreateCommandList = vtable index 12
            eprintln!("  [DX12:Debug] Creating command list... device=0x{:X} alloc=0x{:X}", device, cmd_alloc);
            let hr = vtable_call!(device, 12,
                extern "system" fn(usize, u32, u32, usize, usize, *const [u8; 16], *mut usize) -> i32,
                0u32, D3D12_COMMAND_LIST_TYPE_DIRECT, cmd_alloc, 0usize, &iid_cmd_list, &mut cmd_list);
            if hr < 0 || cmd_list == 0 {
                eprintln!("💀 [DX12] CreateCommandList failed hr=0x{:08X}", hr as u32);
                return 0;
            }
            // Command list is created in recording state — close it immediately
            let hr = vtable_call!(cmd_list, 9, extern "system" fn(usize) -> i32);
            if hr < 0 {
                eprintln!("💀 [DX12] CommandList::Close failed hr=0x{:08X}", hr as u32);
                return 0;
            }
            rs.cmd_list = cmd_list;
            eprintln!("✅ [DX12:Pure] Command List created");

            // ── 10. Create Vertex Buffer ────────────────────
            let vb_size = (CUBE_VERTICES.len() * std::mem::size_of::<f32>()) as u64;
            let heap_props = D3D12HeapProperties {
                heap_type: D3D12_HEAP_TYPE_UPLOAD,
                cpu_page_property: 0,
                memory_pool_preference: 0,
                creation_node_mask: 1,
                visible_node_mask: 1,
            };
            let res_desc = D3D12ResourceDesc {
                dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                alignment: 0,
                width: vb_size,
                height: 1,
                depth_or_array_size: 1,
                mip_levels: 1,
                format: 0, // UNKNOWN
                sample_count: 1,
                sample_quality: 0,
                layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                flags: 0,
            };
            let mut vb_resource: usize = 0;
            // ID3D12Device::CreateCommittedResource = vtable index 27
            let hr = vtable_call!(device, 27,
                extern "system" fn(usize, *const D3D12HeapProperties, u32, *const D3D12ResourceDesc, u32, usize, *const [u8; 16], *mut usize) -> i32,
                &heap_props, 0u32, &res_desc, D3D12_RESOURCE_STATE_GENERIC_READ, 0usize, &iid_resource, &mut vb_resource);
            if hr < 0 || vb_resource == 0 {
                eprintln!("💀 [DX12] CreateCommittedResource(VB) failed hr=0x{:08X}", hr as u32);
                return 0;
            }
            rs.vertex_buffer = vb_resource;

            // Map, copy vertex data, Unmap
            let mut mapped_ptr: *mut u8 = std::ptr::null_mut();
            // ID3D12Resource::Map = vtable index 8
            let hr = vtable_call!(vb_resource, 8,
                extern "system" fn(usize, u32, usize, *mut *mut u8) -> i32,
                0u32, 0usize, &mut mapped_ptr);
            if hr < 0 || mapped_ptr.is_null() {
                eprintln!("💀 [DX12] VB Map failed hr=0x{:08X}", hr as u32);
                return 0;
            }
            std::ptr::copy_nonoverlapping(
                CUBE_VERTICES.as_ptr() as *const u8,
                mapped_ptr,
                vb_size as usize,
            );
            // ID3D12Resource::Unmap = vtable index 9
            vtable_call!(vb_resource, 9,
                extern "system" fn(usize, u32, usize),
                0u32, 0usize);

            // Get GPU virtual address for vertex buffer view
            // ID3D12Resource::GetGPUVirtualAddress = vtable index 11
            let vb_gpu_va = vtable_call!(vb_resource, 11,
                extern "system" fn(usize) -> u64);
            rs.vb_view_buffer = vb_gpu_va as usize;
            rs.vb_view_size = vb_size as u32;
            rs.vb_view_stride = (6 * std::mem::size_of::<f32>()) as u32; // pos(3) + color(3) = 6 floats
            eprintln!("✅ [DX12:Pure] Vertex Buffer created ({} bytes, stride={})", vb_size, rs.vb_view_stride);

            // ── 11. Create Constant Buffer ──────────────────
            let cb_size: u64 = 256; // 256-byte aligned for CBV
            let cb_heap_props = D3D12HeapProperties {
                heap_type: D3D12_HEAP_TYPE_UPLOAD,
                cpu_page_property: 0,
                memory_pool_preference: 0,
                creation_node_mask: 1,
                visible_node_mask: 1,
            };
            let cb_res_desc = D3D12ResourceDesc {
                dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                alignment: 0,
                width: cb_size,
                height: 1,
                depth_or_array_size: 1,
                mip_levels: 1,
                format: 0,
                sample_count: 1,
                sample_quality: 0,
                layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                flags: 0,
            };
            let mut cb_resource: usize = 0;
            let hr = vtable_call!(device, 27,
                extern "system" fn(usize, *const D3D12HeapProperties, u32, *const D3D12ResourceDesc, u32, usize, *const [u8; 16], *mut usize) -> i32,
                &cb_heap_props, 0u32, &cb_res_desc, D3D12_RESOURCE_STATE_GENERIC_READ, 0usize, &iid_resource, &mut cb_resource);
            if hr < 0 || cb_resource == 0 {
                eprintln!("💀 [DX12] CreateCommittedResource(CBV) failed hr=0x{:08X}", hr as u32);
                return 0;
            }
            rs.cbv_buffer = cb_resource;

            // Map and keep mapped (persistent mapping for upload heap)
            let mut cbv_mapped: *mut u8 = std::ptr::null_mut();
            let hr = vtable_call!(cb_resource, 8,
                extern "system" fn(usize, u32, usize, *mut *mut u8) -> i32,
                0u32, 0usize, &mut cbv_mapped);
            if hr < 0 || cbv_mapped.is_null() {
                eprintln!("💀 [DX12] CBV Map failed hr=0x{:08X}", hr as u32);
                return 0;
            }
            rs.cbv_mapped = cbv_mapped;

            let cbv_gpu_va = vtable_call!(cb_resource, 11,
                extern "system" fn(usize) -> u64);
            rs.cbv_gpu_va = cbv_gpu_va;
            eprintln!("✅ [DX12:Pure] Constant Buffer created (256 bytes, GPU VA=0x{:X})", cbv_gpu_va);

            // ── 12. Create PSO ──────────────────────────────
            // Copy shader bytecode before releasing blobs
            let vs_data = vtable_call!(vs_blob, 3, extern "system" fn(usize) -> *const u8);
            let vs_size = vtable_call!(vs_blob, 4, extern "system" fn(usize) -> usize);
            eprintln!("  [DX12:Debug] VS bytecode: ptr=0x{:X} size={}", vs_data as usize, vs_size);
            let mut vs_bytecode = vec![0u8; vs_size];
            std::ptr::copy_nonoverlapping(vs_data, vs_bytecode.as_mut_ptr(), vs_size);

            let ps_data = vtable_call!(ps_blob, 3, extern "system" fn(usize) -> *const u8);
            let ps_size = vtable_call!(ps_blob, 4, extern "system" fn(usize) -> usize);
            let mut ps_bytecode = vec![0u8; ps_size];
            std::ptr::copy_nonoverlapping(ps_data, ps_bytecode.as_mut_ptr(), ps_size);

            com_release(vs_blob);
            com_release(ps_blob);

            // Input element descs
            let sem_position = CString::new("POSITION").unwrap();
            let sem_color = CString::new("COLOR").unwrap();
            let input_elements = [
                D3D12InputElementDesc {
                    semantic_name: sem_position.as_ptr(),
                    semantic_index: 0,
                    format: 6, // DXGI_FORMAT_R32G32B32_FLOAT
                    input_slot: 0,
                    aligned_byte_offset: 0,
                    input_slot_class: D3D12_INPUT_CLASSIFICATION_PER_VERTEX,
                    instance_data_step_rate: 0,
                },
                D3D12InputElementDesc {
                    semantic_name: sem_color.as_ptr(),
                    semantic_index: 0,
                    format: 6, // DXGI_FORMAT_R32G32B32_FLOAT
                    input_slot: 0,
                    aligned_byte_offset: 12,
                    input_slot_class: D3D12_INPUT_CLASSIFICATION_PER_VERTEX,
                    instance_data_step_rate: 0,
                },
            ];

            // Build D3D12_GRAPHICS_PIPELINE_STATE_DESC as raw bytes
            let mut pso_desc = [0u8; GRAPHICS_PSO_DESC_SIZE];
            let p = pso_desc.as_mut_ptr();

            // Offset 0: pRootSignature (8 bytes)
            std::ptr::write_unaligned(p.add(0) as *mut usize, root_sig);
            // Offset 8: VS.pShaderBytecode (8 bytes)
            std::ptr::write_unaligned(p.add(8) as *mut *const u8, vs_bytecode.as_ptr());
            // Offset 16: VS.BytecodeLength (8 bytes)
            std::ptr::write_unaligned(p.add(16) as *mut usize, vs_size);
            // Offset 24: PS.pShaderBytecode (8 bytes)
            std::ptr::write_unaligned(p.add(24) as *mut *const u8, ps_bytecode.as_ptr());
            // Offset 32: PS.BytecodeLength (8 bytes)
            std::ptr::write_unaligned(p.add(32) as *mut usize, ps_size);
            // Offsets 40-87: DS, HS, GS = zeroed (already)
            // Offset 88-135: StreamOutput = zeroed (already)

            // Layout of D3D12_GRAPHICS_PIPELINE_STATE_DESC on x64:
            // 0: pRootSignature (8)
            // 8: VS {ptr(8), size(8)} = 16
            // 24: PS {ptr(8), size(8)} = 16
            // 40: DS (16), 56: HS (16), 72: GS (16) = zeroed
            // 88: StreamOutput {ptr(8), UINT(4), pad(4), ptr(8), UINT(4), UINT(4)} = 32
            // 120: BlendState (328 = 8 + 8*40)
            // 448: SampleMask (4)
            // 452: RasterizerState (44)
            // 496: DepthStencilState (52)
            // 548: InputLayout {ptr(8), UINT(4), pad(4)} = 16
            // 564: IBStripCutValue (4)
            // 568: PrimitiveTopologyType (4)
            // 572: NumRenderTargets (4)
            // 576: RTVFormats[8] (32)
            // 608: DSVFormat (4)
            // 612: SampleDesc {Count(4), Quality(4)} = 8
            // 620: NodeMask (4)
            // 624: CachedPSO {ptr(8), size(8)} = 16
            // 640: Flags (4)
            // Total ~644, padded to 648

            // BlendState at offset 120
            // RenderTarget[0] starts at 120 + 8 = 128
            std::ptr::write_unaligned(p.add(128) as *mut u32, 0); // BlendEnable = FALSE
            std::ptr::write_unaligned(p.add(132) as *mut u32, 0); // LogicOpEnable = FALSE
            std::ptr::write_unaligned(p.add(136) as *mut u32, 1); // SrcBlend = ONE
            std::ptr::write_unaligned(p.add(140) as *mut u32, 0); // DestBlend = ZERO
            std::ptr::write_unaligned(p.add(144) as *mut u32, 1); // BlendOp = ADD
            std::ptr::write_unaligned(p.add(148) as *mut u32, 1); // SrcBlendAlpha = ONE
            std::ptr::write_unaligned(p.add(152) as *mut u32, 0); // DestBlendAlpha = ZERO
            std::ptr::write_unaligned(p.add(156) as *mut u32, 1); // BlendOpAlpha = ADD
            std::ptr::write_unaligned(p.add(160) as *mut u32, 0); // LogicOp = NOOP
            std::ptr::write_unaligned(p.add(164) as *mut u8, 0x0F); // RenderTargetWriteMask = ALL

            // SampleMask at offset 448
            std::ptr::write_unaligned(p.add(448) as *mut u32, 0xFFFFFFFF);

            // RasterizerState at offset 452
            std::ptr::write_unaligned(p.add(452) as *mut u32, D3D12_FILL_MODE_SOLID); // FillMode = 3
            std::ptr::write_unaligned(p.add(456) as *mut u32, D3D12_CULL_MODE_BACK);  // CullMode = 3
            std::ptr::write_unaligned(p.add(460) as *mut u32, 0); // FrontCounterClockwise = FALSE
            std::ptr::write_unaligned(p.add(464) as *mut i32, 0); // DepthBias
            std::ptr::write_unaligned(p.add(468) as *mut f32, 0.0); // DepthBiasClamp
            std::ptr::write_unaligned(p.add(472) as *mut f32, 0.0); // SlopeScaledDepthBias
            std::ptr::write_unaligned(p.add(476) as *mut u32, 1); // DepthClipEnable = TRUE

            // DepthStencilState at offset 496 — all zeroed (no depth test)

            // InputLayout at offset 548
            std::ptr::write_unaligned(p.add(548) as *mut *const D3D12InputElementDesc, input_elements.as_ptr());
            std::ptr::write_unaligned(p.add(556) as *mut u32, 2); // NumElements = 2

            // PrimitiveTopologyType at offset 568
            std::ptr::write_unaligned(p.add(568) as *mut u32, D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE);

            // NumRenderTargets at offset 572
            std::ptr::write_unaligned(p.add(572) as *mut u32, 1);

            // RTVFormats[0] at offset 576
            std::ptr::write_unaligned(p.add(576) as *mut u32, DXGI_FORMAT_R8G8B8A8_UNORM);

            // SampleDesc.Count at offset 612
            std::ptr::write_unaligned(p.add(612) as *mut u32, 1);

            let iid_pso: [u8; 16] = [
                0xf3, 0x30, 0x5a, 0x76, 0x24, 0xf6, 0xe7, 0x4d,
                0x9b, 0xe6, 0x7d, 0x82, 0x14, 0x7a, 0x94, 0xee,
            ];
            let mut pso: usize = 0;
            // ID3D12Device::CreateGraphicsPipelineState = vtable index 10
            let hr = vtable_call!(device, 10,
                extern "system" fn(usize, *const u8, *const [u8; 16], *mut usize) -> i32,
                pso_desc.as_ptr(), &iid_pso, &mut pso);
            if hr < 0 || pso == 0 {
                eprintln!("💀 [DX12] CreateGraphicsPipelineState failed hr=0x{:08X}", hr as u32);
                return 0;
            }
            rs.pso = pso;
            eprintln!("✅ [DX12:Pure] PSO created");

            // Keep vs/ps bytecode alive — they're Vecs that will be dropped after this scope
            // but PSO already consumed them, so it's safe to drop now
            drop(vs_bytecode);
            drop(ps_bytecode);
            drop(sem_position);
            drop(sem_color);

            rs.initialized = true;
            eprintln!("✅ [DX12:Pure] Full pipeline initialized — Device + Queue + SwapChain + RTV + RootSig + Fence + CmdList + VB + CBV + PSO");
            1
        }
    }

    #[cfg(not(target_os = "windows"))]
    { let _ = (vert_s, pixel_s); 1 }
}

/// Create DX12 cube mesh (vertex buffer already created during PSO init)
#[no_mangle]
pub extern "C" fn jdb_dx12_create_cube_mesh() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let rs = get_render();
        if !rs.initialized || rs.vertex_buffer == 0 { return 0; }
        eprintln!("✅ [DX12:Pure] Mesh ready (VB=0x{:X}, {} verts)", rs.vertex_buffer, CUBE_VERTICES.len() / 6);
        1
    }
    #[cfg(not(target_os = "windows"))]
    { 1 }
}

/// Create Constant Buffer View (CBV already created during PSO init)
#[no_mangle]
pub extern "C" fn jdb_dx12_create_cbv() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let rs = get_render();
        if !rs.initialized || rs.cbv_buffer == 0 { return 0; }
        eprintln!("✅ [DX12:Pure] CBV ready (GPU VA=0x{:X})", rs.cbv_gpu_va);
        1
    }
    #[cfg(not(target_os = "windows"))]
    { 1 }
}

/// Update MVP matrix — copy to persistently mapped CBV buffer
#[no_mangle]
pub extern "C" fn jdb_dx12_update_mvp(_cbv: i64, angle: i64) -> i64 {
    #[cfg(target_os = "windows")]
    {
        let rs = get_render();
        if !rs.initialized || rs.cbv_mapped.is_null() { return 0; }
        let mvp = compute_mvp_rowmajor(angle as f32);
        unsafe {
            std::ptr::copy_nonoverlapping(
                mvp.as_ptr() as *const u8,
                rs.cbv_mapped,
                64, // 16 floats * 4 bytes
            );
        }
        1
    }
    #[cfg(not(target_os = "windows"))]
    { let _ = (_cbv, angle); 1 }
}

/// Begin DX12 frame — reset allocator + command list, transition to RENDER_TARGET
#[no_mangle]
pub extern "C" fn jdb_dx12_begin_frame() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let rs = get_render();
        if !rs.initialized { return 0; }
        unsafe {
            let cmd_alloc = rs.cmd_allocator;
            let cmd_list = rs.cmd_list;
            let pso = rs.pso;

            // ID3D12CommandAllocator::Reset = vtable index 8
            let hr = vtable_call!(cmd_alloc, 8, extern "system" fn(usize) -> i32);
            if hr < 0 {
                eprintln!("💀 [DX12] CommandAllocator::Reset failed hr=0x{:08X}", hr as u32);
                return 0;
            }

            // ID3D12GraphicsCommandList::Reset = vtable index 10 (pAllocator, pInitialState)
            let hr = vtable_call!(cmd_list, 10,
                extern "system" fn(usize, usize, usize) -> i32,
                cmd_alloc, pso);
            if hr < 0 {
                eprintln!("💀 [DX12] CommandList::Reset failed hr=0x{:08X}", hr as u32);
                return 0;
            }

            // ResourceBarrier: PRESENT -> RENDER_TARGET
            let barrier = D3D12ResourceBarrier {
                barrier_type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                flags: 0,
                transition: D3D12ResourceTransitionBarrier {
                    resource: rs.render_targets[rs.frame_index as usize],
                    subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                    state_before: D3D12_RESOURCE_STATE_PRESENT,
                    state_after: D3D12_RESOURCE_STATE_RENDER_TARGET,
                },
            };
            // ID3D12GraphicsCommandList::ResourceBarrier = vtable index 26
            vtable_call!(cmd_list, 26,
                extern "system" fn(usize, u32, *const D3D12ResourceBarrier),
                1u32, &barrier);
        }
        1
    }
    #[cfg(not(target_os = "windows"))]
    { 1 }
}

/// Clear render target + set viewport/scissor/render target
#[no_mangle]
pub extern "C" fn jdb_dx12_clear_rtv(_list: i64, r: i64, g: i64, b: i64) -> i64 {
    #[cfg(target_os = "windows")]
    {
        let rs = get_render();
        if !rs.initialized { return 0; }
        unsafe {
            let cmd_list = rs.cmd_list;
            let rtv_heap = rs.rtv_heap;

            // Get RTV handle for current frame
            // ✅ FIX: GetCPUDescriptorHandleForHeapStart returns via *mut usize
            let mut rtv_start_val: usize = 0;
            vtable_call!(rtv_heap, 9,
                extern "system" fn(usize, *mut usize) -> *mut usize,
                &mut rtv_start_val);
            let rtv_handle_val: usize = rtv_start_val + (rs.frame_index as usize) * (rs.rtv_descriptor_size as usize);

            // ClearRenderTargetView = vtable index 48
            // Handle passed as u64 by value in register (already correct)
            let color: [f32; 4] = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0];
            vtable_call!(cmd_list, 48,
                extern "system" fn(usize, u64, *const [f32; 4], u32, usize),
                rtv_handle_val as u64, &color, 0u32, 0usize);

            // RSSetViewports = vtable index 21
            let viewport = D3D12Viewport {
                top_left_x: 0.0, top_left_y: 0.0,
                width: 800.0, height: 600.0,
                min_depth: 0.0, max_depth: 1.0,
            };
            vtable_call!(cmd_list, 21,
                extern "system" fn(usize, u32, *const D3D12Viewport),
                1u32, &viewport);

            // RSSetScissorRects = vtable index 22
            let scissor = D3D12Rect { left: 0, top: 0, right: 800, bottom: 600 };
            vtable_call!(cmd_list, 22,
                extern "system" fn(usize, u32, *const D3D12Rect),
                1u32, &scissor);

            // ✅ FIX: OMSetRenderTargets = vtable index 46
            // Pass handle as *const usize (pointer to the raw value)
            vtable_call!(cmd_list, 46,
                extern "system" fn(usize, u32, *const usize, i32, usize),
                1u32, &rtv_handle_val, 0i32, 0usize);
        }
        1
    }
    #[cfg(not(target_os = "windows"))]
    { let _ = (_list, r, g, b); 1 }
}

/// Set CBV — bind root signature, PSO, and constant buffer
#[no_mangle]
pub extern "C" fn jdb_dx12_set_cbv(_list: i64, _cbv: i64) -> i64 {
    #[cfg(target_os = "windows")]
    {
        let rs = get_render();
        if !rs.initialized { return 0; }
        unsafe {
            let cmd_list = rs.cmd_list;

            // SetGraphicsRootSignature = vtable index 30
            vtable_call!(cmd_list, 30,
                extern "system" fn(usize, usize),
                rs.root_signature);

            // SetPipelineState = vtable index 25
            vtable_call!(cmd_list, 25,
                extern "system" fn(usize, usize),
                rs.pso);

            // SetGraphicsRootConstantBufferView = vtable index 38 (RootParameterIndex, BufferLocation)
            vtable_call!(cmd_list, 38,
                extern "system" fn(usize, u32, u64),
                0u32, rs.cbv_gpu_va);
        }
        1
    }
    #[cfg(not(target_os = "windows"))]
    { let _ = (_list, _cbv); 1 }
}

/// Draw — set topology, vertex buffer, and issue draw call
#[no_mangle]
pub extern "C" fn jdb_dx12_draw(_list: i64, vertex_count: i64) -> i64 {
    #[cfg(target_os = "windows")]
    {
        let rs = get_render();
        if !rs.initialized { return 0; }
        unsafe {
            let cmd_list = rs.cmd_list;

            // IASetPrimitiveTopology = vtable index 20 (D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST = 4)
            vtable_call!(cmd_list, 20,
                extern "system" fn(usize, u32),
                D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

            // IASetVertexBuffers = vtable index 44 (StartSlot, NumViews, pViews)
            let vbv = D3D12VertexBufferView {
                buffer_location: rs.vb_view_buffer as u64,
                size_in_bytes: rs.vb_view_size,
                stride_in_bytes: rs.vb_view_stride,
            };
            vtable_call!(cmd_list, 44,
                extern "system" fn(usize, u32, u32, *const D3D12VertexBufferView),
                0u32, 1u32, &vbv);

            // DrawInstanced = vtable index 12 (VertexCount, InstanceCount, StartVertex, StartInstance)
            vtable_call!(cmd_list, 12,
                extern "system" fn(usize, u32, u32, u32, u32),
                vertex_count as u32, 1u32, 0u32, 0u32);
        }
        1
    }
    #[cfg(not(target_os = "windows"))]
    { let _ = (_list, vertex_count); 1 }
}

/// End frame — transition back to PRESENT, close + execute command list, present, wait for GPU
#[no_mangle]
pub extern "C" fn jdb_dx12_end_frame() -> i64 {
    #[cfg(target_os = "windows")]
    {
        let rs = get_render();
        if !rs.initialized { return 0; }
        unsafe {
            let cmd_list = rs.cmd_list;
            let cmd_queue = rs.command_queue;

            // ResourceBarrier: RENDER_TARGET -> PRESENT
            let barrier = D3D12ResourceBarrier {
                barrier_type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                flags: 0,
                transition: D3D12ResourceTransitionBarrier {
                    resource: rs.render_targets[rs.frame_index as usize],
                    subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                    state_before: D3D12_RESOURCE_STATE_RENDER_TARGET,
                    state_after: D3D12_RESOURCE_STATE_PRESENT,
                },
            };
            vtable_call!(cmd_list, 26,
                extern "system" fn(usize, u32, *const D3D12ResourceBarrier),
                1u32, &barrier);

            // Close command list = vtable index 9
            let hr = vtable_call!(cmd_list, 9, extern "system" fn(usize) -> i32);
            if hr < 0 {
                eprintln!("💀 [DX12] CommandList::Close failed hr=0x{:08X}", hr as u32);
                return 0;
            }

            // ExecuteCommandLists = vtable index 10 on command queue
            let cmd_lists: [usize; 1] = [cmd_list];
            vtable_call!(cmd_queue, 10,
                extern "system" fn(usize, u32, *const usize),
                1u32, cmd_lists.as_ptr());

            // Present = vtable index 8 on swap chain (SyncInterval=1, Flags=0)
            let hr = vtable_call!(rs.swap_chain, 8,
                extern "system" fn(usize, u32, u32) -> i32,
                1u32, 0u32);
            if hr < 0 {
                eprintln!("💀 [DX12] Present failed hr=0x{:08X}", hr as u32);
                return 0;
            }

            // Signal fence
            let fence_val = rs.fence_value;
            // ID3D12CommandQueue::Signal = vtable index 14
            let hr = vtable_call!(cmd_queue, 14,
                extern "system" fn(usize, usize, u64) -> i32,
                rs.fence, fence_val);
            if hr < 0 {
                eprintln!("💀 [DX12] Signal failed hr=0x{:08X}", hr as u32);
                return 0;
            }

            // Wait for GPU to complete
            // ID3D12Fence::GetCompletedValue = vtable index 8
            let completed = vtable_call!(rs.fence, 8,
                extern "system" fn(usize) -> u64);
            if completed < fence_val {
                // ID3D12Fence::SetEventOnCompletion = vtable index 9
                vtable_call!(rs.fence, 9,
                    extern "system" fn(usize, u64, usize) -> i32,
                    fence_val, rs.fence_event);
                WaitForSingleObject(rs.fence_event, 0xFFFFFFFF); // INFINITE
            }

            rs.fence_value += 1;
            rs.frame_index = (rs.frame_index + 1) % BUFFER_COUNT;
        }
        1
    }
    #[cfg(not(target_os = "windows"))]
    { 1 }
}

/// Set DX12 transform (alias for updateMVP — for backward compat)
#[no_mangle]
pub extern "C" fn jdb_dx12_set_transform(_pso: i64, angle: i64) -> i64 {
    jdb_dx12_update_mvp(0, angle)
}

/// Draw mesh (alias — for backward compat)
#[no_mangle]
pub extern "C" fn jdb_dx12_draw_mesh(_pso: i64, _mesh: i64) -> i64 {
    jdb_dx12_draw(0, 36)
}

/// Present (alias — calls end_frame for backward compat)
#[no_mangle]
pub extern "C" fn jdb_dx12_present() -> i64 {
    jdb_dx12_end_frame()
}

/// Destroy DX12 mesh
#[no_mangle]
pub extern "C" fn jdb_dx12_destroy_mesh(_mesh: i64) -> i64 {
    #[cfg(target_os = "windows")]
    unsafe {
        let rs = get_render();
        if rs.vertex_buffer != 0 { com_release(rs.vertex_buffer); rs.vertex_buffer = 0; }
        if rs.cbv_buffer != 0 { com_release(rs.cbv_buffer); rs.cbv_buffer = 0; }
    }
    eprintln!("🔥 [DX12:Pure] Mesh + CBV destroyed");
    1
}

/// Destroy DX12 PSO and all pipeline resources
#[no_mangle]
pub extern "C" fn jdb_dx12_destroy_pso(_pso: i64) -> i64 {
    #[cfg(target_os = "windows")]
    unsafe {
        let rs = get_render();
        // Skip GPU wait — fence not implemented yet
        if rs.pso != 0 { com_release(rs.pso); rs.pso = 0; }
        if rs.root_signature != 0 { com_release(rs.root_signature); rs.root_signature = 0; }
        if rs.cmd_list != 0 { com_release(rs.cmd_list); rs.cmd_list = 0; }
        if rs.cmd_allocator != 0 { com_release(rs.cmd_allocator); rs.cmd_allocator = 0; }
        for i in 0..BUFFER_COUNT as usize {
            if rs.render_targets[i] != 0 { com_release(rs.render_targets[i]); rs.render_targets[i] = 0; }
        }
        if rs.depth_buffer != 0 { com_release(rs.depth_buffer); rs.depth_buffer = 0; }
        if rs.dsv_heap != 0 { com_release(rs.dsv_heap); rs.dsv_heap = 0; }
        if rs.rtv_heap != 0 { com_release(rs.rtv_heap); rs.rtv_heap = 0; }
        if rs.swap_chain != 0 { com_release(rs.swap_chain); rs.swap_chain = 0; }
        if rs.command_queue != 0 { com_release(rs.command_queue); rs.command_queue = 0; }
        if rs.fence != 0 { com_release(rs.fence); rs.fence = 0; }
        if rs.fence_event != 0 { CloseHandle(rs.fence_event); rs.fence_event = 0; }
        // Don't release device — it's shared with dx12.rs context
        rs.device = 0;
        rs.initialized = false;
    }
    eprintln!("🔥 [DX12:Pure] Full pipeline destroyed");
    1
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_dx12_cube_functions_exist() {
        // Without a DX12 device, functions return 0 (not initialized)
        // On non-Windows they return 1 (stub)
        #[cfg(target_os = "windows")]
        {
            // Not initialized → returns 0 or 1 depending on function
            let _ = super::jdb_dx12_create_cube_mesh();
            let _ = super::jdb_dx12_begin_frame();
            let _ = super::jdb_dx12_present();
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(super::jdb_dx12_create_cube_mesh(), 1);
            assert_eq!(super::jdb_dx12_begin_frame(), 1);
            assert_eq!(super::jdb_dx12_present(), 1);
        }
    }

    #[test]
    fn test_mvp_rowmajor() {
        #[cfg(target_os = "windows")]
        {
            let mvp = super::compute_mvp_rowmajor(0.0);
            // Should be a valid matrix with non-zero diagonal
            assert!(mvp[0].abs() > 0.1);
            assert!(mvp[5].abs() > 0.1);
        }
    }
}
