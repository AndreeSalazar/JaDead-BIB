class Jugador {
    int vida;
    int escudo;
}

class Main {
    public static void main(String[] args) {
        Jugador j = new Jugador();
        j.vida = 100;
        j.escudo = 50;
        
        System.out.println(j.vida);
        System.out.println(j.escudo);
    }
}
