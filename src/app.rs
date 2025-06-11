// Use the utility module from crate root
use crate::utility::show_loading_spinner;
use egui::Id;
use crate::db::github::{GithubDb, Repository};

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
    // Loading and toast state
    #[serde(skip)]
    is_loading: bool,
    #[serde(skip)]
    loading_message: String,
    #[serde(skip)]
    toast_message: Option<String>,
    #[serde(skip)]
    toast_timer: f32,
    #[serde(skip)]
    loading_timer: f32,
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
            is_loading: false,
            loading_message: String::new(),
            toast_message: None,
            toast_timer: 0.0,
            loading_timer: 0.0,
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
            let toast_height = 32.0;
            let toast_width = 300.0;
            let rect = egui::Rect::from_min_size(
                ui.max_rect().center_top() + egui::vec2(-toast_width / 2.0, 0.0),
                egui::vec2(toast_width, toast_height),
            );
            let painter = ui.painter();
            painter.rect_filled(rect, 8.0, egui::Color32::from_rgba_unmultiplied(30, 144, 255, 220));
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                msg,
                egui::TextStyle::Button.resolve(ui.style()),
                egui::Color32::WHITE,
            );
        }
    }

    fn trigger_toast(&mut self, message: &str) {
        self.toast_message = Some(message.to_owned());
        self.toast_timer = 2.5; // seconds
    }

    fn update_loading_state(&mut self) {
        // Use a public getter for is_loading
        self.is_loading = self.db.is_loading();
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
        // Loading timer logic (for fake spinner)
        if self.is_loading && self.loading_timer > 0.0 {
            let dt = ctx.input(|i| i.unstable_dt);
            self.loading_timer -= dt;
            if self.loading_timer <= 0.0 {
                self.is_loading = false;
                self.loading_timer = 0.0;
                // Actually switch language and load data here if needed
                if let Some(pending_language) = self.loading_message.strip_prefix("Switching to ").and_then(|s| s.strip_suffix("...")) {
                    self.db.set_language(pending_language.trim());
                    self.db.load_from_indexeddb();
                    self.trigger_toast(&self.loading_message.replace("Switching to ", "Switched to ").replace("...", "!"));
                } else {
                    self.trigger_toast(&self.loading_message.replace("...", "!"));
                }
            }
        }
        // Show loading spinner overlay if loading
        if self.is_loading {
            egui::Area::new(Id::new("loading_spinner_overlay"))
                .fixed_pos((ctx.screen_rect().center().x - 60.0, ctx.screen_rect().center().y - 60.0))
                .show(ctx, |ui| {
                    show_loading_spinner(ui, &self.loading_message, None);
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
            for &lang in LANGUAGE_OPTIONS.iter() {
                let selected = self.db.get_language() == lang;
                if ui.radio(selected, lang).clicked() && !self.is_loading {
                    self.loading_message = format!("Switching to {}...", lang);
                    self.is_loading = true;
                    self.loading_timer = 3.0;
                }
            }
            ui.separator();
            if ui.button("Sync").clicked() {
                self.loading_message = "Syncing repositories...".to_owned();
                self.is_loading = true;
                self.db.sync_and_store();
            }
            if ui.button("Clear Cache").clicked() {
                self.loading_message = "Clearing cache...".to_owned();
                self.is_loading = true;
                self.db.clear_indexeddb();
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
        if self.is_loading {
            // Removed redundant self.update_loading_state() call here
        }

        // Remove this line, it is invalid and causes errors:
        // self.show_toast(ctx, &mut ctx.layer_painter());
        // The toast is already shown via egui::Area above.
    }
}
