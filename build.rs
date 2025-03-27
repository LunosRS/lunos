use core::panic;

fn main() {
    cc::Build::new()
        .cpp(true)
        .file("src/utility/stdout.cpp")
        .include("include")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-O3")
        .compile("fast_stdout");

    match get_os().as_str() {
        "macos" => build_macos(),
        "linux" => build_linux(),
        "windows" => build_windows(),
        _ => panic!("Unsupported target OS"),
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/utility/stdout.cpp");
    println!("cargo:rerun-if-changed=include/stdout.hpp");
}

fn build_macos() {
    println!("Building for macOS...");
    println!("cargo:rustc-link-lib=framework=JavaScriptCore");
}

fn build_linux() {
    println!("Building for Linux...");
    println!("cargo:rustc-link-lib=dylib=javascriptcoregtk-4.1");
    println!("cargo:rustc-link-lib=dylib=javascriptcoregtk-4.0");
}

fn build_windows() {
    panic!("Soon™");
}

fn get_os() -> String {
    let os: &str = std::env::consts::OS;
    os.to_string()
}
