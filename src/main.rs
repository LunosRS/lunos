mod lunos;
mod modules;
mod utility;

use lunos::{help, repl, version};
use once_cell::sync::Lazy;
use rusty_jsc::*;
use std::cell::RefCell;
use std::env;
use std::ffi::CString;
use std::fs;
use std::sync::Arc;

static RUNTIME: Lazy<Arc<JSRuntime>> = Lazy::new(|| Arc::new(JSRuntime::new()));

struct JSRuntime {
    context: *mut OpaqueJSContext,
}

unsafe impl Send for JSRuntime {}
unsafe impl Sync for JSRuntime {}

impl JSRuntime {
    fn new() -> Self {
        unsafe {
            let context = JSGlobalContextCreate(std::ptr::null_mut());
            let console = modules::console::Console::new();
            console.bind_to_context(context);
            modules::lunos::Lunos::bind_to_context(context);
            Self { context }
        }
    }
}

impl Drop for JSRuntime {
    fn drop(&mut self) {
        unsafe {
            JSGlobalContextRelease(self.context);
        }
    }
}

thread_local! {
    static LOCAL_RUNTIME: RefCell<Option<Arc<JSRuntime>>> = const { RefCell::new(None) };
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build_global()
        .unwrap();

    Lazy::force(&RUNTIME);

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        help::show(1);
    }

    match args[1].as_str() {
        "-v" | "--version" => {
            version::show(0);
        }
        "-h" | "--help" => {
            help::show(0);
        }
        "repl" => {
            repl::start_repl(0);
        }
        _ => {}
    }

    let js_file_arg = &args[1];

    let js_file = match fs::canonicalize(js_file_arg) {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(e) => {
            eprintln!("Error resolving file path {}: {}", js_file_arg, e);
            std::process::exit(1);
        }
    };

    let js_code = match fs::read_to_string(&js_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", js_file, e);
            std::process::exit(1);
        }
    };

    let processed_js_code =
        modules::es6::process_es6_modules(&js_file, &js_code) + "\nconsole.flush();";
    let context = modules::es6::get_context();

    unsafe {
        let js_cstr = CString::new(processed_js_code).unwrap();
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

        if result.is_null() {
            println!("Error evaluating script!");
            std::process::exit(1);
        }
    }
}
