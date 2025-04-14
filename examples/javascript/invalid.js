/**
 * An invalid JavaScript example with syntax errors.
 * This file should fail validation with Synx.
 */

function calculateStatistics(data) {
  // Missing closing parenthesis and bracket
  const total = data.reduce((sum, value) => {
    return sum + value
  }
  
  const average = total / data.length;
  
  // Calculate standard deviation
  const squareDiffs = data.map(value => {
    const diff = value - average;
    return diff * diff;
  });
  
  const avgSquareDiff = squareDiffs.reduce((sum, value) => sum + value, 0) / squareDiffs.length;
  const stdDev = Math.sqrt(avgSquareDiff);
  
  return {
    total,
    average,
    stdDev
  };
}

// Test the function
const testData = [10, 20, 30, 40, 50];
const stats = calculateStatistics(testData);

console.log('Statistics:', stats);

