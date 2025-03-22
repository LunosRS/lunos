use super::constants::{ASCII_BANNER, AUTHORS, DESCRIPTION, LICENSE, NAME, VERSION};

pub fn show(exit_code: i32) {
    print!("{}", ASCII_BANNER);
    println!("{} v{}", NAME, VERSION);
    println!("{} by {}", DESCRIPTION, AUTHORS);
    println!("(LEGAL) License: {}", LICENSE);

    std::process::exit(exit_code);
}
