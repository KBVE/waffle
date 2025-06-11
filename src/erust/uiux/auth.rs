// erust/uiux/auth.rs
use egui::{Context, Ui};
use crate::erust::uiux::hcaptcha;

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
        // Debug: Show the actual token if present (for troubleshooting only)
        if let Some(token) = hcaptcha::get_captcha_token() {
            ui.colored_label(egui::Color32::GREEN, format!("Captcha Solved: Token Set\nToken: {}", token));
            ui.ctx().request_repaint();
        } else {
            ui.label("Captcha required");
        }
        if ui.button("Login").clicked() {
            if self.email.is_empty() || self.password.is_empty() || hcaptcha::get_captcha_token().is_none() {
                self.error = Some("All fields and captcha are required".to_string());
            } else {
                let _token = hcaptcha::take_captcha_token();
                self.error = Some("(Stub) Would call Supabase login here".to_string());
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
        // Debug: Show the actual token if present (for troubleshooting only)
        if let Some(token) = hcaptcha::get_captcha_token() {
            ui.colored_label(egui::Color32::GREEN, format!("Captcha Solved: Token Set\nToken: {}", token));
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
                let _token = hcaptcha::take_captcha_token();
                self.error = Some("(Stub) Would call Supabase register here".to_string());
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

// Make AuthWidget public for use in other modules
