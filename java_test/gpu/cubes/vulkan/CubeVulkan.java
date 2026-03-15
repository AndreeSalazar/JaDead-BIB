import gpu.Vulkan;
import gpu.Graphics;
import gpu.OpenGL;

class CubeVulkan {
    public static void main(String[] args) {
        System.out.println("=== JaDead-BIB Vulkan Cube ===");

        System.out.println("[1] Creating window...");
        Graphics.createWindow(800, 600, "JaDead-BIB Vulkan Cube");

        System.out.println("[2] Initializing Vulkan...");
        Vulkan.init();
        int version = Vulkan.getVersion();
        int devices = Vulkan.getDevice();

        System.out.println("[3] Creating Vulkan pipeline...");
        int pipeline = Vulkan.createPipeline(
            "java_test/gpu/cubes/vulkan/shaders/vertex.glsl",
            "java_test/gpu/cubes/vulkan/shaders/fragment.glsl"
        );
        int mesh = Vulkan.createCubeMesh();

        System.out.println("[4] Initializing OpenGL for rendering...");
        OpenGL.init();
        OpenGL.getRenderer();
        int shader = OpenGL.loadShader(
            "java_test/gpu/cubes/opengl/shaders/vertex.glsl",
            "java_test/gpu/cubes/opengl/shaders/fragment.glsl"
        );
        int vao = OpenGL.createCube();

        System.out.println("[5] Rendering spinning cube (300 frames)...");
        int angle = 0;
        for (int i = 0; i < 300; i = i + 1) {
            Graphics.pollEvents();
            OpenGL.clearDepth(20, 25, 40);
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
        Vulkan.destroyMesh(mesh);
        Vulkan.destroyPipeline(pipeline);
        Vulkan.destroy();
        Graphics.destroyWindow();

        System.out.println("=== Vulkan Cube PASSED ===");
    }
}
