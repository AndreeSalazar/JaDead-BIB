import gpu.OpenGL;
import gpu.Graphics;

class CubeOpenGL {
    public static void main(String[] args) {
        System.out.println("=== JaDead-BIB OpenGL Spinning Cube ===");

        System.out.println("[1] Creating window...");
        Graphics.createWindow(800, 600, "JaDead-BIB OpenGL Cube");

        System.out.println("[2] Initializing OpenGL...");
        OpenGL.init();
        OpenGL.getVersion();
        OpenGL.getRenderer();

        System.out.println("[3] Loading shaders...");
        int shader = OpenGL.loadShader(
            "java_test/gpu/cubes/opengl/shaders/vertex.glsl",
            "java_test/gpu/cubes/opengl/shaders/fragment.glsl"
        );

        System.out.println("[4] Creating cube geometry...");
        int vao = OpenGL.createCube();

        System.out.println("[5] Rendering spinning cube (300 frames)...");
        int angle = 0;
        for (int i = 0; i < 300; i = i + 1) {
            Graphics.pollEvents();
            OpenGL.clearDepth(25, 25, 38);
            OpenGL.useShader(shader);
            OpenGL.setUniformMVP(shader, angle);
            OpenGL.drawCube(vao);
            Graphics.swapBuffers();
            angle = angle + 2;
            Graphics.sleep(16);
        }

        System.out.println("[6] Cleanup...");
        OpenGL.destroyShader(shader);
        OpenGL.destroyCube(vao);
        OpenGL.destroy();
        Graphics.destroyWindow();

        System.out.println("=== OpenGL Cube PASSED ===");
    }
}
