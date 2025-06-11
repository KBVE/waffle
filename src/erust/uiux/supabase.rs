use serde::Serialize;
use serde::Deserialize;

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

// Remove all #[wasm_bindgen] functions related to auth, since JS handler is now used

// Add a public Rust function for register that calls the JS handler
pub fn register(email: &str, password: &str, captcha_token: &str) {
    crate::erust::uiux::javascript_interop::send_action_message(
        "register",
        email,
        password,
        captcha_token,
    );
}

// Add a public Rust function for login that calls the JS handler
pub fn login(email: &str, password: &str, captcha_token: &str) {
    crate::erust::uiux::javascript_interop::send_action_message(
        "login",
        email,
        password,
        captcha_token,
    );
}
