const result = Lunos.input("Enter a number between 1 and 10: ");
const randomNumber = Math.floor(Math.random() * 10) + 1;
if(result) {
    console.log(`You ${result === randomNumber ? 'got it!' : 'got it wrong!'}`);
}
