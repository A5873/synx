package main

import "fmt"

func main() {
    // Missing type declaration
    greeting := formatGreeting(42)  // Type error: expected string
    fmt.Println(greeting
}

func formatGreeting(name string) string {
    return "Hello, " + name + "!"
