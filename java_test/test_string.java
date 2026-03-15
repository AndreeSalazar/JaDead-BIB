class TestString {
    public static void main(String[] args) {
        String msg = "JaDead-BIB length test";
        int len = msg.length();

        String a = "JaDead-BIB length test";
        
        if (msg.equals(a)) {
            System.out.println("EQUALS WORKED NATIVELY!");
        }

        String prefix = "Hello, ";
        String suffix = "World! This is Java 1.0 NATIVE.";
        String complete = prefix + suffix;
        System.out.println(complete);
    }
}
