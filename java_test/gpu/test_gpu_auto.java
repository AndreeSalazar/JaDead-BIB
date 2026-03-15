import gpu.Graphics;

class TestGPUAuto {
    public static void main(String[] args) {
        String backend = Graphics.detectBest();
        System.out.println("GPU backend: " + backend);
        System.out.println("gpu auto ok");
    }
}
