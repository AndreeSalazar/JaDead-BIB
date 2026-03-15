import gpu.DirectX12;
import gpu.Graphics;

// ============================================================
// TestDX12 — Ventana + DirectX 12 Nativo
// JaDead-BIB 💀☕ — HISTÓRICO
// Primer Java con DX12 nativo — sin JVM, sin JNI
// Directo a d3d12.dll + dxgi.dll via LoadLibraryA
// Lima, Perú 🇵🇪 — Binary Is Binary 💀🦈
// ============================================================
class TestDX12 {

    static int windowWidth  = 800;
    static int windowHeight = 600;

    public static void main(String[] args) {
        System.out.println("=== JaDead-BIB DirectX 12 Test ===");
        System.out.println("    HISTORICO: Primer Java con DX12 nativo");

        // FASE 1: Detectar disponibilidad
        System.out.println("[1] Checking DX12 availability...");
        boolean available = DirectX12.isAvailable();
        if (!available) {
            System.out.println("SKIP: DirectX 12 not available (Windows 10+ required)");
            return;
        }
        System.out.println("    DX12: AVAILABLE");

        // FASE 2: Crear ventana nativa
        System.out.println("[2] Creating native window...");
        boolean winOk = Graphics.createWindow(windowWidth, windowHeight, "JaDead-BIB DX12 - HISTORICO");
        if (!winOk) {
            System.out.println("FAIL: Window creation failed");
            return;
        }
        System.out.println("    Window: CREATED");

        // FASE 3: Inicializar DX12 (cargar d3d12.dll + dxgi.dll)
        System.out.println("[3] Loading d3d12.dll + dxgi.dll...");
        boolean dxOk = DirectX12.init();
        if (!dxOk) {
            System.out.println("FAIL: DX12 init failed");
            Graphics.destroyWindow();
            return;
        }
        System.out.println("    DX12: INITIALIZED");

        // FASE 4: Obtener Feature Level
        System.out.println("[4] Querying D3D Feature Level...");
        int featureLevel = DirectX12.getFeatureLevel();
        System.out.println("    Feature Level: " + featureLevel);
        if (featureLevel >= 120) {
            System.out.println("    D3D_FEATURE_LEVEL_12_0 or higher");
        }

        // FASE 5: Obtener GPU adapter info
        System.out.println("[5] Querying DXGI adapter...");
        DirectX12.getAdapter();

        // FASE 6: Render loop — ventana activa por 3 segundos
        System.out.println("[6] Window active for 3 seconds...");
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

            // Update window title with frame counter
            if (frames % 30 == 0) {
                int elapsed = (int)(now - startTime);
                Graphics.setTitle("JaDead-BIB DX12 | Frame: " + frames + " | " + elapsed + "ms");
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

        // FASE 7: Cleanup
        System.out.println("[7] Destroying DX12 context...");
        DirectX12.destroy();
        System.out.println("    DX12: DESTROYED");

        System.out.println("[8] Destroying window...");
        Graphics.destroyWindow();
        System.out.println("    Window: DESTROYED");

        System.out.println("=== DX12 Test PASSED ===");
        System.out.println("    Java + DX12 nativo = HISTORICO");
    }
}
