class TestArray {
    public static void main(String[] args) {
        int[] arr = new int[5];
        arr[0] = 42;
        arr[1] = 99;
        arr[4] = 777;
        
        System.out.println(arr[0]);
        System.out.println(arr[1]);
        System.out.println(arr[4]);
        System.out.println(arr.length);
    }
}
