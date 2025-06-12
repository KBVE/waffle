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
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    // Add more fields as needed
}

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

        // --- Login response handling ---
        let mut handled = false;
        if let Some(success) = resp_json.get("success").and_then(|v| v.as_bool()) {
            if success {
                // Try to extract user info from the response
                if let Some(data) = resp_json.get("data") {
                    if let Some(session) = data.get("session") {
                        if let Some(user) = session.get("user") {
                            if let Ok(user_obj) = serde_json::from_value::<User>(user.clone()) {
                                log::info!("Login successful! User: {:?}", user_obj);
                                handled = true;
                                // Example: callback could update state
                                // callback(serde_json::to_value(user_obj).unwrap());
                            }
                        }
                    }
                }
            } else {
                if let Some(error) = resp_json.get("error") {
                    log::error!("Login failed: {:?}", error);
                    handled = true;
                }
            }
        }
        // If not handled, log the message using the JSRust -> Log function
        if !handled {
            // Try to call the JS log function if available
            let log_msg = format!("[JSRustInterop] Unhandled message: {:?}", resp_json);
            let js_log = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("JSRust"));
            if let Ok(js_log_fn) = js_log {
                if js_log_fn.is_function() {
                    let func = js_sys::Function::from(js_log_fn);
                    let _ = func.call2(&JsValue::NULL, &JsValue::from_str("log"), &JsValue::from_str(&log_msg));
                }
            }
            // Also log to Rust console for debugging
            log::info!("[JSRustInterop] Unhandled message: {:?}", resp_json);
        }
        // Call the user callback for further handling
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

/// Call JSRust('user') to request user info from JS and send it to Rust handler
pub fn request_user_from_js() {
    if let Ok(jsrust) = js_sys::Reflect::get(&js_sys::global(), &wasm_bindgen::JsValue::from_str("JSRust")) {
        if jsrust.is_function() {
            let func = js_sys::Function::from(jsrust);
            let _ = func.call1(&wasm_bindgen::JsValue::NULL, &wasm_bindgen::JsValue::from_str("user"));
        }
    }
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

#[wasm_bindgen]
pub fn supabase_session(session: &JsValue) {
    // You can process the session object here or store it as needed
    // For now, just log it for debugging
    log::info!("[JSInterop] Received Supabase session: {:?}", session);
    // TODO: Store or process session as needed
}

#[wasm_bindgen]
pub fn supabase_user(user: &JsValue) {
    // You can process the user object here or store it as needed
    // For now, just log it for debugging
    log::info!("[JSInterop] Received Supabase user: {:?}", user);
    // TODO: Store or process user as needed
}
