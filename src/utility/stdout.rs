use std::io::Write;

pub fn write_stdout(s: &str) {
    write!(std::io::stdout(), "{s}")
        .expect("Failed to get stdout!");
}
