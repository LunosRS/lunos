use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use rusty_jsc::*;
use rustyline::DefaultEditor;
use std::io::{Write, stdout};
use std::{ffi::CString, process};
use std::time::Duration;
use rustyline::error::ReadlineError;
use crate::lunos::constants::{
    ASCII_BANNER, NAME, REPL_HELP, THE_ULTIMATE_QUESTION_AND_ANSWER, VERSION,
};
use crate::lunos::io::colorize;

fn print_welcome() {
    println!("Welcome to Lunos v{VERSION}");
    colorize(
        "[!] Please note: this feature is not fully baked!",
        "yellow",
    );
    println!("Type .help for help");
}

fn clear() {
    execute!(stdout(), Clear(ClearType::All)).unwrap();
    print_welcome();
}

fn handle_command(input: &str, exit_code: i32) -> bool {
    match input {
        ".help" => {
            println!("{REPL_HELP}");
            true
        }
        ".version" => {
            println!("{ASCII_BANNER}{NAME} v{VERSION}");
            true
        }
        ".exit" => {
            colorize("Goodbye!", "green");
            process::exit(exit_code);
        }
        ".clear" => {
            clear();
            true
        }
        ".answer_to_the_ultimate_question_of_life_the_universe_and_everything" => {
            let mut chars = THE_ULTIMATE_QUESTION_AND_ANSWER.chars().peekable();
            while let Some(c) = chars.next() {
                print!("{c}");
                stdout().flush().unwrap();

                if (c == '.' || c == '!' || c == '?') && chars.peek() != Some(&'.') {
                    std::thread::sleep(Duration::from_millis(500));
                } else {
                    std::thread::sleep(Duration::from_millis(15));
                }
            }
            println!();
            true
        }
        _ if input.starts_with('.') => {
            println!("Unknown command: {input}");
            true
        }
        _ => false,
    }
}

pub fn start_repl(exit_code: i32) {
    unsafe {
        let context = JSGlobalContextCreate(std::ptr::null_mut());

        let console = crate::modules::console::Console::new();
        console.bind_to_context(context);

        crate::modules::lunos::Lunos::bind_to_context(context);

        print_welcome();

        let mut rusty_line = DefaultEditor::new().unwrap();
        let mut last_was_ctrl_c = false;

        loop {
            let readline = rusty_line.readline("> ");
            match readline {
                Ok(input) => {
                    last_was_ctrl_c = false;
                    let input = input.trim();

                    if handle_command(input, exit_code) {
                        continue;
                    }

                    rusty_line.add_history_entry(input).unwrap();

                    let js_code = format!("eval('{input}')");
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
                        println!("Error occurred! (Probably a syntax error)");
                    } else {
                        let result_str = JSValueToStringCopy(context, result, std::ptr::null_mut());
                        let mut buffer = [0u8; 1024];
                        let length = JSStringGetUTF8CString(
                            result_str,
                            buffer.as_mut_ptr() as *mut i8,
                            buffer.len(),
                        );

                        if length > 0 {
                            let output = String::from_utf8_lossy(&buffer[..length - 1]);
                            if output != "undefined" {
                                println!("{output}");
                            }
                        }

                        JSStringRelease(result_str);
                    }

                    let flush_code = "console.flush();";
                    let flush_cstr = CString::new(flush_code).unwrap();
                    let flush_script = JSStringCreateWithUTF8CString(flush_cstr.as_ptr());

                    JSEvaluateScript(
                        context,
                        flush_script,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        0,
                        std::ptr::null_mut(),
                    );

                    JSStringRelease(flush_script);
                }
                Err(ReadlineError::Interrupted) => {
                    if last_was_ctrl_c {
                        colorize("Goodbye!", "green");
                        process::exit(exit_code);
                    }
                    last_was_ctrl_c = true;
                    println!("To exit, press Ctrl+C again or Ctrl+D or type .exit");
                }
                Err(ReadlineError::Eof) => {
                    colorize("Goodbye!", "green");
                    process::exit(exit_code);
                }
                Err(err) => {
                    println!("Error: {err}");
                    break;
                }
            }
        }
    }
}