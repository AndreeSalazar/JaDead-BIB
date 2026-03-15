import gpu.DirectX12;
import gpu.Graphics;

class CubeDX12 {
    public static void main(String[] args) {
        System.out.println("========================================");
        System.out.println("  JaDead-BIB DX12 PURO - HISTORICO");
        System.out.println("  Java + DirectX 12 + RTX 3060");
        System.out.println("  Sin JVM - Sin GC - Sin OpenGL");
        System.out.println("  100% DX12 Nativo x86-64");
        System.out.println("========================================");

        System.out.println("[1] Window 800x600...");
        Graphics.createWindow(800, 600, "JaDead-BIB DX12 PURO HISTORICO");

        System.out.println("[2] DirectX 12 init...");
        DirectX12.init();
        DirectX12.getFeatureLevel();
        DirectX12.getAdapter();

        System.out.println("[3] DX12 Pipeline (Device + Queue + SwapChain + PSO)...");
        int pso = DirectX12.createPSO(
            "java_test/gpu/cubes/dx12/shaders/vertex.hlsl",
            "java_test/gpu/cubes/dx12/shaders/pixel.hlsl"
        );

        System.out.println("[4] DX12 Vertex Buffer + Constant Buffer...");
        int mesh = DirectX12.createCubeMesh();
        int cbv = DirectX12.createCBV();

        System.out.println("[5] DX12 PURO Render: 600 frames...");
        int angle = 0;
        for (int i = 0; i < 600; i = i + 1) {
            Graphics.pollEvents();
            DirectX12.beginFrame();
            DirectX12.clearRTV(0, 15, 10, 25);
            DirectX12.updateMVP(cbv, angle);
            DirectX12.setCBV(0, cbv);
            DirectX12.draw(0, 36);
            DirectX12.endFrame();
            angle = angle + 2;
        }

        System.out.println("[6] Cleanup...");
        DirectX12.destroyMesh(mesh);
        DirectX12.destroyPSO(pso);
        DirectX12.destroy();
        Graphics.destroyWindow();

        System.out.println("========================================");
        System.out.println("  DX12 PURO = HISTORICO");
        System.out.println("  Sin OpenGL - Sin JVM - Sin GC");
        System.out.println("  Java + DX12 + HLSL + RTX 3060");
        System.out.println("========================================");
    }
}
