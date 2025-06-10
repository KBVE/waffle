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
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let app: TemplateApp = if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        } else {
            Default::default()
        };
        app
    }

    fn filter_repos<'a>(&self, query: &str) -> Vec<Repository> {
        let repos = self.db.get_repos();
        let repos = repos.lock().unwrap();
        repos.iter()
            .filter(|repo| {
                repo.full_name.as_ref().map_or(false, |name| name.to_lowercase().contains(&query.to_lowercase())) ||
                repo.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&query.to_lowercase()))
            })
            .cloned()
            .collect()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Define available languages here for easy extensibility
        // To add a new language, just add it to this array
        const LANGUAGE_OPTIONS: &[&str] = &["Rust", "Python", "Javascript"];
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Repository Sync & Search");
            ui.label("Select Language:");
            for &lang in LANGUAGE_OPTIONS.iter() {
                let selected = self.db.get_language() == lang;
                if ui.radio(selected, lang).clicked() {
                    self.db.set_language(lang);
                }
            }
            ui.separator();
            if ui.button("Sync").clicked() {
                self.db.sync_and_store();
            }
            if ui.button("Clear Cache").clicked() {
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
            let image_response = ui.add(egui::Image::new(logo_url).fit_to_exact_size(egui::Vec2::new(150.0, 50.0)));
            if image_response.clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab("https://kbve.com/application/rust/"));
            }
            ui.heading("Filtered Repositories");
            let filtered = self.filter_repos(&self.label);
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
        });
    }
}
