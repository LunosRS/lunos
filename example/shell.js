/// Lunos Shell Demo
/// Lunos shell is an easy interface to the
// OS's command line, it provides a result and error

/// Spawn command and collect 'stdout', 'stderr'
const { result, error } = Lunos.shell("sh", "echo Hello, world!");

// Print 'result' if it exists
if(result) console.log("Result:", result);
else console.log("No result");

// Check if 'error' is present, if so, print
if(error) console.log("Error:", error);
else console.log("No error");