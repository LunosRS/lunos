use mime_guess;
use rusty_jsc::*;
use std::ffi::CString;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;

pub struct Lunos;

impl Lunos {
    pub fn bind_to_context(context: *mut OpaqueJSContext) {
        unsafe {
            let global_object = JSContextGetGlobalObject(context);

            let serve_name = CString::new("serve").unwrap();
            let serve_function = JSObjectMakeFunctionWithCallback(
                context,
                JSStringCreateWithUTF8CString(serve_name.as_ptr()),
                Some(Self::serve_callback),
            );

            let input_name = CString::new("input").unwrap();
            let input_function = JSObjectMakeFunctionWithCallback(
                context,
                JSStringCreateWithUTF8CString(input_name.as_ptr()),
                Some(Self::input_callback),
            );

            let argv_name = CString::new("argv").unwrap();
            let argv_function = JSObjectMakeFunctionWithCallback(
                context,
                JSStringCreateWithUTF8CString(argv_name.as_ptr()),
                Some(Self::argv_callback),
            );

            let exit_name = CString::new("exit").unwrap();
            let exit_function = JSObjectMakeFunctionWithCallback(
                context,
                JSStringCreateWithUTF8CString(exit_name.as_ptr()),
                Some(Self::exit_callback),
            );

            let load_file = CString::new("loadFile").unwrap();
            let load_file_function = JSObjectMakeFunctionWithCallback(
                context,
                JSStringCreateWithUTF8CString(exit_name.as_ptr()),
                Some(Self::load_file_callback),
            );

            let shell = CString::new("shell").unwrap();
            let shell_function = JSObjectMakeFunctionWithCallback(
                context,
                JSStringCreateWithUTF8CString(exit_name.as_ptr()),
                Some(Self::shell_callback),
            );

            let lunos_object = JSObjectMake(context, std::ptr::null_mut(), std::ptr::null_mut());

            JSObjectSetProperty(
                context,
                lunos_object,
                JSStringCreateWithUTF8CString(serve_name.as_ptr()),
                serve_function,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );

            JSObjectSetProperty(
                context,
                lunos_object,
                JSStringCreateWithUTF8CString(input_name.as_ptr()),
                input_function,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );

            JSObjectSetProperty(
                context,
                lunos_object,
                JSStringCreateWithUTF8CString(argv_name.as_ptr()),
                argv_function,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );

            JSObjectSetProperty(
                context,
                lunos_object,
                JSStringCreateWithUTF8CString(exit_name.as_ptr()),
                exit_function,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );

            JSObjectSetProperty(
                context,
                lunos_object,
                JSStringCreateWithUTF8CString(load_file.as_ptr()),
                load_file_function,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );

            JSObjectSetProperty(
                context,
                lunos_object,
                JSStringCreateWithUTF8CString(shell.as_ptr()),
                shell_function,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );

            let lunos_name = CString::new("Lunos").unwrap();
            JSObjectSetProperty(
                context,
                global_object,
                JSStringCreateWithUTF8CString(lunos_name.as_ptr()),
                lunos_object,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );
        }
    }

    pub fn argv() -> Vec<String> {
        std::env::args().skip(2).collect()
    }

    unsafe extern "C" fn argv_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        _argument_count: usize,
        _arguments: *const *const OpaqueJSValue,
        _: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        let args = Self::argv();
        let js_array =
            unsafe { JSObjectMakeArray(context, 0, std::ptr::null(), std::ptr::null_mut()) };

        for (i, arg) in args.iter().enumerate() {
            let arg_cstring = CString::new(arg.clone()).unwrap();
            let js_arg = unsafe { JSStringCreateWithUTF8CString(arg_cstring.as_ptr()) };
            let js_value = unsafe { JSValueMakeString(context, js_arg) };
            unsafe { JSStringRelease(js_arg) };

            unsafe {
                JSObjectSetPropertyAtIndex(
                    context,
                    js_array as *mut _,
                    i as u32,
                    js_value,
                    std::ptr::null_mut(),
                );
            }
        }

        js_array
    }

    unsafe extern "C" fn serve_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        _: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        if argument_count < 1 {
            let error_message = "serve() requires an options object";
            let js_error_message =
                unsafe { JSStringCreateWithUTF8CString(error_message.as_ptr() as *const i8) };
            unsafe { JSStringRelease(js_error_message) };
            return unsafe { JSValueMakeUndefined(context) };
        }

        let options_object = unsafe { *arguments };
        if unsafe { !JSValueIsObject(context, options_object) } {
            let error_message = "serve() requires an options object";
            let js_error_message =
                unsafe { JSStringCreateWithUTF8CString(error_message.as_ptr() as *const i8) };
            unsafe { JSStringRelease(js_error_message) };
            return unsafe { JSValueMakeUndefined(context) };
        }

        let response_text =
            unsafe { Self::get_property_as_string(context, options_object, "responseText") }
                .unwrap_or_default();
        let content_type =
            unsafe { Self::get_property_as_string(context, options_object, "contentType") }
                .or_else(|| unsafe {
                    Self::get_property_as_string(context, options_object, "type")
                })
                .unwrap_or_else(|| "text/plain".to_string());
        let port =
            unsafe { Self::get_property_as_u16(context, options_object, "port") }.unwrap_or(9595);
        let static_dir = unsafe { Self::get_property_as_string(context, options_object, "dir") }
            .map(PathBuf::from);
        let file = unsafe { Self::get_property_as_string(context, options_object, "file") }
            .map(PathBuf::from);
        let log_middleware =
            unsafe { Self::get_property_as_bool(context, options_object, "logMiddleware") }
                .unwrap_or(false);

        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
                .await
                .unwrap();
            println!("Server listening on port {}", port);
            if static_dir.is_some() {
                println!(
                    "Serving static files from {}",
                    static_dir.as_ref().unwrap().display()
                );
            }

            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let response_text = response_text.clone();
                        let content_type = content_type.clone();
                        let static_dir_owned = static_dir.as_ref().map(|p| p.to_owned());

                        let file_owned = file.as_ref().map(|p| p.to_owned());
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(
                                stream,
                                &response_text,
                                &content_type,
                                static_dir_owned,
                                file_owned,
                                log_middleware,
                            )
                            .await
                            {
                                eprintln!("Error handling connection from {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                    }
                }
            }
        });

        let result_message = format!("Server started on port {}", port);
        let js_result_message =
            unsafe { JSStringCreateWithUTF8CString(result_message.as_ptr() as *const i8) };
        let js_result = unsafe { JSValueMakeString(context, js_result_message) };
        unsafe { JSStringRelease(js_result_message) };
        js_result
    }

    async fn handle_connection(
        mut stream: TcpStream,
        response_text: &str,
        content_type: &str,
        static_dir: Option<PathBuf>,
        file: Option<PathBuf>,
        log_middleware: bool,
    ) -> io::Result<()> {
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).await?;
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);

        let (method, request_path) = if let Some(method_end) = request.find(' ') {
            let method = &request[..method_end];
            if let Some(path_start) = request[method_end + 1..].find(' ') {
                let path = &request[method_end + 1..method_end + 1 + path_start];
                (method.to_string(), path.to_string())
            } else {
                ("UNKNOWN".to_string(), "/".to_string())
            }
        } else {
            ("UNKNOWN".to_string(), "/".to_string())
        };

        let mut status_code = 200;

        let path_opt = if let Some(path_start) = request.find("GET ") {
            let path_end = request[path_start + 4..]
                .find(' ')
                .unwrap_or(request.len() - path_start - 4);
            let raw_path = &request[path_start + 4..path_start + 4 + path_end];
            if raw_path == "/" {
                Some("index.html".to_string())
            } else {
                Some(raw_path.trim_start_matches('/').to_string())
            }
        } else {
            None
        };

        if path_opt.is_none() {
            if log_middleware {
                let now = SystemTime::now();
                let duration = now.duration_since(UNIX_EPOCH).unwrap();
                let secs = duration.as_secs();
                let millis = duration.subsec_millis();

                let date_time = {
                    let secs_since_epoch = secs;
                    let days_since_epoch = secs_since_epoch / 86400;
                    let secs_in_day = secs_since_epoch % 86400;

                    let years_since_epoch = days_since_epoch / 365;
                    let year = 1970 + years_since_epoch;

                    let mut days_in_year = days_since_epoch % 365;
                    let mut month = 1;

                    // fuck leap years
                    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
                    for days in days_in_month.iter() {
                        if days_in_year < *days {
                            break;
                        }
                        days_in_year -= *days;
                        month += 1;
                        if month > 12 {
                            break;
                        }
                    }

                    let day = days_in_year + 1;

                    let hours = secs_in_day / 3600;
                    let minutes = (secs_in_day % 3600) / 60;
                    let seconds = secs_in_day % 60;

                    format!(
                        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
                        year, month, day, hours, minutes, seconds, millis
                    )
                };

                status_code = 400;
                println!(
                    "{} [Lunos INFO]: {} ..... {} ..... {}",
                    date_time, request_path, method, status_code
                );
            }
            return Ok(());
        }

        let path = path_opt.unwrap();

        if response_text.is_empty() {
            let mut file_path = None;

            if let Some(specific_file) = &file {
                if specific_file.exists() && specific_file.is_file() {
                    file_path = Some(specific_file.clone());
                }
            }

            if file_path.is_none() && static_dir.is_some() {
                let path_from_dir = static_dir.as_ref().unwrap().join(&path);
                if path_from_dir.exists() && path_from_dir.is_file() {
                    file_path = Some(path_from_dir);
                }
            }

            if let Some(file_to_serve) = file_path {
                let mime_type = mime_guess::from_path(&file_to_serve)
                    .first_or_octet_stream()
                    .to_string();

                let content = fs::read(&file_to_serve)?;

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                    mime_type,
                    content.len()
                );

                stream.write_all(response.as_bytes()).await?;
                stream.write_all(&content).await?;
            } else {
                status_code = 404;
                let not_found = "404 Not Found";
                let response = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    not_found.len(),
                    not_found
                );
                stream.write_all(response.as_bytes()).await?;
            }
        } else {
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                content_type,
                response_text.len(),
                response_text
            );

            stream.write_all(response.as_bytes()).await?;
        }

        if log_middleware {
            let now = SystemTime::now();
            let duration = now.duration_since(UNIX_EPOCH).unwrap();
            let secs = duration.as_secs();
            let millis = duration.subsec_millis();

            let date_time = {
                let secs_since_epoch = secs;
                let days_since_epoch = secs_since_epoch / 86400;
                let secs_in_day = secs_since_epoch % 86400;

                let years_since_epoch = days_since_epoch / 365;
                let year = 1970 + years_since_epoch;

                let mut days_in_year = days_since_epoch % 365;
                let mut month = 1;

                let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
                for days in days_in_month.iter() {
                    if days_in_year < *days {
                        break;
                    }
                    days_in_year -= *days;
                    month += 1;
                    if month > 12 {
                        break;
                    }
                }

                let day = days_in_year + 1;

                let hours = secs_in_day / 3600;
                let minutes = (secs_in_day % 3600) / 60;
                let seconds = secs_in_day % 60;

                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
                    year, month, day, hours, minutes, seconds, millis
                )
            };

            println!(
                "{} [Lunos INFO]: {} ..... {} ..... {}",
                date_time, request_path, method, status_code
            );
        }

        Ok(())
    }

    unsafe fn get_property_as_string(
        context: *const OpaqueJSContext,
        object: *const OpaqueJSValue,
        property_name: &str,
    ) -> Option<String> {
        let property_name_cstring = CString::new(property_name).unwrap();
        let property_name =
            unsafe { JSStringCreateWithUTF8CString(property_name_cstring.as_ptr()) };
        let property_value = unsafe {
            JSObjectGetProperty(
                context,
                object as *mut _,
                property_name,
                std::ptr::null_mut(),
            )
        };
        unsafe { JSStringRelease(property_name) };

        if unsafe { JSValueIsString(context, property_value) } {
            let js_string =
                unsafe { JSValueToStringCopy(context, property_value, std::ptr::null_mut()) };
            let c_string = unsafe { JSStringGetCharactersPtr(js_string) };
            let length = unsafe { JSStringGetLength(js_string) };

            let rust_string =
                unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(c_string, length)) };
            unsafe { JSStringRelease(js_string) };

            Some(rust_string)
        } else {
            None
        }
    }

    unsafe fn get_property_as_u16(
        context: *const OpaqueJSContext,
        object: *const OpaqueJSValue,
        property_name: &str,
    ) -> Option<u16> {
        let property_name_cstring = CString::new(property_name).unwrap();
        let property_name =
            unsafe { JSStringCreateWithUTF8CString(property_name_cstring.as_ptr()) };
        let property_value = unsafe {
            JSObjectGetProperty(
                context,
                object as *mut _,
                property_name,
                std::ptr::null_mut(),
            )
        };
        unsafe { JSStringRelease(property_name) };

        if unsafe { JSValueIsNumber(context, property_value) } {
            Some(unsafe { JSValueToNumber(context, property_value, std::ptr::null_mut()) } as u16)
        } else {
            None
        }
    }

    unsafe fn get_property_as_bool(
        context: *const OpaqueJSContext,
        object: *const OpaqueJSValue,
        property_name: &str,
    ) -> Option<bool> {
        let property_name_cstring = CString::new(property_name).unwrap();
        let property_name =
            unsafe { JSStringCreateWithUTF8CString(property_name_cstring.as_ptr()) };
        let property_value = unsafe {
            JSObjectGetProperty(
                context,
                object as *mut _,
                property_name,
                std::ptr::null_mut(),
            )
        };
        unsafe { JSStringRelease(property_name) };

        if unsafe { JSValueIsBoolean(context, property_value) } {
            Some(unsafe { JSValueToBoolean(context, property_value) })
        } else {
            None
        }
    }

    unsafe extern "C" fn input_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        _: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        let mut result = unsafe { JSValueMakeNull(context) };

        if argument_count > 0 {
            let prompt_arg = unsafe { *arguments };

            if unsafe { JSValueIsString(context, prompt_arg) } {
                let js_string =
                    unsafe { JSValueToStringCopy(context, prompt_arg, std::ptr::null_mut()) };
                let c_string = unsafe { JSStringGetCharactersPtr(js_string) };
                let length = unsafe { JSStringGetLength(js_string) };

                let prompt = unsafe {
                    String::from_utf16_lossy(std::slice::from_raw_parts(c_string, length))
                };
                unsafe { JSStringRelease(js_string) };

                print!("{}", prompt);
                io::stdout().flush().unwrap();

                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_ok() {
                    if input.ends_with('\n') {
                        input.pop();
                    }
                    if input.ends_with('\r') {
                        input.pop();
                    }

                    let input_cstring = CString::new(input).unwrap();
                    let js_input = unsafe { JSStringCreateWithUTF8CString(input_cstring.as_ptr()) };
                    result = unsafe { JSValueMakeString(context, js_input) };
                    unsafe { JSStringRelease(js_input) };
                }
            }
        }

        result
    }

    unsafe extern "C" fn exit_callback(
        _context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        _argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        _exception: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        exit(unsafe { *arguments.wrapping_add(0) as i32 });
    }

    unsafe extern "C" fn load_file_callback(
        _context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        _argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        _exception: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        let script_dir: PathBuf = std::env::current_dir().unwrap();
        let js_string =
            unsafe { JSValueToStringCopy(_context, *arguments.add(0), std::ptr::null_mut()) };
        let c_string = unsafe { JSStringGetCharactersPtr(js_string) };
        let length = unsafe { JSStringGetLength(js_string) };
        let relative_path =
            String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(c_string, length) });
        unsafe { JSStringRelease(js_string) };

        let file_path = script_dir.join(relative_path);
        let file_content = fs::read_to_string(&file_path)
            .unwrap_or_else(|err| format!("Failed to read file {}: {}", file_path.display(), err));

        let content_cstring = CString::new(file_content).unwrap();
        let js_content = unsafe { JSStringCreateWithUTF8CString(content_cstring.as_ptr()) };
        unsafe { JSValueMakeString(_context, js_content) }
    }

    unsafe extern "C" fn shell_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        exception: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        if argument_count < 2 {
            unsafe {
                let error_msg =
                    CString::new("shell requires at least 2 arguments: command and args").unwrap();
                let error_str = JSStringCreateWithUTF8CString(error_msg.as_ptr());
                let error = JSValueMakeString(context, error_str);
                JSStringRelease(error_str);
                *exception = error;
                return std::ptr::null();
            }
        }

        let shell = if let Some(shell_str) =
            unsafe { JSValAsString(context, *arguments.offset(0)) }
        {
            shell_str
        } else {
            let error_msg = CString::new("Failed to parse shell command").unwrap();
            let error_str = unsafe { JSStringCreateWithUTF8CString(error_msg.as_ptr()) };
            let error = unsafe { JSValueMakeString(context, error_str) };
            unsafe { JSStringRelease(error_str) };
            unsafe { *exception = error };
            return std::ptr::null();
        };

        let cmd = if let Some(cmd_str) =
            unsafe { JSValAsString(context, *arguments.offset(1)) }
        {
            cmd_str
        } else {
            let error_msg = CString::new("Failed to parse command arguments").unwrap();
            let error_str = unsafe { JSStringCreateWithUTF8CString(error_msg.as_ptr()) };
            let error = unsafe { JSValueMakeString(context, error_str) };
            unsafe { JSStringRelease(error_str) };
            unsafe { *exception = error };
            return std::ptr::null();
        };

        unsafe {
            let output = match std::process::Command::new(&shell)
                .arg("-c")
                .arg(&cmd)
                .output()
            {
                Ok(output) => output,
                Err(e) => {
                    let error_msg =
                        CString::new(format!("Failed to execute command: {}", e)).unwrap();
                    let error_str = JSStringCreateWithUTF8CString(error_msg.as_ptr());
                    let error = JSValueMakeString(context, error_str);
                    JSStringRelease(error_str);
                    *exception = error;
                    return std::ptr::null();
                }
            };

            let result_obj = JSObjectMake(context, std::ptr::null_mut(), std::ptr::null_mut());

            let stdout_str = String::from_utf8_lossy(&output.stdout);
            let stdout_c_str = CString::new(stdout_str.as_bytes()).unwrap();
            let stdout_js_str = JSStringCreateWithUTF8CString(stdout_c_str.as_ptr());
            let stdout_js_val = JSValueMakeString(context, stdout_js_str);
            let result_prop_name = CString::new("result").unwrap();
            let result_prop = JSStringCreateWithUTF8CString(result_prop_name.as_ptr());
            JSObjectSetProperty(
                context,
                result_obj,
                result_prop,
                stdout_js_val,
                0,
                std::ptr::null_mut(),
            );
            JSStringRelease(result_prop);
            JSStringRelease(stdout_js_str);

            let stderr_str = String::from_utf8_lossy(&output.stderr);
            let stderr_c_str = CString::new(stderr_str.as_bytes()).unwrap();
            let stderr_js_str = JSStringCreateWithUTF8CString(stderr_c_str.as_ptr());
            let stderr_js_val = JSValueMakeString(context, stderr_js_str);
            let error_prop_name = CString::new("error").unwrap();
            let error_prop = JSStringCreateWithUTF8CString(error_prop_name.as_ptr());
            JSObjectSetProperty(
                context,
                result_obj,
                error_prop,
                stderr_js_val,
                0,
                std::ptr::null_mut(),
            );
            JSStringRelease(error_prop);
            JSStringRelease(stderr_js_str);
            result_obj
        }
    }
}
