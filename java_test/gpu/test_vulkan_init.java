import gpu.Vulkan;
import gpu.Graphics;

// ============================================================
// TestVulkan — Ventana + Vulkan Init Nativo
// JaDead-BIB 💀☕ — Sin JVM, sin LWJGL
// Directo a vulkan-1.dll via LoadLibraryA
// vkCreateInstance + vkEnumeratePhysicalDevices
// ============================================================
class TestVulkan {

    static int windowWidth  = 800;
    static int windowHeight = 600;

    public static void main(String[] args) {
        System.out.println("=== JaDead-BIB Vulkan Test ===");

        // FASE 1: Detectar disponibilidad
        System.out.println("[1] Checking Vulkan availability...");
        boolean available = Vulkan.isAvailable();
        if (!available) {
            System.out.println("SKIP: Vulkan not available on this system");
            return;
        }
        System.out.println("    Vulkan: AVAILABLE");

        // FASE 2: Crear ventana nativa
        System.out.println("[2] Creating native window...");
        boolean winOk = Graphics.createWindow(windowWidth, windowHeight, "JaDead-BIB Vulkan Test");
        if (!winOk) {
            System.out.println("FAIL: Window creation failed");
            return;
        }
        System.out.println("    Window: CREATED");

        // FASE 3: Crear VkInstance
        System.out.println("[3] Creating Vulkan instance...");
        boolean vkOk = Vulkan.init();
        if (!vkOk) {
            System.out.println("FAIL: vkCreateInstance failed");
            Graphics.destroyWindow();
            return;
        }
        System.out.println("    VkInstance: CREATED");

        // FASE 4: Query API version
        System.out.println("[4] Querying Vulkan API version...");
        int version = Vulkan.getVersion();
        int major = version / 100;
        int minor = version % 100;
        System.out.println("    Vulkan API: " + major + "." + minor);

        // FASE 5: Enumerate physical devices
        System.out.println("[5] Enumerating physical devices...");
        int deviceCount = Vulkan.getDevice();
        System.out.println("    Physical devices: " + deviceCount);

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
            frames = frames + 1;
            Graphics.sleep(16);
        }

        long elapsed = Graphics.timeMs() - startTime;
        System.out.println("    Frames: " + frames);
        System.out.println("    Time:   " + elapsed + "ms");

        // FASE 7: Cleanup
        System.out.println("[7] Destroying Vulkan instance...");
        Vulkan.destroy();
        System.out.println("    VkInstance: DESTROYED");

        System.out.println("[8] Destroying window...");
        Graphics.destroyWindow();
        System.out.println("    Window: DESTROYED");

        System.out.println("=== Vulkan Test PASSED ===");
    }
}
