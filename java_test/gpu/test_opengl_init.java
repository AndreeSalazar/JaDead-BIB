import gpu.OpenGL;

class TestOpenGL {
    public static void main(String[] args) {
        int ver = OpenGL.getVersion();
        System.out.println("OpenGL: " + ver);
        System.out.println("opengl ok");
    }
}
