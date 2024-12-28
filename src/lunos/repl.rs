use javascriptcore_sys::*;
use rustyline::DefaultEditor;
use std::ffi::CString;

pub fn start_repl(exit_code: i32) {
    unsafe {
        let context = JSGlobalContextCreate(std::ptr::null_mut());

        let console = crate::modules::console::Console::new();
        console.bind_to_context(context);

        crate::modules::lunos::Carbon::bind_to_context(context);

        println!("Lunos REPL");
        println!("Type 'exit' to quit.");

        let mut rusty_line = DefaultEditor::new().unwrap();

        loop {
            let readline = rusty_line.readline("Lunos > ");
            match readline {
                Ok(input) => {
                    let input = input.trim();
                    if input.eq_ignore_ascii_case("exit") {
                        std::process::exit(exit_code);
                    }

                    let js_code = format!("{}\nconsole.flush();", input);
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

                    if result.is_null() {
                        println!("Error evaluating...");
                    } else {
                        let result_str = JSValueToStringCopy(context, result, std::ptr::null_mut());
                        let mut buffer = [0u8; 1024];
                        let length = JSStringGetUTF8CString(result_str, buffer.as_mut_ptr() as *mut i8, buffer.len());

                        if length > 0 {
                            let output = String::from_utf8_lossy(&buffer[..length as usize - 1]);
                            if output != "undefined" {
                                println!("{}", output);
                            }
                        }

                        JSStringRelease(result_str);
                    }
                }
                Err(_) => {
                    println!("Error reading input.");
                    break;
                }
            }
        }

        JSGlobalContextRelease(context);
    }
}
