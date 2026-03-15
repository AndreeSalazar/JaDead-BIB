import gpu.Graphics;
import gpu.DirectX12;
import gpu.Vulkan;
import gpu.OpenGL;

class TestGPUAuto {
    public static void main(String[] args) {
        System.out.println("=== JaDead-BIB GPU Auto-Detect Test ===");

        System.out.println("[1] Auto-detecting best GPU backend...");
        Graphics.detectBest();

        System.out.println("[2] Checking DX12...");
        DirectX12.isAvailable();

        System.out.println("[3] Checking Vulkan...");
        Vulkan.isAvailable();

        System.out.println("[4] Checking OpenGL...");
        OpenGL.isAvailable();

        System.out.println("[5] Creating window...");
        Graphics.createWindow(800, 600, "JaDead-BIB GPU Auto-Detect");

        System.out.println("[6] Initializing OpenGL on window...");
        OpenGL.init();
        int ver = OpenGL.getVersion();
        OpenGL.getRenderer();

        System.out.println("[7] Rendering color cycle...");
        OpenGL.clear(200, 50, 50, 255);
        Graphics.swapBuffers();
        Graphics.sleep(1000);

        Graphics.pollEvents();
        OpenGL.clear(50, 200, 50, 255);
        Graphics.swapBuffers();
        Graphics.sleep(1000);

        Graphics.pollEvents();
        OpenGL.clear(50, 50, 200, 255);
        Graphics.swapBuffers();
        Graphics.sleep(1000);

        System.out.println("[8] Cleanup...");
        OpenGL.destroy();
        Graphics.destroyWindow();

        System.out.println("=== GPU Auto-Detect PASSED ===");
    }
}
