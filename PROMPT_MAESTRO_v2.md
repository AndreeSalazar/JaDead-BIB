# 💀🦈 PROMPT MAESTRO v2 — JaDead-BIB DX12 Backend Correcto
**Para: Claude Opus 4.6**
**Proyecto: JaDead-BIB — Java → x86-64 nativo sin JVM**
**Lima, Perú 🇵🇪 — Binary Is Binary**

---

## 🎯 MISIÓN PRINCIPAL

Reemplazar `dx12_cube.rs` por un **backend DX12 interno limpio** para JaDead-BIB.

El archivo actual mezcla lógica de inicialización, pipeline, y cubo TODO junto — no es arquitectura correcta para un compilador. La nueva arquitectura debe tener **funciones `jdb_dx12_*` limpias y separadas** donde **Java (CubeDX12.java) controla todo el flujo**.

---

## 🏗️ ARQUITECTURA OBJETIVO

```
CubeDX12.java (Java — controla TODO)
    ↓ llama
jdb_dx12_init()           ← init base
jdb_dx12_create_swapchain()
jdb_dx12_create_pipeline()
jdb_dx12_create_mesh()
jdb_dx12_create_cbv()
    ↓ render loop en Java
jdb_dx12_begin_frame()
jdb_dx12_clear_rtv()
jdb_dx12_update_mvp()
jdb_dx12_set_cbv()
jdb_dx12_draw()
jdb_dx12_end_frame()
    ↓ cleanup en Java
jdb_dx12_destroy()
```

**Regla de oro:** cada función hace **UNA sola cosa**. Java decide el flujo. Rust solo ejecuta.

---

## 📁 ARCHIVOS

```
Proyecto: C:\Users\andre\OneDrive\Documentos\JaDead-BIB\

REEMPLAZAR completamente:
src/rust/backend/gpu/dx12_cube.rs  ← nuevo backend limpio

NO tocar:
src/rust/backend/gpu/dx12.rs       ← contexto DX12 base (compartido)
src/rust/backend/gpu/window.rs     ← Win32 window
java_test/gpu/cubes/dx12/CubeDX12.java  ← Java OK, no cambiar
Cargo.toml                         ← SIN dependencias externas

Comando: cargo run -- run "java_test/gpu/cubes/dx12/CubeDX12.java"
```

---

## 🔥 BUG CRÍTICO A RESOLVER EN EL NUEVO BACKEND

### El crash actual
```
STATUS_ACCESS_VIOLATION (0xc0000005)
en CreateRenderTargetView (vtable index 20)
```

### Causa raíz
`D3D12_CPU_DESCRIPTOR_HANDLE` es struct de 8 bytes. MSVC x64 ABI lo pasa **por valor en registro** como `u64`. Rust `extern "system"` lo pasa diferente causando ACCESS_VIOLATION.

### Patrón CORRECTO para handles en todo el nuevo backend

```rust
// ❌ NUNCA hacer esto — pasa puntero al struct
vtable_call!(device, 20,
    extern "system" fn(usize, usize, usize, *const D3D12CpuDescriptorHandle),
    resource, 0usize, &handle);

// ✅ SIEMPRE hacer esto — pasa valor usize directo
let handle_value: usize =
    rtv_heap_start + (i * rtv_desc_size);
vtable_call!(device, 20,
    extern "system" fn(usize, usize, usize, usize),
    resource, 0usize, handle_value);
```

### Funciones que necesitan este fix en el nuevo backend
- `CreateRenderTargetView` vtable 20 → handle como `usize` ✅
- `ClearRenderTargetView` vtable 48 → handle como `u64` (ya correcto en original)
- `OMSetRenderTargets` vtable 46 → handle como `usize` ✅
- `GetCPUDescriptorHandleForHeapStart` vtable 9 → retorna en `*mut usize` ✅

---

## 📋 VTABLE INDICES CONFIRMADOS (NO CAMBIAR)

### ID3D12Device
| Método | Índice |
|--------|--------|
| CreateCommandQueue | 8 |
| CreateCommandAllocator | 9 |
| CreateCommandList | 12 |
| CreateDescriptorHeap | 14 |
| GetDescriptorHandleIncrementSize | 15 |
| CreateRootSignature | 16 |
| **CreateRenderTargetView** | **20** |
| CreateCommittedResource | 27 |
| CreateFence | 36 |
| CreateGraphicsPipelineState | 10 |

### ID3D12GraphicsCommandList
| Método | Índice |
|--------|--------|
| Close | 9 |
| Reset | 10 |
| DrawInstanced | 12 |
| RSSetViewports | 21 |
| RSSetScissorRects | 22 |
| SetPipelineState | 25 |
| ResourceBarrier | 26 |
| SetGraphicsRootSignature | 30 |
| SetGraphicsRootConstantBufferView | 38 |
| IASetVertexBuffers | 44 |
| OMSetRenderTargets | 46 |
| ClearRenderTargetView | 48 |

### Otros
| Objeto | Método | Índice |
|--------|--------|--------|
| ID3D12CommandQueue | ExecuteCommandLists | 10 |
| ID3D12CommandQueue | Signal | 14 |
| IDXGISwapChain | Present | 8 |
| IDXGISwapChain | GetBuffer | 9 |
| IDXGIFactory2 | CreateSwapChainForHwnd | 15 |
| ID3D12Fence | GetCompletedValue | 8 |
| ID3D12Fence | SetEventOnCompletion | 9 |
| ID3D12DescriptorHeap | GetCPUDescriptorHandleForHeapStart | 9 |
| ID3D12CommandAllocator | Reset | 8 |
| ID3D12Resource | Map | 8 |
| ID3D12Resource | GetGPUVirtualAddress | 11 |

---

## 🧩 FUNCIONES jdb_dx12_* DEL NUEVO BACKEND

Cada función debe ser **limpia, separada, con una responsabilidad**:

### Grupo 1 — Inicialización
```rust
// Reusa device de dx12.rs, crea Queue + SwapChain
pub extern "C" fn jdb_dx12_create_swapchain(hwnd: usize) -> i64

// Crea RTV heap + descriptors (con fix handle usize)
pub extern "C" fn jdb_dx12_create_rtv_heap() -> i64

// Crea CommandAllocator + CommandList + Fence
pub extern "C" fn jdb_dx12_create_command_resources() -> i64
```

### Grupo 2 — Pipeline
```rust
// Compila shaders HLSL + crea RootSignature + PSO
pub extern "C" fn jdb_dx12_create_pipeline(
    vert_path: *const JdbString,
    pixel_path: *const JdbString,
) -> i64
```

### Grupo 3 — Recursos
```rust
// Crea vertex buffer con datos del cubo
pub extern "C" fn jdb_dx12_create_vertex_buffer() -> i64

// Crea constant buffer 256-byte aligned
pub extern "C" fn jdb_dx12_create_constant_buffer() -> i64
```

### Grupo 4 — Render loop (llamados desde Java cada frame)
```rust
pub extern "C" fn jdb_dx12_begin_frame() -> i64
pub extern "C" fn jdb_dx12_clear_rtv(r: i64, g: i64, b: i64) -> i64
pub extern "C" fn jdb_dx12_update_mvp(angle: i64) -> i64
pub extern "C" fn jdb_dx12_set_pipeline() -> i64
pub extern "C" fn jdb_dx12_draw(vertex_count: i64) -> i64
pub extern "C" fn jdb_dx12_end_frame() -> i64
```

### Grupo 5 — Cleanup
```rust
pub extern "C" fn jdb_dx12_destroy_pipeline() -> i64
pub extern "C" fn jdb_dx12_destroy_resources() -> i64
```

### Aliases de compatibilidad (mantener para no romper Java)
```rust
// jdb_dx12_create_pso → llama create_swapchain + create_rtv_heap + create_command_resources + create_pipeline
// jdb_dx12_create_cube_mesh → llama create_vertex_buffer
// jdb_dx12_create_cbv → llama create_constant_buffer
// jdb_dx12_present → alias de end_frame
// jdb_dx12_destroy_pso → llama destroy_pipeline
// jdb_dx12_destroy_mesh → llama destroy_resources
```

---

## 📊 ESTADO ACTUAL DEL BACKEND (lo que ya funciona)

```
✅ Window Created 800x600
✅ d3d12.dll + dxgi.dll loaded
✅ Feature Level: 12_0
✅ RTX 3060 detected
✅ Device ready (compartido con dx12.rs)
✅ Command Queue created
✅ SwapChain created (2 buffers FLIP_DISCARD)
✅ RTV heap created
✅ RTV desc size=32
✅ handle start=0x280FC18F440
✅ Buffer 0 = 0x280FC187B40
❌ CRASH en CreateRenderTargetView ← FIX con handle usize
```

---

## 🚫 REGLAS ABSOLUTAS

1. **CERO dependencias externas** — Cargo.toml queda vacío. Sin `windows-sys`, sin `windows`, sin nada. JaDead-BIB no usa muletas.

2. **Java controla el flujo** — CubeDX12.java NO cambia. El backend Rust solo ejecuta lo que Java pide.

3. **Una función = una responsabilidad** — no mezclar init + pipeline + render en una sola función gigante como hacía `jdb_dx12_create_pso`.

4. **Handles siempre como usize/u64** — NUNCA pasar `D3D12_CPU_DESCRIPTOR_HANDLE` como struct o puntero a struct en vtable calls.

5. **Rust es motor interno** — el usuario programa Java normal. Rust es transparente.

6. **Vtable indices NO cambian** — ya están verificados con d3d12.h estándar.

---

## ✅ RESULTADO ESPERADO

```
✅ Window Created 800x600
✅ d3d12.dll + dxgi.dll loaded
✅ Feature Level: 12_0
✅ RTX 3060 detected
✅ Device ready
✅ SwapChain created
✅ RTV Heap + 2 Render Targets created  ← sin crash
✅ Command resources created
✅ Shaders compiled (VS + PS)
✅ Root Signature created
✅ PSO created
✅ Vertex Buffer created (cubo 36 verts)
✅ Constant Buffer created (256 bytes)
🎮 Render loop corriendo...
🎮 Cubo DX12 girando en ventana 800x600
```

---

## 💡 CONTEXTO ADICIONAL

**¿Por qué Rust como backend?**
JaDead-BIB compila Java → x86-64 machine code. Rust es el motor que genera ese machine code. No es que el usuario use Rust — el usuario escribe Java normal. Rust está completamente oculto.

**¿Por qué sin dependencias?**
Filosofía Dead-BIB: Binary Is Binary. Si puedes llamar vtable DX12 directamente desde Rust con transmute, no necesitas wrapper. Es más directo, más pequeño, más honesto.

**Hardware objetivo:**
AMD Ryzen 5 5600X + RTX 3060 12GB + Windows 11 + CUDA 12.1

---

## 📈 MÉTRICAS DEL PROYECTO

| Compilador | Estado |
|------------|--------|
| ADead-BIB C/C++ | ✅ v8.0 — 354K líneas — sin LLVM |
| PyDead-BIB Python | ✅ v4.0 — 0.305ms — primer fork (rwdim Dallas TX) |
| JaDead-BIB Java | ✅ v1.0 — 184 tests PASS — 0.1841ms — OpenGL ✅ Vulkan ✅ DX12 ❌ fix hoy |
| FastOS | ✅ 89KB — bootea QEMU — 256-bit ready |

---

*JaDead-BIB v1.0 — Lima, Perú 🇵🇪 — Binary Is Binary 💀🦈*
*Generado: 16/03/2026*
