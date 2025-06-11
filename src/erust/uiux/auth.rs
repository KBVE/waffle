// erust/uiux/auth.rs
use egui::{Context, Ui};
use crate::erust::uiux::hcaptcha;

pub struct AuthWidget {
    pub email: String,
    pub password: String,
    pub captcha_token: Option<String>,
    pub error: Option<String>,
    pub is_register: bool,
}

impl AuthWidget {
    pub fn new(is_register: bool) -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            captcha_token: None,
            error: None,
            is_register,
        }
    }

    pub fn show(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.heading(if self.is_register { "Register" } else { "Login" });
        ui.label("Email:");
        ui.text_edit_singleline(&mut self.email);
        ui.label("Password:");
        ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
        if ui.button("Solve Captcha").clicked() {
            hcaptcha::open_captcha();
        }
        if let Some(token) = hcaptcha::get_captcha_token() {
            self.captcha_token = Some(token);
            ui.label("Captcha solved!");
        } else {
            ui.label("Captcha required");
        }
        if ui.button(if self.is_register { "Register" } else { "Login" }).clicked() {
            if self.email.is_empty() || self.password.is_empty() || self.captcha_token.is_none() {
                self.error = Some("All fields and captcha are required".to_string());
            } else {
                // Here you would call your Supabase API with email, password, and captcha_token
                // Example: supabase_auth(self.email.clone(), self.password.clone(), self.captcha_token.clone())
                self.error = Some("(Stub) Would call Supabase here".to_string());
            }
        }
        if let Some(err) = &self.error {
            ui.colored_label(egui::Color32::RED, err);
        }
    }
}

// Make AuthWidget public for use in other modules
