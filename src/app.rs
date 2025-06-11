// Use the utility module from crate root
use crate::utility::show_loading_spinner_custom;
use egui::Id;
use crate::db::github::{GithubDb, Repository};
use crate::db::idb::LANGUAGES;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    Init,
    Normal,
    Empty,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum LoadingState {
    Idle,
    Loading {
        kind: LoadingKind,
        message: String,
        pending_language: Option<String>,
    },
    Finishing {
        message: String,
    },
    Error(String),
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
    #[serde(skip)]
    app_state: AppState,
    #[serde(skip)]
    pending_app_state: Option<AppState>,
    #[serde(skip)]
    filtered_repos: Option<Vec<Repository>>,
    #[serde(skip)]
    filter_loading: bool,
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
            app_state: AppState::Init,
            pending_app_state: None,
            filtered_repos: None,
            filter_loading: false,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut app: TemplateApp = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        app.app_state = AppState::Init;
        app.db.load_from_indexeddb();
        app.load_filtered_repos_from_idb(&cc.egui_ctx);
        app
    }

    /// Load all repos for the current language from IndexedDB and set filtered_repos
    pub fn load_filtered_repos_from_idb(&mut self, ctx: &egui::Context) {
        let language = self.db.get_language();
        let ctx = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = match crate::db::idb::open_waffle_db().await {
                Ok(db_conn) => crate::db::idb::filter_repos_in_idb::<Repository>(&db_conn, &language, "").await.unwrap_or_default(),
                Err(_) => vec![],
            };
            ctx.data_mut(|d| d.insert_temp(Id::new("waffle_filtered_repos"), result));
            ctx.request_repaint();
        });
    }

    async fn check_empty_and_update_state_async(&mut self) {
        use crate::db::idb;
        use crate::db::github::Repository;
        if let Ok(db) = idb::open_waffle_db().await {
            let language = self.db.get_language();
            let key = format!("latest_{}", language.to_lowercase());
            match idb::get_repo::<Vec<Repository>>(&db, &language, &key).await {
                Ok(Some(repos)) => {
                    if repos.is_empty() {
                        self.pending_app_state = Some(AppState::Empty);
                    } else {
                        self.pending_app_state = Some(AppState::Normal);
                    }
                },
                Ok(None) => {
                    self.pending_app_state = Some(AppState::Empty);
                },
                Err(_) => {
                    self.pending_app_state = Some(AppState::Empty);
                }
            }
        } else {
            self.pending_app_state = Some(AppState::Empty);
        }
    }

    pub fn filter_repos_async(&mut self, query: &str, ctx: &egui::Context) {
        self.filter_loading = true;
        self.filtered_repos = None;
        let query = query.to_string();
        let language = self.db.get_language();
        let ctx = ctx.clone(); // keep this, as it is used in the async block
        wasm_bindgen_futures::spawn_local(async move {
            let result = match crate::db::idb::open_waffle_db().await {
                Ok(db_conn) => crate::db::idb::filter_repos_in_idb::<Repository>(&db_conn, &language, &query).await.unwrap_or_default(),
                Err(_) => vec![],
            };
            ctx.data_mut(|d| d.insert_temp(Id::new("waffle_filtered_repos"), result));
            ctx.request_repaint();
        });
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Toast timer logic
        if let Some(_) = self.toast_message {
            let dt = ctx.input(|i| i.unstable_dt);
            self.toast_timer -= dt;
            if self.toast_timer <= 0.0 {
                self.toast_message = None;
            }
        }
        // Loading state machine
        let mut error_to_trigger: Option<String> = None;
        match &mut self.loading_state {
            LoadingState::Idle => {},
            LoadingState::Loading { kind, message: _, pending_language } => {
                match kind {
                    LoadingKind::LanguageSwitch => {
                        if let Some(lang) = pending_language.take() {
                            self.db.set_language(&lang);
                            self.db.load_from_indexeddb();
                            // Immediately reload filtered repos for the new language
                            let language = self.db.get_language();
                            let ctx = ctx.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                let result = match crate::db::idb::open_waffle_db().await {
                                    Ok(db_conn) => crate::db::idb::filter_repos_in_idb::<Repository>(&db_conn, &language, "").await.unwrap_or_default(),
                                    Err(_) => vec![],
                                };
                                ctx.data_mut(|d| d.insert_temp(Id::new("waffle_filtered_repos"), result));
                                ctx.request_repaint();
                            });
                        }
                    },
                    LoadingKind::Sync => {
                        let db = self.db.clone();
                        let ctx = ctx.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            db.sync_and_store();
                            db.load_from_indexeddb();
                            // After sync, reload filtered repos
                            let language = db.get_language();
                            let result = match crate::db::idb::open_waffle_db().await {
                                Ok(db_conn) => crate::db::idb::filter_repos_in_idb::<Repository>(&db_conn, &language, "").await.unwrap_or_default(),
                                Err(_) => vec![],
                            };
                            ctx.data_mut(|d| d.insert_temp(Id::new("waffle_filtered_repos"), result));
                            ctx.request_repaint();
                        });
                    },
                    LoadingKind::ClearCache => {
                        self.db.clear_indexeddb();
                        self.db.load_from_indexeddb();
                    },
                }
                let message = match kind {
                    LoadingKind::LanguageSwitch => "Finishing language switch...",
                    LoadingKind::Sync => "Finishing sync...",
                    LoadingKind::ClearCache => "Finishing cache clear...",
                };
                self.loading_state = LoadingState::Finishing { message: message.to_string() };
            },
            LoadingState::Finishing { .. } => {
                let app_ptr = self as *mut TemplateApp;
                wasm_bindgen_futures::spawn_local(async move {
                    // SAFETY: This is safe because update() is single-threaded in egui/eframe
                    unsafe {
                        (*app_ptr).check_empty_and_update_state_async().await;
                    }
                });
                // Show toast only if there is data after sync
                if self.app_state == AppState::Normal {
                    self.toast_message = Some("Repositories synced!".to_owned());
                } else if self.app_state == AppState::Empty {
                    self.toast_message = Some("No repositories found. Please sync again.".to_owned());
                }
                self.loading_state = LoadingState::Idle;
            },
            LoadingState::Error(err) => {
                error_to_trigger = Some(err.clone());
                self.loading_state = LoadingState::Idle;
            },
        }
        if let Some(err) = error_to_trigger {
            self.toast_message = Some(format!("Error: {}", err));
        }
        // Apply pending_app_state if set
        if let Some(new_state) = self.pending_app_state.take() {
            self.app_state = new_state;
        }
        // Show loading spinner overlay if loading or finishing
        match &self.loading_state {
            LoadingState::Loading { message, .. } | LoadingState::Finishing { message } => {
                egui::Area::new(Id::new("loading_spinner_overlay"))
                    .fixed_pos((ctx.screen_rect().center().x - 100.0, ctx.screen_rect().center().y - 100.0))
                    .show(ctx, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(18.0, 18.0);
                        ui.add_space(48.0);
                        show_loading_spinner_custom(ui, message, Some(140.0));
                        ui.add_space(48.0);
                    });
            },
            _ => {}
        }
        // Show toast if present
        if self.toast_message.is_some() {
            egui::Area::new(Id::new("toast_area"))
                .fixed_pos((ctx.screen_rect().center().x - 150.0, ctx.screen_rect().bottom() - 60.0))
                .show(ctx, |ui| {
                    show_toast(self, ui);
                });
        }
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Repository Sync & Search");
            ui.label("Select Language:");
            let is_loading = matches!(self.loading_state, LoadingState::Loading { .. });
            for &lang in LANGUAGES.iter() {
                let selected = self.db.get_language() == lang;
                if ui.radio(selected, lang).clicked() && !is_loading {
                    self.loading_state = LoadingState::Loading {
                        kind: LoadingKind::LanguageSwitch,
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
                    message: "Syncing repositories...".to_owned(),
                    pending_language: None,
                };
            }
            if ui.button("Clear Cache").clicked() && !is_loading {
                self.loading_state = LoadingState::Loading {
                    kind: LoadingKind::ClearCache,
                    message: "Clearing cache...".to_owned(),
                    pending_language: None,
                };
            }
            ui.separator();
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.label);
            let filtered = self.filtered_repos.as_ref().cloned().unwrap_or_default();
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
            let filtered = self.filtered_repos.clone().unwrap_or_default();
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

        // Update filtered_repos from egui context temp data if available
        if let Some(repos) = ctx.data(|d| d.get_temp::<Vec<Repository>>(Id::new("waffle_filtered_repos"))) {
            let is_empty = repos.is_empty();
            self.filtered_repos = Some(repos.clone());
            // Set app_state based on whether there is data
            if is_empty {
                self.app_state = AppState::Empty;
            } else {
                self.app_state = AppState::Normal;
            }
        }

        // Show welcome dialog if DB is empty
        if self.app_state == AppState::Empty {
            egui::Window::new("Welcome to the Waffle!")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.heading("Welcome to the Waffle!");
                    ui.label("Please sync the languages you would like to see.");
                });
        }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.update(ctx, frame);
    }
}

// Change show_toast to a free function
fn show_toast(app: &TemplateApp, ui: &mut egui::Ui) {
    if let Some(msg) = &app.toast_message {
        let toast_height = 44.0;
        let toast_width = 360.0;
        let rect = egui::Rect::from_min_size(
            ui.max_rect().center_top() + egui::vec2(-toast_width / 2.0, 0.0),
            egui::vec2(toast_width, toast_height),
        );
        egui::Area::new(Id::new("waffle_toast"))
            .fixed_pos(rect.left_top())
            .show(ui.ctx(), |ui| {
                ui.group(|ui| {
                    ui.label(msg);
                });
            });
    }
}
