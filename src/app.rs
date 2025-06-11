// Use the utility module from crate root
use crate::utility::show_loading_spinner_custom;
use egui::Id;
use crate::db::github::{GithubDb, Repository};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum LoadingState {
    Idle,
    Loading {
        kind: LoadingKind,
        timer: f32,
        message: String,
        pending_language: Option<String>,
    },
    Error,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LoadingKind {
    LanguageSwitch,
    Sync,
    ClearCache,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    #[serde(skip)]
    db: GithubDb,
    #[serde(skip)]
    value: f32,
    #[serde(skip)]
    logo_texture: Option<egui::TextureHandle>,
    #[serde(skip)]
    logo_loaded: bool,
    // Loader and toast state
    #[serde(skip)]
    loading_state: LoadingState,
    #[serde(skip)]
    toast_message: Option<String>,
    #[serde(skip)]
    toast_timer: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            db: GithubDb::new(),
            logo_texture: None,
            logo_loaded: false,
            loading_state: LoadingState::Idle,
            toast_message: None,
            toast_timer: 0.0,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let app: TemplateApp = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        // Attempt to load from IndexedDB on startup
        app.db.load_from_indexeddb();
        app
    }

    fn filter_repos<'a>(&self, query: &str) -> Vec<Repository> {
        let repos = self.db.get_repos();
        let repos = repos.lock().unwrap();
        let selected_language = self.db.get_language();
        repos.iter()
            .filter(|repo| {
                let matches_language = repo.language.as_ref().map_or(false, |lang| lang.eq_ignore_ascii_case(selected_language.as_str()));
                let matches_query = repo.full_name.as_ref().map_or(false, |name| name.to_lowercase().contains(&query.to_lowercase())) ||
                    repo.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&query.to_lowercase()));
                matches_language && matches_query
            })
            .cloned()
            .collect()
    }

    fn show_toast(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        if let Some(msg) = &self.toast_message {
            let toast_height = 44.0;
            let toast_width = 360.0;
            let rect = egui::Rect::from_min_size(
                ui.max_rect().center_top() + egui::vec2(-toast_width / 2.0, 0.0),
                egui::vec2(toast_width, toast_height),
            );
            let painter = ui.painter();
            // Fade out effect based on timer
            let alpha = (self.toast_timer / 2.5).clamp(0.0, 1.0);
            // Dark stone 950 background
            let bg_color = egui::Color32::from_rgba_unmultiplied(18, 24, 27, (240.0 * alpha) as u8);
            // Bright cyan font
            let font_color = egui::Color32::from_rgb(0, 255, 255);
            // Light purple border/accent
            let accent_color = egui::Color32::from_rgb(180, 140, 255);
            // Shadow
            let shadow_rect = rect.expand(10.0);
            painter.rect_filled(shadow_rect, 18.0, egui::Color32::from_rgba_unmultiplied(80, 0, 120, (40.0 * alpha) as u8));
            painter.rect_filled(rect, 12.0, bg_color);
            painter.rect_stroke(rect, 12.0, egui::epaint::Stroke::new(2.0, accent_color), egui::epaint::StrokeKind::Outside);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                msg,
                egui::TextStyle::Button.resolve(ui.style()),
                font_color,
            );
        }
    }

    fn trigger_toast(&mut self, message: &str) {
        self.toast_message = Some(message.to_owned());
        self.toast_timer = 2.5; // seconds
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Toast timer logic
        if let Some(_) = self.toast_message {
            let dt = ctx.input(|i| i.unstable_dt);
            self.toast_timer -= dt;
            if self.toast_timer <= 0.0 {
                self.toast_message = None;
            }
        }
        // Loading state machine
        match &mut self.loading_state {
            LoadingState::Idle => {},
            LoadingState::Loading { kind, timer, message: _, pending_language } => {
                let dt = ctx.input(|i| i.unstable_dt);
                *timer -= dt;
                if *timer <= 0.0 {
                    match kind {
                        LoadingKind::LanguageSwitch => {
                            if let Some(lang) = pending_language.take() {
                                self.db.set_language(&lang);
                                self.db.load_from_indexeddb();
                                self.trigger_toast(&format!("Switched to {}!", lang));
                            }
                        },
                        LoadingKind::Sync => {
                            self.db.sync_and_store();
                            self.trigger_toast("Repositories synced!");
                        },
                        LoadingKind::ClearCache => {
                            self.db.clear_indexeddb();
                            self.trigger_toast("Cache cleared!");
                        },
                    }
                    self.loading_state = LoadingState::Idle;
                }
            },
            LoadingState::Error => {
                // Could show an error toast or overlay here
            },
        }
        // Show loading spinner overlay if loading
        if let LoadingState::Loading { message, .. } = &self.loading_state {
            egui::Area::new(Id::new("loading_spinner_overlay"))
                .fixed_pos((ctx.screen_rect().center().x - 100.0, ctx.screen_rect().center().y - 100.0))
                .show(ctx, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(18.0, 18.0);
                    ui.add_space(48.0);
                    show_loading_spinner_custom(ui, message, Some(140.0));
                    ui.add_space(48.0);
                });
        }
        // Show toast if present
        if self.toast_message.is_some() {
            egui::Area::new(Id::new("toast_area"))
                .fixed_pos((ctx.screen_rect().center().x - 150.0, ctx.screen_rect().bottom() - 60.0))
                .show(ctx, |ui| {
                    self.show_toast(ctx, ui);
                });
        }
        // Define available languages here for easy extensibility
        // To add a new language, just add it to this array
        const LANGUAGE_OPTIONS: &[&str] = &["Rust", "Python", "Javascript"];
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Repository Sync & Search");
            ui.label("Select Language:");
            let is_loading = matches!(self.loading_state, LoadingState::Loading { .. });
            for &lang in LANGUAGE_OPTIONS.iter() {
                let selected = self.db.get_language() == lang;
                if ui.radio(selected, lang).clicked() && !is_loading {
                    self.loading_state = LoadingState::Loading {
                        kind: LoadingKind::LanguageSwitch,
                        timer: 2.0,
                        message: format!("Switching to {}...", lang),
                        pending_language: Some(lang.to_owned()),
                    };
                }
            }
            ui.separator();
            let is_loading = matches!(self.loading_state, LoadingState::Loading { .. });
            if ui.button("Sync").clicked() && !is_loading {
                self.loading_state = LoadingState::Loading {
                    kind: LoadingKind::Sync,
                    timer: 2.0,
                    message: "Syncing repositories...".to_owned(),
                    pending_language: None,
                };
            }
            if ui.button("Clear Cache").clicked() && !is_loading {
                self.loading_state = LoadingState::Loading {
                    kind: LoadingKind::ClearCache,
                    timer: 1.5,
                    message: "Clearing cache...".to_owned(),
                    pending_language: None,
                };
            }
            ui.separator();
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.label);
            let filtered = self.filter_repos(&self.label);
            ui.separator();
            ui.label(format!("Results: {}", filtered.len()));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            // --- Logo image loading and display using egui_extras loader system ---
            let logo_url = "https://kbve.com/assets/images/brand/letter_logo.png";
            let language = self.db.get_language().to_lowercase();
            let logo_link = format!("https://kbve.com/application/{}/", language);
            let image_response = ui.add(
                egui::Image::new(logo_url)
                    .fit_to_exact_size(egui::Vec2::new(150.0, 50.0))
                    .sense(egui::Sense::click())
            );
            if image_response.clicked() {
                ctx.open_url(egui::OpenUrl::new_tab(&logo_link));
            }
            ui.separator();
            ui.heading("Filtered Repositories");
            let filtered = self.filter_repos(&self.label);
            let current_language = self.db.get_language();
            if filtered.is_empty() {
                ui.label(format!("There is no data for {}, please sync.", current_language));
            } else {
                for repo in &filtered {
                    let name = repo.full_name.as_deref().unwrap_or("<unknown>");
                    let desc = repo.description.as_deref().unwrap_or("");
                    let stars = repo.stargazers_count.unwrap_or(0);
                    ui.horizontal(|ui| {
                        ui.label(format!("⭐ {}", stars));
                        ui.hyperlink_to(name, repo.html_url.as_deref().unwrap_or("#"));
                    });
                    if !desc.is_empty() {
                        ui.label(desc);
                    }
                    ui.separator();
                }
            }
        });

        // Show loading spinner if needed
    }
}
