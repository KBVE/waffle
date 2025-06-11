use crate::db::github::Repository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    Init,
    Normal,
    Empty,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct WaffleState {
    pub app_state: AppState,
    pub filtered_repos: Vec<Repository>,
}

impl WaffleState {
    pub fn new() -> Self {
        Self {
            app_state: AppState::Init,
            filtered_repos: Vec::new(),
        }
    }

    pub fn set_empty(&mut self) {
        self.app_state = AppState::Empty;
        self.filtered_repos.clear();
    }

    pub fn set_normal(&mut self, repos: Vec<Repository>) {
        if repos.is_empty() {
            self.set_empty();
        } else {
            self.app_state = AppState::Normal;
            self.filtered_repos = repos;
        }
    }

    pub fn set_error(&mut self, msg: String) {
        self.app_state = AppState::Error(msg);
        self.filtered_repos.clear();
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.app_state, AppState::Normal)
    }
}
