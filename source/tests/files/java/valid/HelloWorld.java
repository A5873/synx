public class HelloWorld {
    public static void main(String[] args) {
        String greeting = formatGreeting("World");
        System.out.println(greeting);
    }

    private static String formatGreeting(String name) {
        return String.format("Hello, %s!", name);
    }
}
