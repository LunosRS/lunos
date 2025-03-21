use std::fs;
use std::path::{Path, PathBuf};
use rusty_jsc::OpaqueJSContext;
use crate::{LOCAL_RUNTIME, RUNTIME};

pub(crate) fn get_context() -> *mut OpaqueJSContext {
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
    path.pop();

    let import_path = if import_path.starts_with("./") || import_path.starts_with("../") {
        import_path.to_string()
    } else {
        format!("./{}", import_path)
    };

    path.join(import_path)
}

fn load_module(path: &Path, loaded_modules: &mut Vec<String>) -> String {
    let path_str = path.to_string_lossy().to_string();

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

                let module_path = if path_part.starts_with("'") && path_part.ends_with("'") {
                    path_part[1..path_part.len()-1].to_string()
                } else if path_part.starts_with("\"") && path_part.ends_with("\"") {
                    path_part[1..path_part.len()-1].to_string()
                } else {
                    continue;
                };

                if !import_part.starts_with("{") && !import_part.contains("*") {
                    let default_name = import_part.trim().to_string();
                    imports.push((vec![format!("default:{}", default_name)], module_path));
                }
                else if import_part.starts_with("{") && import_part.ends_with("}") {
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
            if line.contains("{") && line.contains("}") {
                let start = line.find('{').unwrap_or(0) + 1;
                let end = line.find('}').unwrap_or(line.len());
                if start < end {
                    let names_str = &line[start..end];
                    for name in names_str.split(',') {
                        let clean_name = name.trim();
                        if !clean_name.is_empty() {
                            exports.push(clean_name.to_string());
                        }
                    }
                }
            } else if line.starts_with("export default") {
                exports.push("__default_export_value__".to_string());
            } else {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if parts[1] == "function" || parts[1] == "const" || parts[1] == "let" || parts[1] == "var" || parts[1] == "class" {
                        let name = parts[2].split('(').next().unwrap_or(parts[2]);
                        exports.push(name.to_string());
                    }
                }
            }
        }
    }

    exports
}

fn remove_exports(js_code: &str) -> String {
    js_code.lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("export") {
                if trimmed.contains("{") && trimmed.contains("}") && !trimmed.contains("=") {
                    String::new()
                } else if trimmed.starts_with("export default") {
                    let default_value = trimmed.replacen("export default", "", 1).trim().to_string();
                    let default_value = if default_value.ends_with(';') {
                        default_value[..default_value.len()-1].to_string()
                    } else {
                        default_value
                    };
                    format!("var __default_export_value__ = {};", default_value)
                } else {
                    line.replacen("export ", "", 1)
                }
            } else {
                line.to_string()
            }
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<String>>()
        .join("\n") + "\n"
}

pub(crate) fn process_es6_modules(js_file: &str, js_code: &str) -> String {
    let mut processed_code = String::new();
    let mut loaded_modules = Vec::new();
    let mut default_imports = Vec::new();
    let base_path = Path::new(js_file);

    let imports = extract_imports(js_code);

    for (import_symbols, module_path) in imports {
        let resolved_path = resolve_module_path(base_path, &module_path);
        let module_code = load_module(&resolved_path, &mut loaded_modules);

        let exports = extract_exports(&module_code);

        processed_code.push_str(&remove_exports(&module_code));

        for symbol in import_symbols {
            if symbol.starts_with("default:") {
                let import_name = symbol.split(':').nth(1).unwrap_or("default");
                if exports.contains(&"__default_export_value__".to_string()) {
                    default_imports.push((import_name.to_string(), String::new()));
                } else {
                    eprintln!("Default export not found in module '{}'", module_path);
                    std::process::exit(1);
                }
            } else if exports.contains(&symbol) {
                processed_code.push_str(&format!("var {} = {};\n", symbol, symbol));
            } else {
                eprintln!("Export '{}' not found in module '{}'", symbol, module_path);
                std::process::exit(1);
            }
        }
    }

    let main_code = remove_exports(js_code);
    let main_code_without_imports = main_code.lines()
        .filter(|line| !line.trim().starts_with("import"))
        .collect::<Vec<&str>>()
        .join("\n");

    let processed_main_code = main_code_without_imports;

    processed_code.push_str("function __get_default_export__() { return __default_export_value__; }\n");

    for (import_name, _) in default_imports {
        processed_code.push_str(&format!("var {} = __get_default_export__();\n", import_name));
    }

    processed_code.push_str(&processed_main_code);

    processed_code
}
