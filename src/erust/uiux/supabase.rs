use serde::Serialize;
use serde::Deserialize;
use web_sys::js_sys;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use serde_wasm_bindgen::to_value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupabaseResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthPayload {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProfilePayload {
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterPayload {
    pub email: String,
    pub password: String,
    pub captcha_token: String,
}

#[wasm_bindgen]
pub async fn js_signup(email: &str, password: &str) -> String {
    let payload = AuthPayload { email: email.into(), password: password.into() };
    let js_payload = to_value(&payload).unwrap();
    let promise = js_sys::Promise::resolve(&js_payload);
    let resp = match JsFuture::from(promise).await {
        Ok(js_result) => {
            match serde_wasm_bindgen::from_value::<serde_json::Value>(js_result) {
                Ok(val) => {
                    let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let data = val.get("data").cloned();
                    let error = val.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());
                    serde_json::json!({ "success": success, "data": data, "error": error }).to_string()
                },
                Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("Serde error: {:?}", e) }).to_string(),
            }
        }
        Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("JS error: {:?}", e) }).to_string(),
    };
    resp
}

#[wasm_bindgen]
pub async fn js_login(email: &str, password: &str) -> String {
    let payload = AuthPayload { email: email.into(), password: password.into() };
    let js_payload = to_value(&payload).unwrap();
    let promise = js_sys::Promise::resolve(&js_payload);
    let resp = match JsFuture::from(promise).await {
        Ok(js_result) => {
            match serde_wasm_bindgen::from_value::<serde_json::Value>(js_result) {
                Ok(val) => {
                    let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let data = val.get("data").cloned();
                    let error = val.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());
                    serde_json::json!({ "success": success, "data": data, "error": error }).to_string()
                },
                Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("Serde error: {:?}", e) }).to_string(),
            }
        }
        Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("JS error: {:?}", e) }).to_string(),
    };
    resp
}

#[wasm_bindgen]
pub async fn js_logout() -> String {
    let promise = js_sys::Promise::resolve(&JsValue::NULL);
    let resp = match JsFuture::from(promise).await {
        Ok(js_result) => {
            match serde_wasm_bindgen::from_value::<serde_json::Value>(js_result) {
                Ok(val) => {
                    let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let data = val.get("data").cloned();
                    let error = val.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());
                    serde_json::json!({ "success": success, "data": data, "error": error }).to_string()
                },
                Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("Serde error: {:?}", e) }).to_string(),
            }
        }
        Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("JS error: {:?}", e) }).to_string(),
    };
    resp
}

#[wasm_bindgen]
pub async fn js_session() -> String {
    let promise = js_sys::Promise::resolve(&JsValue::NULL);
    let resp = match JsFuture::from(promise).await {
        Ok(js_result) => {
            match serde_wasm_bindgen::from_value::<serde_json::Value>(js_result) {
                Ok(val) => {
                    let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let data = val.get("data").cloned();
                    let error = val.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());
                    serde_json::json!({ "success": success, "data": data, "error": error }).to_string()
                },
                Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("Serde error: {:?}", e) }).to_string(),
            }
        }
        Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("JS error: {:?}", e) }).to_string(),
    };
    resp
}

#[wasm_bindgen]
pub async fn js_get_user() -> String {
    let promise = js_sys::Promise::resolve(&JsValue::NULL);
    let resp = match JsFuture::from(promise).await {
        Ok(js_result) => {
            match serde_wasm_bindgen::from_value::<serde_json::Value>(js_result) {
                Ok(val) => {
                    let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let data = val.get("data").cloned();
                    let error = val.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());
                    serde_json::json!({ "success": success, "data": data, "error": error }).to_string()
                },
                Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("Serde error: {:?}", e) }).to_string(),
            }
        }
        Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("JS error: {:?}", e) }).to_string(),
    };
    resp
}

#[wasm_bindgen]
pub async fn js_get_profile(user_id: &str) -> String {
    let js_payload = serde_wasm_bindgen::to_value(&ProfilePayload { user_id: user_id.into() }).unwrap();
    let promise = js_sys::Promise::resolve(&js_payload);
    let resp = match JsFuture::from(promise).await {
        Ok(js_result) => {
            match serde_wasm_bindgen::from_value::<serde_json::Value>(js_result) {
                Ok(val) => {
                    let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let data = val.get("data").cloned();
                    let error = val.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());
                    serde_json::json!({ "success": success, "data": data, "error": error }).to_string()
                },
                Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("Serde error: {:?}", e) }).to_string(),
            }
        }
        Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("JS error: {:?}", e) }).to_string(),
    };
    resp
}

#[wasm_bindgen]
pub async fn js_signup_with_captcha(email: &str, password: &str, captcha_token: &str) -> String {
    let js_payload = serde_wasm_bindgen::to_value(&RegisterPayload { email: email.into(), password: password.into(), captcha_token: captcha_token.into() }).unwrap();
    let promise = js_sys::Promise::resolve(&js_payload);
    let resp = match JsFuture::from(promise).await {
        Ok(js_result) => {
            match serde_wasm_bindgen::from_value::<serde_json::Value>(js_result) {
                Ok(val) => {
                    let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let data = val.get("data").cloned();
                    let error = val.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());
                    serde_json::json!({ "success": success, "data": data, "error": error }).to_string()
                },
                Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("Serde error: {:?}", e) }).to_string(),
            }
        }
        Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("JS error: {:?}", e) }).to_string(),
    };
    resp
}

// Add a public Rust function for register that calls js_signup_with_captcha and awaits the result
pub async fn register(email: &str, password: &str, captcha_token: &str) -> String {
    js_signup_with_captcha(email, password, captcha_token).await
}

// Fix js_login_with_captcha to use the captcha_token in the payload
#[wasm_bindgen]
pub async fn js_login_with_captcha(email: &str, password: &str, captcha_token: &str) -> String {
    let js_payload = serde_wasm_bindgen::to_value(&RegisterPayload {
        email: email.into(),
        password: password.into(),
        captcha_token: captcha_token.into(),
    }).unwrap();
    let promise = js_sys::Promise::resolve(&js_payload);
    let resp = match JsFuture::from(promise).await {
        Ok(js_result) => {
            match serde_wasm_bindgen::from_value::<serde_json::Value>(js_result) {
                Ok(val) => {
                    let success = val.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let data = val.get("data").cloned();
                    let error = val.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());
                    serde_json::json!({ "success": success, "data": data, "error": error }).to_string()
                },
                Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("Serde error: {:?}", e) }).to_string(),
            }
        }
        Err(e) => serde_json::json!({ "success": false, "data": null, "error": format!("JS error: {:?}", e) }).to_string(),
    };
    resp
}
