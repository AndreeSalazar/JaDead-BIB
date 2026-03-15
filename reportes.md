# 🗺️ JaDead-BIB: The Ultimate Native Standard API Mapping

Para que **JaDead-BIB** pueda compilar el código fuente en Java (*J2SE 21+*) directamente a nuestro FastOS.bib o x86-64 Nativo (sin JVM y sin basura), necesitamos que el `ImportResolver` intercepte los paquetes estándar de `java.*` y los mapé internamente a nuestra librería de super-rendimiento.

Este documento es el **Reporte y Roadmap** de los componentes más drásticos de la API oficial que reconstruiremos en Rust.

---

## 1. ⚙️ `java.lang` (Core Internals)

### `java.lang.System`
- **`System.out.println(String)`** -> Mapeado directamente a `[ADeadOp::PrintStr]`.
- **`System.currentTimeMillis()`** -> Mapeado directamente a llamadas al SO (Win32 `GetSystemTimeAsFileTime` / Linux `gettimeofday`).
- **`System.gc()`** -> **[OBSOLETO/INTERCEPTADO]**. En JaDead-BIB, `System.gc()` no hará nada o emitirá un warning. La memoria ya está manejada por la arquitectura **GC Plus** determinista descrita en la *Fase 3*, liberándose microscópicamente por regiones o por final de scopes (Zero-Pause).

### `java.lang.String`
- Rediseñado en backend como un arreglo inmutable de bytes ASCII/UTF-8 o C-Strings terminados en null (`\0`), sin el altísimo overhead del object header de Java.

---

## 2. 🧮 `java.lang.Math` (SIMD Accelerated)

Queremos que JaDead-BIB sea un compilador feroz en Videojuegos y Simulaciones Físicas. La API Math tradicional de Java está limitada. En JaDead-BIB usaremos **SIMD (AVX2)** nativo para operaciones vectorizadas usando la extensión `@deadbib:simd`:

- **`Math.sqrt(double)`** -> Map a `VSQRTSD` nativo O(1) vía instrucción en hardware.
- **`Math.sin(double)`** -> Map a instrucción de coprocesador acelerado (`FSIN` o similar algorítmico rápido).
- **`Math.abs(double)`** -> Quita el bit de signo mediante máscaras bit a bit (AND mask).
- **Aceleración Futura (Vector API)** -> En vez de cálculos paralelos densos, soportaríamos algo como `FloatVector` mapeado a `YMM` Registros (256-bit AVX2) que resuelve 8 `float`s al mismo tiempo en 1 ciclo de CPU. 

---

## 3. 📂 `java.io` y `java.nio` (FastOS I/O)

La I/O genérica de la JVM pasa por múltiples abstracciones virtuales.

- **`java.io.File` / `java.nio.file.Files`** -> Mapeados directamente a las Syscalls de Windows (`CreateFileW`, `ReadFile`) Linux (`open`, `read`, `mmap`).
- Los flujos grandes de datos (ej. lectura de texturas 4K) pueden mapearse a Memory-Mapped Files nativos, saltando buffers intermedios de Java.

---

## 4. 🔀 `java.lang.Thread` (OS Green Threads & OS Threads)

A diferencia de la JVM o Project Loom (Virtual Threads), implementaremos un generador interno híbrido de corrutinas estáticas para paralelizar código. Se interceptan clases como `Thread` y las interfaces de concurrencia:

- En Windows: mapeo directo a `CreateThread` o Thread Pools subyacentes del OS.
- Sin recolección de basura, los Hilos en JaDead-BIB no provocan "Stop-The-World" Pauses cruzados.
- Exclusión Estricta Mutua (Mutex) mapeada a `SRWLock` en Windows (ultra rápido).

---

## 5. 💀 Exclusivas de JaDead-BIB (Nuevas APIs / Directivas AST)

El compilador ahora detecta Comportamientos Indefinidos en Fase 4.8. Para usar el máximo poder, ofreceremos Anotaciones de Control Exclusivas (inspiradas del *UB Detector* y *GC Plus*).

- `@deadbib:region("GameLevel1")`: Define manualmente el Memory Region (Módulo 4) en una clase.
- `@deadbib:gpu`: Al marcar un método `public static void procesarFisicas()`, el backend `ISATranslator` intercepta y empaqueta el contenido en un kernel SPIR-V/CUDA ejecutándose directamente en la Tarjeta Gráfica en `jab run`.
- `@deadbib:pure`: Fuerza al compilador a aislar estáticamente que el método no toca ninguna variable global, permitiendo a ADeadOp elidir comprobaciones de cache o mutabilidad de memoria cruzada completamente.

---
**Estado Oficial**: Proyecto compilador en *Release Readiness* inicial. Fase "Garbage Collection 2.0" superada en código nativo (Scope, Loop, Bounds, Regions, Cycles OK). Preparación completada para el primer parche Beta a Desarrolladores de Videojuegos y FastOS.
