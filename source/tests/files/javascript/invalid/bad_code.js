// This file contains deliberate ESLint violations

// Missing 'use strict' directive
var x = 10; // Using var instead of let/const

// No JSDoc comments
function addNumbers(a, b) {
  return a+b; // Missing spaces around operator
}

// Inconsistent quotes
const message = "This uses double quotes";
const name = 'This uses single quotes';

// Unused variable
const unused = 'This variable is never used';

// Console statements left in code
console.log("This should be removed");

// Inconsistent semicolon usage
const value = 42
const array = [1,2,3]; // Missing spaces in array literal

// If statement without braces
if (value > 10)
  console.log('Value is greater than 10');

// == instead of ===
if (value == '42') {
  console.log('Equality without type checking');
}

// Dangling promise
fetch('https://example.com/api')
  .then(response => response.json())
  // Missing .catch() to handle errors

// Function declaration inside a block (hoisting issue)
{
  function nestedFunction() {
    return 'This should be a function expression';
  }
}

// Unreachable code
function unreachableDemo() {
  return 'Early return';
  console.log('This will never execute'); // Unreachable code
}

