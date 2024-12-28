mod lunos;
mod modules;

use javascriptcore_sys::*;
use lunos::{help, version};
use std::env;
use std::ffi::CString;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        help::show(1);
    }

    match args[1].as_str() {
        "-v" | "--version" => {
            version::show(0);
        },
        "-h" | "--help" => {
            help::show(0);
        },
        _ => {
            help::show(1);
        },
    }

    let js_file = &args[1];
    let js_code = match fs::read_to_string(js_file) {
        Ok(content) => content + "\nconsole.flush();",
        Err(e) => {
            eprintln!("Error reading file {}: {}", js_file, e);
            std::process::exit(1);
        }
    };

    unsafe {
        let context = JSGlobalContextCreate(std::ptr::null_mut());

        // Console.*
        let console = modules::console::Console::new();
        console.bind_to_context(context);

        // Lunos.*
        modules::lunos::Carbon::bind_to_context(context);

        let js_cstr = CString::new(js_code).unwrap();
        let script = JSStringCreateWithUTF8CString(js_cstr.as_ptr());

        let result = JSEvaluateScript(
            context,
            script,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
        );

        JSStringRelease(script);
        JSGlobalContextRelease(context);

        if result.is_null() {
            std::process::exit(1);
        }
    }
}
