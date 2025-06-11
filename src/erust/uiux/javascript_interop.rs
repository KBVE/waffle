// javascript_interop.rs - Rust <-> JS message bridge for unified JS calls
use wasm_bindgen::JsValue;
use web_sys::js_sys;

/// Send a message object to the global JSRust JS handler.
pub fn send_jsrust_message(message: &js_sys::Object) {
    let _ = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("JSRust"))
        .and_then(|f| {
            if f.is_function() {
                let func = js_sys::Function::from(f);
                func.call1(&JsValue::NULL, message).ok();
                Ok(())
            } else {
                Ok(())
            }
        });
}

/// Helper to build and send a login/register message to JSRust
pub fn send_action_message(action: &str, email: &str, password: &str, captcha_token: &str) {
    let msg = js_sys::Object::new();
    js_sys::Reflect::set(&msg, &JsValue::from_str("action"), &JsValue::from_str(action)).ok();
    js_sys::Reflect::set(&msg, &JsValue::from_str("email"), &JsValue::from_str(email)).ok();
    js_sys::Reflect::set(&msg, &JsValue::from_str("password"), &JsValue::from_str(password)).ok();
    js_sys::Reflect::set(&msg, &JsValue::from_str("captcha_token"), &JsValue::from_str(captcha_token)).ok();
    send_jsrust_message(&msg);
}

/// Register a JS callback handler for responses from JS to Rust
/// The callback will be called with a JsValue (the response object)
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn set_jsrust_response_handler(cb: &js_sys::Function) {
    // Store the callback globally in JS (window.JSRustResponseHandler)
    let _ = js_sys::Reflect::set(
        &js_sys::global(),
        &JsValue::from_str("JSRustResponseHandler"),
        cb,
    );
}

/// Call this from JS to send a response back to Rust
#[wasm_bindgen]
pub fn handle_jsrust_response(response: &JsValue) {
    // If a handler is set, call it with the response
    if let Ok(handler) = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("JSRustResponseHandler")) {
        if handler.is_function() {
            let func = js_sys::Function::from(handler);
            let _ = func.call1(&JsValue::NULL, response);
        }
    }
}
