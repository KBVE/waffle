// hcaptcha.rs - hCaptcha integration for WASM/egui
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::js_sys;
use std::cell::RefCell;

thread_local! {
    static LAST_TOKEN: RefCell<Option<String>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn pass_captcha_token(token: String) {
    LAST_TOKEN.with(|t| t.replace(Some(token)));
}

pub fn get_captcha_token() -> Option<String> {
    LAST_TOKEN.with(|t| t.borrow().clone())
}

pub fn clear_captcha_token() {
    LAST_TOKEN.with(|t| t.replace(None));
}

/// Call this from Rust to open the captcha overlay
pub fn open_captcha() {
    let _ = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("JSRust"))
        .and_then(|f| if f.is_function() {
            let func = js_sys::Function::from(f);
            func.call1(&JsValue::NULL, &JsValue::from_str("openCaptcha")).ok();
            Ok(())
        } else { Ok(()) });
}

/// Call this from Rust to close the captcha overlay
pub fn close_captcha() {
    let _ = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("JSRust"))
        .and_then(|f| if f.is_function() {
            let func = js_sys::Function::from(f);
            func.call1(&JsValue::NULL, &JsValue::from_str("closeCaptcha")).ok();
            Ok(())
        } else { Ok(()) });
}
