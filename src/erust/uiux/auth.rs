// erust/uiux/auth.rs
use egui::{Context, Ui};
use crate::erust::uiux::hcaptcha;
use crate::erust::uiux::javascript_interop;

pub enum AuthView {
    Login,
    Register,
    Help,
}

pub struct AuthWidget {
    pub email: String,
    pub password: String,
    pub confirm_password: String, // Only used for Register
    pub captcha_token: Option<String>,
    pub error: Option<String>,
    pub view: AuthView,
    pub is_registered: bool,
}

impl AuthWidget {
    pub fn new(is_registered: bool) -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            captcha_token: None,
            error: None,
            view: if is_registered { AuthView::Login } else { AuthView::Register },
            is_registered,
        }
    }

    pub fn show(&mut self, ctx: &Context, ui: &mut Ui) {
        match self.view {
            AuthView::Login => self.show_login(ctx, ui),
            AuthView::Register => self.show_register(ctx, ui),
            AuthView::Help => self.show_help(ctx, ui),
        }
    }

    fn show_login(&mut self, _ctx: &Context, ui: &mut Ui) {
        ui.heading("Login");
        ui.separator();
        ui.label("Email:");
        ui.text_edit_singleline(&mut self.email);
        ui.label("Password:");
        ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
        ui.separator();

        if ui.button("Solve Captcha").clicked() {
            hcaptcha::open_captcha();
        }
        // Always show the captcha status message, and force a repaint if token is set
        // Only show the solved message, not the token
        if hcaptcha::get_captcha_token().is_some() {
            ui.colored_label(egui::Color32::GREEN, "Captcha Solved: Token Set");
            ui.ctx().request_repaint();
        } else {
            ui.label("Captcha required");
        }
        if ui.button("Login").clicked() {
            if self.email.is_empty() || self.password.is_empty() || hcaptcha::get_captcha_token().is_none() {
                self.error = Some("All fields and captcha are required".to_string());
            } else {
                let _token = hcaptcha::take_captcha_token();
                // self.error = Some("(Stub) Would call Supabase login here".to_string());
                login_with_js(self.email.as_str(), self.password.as_str(), _token.as_deref().unwrap_or(""));
            }
        }
        if let Some(err) = &self.error {
            ui.colored_label(egui::Color32::RED, err);
        }
        ui.horizontal(|ui| {
            if ui.button("Register").clicked() {
                self.view = AuthView::Register;
                self.error = None;
            }
            if ui.button("Help").clicked() {
                self.view = AuthView::Help;
                self.error = None;
            }
        });
    }

    fn show_register(&mut self, _ctx: &Context, ui: &mut Ui) {
        ui.heading("Register");
        ui.label("Email:");
        ui.text_edit_singleline(&mut self.email);
        ui.label("Password:");
        ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
        ui.label("Confirm Password:");
        ui.add(egui::TextEdit::singleline(&mut self.confirm_password).password(true));
        if ui.button("Solve Captcha").clicked() {
            hcaptcha::open_captcha();
        }
        // Always show the captcha status message, and force a repaint if token is set
        // Only show the solved message, not the token
        if hcaptcha::get_captcha_token().is_some() {
            ui.colored_label(egui::Color32::GREEN, "Captcha Solved: Token Set");
            ui.ctx().request_repaint();
        } else {
            ui.label("Captcha required");
        }
        if ui.button("Register").clicked() {
            if self.email.is_empty() || self.password.is_empty() || self.confirm_password.is_empty() || hcaptcha::get_captcha_token().is_none() {
                self.error = Some("All fields and captcha are required".to_string());
            } else if self.password != self.confirm_password {
                self.error = Some("Passwords do not match".to_string());
            } else {
                let token = hcaptcha::take_captcha_token();
                if let Some(token) = token {
                    let email = self.email.clone();
                    let password = self.password.clone();
                    self.error = Some("Processing registration...".to_string());
                    crate::erust::uiux::auth::register_with_js(
                        email.as_str(),
                        password.as_str(),
                        token.as_str(),
                    );
                } else {
                    self.error = Some("Captcha token missing".to_string());
                }
            }
        }
        // Show spinner if processing
        if let Some(err) = &self.error {
            if err == "Processing registration..." {
                ui.horizontal(|ui| {
                    ui.label("Processing registration...");
                    ui.add(egui::Spinner::default());
                });
            } else {
                ui.colored_label(egui::Color32::RED, err);
            }
        }
        ui.horizontal(|ui| {
            if ui.button("Back to Login").clicked() {
                self.view = AuthView::Login;
                self.error = None;
            }
            if ui.button("Help").clicked() {
                self.view = AuthView::Help;
                self.error = None;
            }
        });
    }

    fn show_help(&mut self, _ctx: &Context, ui: &mut Ui) {
        ui.heading("Forgot your password?");
        ui.label("Enter your email to receive a reset link:");
        ui.text_edit_singleline(&mut self.email);
        if ui.button("Send Reset Link").clicked() {
            if self.email.is_empty() {
                self.error = Some("Email is required".to_string());
            } else {
                self.error = Some("(Stub) Would call Supabase password reset here".to_string());
            }
        }
        if let Some(err) = &self.error {
            ui.colored_label(egui::Color32::RED, err);
        }
        ui.horizontal(|ui| {
            if ui.button("Back to Login").clicked() {
                self.view = AuthView::Login;
                self.error = None;
            }
            if ui.button("Register").clicked() {
                self.view = AuthView::Register;
                self.error = None;
            }
        });
    }
}

/// Call this to trigger a register action via JS handler
pub fn register_with_js(email: &str, password: &str, captcha_token: &str) {
    javascript_interop::send_action_message(
        "register",
        email,
        password,
        captcha_token,
    );
}

/// Call this to trigger a login action via JS handler
pub fn login_with_js(email: &str, password: &str, captcha_token: &str) {
    javascript_interop::send_action_message(
        "login",
        email,
        password,
        captcha_token,
    );
}

// Make AuthWidget public for use in other modules
