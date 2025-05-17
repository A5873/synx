#!/usr/bin/env python3
"""
A simple calculator module with type hints.

This module demonstrates proper PEP8 compliance and type hinting.
"""
from typing import Union, List, Optional


class Calculator:
    """A simple calculator that performs basic arithmetic operations."""

    def __init__(self) -> None:
        """Initialize the calculator with a clean history."""
        self.history: List[float] = []

    def add(self, a: float, b: float) -> float:
        """
        Add two numbers and store the result in history.

        Args:
            a: First number
            b: Second number

        Returns:
            The sum of a and b
        """
        result = a + b
        self.history.append(result)
        return result

    def subtract(self, a: float, b: float) -> float:
        """
        Subtract b from a and store the result in history.

        Args:
            a: First number
            b: Second number

        Returns:
            The difference between a and b
        """
        result = a - b
        self.history.append(result)
        return result

    def get_last_result(self) -> Optional[float]:
        """
        Get the last calculation result.

        Returns:
            The last result or None if no calculations have been performed
        """
        if not self.history:
            return None
        return self.history[-1]


def main() -> None:
    """Execute a simple demonstration of the calculator."""
    calc = Calculator()
    print(f"5 + 3 = {calc.add(5, 3)}")
    print(f"10 - 4 = {calc.subtract(10, 4)}")
    print(f"Last result: {calc.get_last_result()}")


if __name__ == "__main__":
    main()

