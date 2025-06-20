// Use the utility module from crate root
use crate::utility::show_loading_spinner_custom;
use egui::Id;
use crate::db::github::{GithubDb, Repository};
use crate::db::idb::LANGUAGES;
use crate::erust::uiux::search::SearchWidget;
use crate::erust::state::{AppState, WaffleState};
use crate::erust::uiux::auth::AuthWidget;
use crate::erust::uiux::user::User;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq)]
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
    waffle_state: WaffleState,
    #[serde(skip)]
    filtered_repos: Option<Vec<Repository>>,
    #[serde(skip)]
    filter_loading: bool,
    #[serde(skip)]
    search_widget: Option<SearchWidget>,
    #[serde(skip)]
    auth_widget: AuthWidget,
    #[serde(skip)]
    user: User,
    #[serde(skip)]
    show_auth_window: bool, // Track if auth window should be shown
    #[serde(skip)]
    show_welcome: bool, // Track if welcome window should be shown
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
            waffle_state: WaffleState::new(),
            filtered_repos: None,
            filter_loading: false,
            search_widget: Some(SearchWidget::new()),
            auth_widget: AuthWidget::new(false),
            user: User::default(),
            show_auth_window: false,
            show_welcome: true, // Show welcome window by default
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
        // --- Call JSRust to request user info when app is ready ---
        crate::erust::uiux::javascript_interop::request_user_from_js();
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
        if let Some(widget) = &mut self.search_widget {
            widget.query = query.to_string();
            widget.search(&self.db.get_language(), ctx);
        }
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
            // --- Show app state at the bottom ---
            ui.separator();
            ui.label(format!("App State: {:?}", self.waffle_state.app_state));
            if !self.waffle_state.log.is_empty() {
                ui.separator();
                ui.label("App Log:");
                ui.label(&self.waffle_state.log);
            }
            // Add Login/Register buttons if not authenticated
            if !self.user.is_logged_in() {
                if ui.button("Login").clicked() {
                    self.auth_widget.view = crate::erust::uiux::auth::AuthView::Login;
                    self.show_auth_window = true;
                }
                if ui.button("Register").clicked() {
                    self.auth_widget.view = crate::erust::uiux::auth::AuthView::Register;
                    self.show_auth_window = true;
                }
            }
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
            // --- Show welcome and email if authenticated ---
            if self.user.is_logged_in() {
                ui.heading("Welcome,");
                if let Some(email) = &self.user.email {
                    ui.label(format!("Email: {}", email));
                }
                ui.separator();
            }
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
        if let Some(widget) = &mut self.search_widget {
            widget.update_results_from_ctx(ctx);
            // Use WaffleState to manage app state
            if !widget.results.is_empty() {
                self.waffle_state.set_ready(widget.results.clone());
            } else {
                self.waffle_state.set_empty();
            }
            self.filtered_repos = Some(self.waffle_state.filtered_repos.clone());
            self.app_state = self.waffle_state.app_state.clone();
        }

        // Show welcome dialog if DB is empty
        if self.app_state == AppState::Empty && self.show_welcome {
            egui::Window::new("Welcome to the Waffle!")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .open(&mut self.show_welcome)
                .show(ctx, |ui| {
                    ui.heading("Welcome to the Waffle!");
                    ui.label("Please sync the languages you would like to see.");
                });
        }

        // --- Always-visible bottom panel with Logout button ---
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Logout").clicked() {
                    crate::erust::uiux::javascript_interop::send_action_message(
                        "logout",
                        "",
                        "",
                        "",
                    );
                    self.user = User::default(); // Reset user to blank/default
                    self.toast_message = Some("Sent logout request to JS".to_string());
                    ctx.request_repaint(); // Ensure UI updates
                }
                ui.label("Waffle v0.1.0");
            });
        });

        // --- Authentication Widget (Login/Register) ---
        // Only show the authentication window if user is not logged in and show_auth_window is true
        if !self.user.is_logged_in() && self.show_auth_window {
            egui::Window::new("Authentication")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0.0, 40.0))
                .open(&mut self.show_auth_window) // Adds the X close button
                .show(ctx, |ui| {
                    self.auth_widget.show(ctx, ui);
                });
        }

        // --- Check for new Supabase user and update state ---
        if let Some(new_user) = crate::erust::uiux::javascript_interop::take_supabase_user() {
            if new_user.is_authenticated {
                self.user = new_user;
                ctx.request_repaint();
            }
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
