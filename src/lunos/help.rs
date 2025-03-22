use super::constants::{ASCII_BANNER, HELP, NAME, VERSION};

pub fn show(exit_code: i32) {
    print!("{}", ASCII_BANNER);
    println!("{} v{}", NAME, VERSION);
    print!("{}", HELP);

    std::process::exit(exit_code);
}
