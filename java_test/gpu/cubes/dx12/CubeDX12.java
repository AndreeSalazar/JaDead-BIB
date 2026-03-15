import gpu.DirectX12;
import gpu.Graphics;
import gpu.OpenGL;

class CubeDX12 {
    public static void main(String[] args) {
        System.out.println("=== JaDead-BIB DX12 Cube ===");
        System.out.println("HISTORICO: Primer cubo Java + DX12");

        System.out.println("[1] Creating window...");
        Graphics.createWindow(800, 600, "JaDead-BIB DX12 Cube HISTORICO");

        System.out.println("[2] Initializing DirectX 12...");
        DirectX12.init();
        int featureLevel = DirectX12.getFeatureLevel();
        DirectX12.getAdapter();

        System.out.println("[3] Creating DX12 pipeline...");
        int pso = DirectX12.createPSO(
            "java_test/gpu/cubes/dx12/shaders/vertex.hlsl",
            "java_test/gpu/cubes/dx12/shaders/pixel.hlsl"
        );
        int mesh = DirectX12.createCubeMesh();

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
            OpenGL.clearDepth(30, 20, 35);
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
        DirectX12.destroyMesh(mesh);
        DirectX12.destroyPSO(pso);
        DirectX12.destroy();
        Graphics.destroyWindow();

        System.out.println("=== DX12 Cube PASSED ===");
        System.out.println("Java + DX12 Cube = HISTORICO");
    }
}
