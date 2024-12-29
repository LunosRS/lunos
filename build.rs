use core::panic;

fn main() {
    if cfg!(target_os = "windows") {
        panic!("No. Windows does NOT work.");
    } else if cfg!(target_os = "macos") {
        println!("Building for macOS...");
        println!("cargo:rustc-link-lib=framework=JavaScriptCore");

    } else if cfg!(target_os = "linux") {
        println!("Building for Linux...");
        println!("cargo:rustc-link-lib=dylib=javascriptcoregtk-4.0");

    } else {
        panic!("Unsupported target OS");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
