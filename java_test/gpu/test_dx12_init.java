import gpu.DirectX12;
import gpu.Graphics;

class TestDX12 {
    public static void main(String[] args) {
        System.out.println("=== JaDead-BIB DirectX 12 Test ===");
        System.out.println("HISTORICO: Primer Java con DX12 nativo");

        System.out.println("[1] Creating window...");
        Graphics.createWindow(800, 600, "JaDead-BIB DX12 HISTORICO");

        System.out.println("[2] Loading d3d12.dll + dxgi.dll...");
        DirectX12.init();

        System.out.println("[3] Querying D3D Feature Level...");
        int featureLevel = DirectX12.getFeatureLevel();

        System.out.println("[4] Querying DXGI adapter...");
        DirectX12.getAdapter();

        System.out.println("[5] Window visible for 3 seconds...");
        Graphics.sleep(3000);

        System.out.println("[6] Cleanup...");
        DirectX12.destroy();
        Graphics.destroyWindow();

        System.out.println("=== DX12 Test PASSED ===");
        System.out.println("Java + DX12 nativo = HISTORICO");
    }
}
