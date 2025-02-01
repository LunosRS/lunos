use core::panic;

fn main() {
    match get_os().as_str() {
        "macos" => build_macos(),
        "linux" => build_linux(),
        "windows" => build_windows(),
        _ => panic!("Unsupported target OS"),
    }

    println!("cargo:rerun-if-changed=build.rs");
}

fn build_macos() {
    println!("Building for macOS...");
    println!("cargo:rustc-link-lib=framework=JavaScriptCore");
}

fn build_linux() {
    println!("Building for Linux...");
    println!("cargo:rustc-link-lib=dylib=javascriptcoregtk-4.0");
}

fn build_windows() {
    panic!("Soon™");
}

fn get_os() -> String {
    let os: &str = std::env::consts::OS;
    os.to_string()
}
