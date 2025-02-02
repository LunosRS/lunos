use crate::utility::stdout::write_stdout;
use javascriptcore_sys::*;
use once_cell::sync::Lazy;
use std::ffi::CString;
use std::sync::Mutex;

const BUF_SIZE: usize = 1024 * 1024; // 1MB buffer
const CHUNK_SIZE: usize = 1000; // Process logs in chunks

// Pre-compute color codes
static COLORS: Lazy<JSColors> = Lazy::new(|| JSColors {
    null: b"\x1b[90m",      // Gray
    undefined: b"\x1b[90m", // Gray
    boolean: b"\x1b[35m",   // Magenta
    number: b"\x1b[33m",    // Yellow
    string: b"",            // No color
    array: b"\x1b[36m",     // Cyan
    object: b"\x1b[34m",    // Blue
    unknown: b"\x1b[37m",   // White
    reset: b"\x1b[0m",
});

struct JSColors {
    null: &'static [u8],
    undefined: &'static [u8],
    boolean: &'static [u8],
    number: &'static [u8],
    string: &'static [u8],
    array: &'static [u8],
    object: &'static [u8],
    unknown: &'static [u8],
    reset: &'static [u8],
}

#[derive(Copy, Clone)]
enum JSType {
    Null,
    Undefined,
    Boolean,
    Number,
    String,
    Array,
    Object,
    Unknown,
}

pub struct Console {
    buffer: Mutex<Vec<u8>>,
}

impl Console {
    pub fn new() -> Self {
        Console {
            buffer: Mutex::new(Vec::with_capacity(BUF_SIZE)),
        }
    }

    pub fn bind_to_context(self: &Self, context: *mut OpaqueJSContext) {
        unsafe {
            let global_object = JSContextGetGlobalObject(context);
            let console = JSObjectMake(context, std::ptr::null_mut(), std::ptr::null_mut());

            // Pre-compute all CStrings
            let function_names = [
                (
                    "log",
                    Self::log_callback as unsafe extern "C" fn(_, _, _, _, _, _) -> _,
                ),
                (
                    "warn",
                    Self::warn_callback as unsafe extern "C" fn(_, _, _, _, _, _) -> _,
                ),
                (
                    "error",
                    Self::error_callback as unsafe extern "C" fn(_, _, _, _, _, _) -> _,
                ),
                (
                    "flush",
                    Self::flush_callback as unsafe extern "C" fn(_, _, _, _, _, _) -> _,
                ),
            ];

            for (name, callback) in function_names.iter() {
                let name_cstr = CString::new(*name).unwrap();
                let js_string = JSStringCreateWithUTF8CString(name_cstr.as_ptr());
                let function =
                    JSObjectMakeFunctionWithCallback(context, js_string, Some(*callback));
                JSObjectSetProperty(
                    context,
                    console,
                    js_string,
                    function,
                    kJSPropertyAttributeNone,
                    std::ptr::null_mut(),
                );
                JSStringRelease(js_string);
            }

            let console_cstr = CString::new("console").unwrap();
            let console_js_string = JSStringCreateWithUTF8CString(console_cstr.as_ptr());
            JSObjectSetProperty(
                context,
                global_object,
                console_js_string,
                console,
                kJSPropertyAttributeNone,
                std::ptr::null_mut(),
            );
            JSStringRelease(console_js_string);
        }
    }

    #[inline(always)]
    unsafe fn get_value_type(
        context: *const OpaqueJSContext,
        value: *const OpaqueJSValue,
    ) -> JSType {
        if JSValueIsNull(context, value) != false {
            JSType::Null
        } else if JSValueIsUndefined(context, value) != false {
            JSType::Undefined
        } else if JSValueIsBoolean(context, value) != false {
            JSType::Boolean
        } else if JSValueIsNumber(context, value) != false {
            JSType::Number
        } else if JSValueIsString(context, value) != false {
            JSType::String
        } else if JSValueIsObject(context, value) != false {
            if JSValueIsArray(context, value) != false {
                JSType::Array
            } else {
                JSType::Object
            }
        } else {
            JSType::Unknown
        }
    }

    #[inline(always)]
    unsafe fn get_value_color(value_type: JSType) -> &'static [u8] {
        match value_type {
            JSType::Null => COLORS.null,
            JSType::Undefined => COLORS.undefined,
            JSType::Boolean => COLORS.boolean,
            JSType::Number => COLORS.number,
            JSType::String => COLORS.string,
            JSType::Array => COLORS.array,
            JSType::Object => COLORS.object,
            JSType::Unknown => COLORS.unknown,
        }
    }

    unsafe fn write_value(
        context: *const OpaqueJSContext,
        arg: *const OpaqueJSValue,
        buffer: &mut Vec<u8>,
        is_first: bool,
    ) {
        if !is_first {
            buffer.extend_from_slice(b" ");
        }

        let value_type = Self::get_value_type(context, arg);
        buffer.extend_from_slice(Self::get_value_color(value_type));

        let js_string = JSValueToStringCopy(context, arg, std::ptr::null_mut());
        let c_string = JSStringGetCharactersPtr(js_string);
        let length = JSStringGetLength(js_string);
        let rust_string = String::from_utf16_lossy(std::slice::from_raw_parts(c_string, length));

        buffer.extend_from_slice(rust_string.as_bytes());
        buffer.extend_from_slice(COLORS.reset);

        JSStringRelease(js_string);
    }

    unsafe fn process_arguments(
        argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        context: *const OpaqueJSContext,
        buffer: &mut Vec<u8>,
    ) {
        for i in 0..argument_count {
            let arg = *arguments.add(i);
            Self::write_value(context, arg, buffer, i == 0);
        }
        buffer.extend_from_slice(b"\n");
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
        Self::process_arguments(argument_count, arguments, context, &mut buffer);

        // Auto-flush on large chunks
        if buffer.len() >= CHUNK_SIZE {
            Self::flush_buffer(&mut buffer);
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
        buffer.extend_from_slice(COLORS.number);
        Self::process_arguments(argument_count, arguments, context, &mut buffer);
        buffer.extend_from_slice(COLORS.reset);
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
        buffer.extend_from_slice(b"\x1b[31m");
        Self::process_arguments(argument_count, arguments, context, &mut buffer);
        buffer.extend_from_slice(COLORS.reset);
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

    fn flush_buffer(buffer: &mut Vec<u8>) {
        if !buffer.is_empty() {
            write_stdout(std::str::from_utf8(buffer).unwrap_or(""));
            buffer.clear();
        }
    }

    fn get_instance() -> &'static Console {
        static INSTANCE: once_cell::sync::Lazy<Console> = once_cell::sync::Lazy::new(Console::new);
        &INSTANCE
    }
}
