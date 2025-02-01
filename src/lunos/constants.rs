use std::env;

pub const ASCII_BANNER: &str = r#"(
 )\ )
(()/(   (
 /(_)) ))\   (      (   (
(_))  /((_)  )\ )   )\  )\
| |  (_))(  _(_/(  ((_)((_)
| |__| || || ' \))/ _ \(_-<
|____|\_,_||_||_| \___//__/
"#;
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const LICENSE: &str = env!("CARGO_PKG_LICENSE");
pub const HELP: &str = r#"Lunos is a Blazingly fast JavaScript runtime
Usage: lunos <command> [...flags] [...args]
Help:
  Flags:
    -h / --help / none    show this screen
    -v / --version        show version info
    repl                  start the repl
    <js_file>             execute a js file
"#;
pub const REPL_HELP: &str = r#"Lunos REPL help:
    Commands:
      .help      show this screen
      .exit      leave the repl
      .clear     clear the screen
      .version   show version info
"#;
pub const THE_ULTIMATE_QUESTION_AND_ANSWER: &str = r#"
Both of the men had been trained for this moment, their lives had been a preparation for it, they had been selected at birth as those who would witness the answer, but even so they found themselves gasping and squirming like excited children.
"And you're ready to give it to us?" urged Loonsuawl.
"I am."
"Now?"
"Now," said Deep Thought.
They both licked their dry lips.
"Though I don't think," added Deep Thought, "that you're going to like it."
"Doesn't matter!" said Phouchg. "We must know it! Now!"
"Now?" inquired Deep Thought.
"Yes! Now..."
"All right," said the computer, and settled into silence again. The two men fidgeted. The tension was unbearable.
"You're really not going to like it," observed Deep Thought.
"Tell us!"
"All right," said Deep Thought. "The Answer to the Great Question..."
"Yes..!"
"Of Life, the Universe and Everything..." said Deep Thought.
"Yes...!"
"Is..." said Deep Thought, and paused.
"Yes...!"
"Is..."
"Yes...!!!...?"
"Forty-two," said Deep Thought, with infinite majesty and calm.”

― Douglas Adams, The Hitchhiker’s Guide to the Galaxy
"#;
