use javascriptcore_sys::*;
use std::arch::asm;
use std::ffi::CString;

const BUFFER_SIZE: usize = 65536;
const MAX_CACHED_NUM: usize = 100_000;
const WARNING_PREFIX: [u8; 18] = *b"\x1b[33mWARNING:\x1b[0m ";
const ERROR_PREFIX: [u8; 16] = *b"\x1b[31mERROR:\x1b[0m ";

#[repr(C, align(64))]
struct OutputBuffer {
    data: [u8; BUFFER_SIZE],
    pos: usize,
}

static mut OUTPUT_BUFFER: OutputBuffer = OutputBuffer {
    data: [0; BUFFER_SIZE],
    pos: 0,
};

#[allow(long_running_const_eval)]
const NUMBER_CACHE: ([[u8; 16]; MAX_CACHED_NUM], [usize; MAX_CACHED_NUM]) = {
    let mut cache = [[0u8; 16]; MAX_CACHED_NUM];
    let mut lengths = [0usize; MAX_CACHED_NUM];
    let mut i = 0;
    while i < MAX_CACHED_NUM {
        let mut buf = [0u8; 16];
        let mut pos = 0;
        {
            let mut num = i;
            let mut digits = [0u8; 20];
            let mut digit_count = 0;
            if num == 0 {
                digits[0] = b'0';
                digit_count = 1;
            } else {
                while num > 0 {
                    digits[digit_count] = (num % 10) as u8 + b'0';
                    num /= 10;
                    digit_count += 1;
                }
            }
            let mut j = digit_count;
            while j > 0 {
                j -= 1;
                buf[pos] = digits[j];
                pos += 1;
            }
            buf[pos] = b'\n';
            pos += 1;
            lengths[i] = pos;
        }
        cache[i] = buf;
        i += 1;
    }
    (cache, lengths)
};

#[inline(always)]
unsafe fn write_stdout(buf: &[u8]) {
    let _ret: i32;
    let fd: i32 = 1;
    asm!(
        "syscall",
        inlateout("rax") 1i32 => _ret,
        in("rdi") fd,
        in("rsi") buf.as_ptr(),
        in("rdx") buf.len(),
        out("rcx") _,
        out("r11") _,
        options(nostack)
    );
}

#[inline(always)]
unsafe fn flush_buffer() {
    if OUTPUT_BUFFER.pos > 0 {
        write_stdout(&OUTPUT_BUFFER.data[..OUTPUT_BUFFER.pos]);
        OUTPUT_BUFFER.pos = 0;
    }
}

#[inline(always)]
unsafe fn buffer_write(buf: &[u8]) {
    if OUTPUT_BUFFER.pos + buf.len() >= BUFFER_SIZE {
        flush_buffer();
    }
    OUTPUT_BUFFER.data[OUTPUT_BUFFER.pos..OUTPUT_BUFFER.pos + buf.len()].copy_from_slice(buf);
    OUTPUT_BUFFER.pos += buf.len();
}

#[inline(always)]
unsafe fn write_number(num: usize) {
    if num < MAX_CACHED_NUM {
        buffer_write(&NUMBER_CACHE.0[num][..NUMBER_CACHE.1[num]]);
    } else {
        let mut buf = [0u8; 20];
        let mut pos = 0;
        if num == 0 {
            buf[0] = b'0';
            pos = 1;
        } else {
            let mut n = num;
            while n > 0 {
                buf[pos] = (n % 10) as u8 + b'0';
                n /= 10;
                pos += 1;
            }
            let mut i = 0;
            let mut j = pos - 1;
            while i < j {
                buf.swap(i, j);
                i += 1;
                j -= 1;
            }
        }
        buf[pos] = b'\n';
        pos += 1;
        buffer_write(&buf[..pos]);
    }
}

pub struct Console;

impl Console {
    #[inline(always)]
    pub unsafe fn bind_to_context(ctx: JSGlobalContextRef) {
        let console_name = CString::new("console").unwrap();
        let console_str = JSStringCreateWithUTF8CString(console_name.as_ptr());
        let console_obj = JSObjectMake(ctx, std::ptr::null_mut(), std::ptr::null_mut());

        let methods = [
            ("log", console_log as *const ()),
            ("warn", console_warn as *const ()),
            ("error", console_error as *const ()),
            ("flush", console_flush as *const ()),
        ];

        for (name, callback) in methods.iter() {
            let name = CString::new(*name).unwrap();
            let str_ref = JSStringCreateWithUTF8CString(name.as_ptr());
            let fn_obj = JSObjectMakeFunctionWithCallback(
                ctx,
                str_ref,
                Some(std::mem::transmute(*callback)),
            );
            JSObjectSetProperty(ctx, console_obj, str_ref, fn_obj, 0, std::ptr::null_mut());
            JSStringRelease(str_ref);
        }

        let global = JSContextGetGlobalObject(ctx);
        JSObjectSetProperty(
            ctx,
            global,
            console_str,
            console_obj,
            0,
            std::ptr::null_mut(),
        );
        JSStringRelease(console_str);
    }
}

#[inline(always)]
unsafe fn is_number(ctx: JSContextRef, value: JSValueRef) -> bool {
    JSValueGetType(ctx, value) as u32 == 1
}

unsafe extern "C" fn console_log(
    ctx: JSContextRef,
    _function: JSObjectRef,
    _this: JSObjectRef,
    argument_count: usize,
    arguments: *const JSValueRef,
    _exception: *mut JSValueRef,
) -> JSValueRef {
    if argument_count == 1 {
        let arg = *arguments;
        if is_number(ctx, arg) {
            let num = JSValueToNumber(ctx, arg, std::ptr::null_mut());
            if num >= 0.0 {
                write_number(num as usize);
                if OUTPUT_BUFFER.pos > BUFFER_SIZE / 2 {
                    flush_buffer();
                }
                return JSValueMakeUndefined(ctx);
            }
        }
    }

    for i in 0..argument_count {
        let arg = *arguments.offset(i as isize);
        let str_ref = JSValueToStringCopy(ctx, arg, std::ptr::null_mut());
        let max_size = JSStringGetMaximumUTF8CStringSize(str_ref);

        let mut local_buf = [0u8; 1024];
        JSStringGetUTF8CString(str_ref, local_buf.as_mut_ptr() as *mut _, max_size);

        let str_len = (0..max_size).take_while(|&j| local_buf[j] != 0).count();
        buffer_write(&local_buf[..str_len]);

        if i < argument_count - 1 {
            buffer_write(b" ");
        }

        JSStringRelease(str_ref);
    }

    buffer_write(b"\n");

    if OUTPUT_BUFFER.pos > BUFFER_SIZE / 2 {
        flush_buffer();
    }

    JSValueMakeUndefined(ctx)
}

unsafe extern "C" fn console_warn(
    ctx: JSContextRef,
    function: JSObjectRef,
    this: JSObjectRef,
    argument_count: usize,
    arguments: *const JSValueRef,
    exception: *mut JSValueRef,
) -> JSValueRef {
    write_stdout(&WARNING_PREFIX);
    console_log(ctx, function, this, argument_count, arguments, exception)
}

unsafe extern "C" fn console_error(
    ctx: JSContextRef,
    function: JSObjectRef,
    this: JSObjectRef,
    argument_count: usize,
    arguments: *const JSValueRef,
    exception: *mut JSValueRef,
) -> JSValueRef {
    write_stdout(&ERROR_PREFIX);
    console_log(ctx, function, this, argument_count, arguments, exception)
}

unsafe extern "C" fn console_flush(
    ctx: JSContextRef,
    _function: JSObjectRef,
    _this: JSObjectRef,
    _argument_count: usize,
    _arguments: *const JSValueRef,
    _exception: *mut JSValueRef,
) -> JSValueRef {
    flush_buffer();
    JSValueMakeUndefined(ctx)
}
