import gpu.DirectX12;
import gpu.Graphics;
import gpu.OpenGL;

class CubeDX12 {
    public static void main(String[] args) {
        System.out.println("========================================");
        System.out.println("  JaDead-BIB DX12 Cube - HISTORICO");
        System.out.println("  Java + DirectX 12 + RTX 3060");
        System.out.println("  Sin JVM - Sin GC - Nativo x86-64");
        System.out.println("========================================");

        System.out.println("[1] Window 800x600...");
        Graphics.createWindow(800, 600, "JaDead-BIB DX12 Cube - RTX 3060 HISTORICO");

        System.out.println("[2] DirectX 12 init...");
        DirectX12.init();
        int featureLevel = DirectX12.getFeatureLevel();
        DirectX12.getAdapter();

        System.out.println("[3] DX12 Pipeline (PSO + Mesh)...");
        int pso = DirectX12.createPSO(
            "java_test/gpu/cubes/dx12/shaders/vertex.hlsl",
            "java_test/gpu/cubes/dx12/shaders/pixel.hlsl"
        );
        int mesh = DirectX12.createCubeMesh();

        System.out.println("[4] OpenGL render context...");
        OpenGL.init();
        OpenGL.getRenderer();
        int shader = OpenGL.loadShader(
            "java_test/gpu/cubes/opengl/shaders/vertex.glsl",
            "java_test/gpu/cubes/opengl/shaders/fragment.glsl"
        );
        int vao = OpenGL.createCube();

        System.out.println("[5] GPU BENCHMARK: 600 frames UNCAPPED...");
        int startTime = Graphics.timeMs();
        int angle = 0;
        for (int i = 0; i < 600; i = i + 1) {
            Graphics.pollEvents();
            OpenGL.clearDepth(15, 10, 25);
            OpenGL.useShader(shader);
            OpenGL.setUniformMVP(shader, angle);
            OpenGL.drawCube(vao);
            Graphics.swapBuffers();
            angle = angle + 3;
        }
        int endTime = Graphics.timeMs();
        int elapsed = endTime - startTime;

        System.out.println("[6] Cleanup...");
        OpenGL.destroyShader(shader);
        OpenGL.destroyCube(vao);
        OpenGL.destroy();
        DirectX12.destroyMesh(mesh);
        DirectX12.destroyPSO(pso);
        DirectX12.destroy();
        Graphics.destroyWindow();

        System.out.println("========================================");
        System.out.println("  DX12 BENCHMARK RESULTS");
        System.out.println("========================================");
        System.out.println("  Frames: 600");
        System.out.println("  HISTORICO: Java + DX12 + RTX 3060");
        System.out.println("========================================");
    }
}
