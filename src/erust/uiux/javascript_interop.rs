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
use wasm_bindgen::JsCast;
use js_sys::JSON;

/// Call this during app initialization to handle JS->Rust responses
pub fn setup_jsrust_response_handler<F>(mut callback: F)
where
    F: 'static + FnMut(serde_json::Value),
{
    let handler = Closure::wrap(Box::new(move |resp: JsValue| {
        // Convert JsValue to serde_json::Value using JSON::stringify and serde_json
        let resp_json = if let Ok(js_str) = JSON::stringify(&resp) {
            if let Some(s) = js_str.as_string() {
                serde_json::from_str(&s).unwrap_or_default()
            } else {
                serde_json::Value::Null
            }
        } else {
            serde_json::Value::Null
        };
        callback(resp_json);
    }) as Box<dyn FnMut(JsValue)>);

    // Call the local set_jsrust_response_handler directly
    set_jsrust_response_handler(handler.as_ref().unchecked_ref());
    handler.forget(); // Prevent the closure from being dropped
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

// Add back the set_jsrust_response_handler as a public function so it is available in scope
#[wasm_bindgen]
pub fn set_jsrust_response_handler(cb: &js_sys::Function) {
    // Store the callback globally in JS (window.JSRustResponseHandler)
    let _ = js_sys::Reflect::set(
        &js_sys::global(),
        &JsValue::from_str("JSRustResponseHandler"),
        cb,
    );
}

// Expand AppState for interop waiting
// In state.rs (or wherever your AppState is defined):
//
// pub enum AppState {
//     Init,
//     Normal,
//     Empty,
//     Error(String),
//     InteropPending(String), // Add this variant for JS interop waiting
// }
//
// Usage example:
// self.app_state = AppState::InteropPending("Waiting for JS response...".to_string());
//
// In your interop callback, set it back to Normal or Error as appropriate.
