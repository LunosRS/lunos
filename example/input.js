const result = Lunos.input("Enter a number between 1 and 10: ");
const randomNumber = Math.floor(Math.random() * 10) + 1;

console.log(`The random number is ${randomNumber}`);
console.log(`Your guess is ${result}`);
