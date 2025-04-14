#!/usr/bin/env python3
"""
An invalid Python example with a syntax error.
This file should fail validation with Synx.
"""

def calculate_average(numbers):
    """Calculate the average of a list of numbers."""
    total = sum(numbers
    count = len(numbers)  # Missing closing parenthesis above
    
    if count == 0:
        return 0
    
    return total / count

if __name__ == "__main__":
    data = [10, 15, 20, 25, 30]
    print(f"The average is: {calculate_average(data)}")

