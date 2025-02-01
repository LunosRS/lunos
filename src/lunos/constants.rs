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