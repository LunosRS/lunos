fn main() {
    if cfg!(target_os = "windows") {
        println!("[!] THIS IS NOT STABLE\nBulding for windows...");
        println!("cargo:rustc-link-search=native=.build/windows/win/lib32");
        println!("cargo:rustc-link-lib=dylib=JavaScriptCore");
        println!("cargo:rustc-link-lib=dylib=WTF");
    } else if cfg!(target_os = "macos") {
        println!("Building for macOS...");
        println!("cargo:rustc-link-lib=framework=JavaScriptCore");
    } else if cfg!(target_os = "linux") {
        println!("Building for Linux...");
        println!("cargo:rustc-link-lib=dylib=javascriptcoregtk-4.0");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
