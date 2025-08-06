use std::process::Command;

fn main() {
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

    // linux can have the older 4.0 if using a pkg
    // manager like apt. To maintain compatibility,
    // we use dynamically select the version
    if let Ok(output) = Command::new("pkg-config")
        .args(["--libs", "javascriptcoregtk-4.1"])
        .output() {
        if output.status.success() {
            println!("cargo:rustc-link-lib=dylib=javascriptcoregtk-4.1");
            return;
        }
    }

    if let Ok(output) = Command::new("pkg-config")
        .args(["--libs", "javascriptcoregtk-4.0"])
        .output() {
        if output.status.success() {
            println!("cargo:rustc-link-lib=dylib=javascriptcoregtk-4.0");
            return;
        }
    }

    panic!("Could not find JavaScriptCore GTK library (tried 4.1 and 4.0)");
}

fn build_windows() {
    println!("This builds with hopes and prayers!");
    panic!("Soon™");
}

fn get_os() -> String {
    let os: &str = std::env::consts::OS;
    os.to_string()
}
