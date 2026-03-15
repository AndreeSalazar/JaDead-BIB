public class Jugador {
    private String nombre;
    private int vida;
    
    public Jugador(String nombre) {
        this.nombre = nombre;  
        this.vida = 100;
    }
    
    public int atacar() {
        System.out.println("Hello from FastOS Native RAM!");
        return 10;
    }
}
