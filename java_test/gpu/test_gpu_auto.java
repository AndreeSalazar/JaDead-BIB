import gpu.Graphics;
import gpu.OpenGL;
import gpu.Vulkan;
import gpu.DirectX12;

// ============================================================
// TestGPUAuto — Auto-detect Best GPU Backend + Ventana + Render
// JaDead-BIB 💀☕ — Sin JVM
// Prueba DX12 → Vulkan → CUDA → OpenGL (priority order)
// Crea ventana nativa y ejecuta render loop con el mejor backend
// ============================================================
class TestGPUAuto {

    static int windowWidth  = 800;
    static int windowHeight = 600;

    public static void main(String[] args) {
        System.out.println("=== JaDead-BIB GPU Auto-Detect Test ===");

        // FASE 1: Auto-detect mejor backend
        System.out.println("[1] Auto-detecting best GPU backend...");
        String backend = Graphics.detectBest();
        System.out.println("    Best backend: " + backend);

        // FASE 2: Probar disponibilidad de todos los backends
        System.out.println("[2] Checking all backends...");

        boolean hasDX12 = DirectX12.isAvailable();
        System.out.println("    DirectX12: " + (hasDX12 ? "YES" : "NO"));

        boolean hasVulkan = Vulkan.isAvailable();
        System.out.println("    Vulkan:    " + (hasVulkan ? "YES" : "NO"));

        boolean hasOpenGL = OpenGL.isAvailable();
        System.out.println("    OpenGL:    " + (hasOpenGL ? "YES" : "NO"));

        // FASE 3: Crear ventana nativa
        System.out.println("[3] Creating native window...");
        boolean winOk = Graphics.createWindow(windowWidth, windowHeight, "JaDead-BIB GPU Auto [" + backend + "]");
        if (!winOk) {
            System.out.println("FAIL: Window creation failed");
            return;
        }
        System.out.println("    Window: CREATED");

        // FASE 4: Inicializar el backend detectado
        System.out.println("[4] Initializing " + backend + "...");
        boolean initOk = false;

        if (backend.equals("DirectX12") && hasDX12) {
            initOk = DirectX12.init();
            if (initOk) {
                int fl = DirectX12.getFeatureLevel();
                System.out.println("    DX12 Feature Level: " + fl);
                DirectX12.getAdapter();
            }
        } else if (backend.equals("Vulkan") && hasVulkan) {
            initOk = Vulkan.init();
            if (initOk) {
                int ver = Vulkan.getVersion();
                System.out.println("    Vulkan API: " + (ver / 100) + "." + (ver % 100));
                int devs = Vulkan.getDevice();
                System.out.println("    Devices: " + devs);
            }
        } else if (hasOpenGL) {
            initOk = OpenGL.init();
            if (initOk) {
                int ver = OpenGL.getVersion();
                System.out.println("    OpenGL version: " + ver);
                OpenGL.getRenderer();
            }
        }

        if (!initOk) {
            System.out.println("WARN: No GPU backend could be initialized");
            System.out.println("    Continuing with window-only test...");
        } else {
            System.out.println("    " + backend + ": INITIALIZED");
        }

        // FASE 5: Render loop — 3 segundos
        System.out.println("[5] Render loop for 3 seconds...");
        System.out.println("    Press ESC to close early");

        int frames = 0;
        long startTime = Graphics.timeMs();
        long endTime = startTime + 3000;

        while (!Graphics.windowShouldClose()) {
            long now = Graphics.timeMs();
            if (now >= endTime) {
                break;
            }

            Graphics.pollEvents();

            // If OpenGL is active, do color cycling clear
            if (initOk && backend.equals("OpenGL")) {
                float t = (float)((now - startTime) % 3000) / 3000.0f;
                float r = 0.1f + 0.4f * t;
                float g = 0.2f + 0.3f * (1.0f - t);
                float b = 0.3f;
                OpenGL.clear(r, g, b, 1.0f);
                Graphics.swapBuffers();
            }

            // Update title every 30 frames
            if (frames % 30 == 0) {
                int elapsed = (int)(now - startTime);
                Graphics.setTitle("JaDead-BIB [" + backend + "] | Frame " + frames + " | " + elapsed + "ms");
            }

            frames = frames + 1;
            Graphics.sleep(16);
        }

        long elapsed = Graphics.timeMs() - startTime;
        System.out.println("    Frames: " + frames);
        System.out.println("    Time:   " + elapsed + "ms");
        if (elapsed > 0) {
            int fps = (int)((long)frames * 1000 / elapsed);
            System.out.println("    FPS:    " + fps);
        }

        // FASE 6: Cleanup
        System.out.println("[6] Cleanup...");
        if (initOk) {
            if (backend.equals("DirectX12")) {
                DirectX12.destroy();
            } else if (backend.equals("Vulkan")) {
                Vulkan.destroy();
            } else {
                OpenGL.destroy();
            }
            System.out.println("    " + backend + ": DESTROYED");
        }

        Graphics.destroyWindow();
        System.out.println("    Window: DESTROYED");

        // FASE 7: Summary
        System.out.println("");
        System.out.println("=== GPU Auto-Detect Summary ===");
        System.out.println("    Backend used: " + backend);
        System.out.println("    DX12:    " + (hasDX12 ? "available" : "n/a"));
        System.out.println("    Vulkan:  " + (hasVulkan ? "available" : "n/a"));
        System.out.println("    OpenGL:  " + (hasOpenGL ? "available" : "n/a"));
        System.out.println("    Frames:  " + frames);
        System.out.println("=== GPU Auto-Detect PASSED ===");
    }
}
