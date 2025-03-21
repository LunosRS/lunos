mod lunos;
mod modules;
mod utility;

use rusty_jsc::*;
use lunos::{help, repl, version};
use std::env;
use std::ffi::CString;
use std::fs;
use std::cell::RefCell;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use once_cell::sync::Lazy;

static RUNTIME: Lazy<Arc<JSRuntime>> = Lazy::new(|| {
    Arc::new(JSRuntime::new())
});

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
    static LOCAL_RUNTIME: RefCell<Option<Arc<JSRuntime>>> = RefCell::new(None);
}

fn get_context() -> *mut OpaqueJSContext {
    LOCAL_RUNTIME.with(|runtime| {
        let mut runtime = runtime.borrow_mut();
        if runtime.is_none() {
            *runtime = Some(RUNTIME.clone());
        }
        runtime.as_ref().unwrap().context
    })
}

fn resolve_module_path(base_path: &Path, import_path: &str) -> PathBuf {
    let mut path = base_path.to_path_buf();
    path.pop(); // Remove the filename to get the directory

    let import_path = if import_path.starts_with("./") || import_path.starts_with("../") {
        import_path.to_string()
    } else {
        format!("./{}", import_path)
    };

    path.join(import_path)
}

fn load_module(path: &Path, loaded_modules: &mut Vec<String>) -> String {
    let path_str = path.to_string_lossy().to_string();

    // Check if module is already loaded
    if loaded_modules.contains(&path_str) {
        return String::new();
    }

    loaded_modules.push(path_str);

    let mut file_path = path.to_path_buf();
    if !file_path.exists() {
        file_path = path.with_extension("js");
        if !file_path.exists() {
            eprintln!("Module not found: {}", path.display());
            std::process::exit(1);
        }
    }

    match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading module {}: {}", file_path.display(), e);
            std::process::exit(1);
        }
    }
}

fn extract_imports(js_code: &str) -> Vec<(Vec<String>, String)> {
    let mut imports = Vec::new();

    for line in js_code.lines() {
        let line = line.trim();
        if line.starts_with("import") && line.contains("from") {
            let parts: Vec<&str> = line.split("from").collect();
            if parts.len() == 2 {
                let import_part = parts[0].trim().replace("import", "").trim().to_string();
                let path_part = parts[1].trim().replace(";", "").trim().to_string();

                // Extract module path
                let module_path = if path_part.starts_with("'") && path_part.ends_with("'") {
                    path_part[1..path_part.len()-1].to_string()
                } else if path_part.starts_with("\"") && path_part.ends_with("\"") {
                    path_part[1..path_part.len()-1].to_string()
                } else {
                    continue;
                };

                // Extract imported symbols
                if import_part.starts_with("{") && import_part.ends_with("}") {
                    let symbols_str = import_part[1..import_part.len()-1].trim();
                    let symbols = symbols_str.split(',')
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<String>>();

                    imports.push((symbols, module_path));
                }
            }
        }
    }

    imports
}

fn extract_exports(js_code: &str) -> Vec<String> {
    let mut exports = Vec::new();

    for line in js_code.lines() {
        let line = line.trim();
        if line.starts_with("export") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if parts[1] == "function" || parts[1] == "const" || parts[1] == "let" || parts[1] == "var" || parts[1] == "class" {
                    // Extract the function/variable name, handling potential parentheses
                    let name = parts[2].split('(').next().unwrap_or(parts[2]);
                    exports.push(name.to_string());
                }
            }
        }
    }

    exports
}

fn remove_exports(js_code: &str) -> String {
    js_code.lines()
        .map(|line| {
            if line.trim().starts_with("export") {
                line.replacen("export ", "", 1)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n") + "\n"
}

fn process_es6_modules(js_file: &str, js_code: &str) -> String {
    let mut processed_code = String::new();
    let mut loaded_modules = Vec::new();
    let base_path = Path::new(js_file);

    // Extract imports
    let imports = extract_imports(js_code);

    // Process each import
    for (import_symbols, module_path) in imports {
        let resolved_path = resolve_module_path(base_path, &module_path);
        let module_code = load_module(&resolved_path, &mut loaded_modules);

        // Extract exports from the module
        let exports = extract_exports(&module_code);

        // Add module code to processed code (without export keywords)
        processed_code.push_str(&remove_exports(&module_code));

        // Add variable declarations for imported symbols
        for symbol in import_symbols {
            if exports.contains(&symbol) {
                processed_code.push_str(&format!("var {} = {};\n", symbol, symbol));
            } else {
                eprintln!("Export '{}' not found in module '{}'", symbol, module_path);
                std::process::exit(1);
            }
        }
    }

    // Add the main module code (without export keywords and import statements)
    let main_code = remove_exports(js_code);
    let main_code_without_imports = main_code.lines()
        .filter(|line| !line.trim().starts_with("import"))
        .collect::<Vec<&str>>()
        .join("\n");

    processed_code.push_str(&main_code_without_imports);

    processed_code
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build_global()
        .unwrap();

    Lazy::force(&RUNTIME);

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
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

    let js_file = &args[1];
    let js_code = match fs::read_to_string(js_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", js_file, e);
            std::process::exit(1);
        }
    };

    // Process ES6 modules
    let processed_js_code = process_es6_modules(js_file, &js_code) + "\nconsole.flush();";

    let context = get_context();

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
