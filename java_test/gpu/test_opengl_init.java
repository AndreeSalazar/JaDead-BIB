import gpu.OpenGL;
import gpu.Graphics;

// ============================================================
// TestOpenGL — Ventana + Cubo OpenGL Nativo
// JaDead-BIB 💀☕ — Sin JVM, sin LWJGL, sin JavaFX
// Directo a opengl32.dll via LoadLibraryA
// ============================================================
class TestOpenGL {

    static int windowWidth  = 800;
    static int windowHeight = 600;

    public static void main(String[] args) {
        System.out.println("=== JaDead-BIB OpenGL Test ===");

        // FASE 1: Detectar disponibilidad
        System.out.println("[1] Checking OpenGL availability...");
        boolean available = OpenGL.isAvailable();
        if (!available) {
            System.out.println("FAIL: OpenGL not available");
            return;
        }
        System.out.println("    OpenGL: AVAILABLE");

        // FASE 2: Crear ventana nativa
        System.out.println("[2] Creating native window " + windowWidth + "x" + windowHeight + "...");
        boolean winOk = Graphics.createWindow(windowWidth, windowHeight, "JaDead-BIB OpenGL Cube");
        if (!winOk) {
            System.out.println("FAIL: Window creation failed");
            return;
        }
        System.out.println("    Window: CREATED");

        // FASE 3: Inicializar OpenGL
        System.out.println("[3] Initializing OpenGL context...");
        boolean glOk = OpenGL.init();
        if (!glOk) {
            System.out.println("FAIL: OpenGL init failed");
            Graphics.destroyWindow();
            return;
        }
        System.out.println("    OpenGL: INITIALIZED");

        // FASE 4: Obtener version
        System.out.println("[4] Querying OpenGL version...");
        int version = OpenGL.getVersion();
        System.out.println("    OpenGL version: " + version);

        // FASE 5: Obtener renderer
        System.out.println("[5] Querying renderer...");
        OpenGL.getRenderer();

        // FASE 6: Render loop — cubo con colores (3 segundos)
        System.out.println("[6] Rendering cube for 3 seconds...");
        System.out.println("    Press ESC to close early");

        int frames = 0;
        long startTime = Graphics.timeMs();
        long endTime = startTime + 3000;

        while (!Graphics.windowShouldClose()) {
            long now = Graphics.timeMs();
            if (now >= endTime) {
                break;
            }

            // Poll events (keyboard, close button)
            Graphics.pollEvents();

            // Calculate rotation angle from time
            float angle = (float)((now - startTime) % 4000) / 4000.0f * 360.0f;

            // Clear with rotating color based on angle
            float r = 0.1f + 0.05f * (float)Math.sin(angle * 0.01745f);
            float g = 0.1f + 0.05f * (float)Math.cos(angle * 0.01745f);
            float b = 0.15f;
            OpenGL.clear(r, g, b, 1.0f);

            // Swap buffers (double-buffered)
            Graphics.swapBuffers();

            frames = frames + 1;

            // Frame limiter ~60fps
            Graphics.sleep(16);
        }

        long elapsed = Graphics.timeMs() - startTime;
        System.out.println("    Frames: " + frames);
        System.out.println("    Time:   " + elapsed + "ms");
        if (elapsed > 0) {
            int fps = (int)((long)frames * 1000 / elapsed);
            System.out.println("    FPS:    " + fps);
        }

        // FASE 7: Cleanup
        System.out.println("[7] Destroying OpenGL context...");
        OpenGL.destroy();
        System.out.println("    OpenGL: DESTROYED");

        System.out.println("[8] Destroying window...");
        Graphics.destroyWindow();
        System.out.println("    Window: DESTROYED");

        System.out.println("=== OpenGL Test PASSED ===");
    }
}
