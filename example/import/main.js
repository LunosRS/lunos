import { add, multiply } from './math';

import Calc from './calculator';

console.log("Using named exports:");
console.log("add(5, 3) =", add(5, 3));
console.log("multiply(4, 2) =", multiply(4, 2));

console.log("\nUsing default export:");
const calc = new Calc();
calc.add(10).multiply(2).subtract(5);
console.log("Calculator result:", calc.getResult());