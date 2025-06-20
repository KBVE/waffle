use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseUserRaw {
    pub id: String,
    pub aud: String,
    pub role: String,
    pub email: String,
    pub email_confirmed_at: Option<String>,
    pub phone: Option<String>,
    pub confirmed_at: Option<String>,
    pub last_sign_in_at: Option<String>,
    pub app_metadata: AppMetadata,
    pub user_metadata: UserMetadata,
    pub identities: Vec<Identity>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub is_anonymous: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    pub provider: Option<String>,
    pub providers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMetadata {
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub phone_verified: Option<bool>,
    pub sub: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub identity_id: Option<String>,
    pub id: Option<String>,
    pub user_id: Option<String>,
    pub identity_data: Option<UserMetadata>,
    pub provider: Option<String>,
    pub last_sign_in_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub email: Option<String>,
}

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
