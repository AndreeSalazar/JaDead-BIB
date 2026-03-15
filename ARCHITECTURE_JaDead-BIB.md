# JaDead-BIB 💀☕
> **Java → x86-64 Nativo — Sin JVM — Sin GC — Sin Runtime — Sin Oracle**

```
James Gosling:    'write once, run anywhere'
Bjarne Stroustrup:'zero overhead abstractions'  
Grace Hopper:     'la máquina sirve al humano'
JaDead-BIB 2026:  hereda ADead-BIB v8.0 + JIT 2.0 — Java nativo — sin JVM
```

---

## Filosofía Central

```
SIN JVM — NUNCA
  java.exe      ❌ eliminado
  HotSpot JIT   ❌ eliminado (reemplazado por JIT 2.0)
  GC pauses     ❌ eliminado (ownership estático)
  "Stop World"  ❌ nunca más
  runtime 200MB ❌ 0 bytes

SIN INTERMEDIARIO
  Java → javac → .class → JVM → JIT → machine code  ❌ viejo
  Java → IR ADeadOp → x86-64 nativo                 ✅ JaDead-BIB

HEREDA Dead-BIB familia:
  IR ADeadOp SSA    → reutilizado 100%  ✅
  ISA x86-64        → reutilizado 100%  ✅
  UB Detector       → extendido Java    ✅
  JIT 2.0 Killer    → integrado         ✅
  PE/ELF output     → reutilizado 100%  ✅
  BG Binary Guardian→ reutilizado 100%  ✅

JAVA SINTAXIS COMPLETA:
  Java 8  → streams, lambdas, Optional     ✅
  Java 11 → var, HTTP client, String API   ✅
  Java 17 → records, sealed, pattern match ✅
  Java 21 → virtual threads, sequenced     ✅

HELLO WORLD:
  CPython:    30MB runtime                 ❌
  JVM:        200MB runtime                ❌
  JaDead-BIB: ~2KB PE nativo               ✅
```

---

## Comparación

| Característica | JVM HotSpot | GraalVM | JaDead-BIB |
|---|---|---|---|
| Sin runtime | ❌ 200MB | ❌ 50MB | ✅ **0 bytes** |
| Sin GC pauses | ❌ | ❌ partial | ✅ **ownership** |
| Sin JVM | ❌ | ❌ | ✅ |
| Sin LLVM | ✅ | ❌ | ✅ |
| JIT 2.0 | ❌ aprende | ❌ aprende | ✅ **ya sabe** |
| Hello World | 200MB | 50MB | **~2KB** |
| Startup | ~2000ms | ~100ms | **0.305ms** |
| UB compile-time | ❌ | partial | ✅ **15+ tipos** |
| OpenGL/Vulkan/DX12 | via JOGL | via JOGL | ✅ **nativo directo** |

---

## Pipeline Completo

```
Java Source (.java)
        │
        ▼
┌─────────────────────────────────────────────────────┐
│  FRONTEND (★ JaDead-BIB v1.0)                       │
│                                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │ Preprocessor│→ │Import Elim. │→ │   Lexer     │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  │
│         │                                │          │
│         ▼                                ▼          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │   Parser    │→ │Type Checker │→ │  IR Gen     │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────┐
│  MIDDLE-END (heredado ADead-BIB v8.0)               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │ UB Detector │→ │  Optimizer  │→ │ Reg Alloc   │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────┐
│  JIT 2.0 KILLER (heredado PyDead-BIB)               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │Dispatch Pre │→ │Instant Image│→ │VirtualAlloc │  │
│  │  Resolved   │  │Pre-Patched  │  │   → JMP     │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  │
│  "El CPU no piensa — ya sabe. La RAM no espera."    │
└─────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────┐
│  BACKEND (heredado ADead-BIB v8.0)                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │Bit Resolver │→ │ISA Compiler │→ │ PE/ELF/Po   │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────┐
│  GPU DISPATCH (nuevo + heredado PyDead-BIB)         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │   OpenGL    │  │   Vulkan    │  │   DX12      │  │
│  │  ctypes     │  │  SPIR-V     │  │  HLSL       │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────┘
        │
        ▼
   .exe / ELF (nativo, sin JVM, sin runtime)
```

---

## Frontend Java — Fases

### Phase 01 — PREPROCESSOR
```
ja_preprocessor.rs:
→ encoding detection UTF-8
→ annotation processing (@Override, @FunctionalInterface)
→ package resolution
→ import tree shaking (como header_main.h en ADead-BIB)
→ generics erasure preparation
→ fastos.bib cache (CACHE HIT = nanosegundos)
```

### Phase 02 — IMPORT ELIMINATOR
```
ja_import_resolver.rs:
→ java.lang.* → inline nativo (String, Math, System)
→ java.util.* → estructuras nativas (ArrayList → HeapAlloc)
→ java.io.* → syscalls directos
→ sin classpath en runtime — NUNCA
→ "ClassNotFoundException" en compile time — no runtime ✅
```

### Phase 03 — LEXER
```
ja_lexer.rs:
Tokens Java completos:
→ Keywords: class, interface, enum, record, sealed (Java 17+)
→ Primitivos: int, long, float, double, boolean, char, byte, short
→ Literales: 42, 3.14f, 0xFF, 0b1010, "string", 'c'
→ Operadores: instanceof (pattern matching Java 16+)
→ switch expressions (Java 14+)
→ Text blocks (Java 15+): """..."""
→ var (Java 10+): tipo inferido
```

### Phase 04 — PARSER / AST
```
ja_parser.rs:
→ Classes, interfaces, abstract, enums
→ Records (Java 16+): record Point(int x, int y)
→ Sealed classes (Java 17+): sealed interface Shape
→ Pattern matching: if (obj instanceof String s)
→ Switch expressions: yield
→ Lambdas: (x) -> x * 2
→ Streams: list.stream().filter().map().collect()
→ Generics: List<T>, Map<K,V>
→ Annotations completas
→ Inner classes, anonymous classes
→ try-with-resources
→ Multi-catch: catch (A | B e)
```

### Phase 05 — TYPE CHECKER
```
ja_types.rs:
Java tiene tipos ESTÁTICOS → ventaja sobre Python:

→ int    → I32/I64
→ long   → I64
→ float  → F32 (XMM)
→ double → F64 (XMM/YMM)
→ boolean→ I8
→ char   → I16
→ String → Ptr (.data section)
→ T[]    → HeapAlloc array
→ List<T>→ HeapAlloc + vtable
→ generics → monomorphization (como Rust/C++ templates)

VENTAJA vs PyDead-BIB:
Java tipos explícitos desde siempre →
type checker más fácil →
menos inferencia necesaria →
machine code más directo 💀🦈
```

### Phase 06 — IR ADeadOp (heredado 100%)
```
ja_to_ir.rs:
Java AST → ADeadOp SSA-form

int suma(int a, int b) → ADD RAX, RBX directo
String s = "hola"     → LoadString .data
new ArrayList<>()     → HeapAlloc
lambda (x) -> x*2     → inline function
stream.filter()       → loop IR unrolled
```

---

## Java Completo Soportado

### Primitivos y Tipos
```java
// Todos compilados a tipos nativos
int    x = 42;          // → RAX I32
long   l = 100L;        // → RAX I64
float  f = 3.14f;       // → XMM0 F32
double d = 3.14;        // → XMM0 F64
boolean b = true;       // → 1 byte
char   c = 'A';         // → I16 literal
String s = "hola";      // → .data section

// var (Java 10+)
var nombre = "Eddi";    // → inferido String → Ptr
var numero = 42;        // → inferido int → I32
```

### Clases y OOP
```java
// Clase → struct + vtable (como ADead-BIB C++)
public class Jugador {
    private String nombre;
    private int vida;
    
    public Jugador(String nombre) {
        this.nombre = nombre;  // → struct field offset
        this.vida = 100;
    }
    
    public int atacar() {
        return 10;             // → vtable entry
    }
}

// Herencia → vtable override
public class Guerrero extends Jugador {
    @Override
    public int atacar() {
        return 25;             // → vtable override
    }
}

// Interface → vtable pura
public interface Atacable {
    int atacar();              // → vtable entry abstracta
}
```

### Records (Java 16+)
```java
// Record → struct inmutable nativo
record Point(int x, int y) {}
record Jugador(String nombre, int vida) {}

// JaDead-BIB compila:
// Point → struct { i32 x; i32 y; }
// sin GC, sin overhead, 8 bytes exactos
```

### Sealed Classes (Java 17+)
```java
// Sealed → jump table exhaustivo
sealed interface Shape 
    permits Circle, Rectangle, Triangle {}

record Circle(double radio) implements Shape {}
record Rectangle(double w, double h) implements Shape {}

// switch exhaustivo → branchless O(1)
double area = switch (shape) {
    case Circle c    -> Math.PI * c.radio() * c.radio();
    case Rectangle r -> r.w() * r.h();
    case Triangle t  -> 0.5 * t.base() * t.altura();
};
```

### Lambdas y Streams
```java
// Lambda → inline function nativa
var doble = (int x) -> x * 2;
// → mov rax, rdi; shl rax, 1; ret

// Stream → loop IR unrolled
List<Integer> nums = List.of(1, 2, 3, 4, 5);
var suma = nums.stream()
               .filter(n -> n % 2 == 0)
               .mapToInt(n -> n * 2)
               .sum();
// JaDead-BIB: detecta stream de int →
// genera loop AVX2 automático 💀🦈
```

### Generics → Monomorphization
```java
// Generics → especializados en compile time
// (como C++ templates, no erasure de JVM)

class Stack<T> {
    private T[] data;
    public void push(T item) { ... }
    public T pop() { ... }
}

Stack<Integer> si = new Stack<>();  // → Stack_I32 nativo
Stack<String>  ss = new Stack<>();  // → Stack_Ptr nativo
// sin boxing, sin overhead de tipo genérico 💀🦈
```

### Concurrencia sin GIL ni locks innecesarios
```java
// Virtual threads (Java 21+) → ownership estático
Thread.ofVirtual().start(() -> {
    procesar(datos);
});
// JaDead-BIB: sin GIL, ownership estático
// → threads reales en Ryzen 5 5600X 12 threads 💀🦈
```

---

## UB Detector Java — 15+ Tipos

```rust
pub enum JavaUB {
    // Heredados de C/C++ (ADead-BIB)
    NullDeref,              // null.method() → NullPointerException pre-detectado
    ArrayIndexOutOfBounds,  // arr[100] con arr[10] → pre-detectado
    DivisionByZero,         // x / 0 → pre-detectado
    IntegerOverflow,        // int + int overflow → warning
    
    // Java-specific
    ClassCastException,     // (String) integer → pre-detectado
    StackOverflow,          // recursión sin base → pre-detectado
    ConcurrentModification, // modificar lista en foreach → pre-detectado
    NegativeArraySize,      // new int[-1] → pre-detectado
    StringIndexOutOfBounds, // "hola".charAt(100) → pre-detectado
    NumberFormatException,  // Integer.parseInt("abc") → warning
    EmptyOptional,          // Optional.get() sin isPresent() → pre-detectado
    UncheckedCast,          // cast genérico sin verificar → warning
    DeadLock,               // patrones de deadlock → warning
    ResourceLeak,           // stream/file sin close() → pre-detectado
    UnsafePublicField,      // campo público mutable en record → warning
}

// JVM: todos → excepción en RUNTIME ❌
// JaDead-BIB: todos → detectados COMPILE TIME ✅
```

---

## JIT 2.0 Killer — Heredado y Mejorado

```
JVM HotSpot JIT 1.0:
"no sé los tipos de Java en runtime"

MENTIRA — Java tiene tipos estáticos.
El problema era diferente: el JVM
apostaba a que el código se calientara
para optimizar. Ineficiente.

JaDead-BIB JIT 2.0:
Java tiene tipos estáticos DESDE SIEMPRE →
JaDead-BIB los conoce en compile time →
dispatch pre-resuelta →
instant image →
VirtualAlloc → JMP →
0.305ms

"El CPU no piensa — ya sabe.
 La RAM no espera — ya recibe."

Java + JIT 2.0 = MÁS fácil que Python + JIT 2.0
porque Java ya tenía tipos explícitos 💀🦈
```

---

## GPU Dispatch — OpenGL + Vulkan + DX12

### OpenGL Nativo
```java
// Java normal usa JOGL (librería pesada)
// JaDead-BIB: ctypes directo a opengl32.dll

import gpu.OpenGL;  // → JaDead-BIB resuelve a opengl32.dll

OpenGL.glClear(OpenGL.GL_COLOR_BUFFER_BIT);
OpenGL.glBegin(OpenGL.GL_TRIANGLES);
OpenGL.glVertex3f(0.0f, 1.0f, 0.0f);
OpenGL.glEnd();

// JaDead-BIB genera:
// DllLoad("opengl32.dll")
// GetProc("glClear") → call directo
// sin JOGL, sin overhead, sin JVM 💀🦈
```

### Vulkan Nativo
```java
// Java normal: no tiene Vulkan nativo oficial
// JaDead-BIB: ctypes → vulkan-1.dll + SPIR-V

import gpu.Vulkan;

var instance = Vulkan.vkCreateInstance(info);
var device   = Vulkan.vkCreateDevice(physDev, info);
var shader   = Vulkan.vkCreateShaderModule(device, spirv);
Vulkan.vkCmdDispatch(cmdBuf, 64, 64, 1);

// IR instructions:
// VkInit → VkDeviceCreate → VkShaderLoad
// → VkDispatch → VkBufferRead
// RTX 3060 → 24,119 GFLOPS desde Java compilado 💀🦈
```

### DirectX 12 Nativo
```java
// Solo Windows — DX12 directo
import gpu.DirectX12;

var device = DirectX12.D3D12CreateDevice(adapter, featureLevel);
var queue  = DirectX12.CreateCommandQueue(device, desc);
var list   = DirectX12.CreateCommandList(device, type, allocator);

DirectX12.DrawInstanced(list, vertexCount, 1, 0, 0);

// JaDead-BIB genera:
// DllLoad("d3d12.dll")
// GetProc("D3D12CreateDevice") → call directo
// sin overhead JVM, sin JOGL, directo al metal 💀🦈
```

### CUDA desde Java
```java
// Java normal: no tiene CUDA oficial sin JVM
// JaDead-BIB: ctypes → nvcuda.dll

import gpu.CUDA;

CUDA.cuInit(0);
var ctx = CUDA.cuCtxCreate(0, device);
var mem = CUDA.cuMemAlloc(size);
CUDA.cuMemcpyHtoD(mem, hostData, size);
CUDA.cuLaunchKernel(kernel, gridX, gridY, gridZ,
                    blockX, blockY, blockZ,
                    sharedMem, stream, params);

// RTX 3060 → 24,119 GFLOPS desde Java compilado
// sin JVM, sin JCuda, sin overhead 💀🦈
```

---

## Minecraft — El Caso Obvio

```java
// Minecraft Java Edition problema real:
// GC "Stop The World" → chunk loading lag
// jugadores lo conocen y odian

// Con JaDead-BIB:
public class ChunkLoader {
    // sin GC → sin Stop The World
    // ownership estático → sin pauses
    // 0.305ms startup
    // chunk loading sin lag
    
    public void loadChunk(int x, int z) {
        // machine code nativo
        // sin JVM overhead
        // sin GC interference
    }
}

// Microsoft/Mojang:
// "el lag de Minecraft desaparece?"
// "._."
// "cuánto cuesta?"
// Techne License: "10%"
// "firmado" 💀🦈
// Oracle: "oye espera—"
// Netflix lawyers: "nosotros pagamos" 😂
```

---

## Configuración — jab.toml

```toml
[project]
name = "mi_app_java"
version = "0.1.0"
lang = "java"
standard = "java21"

[build]
src = "src/"
output = "bin/"

[java]
version = "21"           # Java 21 LTS
generics = "monomorphize" # como C++ templates
gc = "none"              # ownership estático
ub_mode = "warn"         # --warn-ub (no strict, respeta a Bjarne)
jit = "2.0"              # JIT Killer 2.0 activado

[gpu]
backend = "auto"         # detecta OpenGL/Vulkan/DX12/CUDA
device = 0               # RTX 3060
spirv = true             # Vulkan SPIR-V habilitado
```

---

## Comandos CLI

```bash
# Compilar Java
jab java Archivo.java -o output.exe

# Target específico
jab java Archivo.java --target windows    # PE x64
jab java Archivo.java --target linux      # ELF x64
jab java Archivo.java --target fastos256  # FastOS 256-bit

# JIT 2.0 — ejecutar sin .exe
jab run Archivo.java

# Step Mode — ver todas las fases
jab step Archivo.java

# UB modes
jab java Archivo.java --warn-ub    # avisa, no bloquea
jab java Archivo.java --strict-ub  # bloquea en UB
jab java Archivo.java              # sin flags, compila igual

# Build proyecto
jab build   # lee jab.toml

# Crear proyecto
jab create mi_app
jab create mi_juego --gpu vulkan
```

---

## Step Mode Java

```
╔══════════════════════════════════════════════════════════════╗
║   JaDead-BIB Step Compiler — Deep Analysis Mode 💀☕        ║
╚══════════════════════════════════════════════════════════════╝
  Source:   Jugador.java
  Language: Java 21

--- Phase 01: PREPROCESSOR ---
[PREPROC]  encoding: UTF-8 detectado
[PREPROC]  annotations: @Override, @FunctionalInterface
[PREPROC]  package: com.juego → resuelto

--- Phase 02: IMPORT ELIMINATOR ---
[IMPORT]   java.lang.Math → SIMD inline
[IMPORT]   java.util.List → HeapAlloc nativo
[IMPORT]   sin classpath en runtime — NUNCA

--- Phase 03: LEXER ---
[LEXER]    342 tokens generados
[LEXER]    generics: 8 detectados → monomorphize

--- Phase 04: PARSER ---
[PARSER]   class Jugador (3 fields, 5 methods)
[PARSER]   record Point (2 components) → struct 8 bytes
[PARSER]   sealed Shape → 3 permits → jump table O(1)
[PARSER]   lambda × 4 → inline functions

--- Phase 05: TYPE CHECKER ---
[TYPES]    int vida → I32 garantizado
[TYPES]    String nombre → Ptr .data
[TYPES]    List<Integer> → HeapAlloc_I32 (sin boxing)
[TYPES]    Generic<T> → monomorphized ✓

--- Phase 06: IR (ADeadOp SSA-form) ---
[IR]       12 functions compiled
[IR]       87 IR statements total
[IR]       GC eliminado — ownership estático ✓
[IR]       JVM eliminado — machine code directo ✓

--- Phase 07: UB DETECTOR ---
[UB]       ⚠ NullDeref posible — línea 23
           jugador.getNombre() sin null check
           fix: if (jugador != null)
[UB]       ✓ sin UB críticos detectados

--- Phase 08: OPTIMIZER ---
[OPT]      stream List<Integer> → AVX2 loop ★
[OPT]      constexpr Math.PI → literal compilado
[OPT]      dead code eliminado: 3 branches

--- Phase 09: JIT 2.0 KILLER ---
[JIT]      dispatch table pre-resuelta ✓
[JIT]      instant image pre-patched ✓
[JIT]      CPU: Ryzen 5 5600X AVX2 ✓ SSE4.2 ✓
[JIT]      VirtualAlloc → JMP directo

--- Phase 10: REGISTER ALLOCATOR ---
[REGALLOC] LinearScan — spill 0 — 13 registros OK

--- Phase 11: OUTPUT ---
[OUTPUT]   Target: Windows PE x64
[OUTPUT]   Code:  3,847 bytes (.text)
[OUTPUT]   Data:  312 bytes (.data)
[OUTPUT]   JVM mismo programa: 200MB runtime 💀

✅ JaDead-BIB compilation complete
   Sin JVM — Sin GC — Sin Runtime — Sin Oracle 💀☕
   time-to-RAM: 0.305ms
```

---

## Estructura del Proyecto

```
JaDead-BIB/
├── Cargo.toml
├── src/rust/
│   ├── lib.rs
│   ├── main.rs              # CLI jab
│   ├── frontend/java/       # Frontend Java (★ nuevo)
│   │   ├── ja_preprocessor.rs
│   │   ├── ja_import_resolver.rs
│   │   ├── ja_lexer.rs
│   │   ├── ja_parser.rs
│   │   ├── ja_ast.rs
│   │   ├── ja_types.rs
│   │   ├── ja_generics.rs   # Monomorphization
│   │   └── ja_to_ir.rs
│   ├── middle/              # HEREDADO ADead-BIB
│   │   ├── ir.rs
│   │   └── ub_detector.rs   # + Java UB types
│   └── backend/             # HEREDADO ADead-BIB
│       ├── isa.rs
│       ├── jit.rs           # JIT 2.0 HEREDADO PyDead-BIB
│       ├── gpu/
│       │   ├── opengl.rs    # OpenGL ctypes dispatch
│       │   ├── vulkan.rs    # Vulkan + SPIR-V
│       │   ├── dx12.rs      # DirectX 12
│       │   └── cuda.rs      # CUDA nvcuda.dll
│       └── output/
│           ├── pe.rs        # Windows .exe
│           └── elf.rs       # Linux ELF
```

---

## Roadmap

### v1.0 — Java Core ✅ (objetivo)
- [ ] Lexer Java completo (Java 8 → 21)
- [ ] Parser: clases, interfaces, enums, records
- [ ] Type checker con monomorphization
- [ ] IR generation desde Java AST
- [ ] UB detector 15+ tipos Java
- [ ] Hello World → 2KB PE nativo

### v1.1 — Java Modern
- [ ] Records (Java 16+)
- [ ] Sealed classes (Java 17+)
- [ ] Pattern matching completo
- [ ] Switch expressions
- [ ] Text blocks

### v1.2 — GPU Dispatch
- [ ] OpenGL → opengl32.dll ctypes
- [ ] Vulkan → vulkan-1.dll + SPIR-V
- [ ] DX12 → d3d12.dll
- [ ] CUDA → nvcuda.dll

### v2.0 — Production Ready
- [ ] Virtual threads (Java 21)
- [ ] Generics monomorphization completa
- [ ] Streams → AVX2 automático
- [ ] Minecraft compatible (sin GC lag)
- [ ] PyPI-style package registry

---

## Relación con la Familia Dead-BIB

```
ADead-BIB v8.0:    C/C++   → Bjarne filosofía  (flexible con UB)
PyDead-BIB v4.0:   Python  → Guido filosofía   (UB obligatorio)
JaDead-BIB v1.0:   Java    → Gosling filosofía  (warn UB, no strict)

Comparten (100% heredado):
→ IR ADeadOp SSA ✅
→ ISA x86-64 ✅
→ JIT 2.0 Killer ✅
→ PE/ELF output ✅
→ Binary Guardian ✅
→ Binary Is Binary ✅

No comparten (intencionalmente separados):
→ frontend ❌ (cada lenguaje su parser)
→ UB philosophy ❌ (cada lenguaje sus reglas)
→ type system ❌ (Java generics ≠ Python duck)
→ GC model ❌ (Java GC eliminado diferente)

= familia, no monolito
= hermanos, no GCC
= cada uno pequeño y enfocado
= juntos cubren todo 💀☕🦈🇵🇪
```

---

## Por qué Oracle no puede demandar

```
Oracle demandó a Google por:
→ usar APIs de Java en Android
→ reimplementar java.* packages
→ caso duró 10 años (2010-2021)
→ Google GANÓ en Suprema Corte

JaDead-BIB:
→ NO reimplementa java.* packages
→ NO usa bytecode .class de Oracle
→ NO usa JVM de Oracle
→ NO usa HotSpot de Oracle
→ COMPILA la sintaxis del lenguaje
→ sintaxis = no patentable (Google ganó)
→ Gosling creó Java ANTES que Oracle
→ Oracle compró Sun en 2010

= JaDead-BIB compila Java sintaxis
= no usa IP de Oracle
= Google precedent ✅
= legalmente sólido

Oracle: "._."
Netflix lawyers: *tranquilos* 😂💀🦈🇵🇪
```

---

## Licencia

**TECHNE LICENSE v1.0**

```
Uso personal / estudiantes / OSS:  GRATIS
Empresa < $1M revenue:             GRATIS
Empresa > $1M con JaDead-BIB:      10% royalty
Microsoft/Oracle/Google:            10% o negociamos
Minecraft lag fix:                  Netflix paga abogados 😂
```

---

*JaDead-BIB v1.0 — 2026*
*"Java sin JVM — sin GC — sin Oracle — sin runtime — 16 hasta 256 bits"*
*Hereda ADead-BIB v8.0 + JIT 2.0 PyDead-BIB — IR probado — codegen probado*
*Eddi Andreé Salazar Matos — Lima, Perú 🇵🇪 — 1 dev — Binary Is Binary 💀☕🦈*


"Referencias para que tomes como es "C:\Users\andre\OneDrive\Documentos\ADead-BIB" el origen que nació"

"C:\Users\andre\OneDrive\Documentos\PyDead-BIB = La referencia perfecta para aplicar JaDead-BIB lo aprendido"