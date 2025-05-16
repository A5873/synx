package main

import "fmt"

func formatGreeting(name string) string {
    return fmt.Sprintf("Hello, %s!", name)
}

func main() {
    greeting := formatGreeting("World")
    fmt.Println(greeting)
}
