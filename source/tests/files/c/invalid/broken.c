#include <stdio.h>
#include <stdlib.h>

/* This file contains deliberate errors for testing */

int main() {
    // Missing semicolon
    printf("Hello, World!")
    
    // Memory leak (missing free)
    char* message = malloc(20 * sizeof(char));
    sprintf(message, "This will leak memory");
    
    // Uninitialized variable
    int value;
    printf("Value: %d\n", value);
    
    // Missing return statement
}

