use crate::{LOCAL_RUNTIME, RUNTIME};
use rusty_jsc::OpaqueJSContext;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub(crate) fn get_context() -> *mut OpaqueJSContext {
    LOCAL_RUNTIME.with(|runtime| {
        let mut runtime = runtime.borrow_mut();
        if runtime.is_none() {
            *runtime = Some(RUNTIME.clone());
        }
        runtime.as_ref().unwrap().context
    })
}

fn find_node_modules(start_dir: &Path) -> Option<PathBuf> {
    let mut current_dir = start_dir.to_path_buf();

    if !current_dir.exists() {
        if let Ok(cwd) = std::env::current_dir() {
            current_dir = cwd;
        } else {
            return None;
        }
    }

    let node_modules = current_dir.join("node_modules");
    if node_modules.exists() && node_modules.is_dir() {
        return Some(node_modules);
    }

    while current_dir.pop() {
        let node_modules = current_dir.join("node_modules");
        if node_modules.exists() && node_modules.is_dir() {
            return Some(node_modules);
        }
    }

    None
}

fn resolve_package_main(package_dir: &Path) -> Option<PathBuf> {
    let package_json_path = package_dir.join("package.json");

    if !package_json_path.exists() {
        let index_js = package_dir.join("index.js");
        if index_js.exists() {
            return Some(index_js);
        }
        return None;
    }

    let mut file = match fs::File::open(&package_json_path) {
        Ok(file) => file,
        Err(_) => return None,
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return None;
    }

    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with("\"main\"") || line.starts_with("'main'") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                let main_value = parts[1].trim();
                let main_value = main_value.trim_end_matches(',');
                let main_value = main_value.trim_matches(|c| c == '"' || c == '\'');

                let main_path = package_dir.join(main_value);
                if main_path.exists() {
                    return Some(main_path);
                }

                let main_path_with_js = if !main_value.ends_with(".js") {
                    package_dir.join(format!("{}.js", main_value))
                } else {
                    main_path
                };

                if main_path_with_js.exists() {
                    return Some(main_path_with_js);
                }
            }
        }
    }

    let index_js = package_dir.join("index.js");
    if index_js.exists() {
        return Some(index_js);
    }

    None
}

fn resolve_module_path(base_path: &Path, import_path: &str) -> PathBuf {
    if import_path.starts_with("./") || import_path.starts_with("../") {
        let mut path = base_path.to_path_buf();
        path.pop();
        return path.join(import_path);
    }

    let base_dir = base_path.parent().unwrap_or(Path::new("."));

    if let Some(node_modules_dir) = find_node_modules(base_dir) {
        let parts: Vec<&str> = import_path.split('/').collect();
        let package_name = parts[0];
        let package_dir = node_modules_dir.join(package_name);

        if !package_dir.exists() {
            eprintln!("Package '{}' not found in node_modules", package_name);
            std::process::exit(1);
        }

        if parts.len() > 1 {
            let submodule_path = parts[1..].join("/");
            let full_path = package_dir.join(&submodule_path);

            if full_path.exists() {
                return full_path;
            }

            let full_path_js = package_dir.join(format!("{}.js", submodule_path));
            if full_path_js.exists() {
                return full_path_js;
            }

            eprintln!(
                "Submodule '{}' not found in package '{}'",
                submodule_path, package_name
            );
            std::process::exit(1);
        }

        if let Some(main_path) = resolve_package_main(&package_dir) {
            return main_path;
        }

        eprintln!(
            "Could not resolve main entry point for package '{}'",
            package_name
        );
        std::process::exit(1);
    }

    let mut path = base_path.to_path_buf();
    path.pop();
    path.join(format!("./{}", import_path))
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
                    path_part[1..path_part.len() - 1].to_string()
                } else if path_part.starts_with("\"") && path_part.ends_with("\"") {
                    path_part[1..path_part.len() - 1].to_string()
                } else {
                    continue;
                };

                if !import_part.starts_with("{") && !import_part.contains("*") {
                    let default_name = import_part.trim().to_string();
                    imports.push((vec![format!("default:{}", default_name)], module_path));
                } else if import_part.starts_with("{") && import_part.ends_with("}") {
                    let symbols_str = import_part[1..import_part.len() - 1].trim();
                    let symbols = symbols_str
                        .split(',')
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

            if line.contains("{") && line.contains("}") && !line.starts_with("export default") {
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
                    if parts[1] == "function"
                        || parts[1] == "const"
                        || parts[1] == "let"
                        || parts[1] == "var"
                        || parts[1] == "class"
                    {
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
    let mut result = Vec::new();
    let mut in_default_export_object = false;
    let mut default_object_content = String::new();
    let mut brace_count = 0;

    for line in js_code.lines() {
        let trimmed = line.trim();

        if in_default_export_object {
            brace_count += trimmed.chars().filter(|&c| c == '{').count();
            brace_count -= trimmed.chars().filter(|&c| c == '}').count();

            default_object_content.push_str(line);
            default_object_content.push('\n');

            if brace_count == 0 {
                in_default_export_object = false;
                let start = default_object_content.find('{').unwrap_or(0);
                let end = default_object_content
                    .rfind('}')
                    .unwrap_or(default_object_content.len());
                if start < end {
                    let object_content = &default_object_content[start..=end];
                    result.push(format!(
                        "var __default_export_value__ = {};",
                        object_content
                    ));
                }
                default_object_content.clear();
            }
            continue;
        }

        if trimmed.starts_with("export") {
            if trimmed.contains("{") && trimmed.contains("}") && !trimmed.contains("=") {
                continue;
            } else if trimmed.starts_with("export default {") {
                in_default_export_object = true;
                brace_count = 1;
                default_object_content.push_str(trimmed.replacen("export default", "", 1).trim());
                default_object_content.push('\n');
            } else if trimmed.starts_with("export default") {
                let default_value = trimmed.replacen("export default", "", 1).trim().to_string();
                let default_value = if default_value.ends_with(';') {
                    default_value[..default_value.len() - 1].to_string()
                } else {
                    default_value
                };
                result.push(format!("var __default_export_value__ = {};", default_value));
            } else {
                result.push(line.replacen("export ", "", 1));
            }
        } else {
            result.push(line.to_string());
        }
    }

    result
        .into_iter()
        .filter(|line| !line.is_empty())
        .collect::<Vec<String>>()
        .join("\n")
        + "\n"
}

use std::collections::HashMap;

pub(crate) fn process_es6_modules(js_file: &str, js_code: &str) -> String {
    let mut processed_code = String::new();
    let mut loaded_modules = Vec::new();
    let mut module_exports = HashMap::new();
    let mut default_imports = Vec::new();
    let base_path = Path::new(js_file);
    let imports = extract_imports(js_code);

    for (_, module_path) in &imports {
        let resolved_path = resolve_module_path(base_path, module_path);
        let path_str = resolved_path.to_string_lossy().to_string();

        if module_exports.contains_key(&path_str) {
            continue;
        }

        let module_code = read_module_code(&resolved_path);
        let exports = extract_exports(&module_code);

        let processed_module_code = remove_exports(&module_code);
        processed_code.push_str(&processed_module_code);

        module_exports.insert(path_str.clone(), (exports, module_code));
        loaded_modules.push(path_str);
    }

    for (import_symbols, module_path) in imports {
        let resolved_path = resolve_module_path(base_path, &module_path);
        let path_str = resolved_path.to_string_lossy().to_string();

        let (exports, _module_code) = module_exports.get(&path_str).unwrap_or_else(|| {
            std::process::exit(1);
        });

        let _module_prefix = module_path
            .replace("/", "_")
            .replace("-", "_")
            .replace(".", "_")
            .replace("'", "")
            .replace("\"", "");

        for symbol in import_symbols {
            if symbol.starts_with("default:") {
                let import_name = symbol.split(':').nth(1).unwrap_or("default");
                if exports.contains(&"__default_export_value__".to_string()) {
                    default_imports.push((import_name.to_string(), String::new()));
                } else {
                    std::process::exit(1);
                }
            } else if exports.contains(&symbol) {
                continue
            } else {
                std::process::exit(1);
            }
        }
    }

    let main_code = remove_exports(js_code);
    let main_code_without_imports = main_code
        .lines()
        .filter(|line| !line.trim().starts_with("import"))
        .collect::<Vec<&str>>()
        .join("\n");

    let processed_main_code = main_code_without_imports;

    processed_code
        .push_str("\nfunction __get_default_export__() { return __default_export_value__; }\n");

    for (import_name, _) in default_imports {
        processed_code.push_str(&format!(
            "var {} = __get_default_export__();\n",
            import_name
        ));
    }

    processed_code.push_str(&processed_main_code);

    processed_code
}

fn read_module_code(resolved_path: &Path) -> String {
    match fs::read_to_string(&resolved_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading module {}: {}", resolved_path.display(), e);
            std::process::exit(1);
        }
    }
}