use javascriptcore_sys::*;
use std::arch::asm;
use std::ffi::CString;

const BUFFER_SIZE: usize = 8192;
const WARNING_PREFIX: [u8; 18] = *b"\x1b[33mWARNING:\x1b[0m ";
const ERROR_PREFIX: [u8; 16] = *b"\x1b[31mERROR:\x1b[0m ";

#[repr(C, align(64))]
struct AlignedBuffer([u8; BUFFER_SIZE]);

static mut OUTPUT_BUFFER: AlignedBuffer = AlignedBuffer([0; BUFFER_SIZE]);

#[allow(long_running_const_eval)]
const NUMBER_CACHE: ([[u8; 16]; 1000000], [usize; 1000000]) = {
    let mut cache = [[0u8; 16]; 1000000];
    let mut lengths = [0usize; 1000000];
    let mut i = 0;
    while i < 10000 {
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
unsafe fn write_number(num: usize) {
    match num {
        0..=9 => write_stdout(&NUMBER_CACHE.0[num][..NUMBER_CACHE.1[num]]),
        _ => {
            let num_str = num.to_string();
            OUTPUT_BUFFER.0[..num_str.len()].copy_from_slice(num_str.as_bytes());
            OUTPUT_BUFFER.0[num_str.len()] = b'\n';
            write_stdout(&OUTPUT_BUFFER.0[..=num_str.len()]);
        }
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
                return JSValueMakeUndefined(ctx);
            }
        }
    }

    let mut pos = 0;
    for i in 0..argument_count {
        let arg = *arguments.offset(i as isize);
        let str_ref = JSValueToStringCopy(ctx, arg, std::ptr::null_mut());
        let max_size = JSStringGetMaximumUTF8CStringSize(str_ref);

        #[allow(static_mut_refs)]
        JSStringGetUTF8CString(
            str_ref,
            OUTPUT_BUFFER.0.as_mut_ptr().add(pos) as *mut _,
            max_size,
        );

        #[allow(static_mut_refs)]
        let str_len = (0..max_size)
            .take_while(|&j| *OUTPUT_BUFFER.0.as_ptr().add(pos + j) != 0)
            .count();
        pos += str_len;

        if i < argument_count - 1 {
            OUTPUT_BUFFER.0[pos] = b' ';
            pos += 1;
        }

        JSStringRelease(str_ref);
    }

    OUTPUT_BUFFER.0[pos] = b'\n';
    pos += 1;

    write_stdout(&OUTPUT_BUFFER.0[..pos]);
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
