import gpu.Vulkan;

class TestVulkan {
    public static void main(String[] args) {
        int ver = Vulkan.getVersion();
        System.out.println("Vulkan: " + ver);
        System.out.println("vulkan ok");
    }
}
