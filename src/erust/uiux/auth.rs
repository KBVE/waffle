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
                    let error_ptr = std::rc::Rc::new(std::cell::RefCell::new(None));
                    let error_ptr_clone = error_ptr.clone();
                    let ctx = ui.ctx().clone();
                    // Show spinner while processing
                    self.error = Some("Processing registration...".to_string());
                    wasm_bindgen_futures::spawn_local(async move {
                        let resp = crate::erust::uiux::supabase::register(&email, &password, &token).await;
                        let resp_json: serde_json::Value = serde_json::from_str(&resp).unwrap_or_default();
                        let success = resp_json.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                        let error = resp_json.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());
                        ctx.request_repaint();
                        let mut err_mut = error_ptr_clone.borrow_mut();
                        if success {
                            *err_mut = Some("Registration successful!".to_string());
                        } else {
                            *err_mut = Some(error.unwrap_or_else(|| "Registration failed".to_string()));
                        }
                    });
                    self.error = error_ptr.borrow().clone();
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

// Make AuthWidget public for use in other modules
