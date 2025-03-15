use super::constants::{ASCII_BANNER, NAME, VERSION, DESCRIPTION, AUTHORS, LICENSE};

pub fn show(exit_code: i32) {
    print!("{}", ASCII_BANNER);
    println!("{} v{}", NAME, VERSION);
    println!("{} by {}", DESCRIPTION, AUTHORS);
    println!("(LEGAL) License: {}", LICENSE);

    std::process::exit(exit_code);
}
