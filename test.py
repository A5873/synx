#!/usr/bin/env python3
"""
This is a test file for Synx validation.
It contains a deliberate syntax error to test validation.
"""

def calculate_fibonacci(n):
    """Calculate the nth Fibonacci number recursively."""
    if n <= 1:
        return n
    else:
        return calculate_fibonacci(n-1) + calculate_fibonacci(n-2)
    
# This function has a missing closing parenthesis above

def is_prime(num):
    """Check if a number is prime."""
    if num < 2:
        return False
    for i in range(2, int(num**0.5) + 1):
        if num % i == 0:
            return False
    return True

if __name__ == "__main__":
    # This will never execute due to the syntax error above
    print("Testing fibonacci calculation:")
    for i in range(10):
        print(f"Fibonacci({i}) = {calculate_fibonacci(i)}")
        
    print("\nTesting prime numbers:")
    for i in range(20):
        if is_prime(i):
            print(f"{i} is prime")

