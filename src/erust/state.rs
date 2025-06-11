use crate::db::github::Repository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    Init,
    Empty,
    Syncing,
    Normal,
    Ready,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct WaffleState {
    pub app_state: AppState,
    pub filtered_repos: Vec<Repository>,
    pub log: String,
}

impl WaffleState {
    pub fn new() -> Self {
        Self {
            app_state: AppState::Init,
            filtered_repos: Vec::new(),
            log: String::new(),
        }
    }

    pub fn set_empty(&mut self) {
        self.app_state = AppState::Empty;
        self.filtered_repos.clear();
    }

    pub fn set_syncing(&mut self) {
        self.app_state = AppState::Syncing;
    }

    pub fn set_ready(&mut self, repos: Vec<Repository>) {
        if repos.is_empty() {
            self.set_empty();
        } else {
            self.app_state = AppState::Ready;
            self.filtered_repos = repos;
        }
    }

    pub fn set_error(&mut self, msg: String) {
        self.app_state = AppState::Error(msg.clone());
        self.log.push_str(&format!("Error: {}\n", msg));
        self.filtered_repos.clear();
    }

    pub fn log(&mut self, msg: &str) {
        self.log.push_str(msg);
        self.log.push('\n');
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.app_state, AppState::Ready)
    }
}
