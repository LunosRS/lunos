console.log("Hello", "log!");
console.log("Hello, log!");

console.warn("Hello", "warn!");
console.warn("Hello, warn!");

console.error("Hello, error!");
console.error("Hello", "error!");

// 'console.flush()' is a Lunos feature to
// flush the console.x() buffer.
// This is handled automatically but if something
// isn't printing, this is probably the reason.
// We are still working on the smart_flush functionality.
// console.flush();