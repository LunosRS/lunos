use super::constants::{ASCII_BANNER, AUTHORS, DESCRIPTION, LICENSE, NAME, VERSION};

pub fn show(exit_code: i32) {
    print!("{ASCII_BANNER}");
    println!("{NAME} v{VERSION}");
    println!("{DESCRIPTION} by {AUTHORS}");
    println!("(LEGAL) License: {LICENSE}");

    std::process::exit(exit_code);
}
