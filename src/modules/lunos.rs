use javascriptcore_sys::*;
use std::{ffi::CString, thread};
use tiny_http::{Server, Response};

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

            let lunos_object = JSObjectMake(context, std::ptr::null_mut(), std::ptr::null_mut());
            JSObjectSetProperty(
                context,
                lunos_object,
                JSStringCreateWithUTF8CString(serve_name.as_ptr()),
                serve_function,
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

    unsafe extern "C" fn serve_callback(
        context: *const OpaqueJSContext,
        _: *mut OpaqueJSValue,
        _: *mut OpaqueJSValue,
        argument_count: usize,
        arguments: *const *const OpaqueJSValue,
        _: *mut *const OpaqueJSValue,
    ) -> *const OpaqueJSValue {
        if argument_count < 1 {
            eprintln!("Lunos.serve expects 1 argument (a configuration object).");
            return JSValueMakeUndefined(context);
        }

        let config = *arguments.add(0);
        if JSValueIsObject(context, config) == false {
            eprintln!("Argument to Lunos.serve must be an object.");
            return JSValueMakeUndefined(context);
        }

        let port = Self::get_property_as_u16(context, config, "port");
        if port.is_none() {
            eprintln!("Invalid or missing 'port' property in configuration object.");
            return JSValueMakeUndefined(context);
        }

        let port = port.unwrap();
        let content_type = Self::get_property_as_string(context, config, "type").unwrap_or("text/plain".to_string());
        let response_text = Self::get_property_as_string(context, config, "return").unwrap_or("Hello, World!".to_string());

        thread::spawn(move || {
            let server = Server::http(format!("0.0.0.0:{}", port)).unwrap();
            println!("Listening on :{}", port);

            for request in server.incoming_requests() {
                let response = Response::from_string(response_text.clone())
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap());
                let _ = request.respond(response);
            }
        });

        JSValueMakeUndefined(context)
    }

    unsafe fn get_property_as_string(
        context: *const OpaqueJSContext,
        object: *const OpaqueJSValue,
        property_name: &str,
    ) -> Option<String> {
        let property_name_cstring = CString::new(property_name).unwrap();
        let property_name = JSStringCreateWithUTF8CString(property_name_cstring.as_ptr());
        let property_value = JSObjectGetProperty(context, object as *mut _, property_name, std::ptr::null_mut());
        JSStringRelease(property_name);

        if JSValueIsString(context, property_value) != false {
            let js_string = JSValueToStringCopy(context, property_value, std::ptr::null_mut());
            let c_string = JSStringGetCharactersPtr(js_string);
            let length = JSStringGetLength(js_string);

            let rust_string = String::from_utf16_lossy(std::slice::from_raw_parts(c_string, length));
            JSStringRelease(js_string);

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
        let property_name = JSStringCreateWithUTF8CString(property_name_cstring.as_ptr());
        let property_value = JSObjectGetProperty(context, object as *mut _, property_name, std::ptr::null_mut());
        JSStringRelease(property_name);

        if JSValueIsNumber(context, property_value) != false {
            Some(JSValueToNumber(context, property_value, std::ptr::null_mut()) as u16)
        } else {
            None
        }
    }
}
