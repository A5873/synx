#!/usr/bin/env python3
"""This module contains deliberate PEP8 and type hint violations."""
import random, sys, os # Multiple imports on one line

class badCalculator: # Class name should use CamelCase
    def __init__(self):
        self.history=[] # Missing spaces around operator
        
    def Add(self, x, y): # Method should be lowercase
        # Missing type hints
        result=x+y # Missing spaces
        self.history.append(result)
        return result
    
    def subtract( self, x , y ): # Inconsistent spacing
        """Subtract two numbers.
        Args:
            x: First number
            y: Second number # Missing proper indentation
        Returns:
            Difference between x and y"""
        # Missing consistent docstring format
        result = x-y
        self.history.append(result)
        return result
    
    def getHistory(self):
        for i in range(0,len(self.history)): # Unnecessary 0 in range
            print (i, self.history[i])        # Extra space after print
    
    def process_input(self, value): # No type hints
        if value == None: return 0 # Should be 'is None', and statement on same line
        elif type(value) == str: # Should use isinstance
            try:
                return float(value)
            except:  # Bare except clause
                print ("Error converting value")
                return None


# Extraneous whitespace at end of line 
def main(): 
    calc = badCalculator()
    print("5 + 3 =", calc.Add(5, 3)) # Should use f-string
    print("Last result:", calc.history[-1]) # Direct access to internals

if __name__ == "__main__":
    main()

