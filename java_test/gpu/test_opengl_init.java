import gpu.OpenGL;
import gpu.Graphics;

class TestOpenGL {
    public static void main(String[] args) {
        System.out.println("=== JaDead-BIB OpenGL Test ===");

        System.out.println("[1] Creating window...");
        Graphics.createWindow(800, 600, "JaDead-BIB OpenGL");

        System.out.println("[2] Initializing OpenGL...");
        OpenGL.init();

        System.out.println("[3] Querying OpenGL version...");
        int version = OpenGL.getVersion();

        System.out.println("[4] Querying renderer...");
        OpenGL.getRenderer();

        System.out.println("[5] Rendering red frame...");
        OpenGL.clear(200, 50, 50, 255);
        Graphics.swapBuffers();
        Graphics.sleep(1500);

        System.out.println("[6] Rendering green frame...");
        Graphics.pollEvents();
        OpenGL.clear(50, 200, 50, 255);
        Graphics.swapBuffers();
        Graphics.sleep(1500);

        System.out.println("[7] Rendering blue frame...");
        Graphics.pollEvents();
        OpenGL.clear(50, 50, 200, 255);
        Graphics.swapBuffers();
        Graphics.sleep(1500);

        System.out.println("[8] Cleanup...");
        OpenGL.destroy();
        Graphics.destroyWindow();

        System.out.println("=== OpenGL Test PASSED ===");
    }
}
