/**
 * A module for handling math operations with proper JSDoc.
 * @module MathUtils
 */

/**
 * Adds two numbers and returns the result.
 * @param {number} a - The first number.
 * @param {number} b - The second number.
 * @returns {number} The sum of a and b.
 */
export function add(a, b) {
  return a + b;
}

/**
 * Multiplies two numbers and returns the result.
 * @param {number} a - The first number.
 * @param {number} b - The second number.
 * @returns {number} The product of a and b.
 */
export function multiply(a, b) {
  return a * b;
}

/**
 * A class representing a calculator.
 */
export class Calculator {
  /**
   * Create a calculator instance.
   * @param {number} [initialValue=0] - The initial value.
   */
  constructor(initialValue = 0) {
    this.value = initialValue;
  }

  /**
   * Add a number to the current value.
   * @param {number} num - The number to add.
   * @returns {Calculator} The calculator instance for chaining.
   */
  add(num) {
    this.value += num;
    return this;
  }

  /**
   * Get the current value.
   * @returns {number} The current value.
   */
  getValue() {
    return this.value;
  }
}

// Example usage
const calc = new Calculator(10);
const result = calc.add(5).add(3).getValue();
console.log(`Result: ${result}`);

