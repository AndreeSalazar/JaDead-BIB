import gpu.Vulkan;
import gpu.Graphics;

class TestVulkan {
    public static void main(String[] args) {
        System.out.println("=== JaDead-BIB Vulkan Test ===");

        System.out.println("[1] Creating window...");
        Graphics.createWindow(800, 600, "JaDead-BIB Vulkan");

        System.out.println("[2] Creating Vulkan instance...");
        Vulkan.init();

        System.out.println("[3] Querying Vulkan API version...");
        int version = Vulkan.getVersion();

        System.out.println("[4] Enumerating physical devices...");
        int deviceCount = Vulkan.getDevice();

        System.out.println("[5] Window visible for 3 seconds...");
        Graphics.sleep(3000);

        System.out.println("[6] Cleanup...");
        Vulkan.destroy();
        Graphics.destroyWindow();

        System.out.println("=== Vulkan Test PASSED ===");
    }
}
