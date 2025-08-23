use crate::utility::stdout::write_stdout;
use once_cell::sync::Lazy;
use rusty_jsc::*;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::sync::Mutex;

const BUF_SIZE: usize = 1024 * 1024;
const CHUNK_SIZE: usize = 1000;

static COLORS: Lazy<JSColors> = Lazy::new(|| JSColors {
    null: b"\x1b[90m",
    undefined: b"\x1b[90m",
    boolean: b"\x1b[33m",
    number: b"\x1b[33m",
    string: b"\x1b[32m",
    array: b"\x1b[36m",
    object: b"\x1b[34m",
    unknown: b"\x1b[37m",
    reset: b"\x1b[0m",
});

type JSCallback = unsafe extern "C" fn(
    *const OpaqueJSContext,
    *mut OpaqueJSValue,
    *mut OpaqueJSValue,
    usize,
    *const *const OpaqueJSValue,
    *mut *const OpaqueJSValue,
) -> *const OpaqueJSValue;

static FUNCTION_NAMES: Lazy<[(CString, JSCallback); 4]> = Lazy::new(|| {
    [
        (CString::new("log").unwrap(), Console::log_callback),
        (CString::new("warn").unwrap(), Console::warn_callback),
        (CString::new("error").unwrap(), Console::error_callback),
        (CString::new("flush").unwrap(), Console::flush_callback),
    ]
});

static CONSOLE_STR: Lazy<CString> = Lazy::new(|| CString::new("console").unwrap());

#[repr(u8)]
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

pub struct Console {
    buffer: Mutex<Vec<u8>>,
}

impl Console {
    #[inline(always)]
    pub fn new() -> Self {
        Console {
            buffer: Mutex::new(Vec::with_capacity(BUF_SIZE)),
        }
    }

    pub fn bind_to_context(&self, context: *mut OpaqueJSContext) {
        unsafe {
            let global_object = JSContextGetGlobalObject(context);
            let console = JSObjectMake(context, std::ptr::null_mut(), std::ptr::null_mut());

            for (name, callback) in FUNCTION_NAMES.iter() {
                let js_string = JSStringCreateWithUTF8CString(name.as_ptr());
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

            let console_js_string = JSStringCreateWithUTF8CString(CONSOLE_STR.as_ptr());
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
        unsafe {
            if JSValueIsNull(context, value) {
                JSType::Null
            } else if JSValueIsUndefined(context, value) {
                JSType::Undefined
            } else if JSValueIsBoolean(context, value) {
                JSType::Boolean
            } else if JSValueIsNumber(context, value) {
                JSType::Number
            } else if JSValueIsString(context, value) {
                JSType::String
            } else if JSValueIsObject(context, value) {
                if JSValueIsArray(context, value) {
                    JSType::Array
                } else {
                    JSType::Object
                }
            } else {
                JSType::Unknown
            }
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

    #[inline(always)]
    unsafe fn write_value(
        context: *const OpaqueJSContext,
        arg: *const OpaqueJSValue,
        buffer: &mut Vec<u8>,
        is_first: bool,
    ) {
        if !is_first {
            buffer.push(b' ');
        }

        let value_type = unsafe { Self::get_value_type(context, arg) };
        buffer.extend_from_slice(unsafe { Self::get_value_color(value_type) });

        let mut js_string = std::ptr::null_mut();
        if matches!(value_type, JSType::Object | JSType::Array) {
            js_string = unsafe { JSValueCreateJSONString(context, arg, 2, std::ptr::null_mut()) };
        }
        if js_string.is_null() {
            js_string = unsafe { JSValueToStringCopy(context, arg, std::ptr::null_mut()) };
        }
        let c_string = unsafe { JSStringGetCharactersPtr(js_string) };
        let length = unsafe { JSStringGetLength(js_string) };

        if length < 128 {
            let mut stack_buf = [MaybeUninit::<u8>::uninit(); 256];
            let mut pos = 0;
            for i in 0..length {
                let c = unsafe { *c_string.add(i) };
                if c < 128 {
                    stack_buf[pos].write(c as u8);
                    pos += 1;
                } else {
                    let bytes = (c as u16).to_le_bytes();
                    stack_buf[pos].write(bytes[0]);
                    stack_buf[pos + 1].write(bytes[1]);
                    pos += 2;
                }
            }
            buffer.extend_from_slice(unsafe {
                std::slice::from_raw_parts(stack_buf.as_ptr() as *const u8, pos)
            });
        } else {
            let rust_string =
                String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(c_string, length) });
            buffer.extend_from_slice(rust_string.as_bytes());
        }

        buffer.extend_from_slice(COLORS.reset);
        unsafe { JSStringRelease(js_string) };
    }

    #[inline(always)]
    unsafe fn process_arguments(
        argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        context: *const OpaqueJSContext,
        buffer: &mut Vec<u8>,
    ) {
        for i in 0..argument_count {
            let arg = unsafe { *arguments.add(i) };
            unsafe { Self::write_value(context, arg, buffer, i == 0) };
        }
        buffer.push(b'\n');
    }

    unsafe extern "C" fn log_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        _: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        if let Ok(mut buffer) = Console::get_instance().buffer.lock() {
            unsafe { Self::process_arguments(argument_count, arguments, context, &mut buffer) };

            if buffer.len() >= CHUNK_SIZE {
                Self::flush_buffer(&mut buffer);
            }
        }

        unsafe { JSValueMakeUndefined(context) }
    }

    unsafe extern "C" fn warn_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        _: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        if let Ok(mut buffer) = Console::get_instance().buffer.lock() {
            buffer.extend_from_slice(COLORS.number);
            unsafe { Self::process_arguments(argument_count, arguments, context, &mut buffer) };
            buffer.extend_from_slice(COLORS.reset);
        }
        unsafe { JSValueMakeUndefined(context) }
    }

    unsafe extern "C" fn error_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        _: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        if let Ok(mut buffer) = Console::get_instance().buffer.lock() {
            buffer.extend_from_slice(b"\x1b[31m");
            unsafe { Self::process_arguments(argument_count, arguments, context, &mut buffer) };
            buffer.extend_from_slice(COLORS.reset);
        }
        unsafe { JSValueMakeUndefined(context) }
    }

    unsafe extern "C" fn flush_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        _: usize,
        _: *const *const OpaqueJSValue,
        _: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        if let Ok(mut buffer) = Console::get_instance().buffer.lock() {
            Self::flush_buffer(&mut buffer);
        }
        unsafe { JSValueMakeUndefined(context) }
    }

    #[inline(always)]
    fn flush_buffer(buffer: &mut Vec<u8>) {
        if !buffer.is_empty() {
            unsafe {
                write_stdout(std::str::from_utf8_unchecked(buffer));
            }
            buffer.clear();
        }
    }

    #[inline(always)]
    fn get_instance() -> &'static Console {
        static INSTANCE: Lazy<Console> = Lazy::new(Console::new);
        &INSTANCE
    }
}
