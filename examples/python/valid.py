#!/usr/bin/env python3
"""
A valid Python example that calculates Fibonacci numbers.
This file should pass validation with Synx.
"""

def fibonacci(n):
    """Calculate the nth Fibonacci number recursively with memoization."""
    memo = {0: 0, 1: 1}
    
    def fib_memo(n):
        if n not in memo:
            memo[n] = fib_memo(n-1) + fib_memo(n-2)
        return memo[n]
    
    return fib_memo(n)

def is_prime(num):
    """Check if a number is prime."""
    if num < 2:
        return False
    for i in range(2, int(num**0.5) + 1):
        if num % i == 0:
            return False
    return True

if __name__ == "__main__":
    print("Fibonacci Sequence:")
    for i in range(10):
        print(f"fibonacci({i}) = {fibonacci(i)}")
    
    print("\nPrime Fibonacci Numbers:")
    for i in range(20):
        fib = fibonacci(i)
        if is_prime(fib):
            print(f"fibonacci({i}) = {fib} is prime")

