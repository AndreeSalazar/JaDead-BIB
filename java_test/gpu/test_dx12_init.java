import gpu.DirectX12;

class TestDX12 {
    public static void main(String[] args) {
        int level = DirectX12.getFeatureLevel();
        System.out.println("DX12 feature level: " + level);
        System.out.println("dx12 ok");
    }
}
