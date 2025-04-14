#!/usr/bin/env node
/**
 * A valid JavaScript example implementing a data processing utility.
 * This file should pass validation with Synx.
 */

// Data processing class with modern JS features
class DataProcessor {
  constructor(data = []) {
    this.data = data;
    this.errors = [];
  }

  // Process array of numbers with error handling
  processNumbers() {
    try {
      // Check if we have data
      if (!this.data.length) {
        throw new Error('No data to process');
      }

      // Array methods and arrow functions
      const results = {
        sum: this.data.reduce((acc, val) => acc + val, 0),
        average: this.data.reduce((acc, val) => acc + val, 0) / this.data.length,
        min: Math.min(...this.data),
        max: Math.max(...this.data),
        median: this.#calculateMedian(),
        processed: this.data.map(x => ({
          original: x,
          doubled: x * 2,
          squared: x ** 2
        }))
      };

      return results;
    } catch (error) {
      this.errors.push(error.message);
      return null;
    }
  }

  // Private method (using modern JS feature)
  #calculateMedian() {
    const sorted = [...this.data].sort((a, b) => a - b);
    const middle = Math.floor(sorted.length / 2);
    
    if (sorted.length % 2 === 0) {
      return (sorted[middle - 1] + sorted[middle]) / 2;
    } else {
      return sorted[middle];
    }
  }

  // Generator function to yield processed items one by one
  *processedItemsGenerator() {
    for (const item of this.data) {
      yield {
        original: item,
        squared: item ** 2
      };
    }
  }
}

// Example usage
const numbers = [12, 5, 8, 21, 16, 7, 3, 9];
const processor = new DataProcessor(numbers);
const results = processor.processNumbers();

console.log('Processing results:', results);

// Using the generator
console.log('\nProcessed items one by one:');
for (const item of processor.processedItemsGenerator()) {
  console.log(item);
}

