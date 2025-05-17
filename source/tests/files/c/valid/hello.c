#include <stdio.h>
#include <stdlib.h>

/**
 * A simple "Hello, World!" program with proper memory management
 */
int main() {
    char* message = malloc(14 * sizeof(char));
    if (message == NULL) {
        fprintf(stderr, "Memory allocation failed\n");
        return 1;
    }
    
    sprintf(message, "Hello, World!");
    printf("%s\n", message);
    
    free(message);
    return 0;
}

