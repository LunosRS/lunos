fn main() {
    if cfg!(target_os = "windows") {
        let arch = if cfg!(target_arch = "x86_64") {
            ".build/windows/bin64"
        } else {
            ".build/windows/bin32"
        };

        println!("cargo:rustc-link-search=native={}/", arch);
        println!("cargo:rustc-link-lib=static=JavaScriptCore");
        println!("cargo:rustc-link-lib=static=WTF");
        println!("cargo:rustc-link-lib=dylib=JavaScriptCore");
        println!("cargo:rustc-link-lib=dylib=WTF");

    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=JavaScriptCore");

    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=javascriptcoregtk-4.0");

    } else {
        panic!("Unsupported target OS");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
