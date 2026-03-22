# ABI Translators — ADead-BIB Binary Frontend

> **PE / ELF / SPIR-V / DXIL → ADead-BIB IR**
> Any binary in, universal IR out.

## Role

Single responsibility:

```text
Convert external binary formats → ADead-BIB IR
```

Does NOT execute. Does NOT optimize. Does NOT compile from source.
Only: **Parse → Decode → Map → Emit ADead-BIB IR**

## Architecture

```text
  .exe / .dll          .elf / .so         .spv            .dxbc / .dxil
       ↓                    ↓               ↓                   ↓
  ┌─────────┐        ┌──────────┐     ┌──────────┐      ┌───────────┐
  │ PE      │        │ ELF      │     │ SPIR-V   │      │ DXIL      │
  │ Parser  │        │ Parser   │     │ Parser   │      │ Parser    │
  │ Decoder │        │ Decoder  │     │ Decoder  │      │ Decoder   │
  │ Mapper  │        │ Mapper   │     │ Mapper   │      │ Mapper    │
  └────┬────┘        └────┬─────┘     └────┬─────┘      └─────┬─────┘
       └──────────┬───────┴────────┬───────┘                  │
                  ↓                ↓                          ↓
            ABIB_Module      ABIB_Module               ABIB_Module
              (CPU IR)         (CPU IR)                  (GPU IR)
```

## Pipeline (every translator)

```text
Binary File → Parser → Decoder → IR Mapper → ABIB_Module
```

## ABIB_Module Structure

```text
ABIB_Module
├── functions[]      (ABIB_Function)
│     └── blocks[]   (ABIB_Block)
│           └── instructions[]  (ABIB_Instruction)
├── globals[]        (ABIB_Global)
├── imports[]        (ABIB_Import)
├── exports[]        (ABIB_Export)
├── gpu_kernels[]    (ABIB_GpuKernel)
└── relocations[]    (ABIB_Relocation)
```

## Registered Translators

| Translator | Format | Domain | Status |
|------------|--------|--------|--------|
| **PE Translator** | Windows PE/PE32+ (.exe/.dll) | CPU | Complete |
| **ELF Translator** | Linux ELF64 (.elf/.so) | CPU | Complete |
| **SPIR-V Translator** | Vulkan/OpenCL (.spv) | GPU | Complete |
| **DXIL Translator** | DirectX 12 DXBC/DXIL | GPU | Complete |

## Usage

### CLI

```bash
# Translate a PE executable
abi-translate program.exe

# Translate with full IR dump
abi-translate program.exe --dump

# Translate a SPIR-V shader
abi-translate shader.spv --dump

# Save IR to file
abi-translate program.exe -o output.abib

# List translators
abi-translate --list

# Run built-in demo (PE + SPIR-V)
abi-translate --demo
```

### Options

| Flag | Description |
|------|-------------|
| `-o <path>` | Save IR dump to file |
| `--info` | Show module summary |
| `--dump` | Dump all decoded instructions |
| `--demo` | Run built-in demo |
| `--list` | List registered translators |

### Rust API

```rust
use abi_translators::core::registry;
use abi_translators::core::translator::BinaryView;

let reg = registry::build_default_registry();
let view = BinaryView::from_file("program.exe")?;
let module = reg.translate(&view)?;

println!("Functions: {}", module.functions.len());
println!("Imports:   {}", module.imports.len());
println!("Instructions: {}", module.total_instructions());
```

## Project Structure

```text
abi_translators/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs                # CLI entry point
    ├── lib.rs                 # Library root
    ├── core/
    │   ├── mod.rs             # Core module
    │   ├── ir.rs              # ABIB IR types (Module, Function, Block, Instruction)
    │   ├── translator.rs      # ABIBTranslator trait + BinaryView
    │   ├── registry.rs        # TranslatorRegistry + auto-detection
    │   └── context.rs         # TranslationContext (IR builder)
    ├── cpu/
    │   ├── mod.rs             # CPU translators module
    │   ├── pe.rs              # PE parser + decoder + pe_to_abib
    │   └── elf.rs             # ELF parser + decoder + elf_to_abib
    ├── gpu/
    │   ├── mod.rs             # GPU translators module
    │   ├── spirv.rs           # SPIR-V parser + decoder + spirv_to_abib
    │   └── dxil.rs            # DXBC/DXIL parser + decoder + dxil_to_abib
    └── utils/
        ├── mod.rs             # Utilities module
        ├── binary_reader.rs   # Cursor-based binary reader
        ├── x86_decoder.rs     # x86-64 instruction decoder → ABIB ops
        └── symbol_table.rs    # Symbol resolution table
```

## x86-64 Decoder Coverage

Supported instructions:

- **Data**: MOV, PUSH, POP, LEA, XCHG, MOVZX
- **Arithmetic**: ADD, SUB, CMP, INC, DEC, NEG
- **Bitwise**: AND, OR, XOR, NOT, SHL, SHR
- **Control**: CALL, RET, JMP, Jcc (JE/JNE/JG/JL/JA/JB...)
- **System**: SYSCALL, INT, HLT, UD2, NOP
- **Stack**: ENTER, LEAVE

## GPU IR Opcodes

| Opcode | Source |
|--------|--------|
| GpuLoad | SPIR-V OpLoad, DXIL ld_uav |
| GpuStore | SPIR-V OpStore, DXIL store_uav |
| GpuAdd | SPIR-V OpIAdd/OpFAdd, DXIL add |
| GpuMul | SPIR-V OpIMul/OpFMul, DXIL mul |
| GpuFma | DXIL mad |
| GpuDot | SPIR-V OpDot, DXIL dp3/dp4 |
| GpuBarrier | SPIR-V OpControlBarrier, DXIL sync |
| GpuSync | DXIL sync |

## Build

```bash
cd abi_translators
cargo build --release
```

## Demo Output

```text
=== ADead-BIB ABI Translator — Demo ===

--- Demo 1: Synthetic PE Binary ---
  Format: PE
  Size: 1024 bytes
  Translator: PE Translator (Windows x86-64)
=== ABIB Module: demo.exe ===
  Source:       PE
  Type:         Cpu
  Arch:         x86-64
  Functions:    1
  Instructions: 4 (SUB, MOV, ADD, RET)

--- Demo 2: Synthetic SPIR-V Binary ---
  Format: SPIRV
  Size: 148 bytes
  Translator: SPIR-V Translator (Vulkan/OpenCL)
=== ABIB Module: demo.spv ===
  Source:       SPIRV
  Type:         Gpu
  GPU Kernels:  1 (Compute, local_size=[64, 1, 1])
  Entry Point:  main
```

## License

Part of ADead-BIB — Binary Is Binary.
