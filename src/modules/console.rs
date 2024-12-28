use javascriptcore_sys::*;
use std::ffi::CString;
use std::sync::Mutex;

pub struct Console {
    buffer: Mutex<Vec<String>>,
}

impl Console {
    pub fn new() -> Self {
        Console {
            buffer: Mutex::new(Vec::new()),
        }
    }

    pub fn bind_to_context(self: &Self, context: *mut OpaqueJSContext) {
        unsafe {
            let global_object = JSContextGetGlobalObject(context);

            let console = JSObjectMake(context, std::ptr::null_mut(), std::ptr::null_mut());

            let log_name = CString::new("log").unwrap();
            let log_function = JSObjectMakeFunctionWithCallback(
                context,
                JSStringCreateWithUTF8CString(log_name.as_ptr()),
                Some(Self::log_callback),
            );
            JSObjectSetProperty(
                context,
                console,
                JSStringCreateWithUTF8CString(log_name.as_ptr()),
                log_function,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );

            let warn_name = CString::new("warn").unwrap();
            let warn_function = JSObjectMakeFunctionWithCallback(
                context,
                JSStringCreateWithUTF8CString(warn_name.as_ptr()),
                Some(Self::warn_callback),
            );
            JSObjectSetProperty(
                context,
                console,
                JSStringCreateWithUTF8CString(warn_name.as_ptr()),
                warn_function,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );

            let error_name = CString::new("error").unwrap();
            let error_function = JSObjectMakeFunctionWithCallback(
                context,
                JSStringCreateWithUTF8CString(error_name.as_ptr()),
                Some(Self::error_callback),
            );
            JSObjectSetProperty(
                context,
                console,
                JSStringCreateWithUTF8CString(error_name.as_ptr()),
                error_function,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );

            let flush_name = CString::new("flush").unwrap();
            let flush_function = JSObjectMakeFunctionWithCallback(
                context,
                JSStringCreateWithUTF8CString(flush_name.as_ptr()),
                Some(Self::flush_callback),
            );
            JSObjectSetProperty(
                context,
                console,
                JSStringCreateWithUTF8CString(flush_name.as_ptr()),
                flush_function,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );

            let console_name = CString::new("console").unwrap();
            JSObjectSetProperty(
                context,
                global_object,
                JSStringCreateWithUTF8CString(console_name.as_ptr()),
                console,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );
        }
    }

    unsafe extern "C" fn log_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        _: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        let mut buffer = Console::get_instance().buffer.lock().unwrap();

        for i in 0..argument_count {
            let arg = *arguments.add(i);
            let js_string = JSValueToStringCopy(context, arg, std::ptr::null_mut());
            let c_string = JSStringGetCharactersPtr(js_string);
            let length = JSStringGetLength(js_string);

            let rust_string =
                String::from_utf16_lossy(std::slice::from_raw_parts(c_string, length));
            buffer.push(rust_string);

            JSStringRelease(js_string);
        }

        JSValueMakeUndefined(context)
    }

    unsafe extern "C" fn warn_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        _: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        let mut buffer = Console::get_instance().buffer.lock().unwrap();

        for i in 0..argument_count {
            let arg = *arguments.add(i);
            let js_string = JSValueToStringCopy(context, arg, std::ptr::null_mut());
            let c_string = JSStringGetCharactersPtr(js_string);
            let length = JSStringGetLength(js_string);

            let rust_string =
                String::from_utf16_lossy(std::slice::from_raw_parts(c_string, length));
            buffer.push(rust_string);

            JSStringRelease(js_string);
        }

        JSValueMakeUndefined(context)
    }

    unsafe extern "C" fn error_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        _: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        let mut buffer = Console::get_instance().buffer.lock().unwrap();

        for i in 0..argument_count {
            let arg = *arguments.add(i);
            let js_string = JSValueToStringCopy(context, arg, std::ptr::null_mut());
            let c_string = JSStringGetCharactersPtr(js_string);
            let length = JSStringGetLength(js_string);

            let rust_string =
                String::from_utf16_lossy(std::slice::from_raw_parts(c_string, length));
            buffer.push(rust_string);

            JSStringRelease(js_string);
        }

        JSValueMakeUndefined(context)
    }

    unsafe extern "C" fn flush_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        _: usize,
        _: *const *const OpaqueJSValue,
        _: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        let mut buffer = Console::get_instance().buffer.lock().unwrap();
        Self::flush_buffer(&mut buffer);

        JSValueMakeUndefined(context)
    }

    fn flush_buffer(buffer: &mut Vec<String>) {
        if !buffer.is_empty() {
            println!("{}", buffer.join("\n"));
            buffer.clear();
        }
    }

    fn get_instance() -> &'static Console {
        static INSTANCE: once_cell::sync::Lazy<Console> = once_cell::sync::Lazy::new(Console::new);
        &INSTANCE
    }
}
