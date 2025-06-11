use ehttp::{self};
#[cfg(target_arch = "wasm32")]
use ehttp::{Mode};
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::db::idb;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Owner {
    pub site_admin: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct License {
    pub node_id: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Repository {
    pub id: Option<u64>,
    pub node_id: Option<String>,
    pub name: Option<String>,
    pub full_name: Option<String>,
    pub private: Option<bool>,
    pub owner: Option<Owner>,
    pub html_url: Option<String>,
    pub description: Option<String>,
    pub fork: Option<bool>,
    pub url: Option<String>,
    pub forks_url: Option<String>,
    pub keys_url: Option<String>,
    pub collaborators_url: Option<String>,
    pub teams_url: Option<String>,
    pub hooks_url: Option<String>,
    pub issue_events_url: Option<String>,
    pub events_url: Option<String>,
    pub assignees_url: Option<String>,
    pub branches_url: Option<String>,
    pub tags_url: Option<String>,
    pub blobs_url: Option<String>,
    pub git_tags_url: Option<String>,
    pub git_refs_url: Option<String>,
    pub trees_url: Option<String>,
    pub statuses_url: Option<String>,
    pub languages_url: Option<String>,
    pub stargazers_url: Option<String>,
    pub contributors_url: Option<String>,
    pub subscribers_url: Option<String>,
    pub subscription_url: Option<String>,
    pub commits_url: Option<String>,
    pub git_commits_url: Option<String>,
    pub comments_url: Option<String>,
    pub issue_comment_url: Option<String>,
    pub contents_url: Option<String>,
    pub compare_url: Option<String>,
    pub merges_url: Option<String>,
    pub archive_url: Option<String>,
    pub downloads_url: Option<String>,
    pub issues_url: Option<String>,
    pub pulls_url: Option<String>,
    pub milestones_url: Option<String>,
    pub notifications_url: Option<String>,
    pub labels_url: Option<String>,
    pub releases_url: Option<String>,
    pub deployments_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub pushed_at: Option<String>,
    pub git_url: Option<String>,
    pub ssh_url: Option<String>,
    pub clone_url: Option<String>,
    pub svn_url: Option<String>,
    pub homepage: Option<String>,
    pub size: Option<u64>,
    pub stargazers_count: Option<u64>,
    pub watchers_count: Option<u64>,
    pub language: Option<String>,
    pub has_issues: Option<bool>,
    pub has_projects: Option<bool>,
    pub has_downloads: Option<bool>,
    pub has_wiki: Option<bool>,
    pub has_pages: Option<bool>,
    pub has_discussions: Option<bool>,
    pub forks_count: Option<u64>,
    pub mirror_url: Option<String>,
    pub archived: Option<bool>,
    pub disabled: Option<bool>,
    pub open_issues_count: Option<u64>,
    pub license: Option<License>,
    pub allow_forking: Option<bool>,
    pub is_template: Option<bool>,
    pub web_commit_signoff_required: Option<bool>,
    pub topics: Option<Vec<String>>,
    pub visibility: Option<String>,
    pub forks: Option<u64>,
    pub open_issues: Option<u64>,
    pub watchers: Option<u64>,
    pub default_branch: Option<String>,
    pub score: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
    pub total_count: Option<u64>,
    pub incomplete_results: Option<bool>,
    pub items: Vec<Repository>,
}

#[derive(Clone)]
pub struct GithubDb {
    repos: Arc<Mutex<Vec<Repository>>>,
    error: Arc<Mutex<Option<String>>>,
    is_loading: Arc<Mutex<bool>>,
    pub language: Arc<Mutex<String>>, // Add language selection
}

impl GithubDb {
    pub fn new() -> Self {
        Self {
            repos: Arc::new(Mutex::new(Vec::new())),
            error: Arc::new(Mutex::new(None)),
            is_loading: Arc::new(Mutex::new(false)),
            language: Arc::new(Mutex::new("Rust".to_string())),
        }
    }

    pub fn set_language(&self, lang: &str) {
        *self.language.lock().unwrap() = lang.to_string();
    }

    pub fn get_language(&self) -> String {
        self.language.lock().unwrap().clone()
    }

    pub fn clear_indexeddb(&self) {
        let language = self.get_language();
        let key = format!("latest_{}", language.to_lowercase());
        let error = Arc::clone(&self.error);
        wasm_bindgen_futures::spawn_local(async move {
            match idb::open_waffle_db_with_languages(&[&language]).await {
                Ok(db) => {
                    if let Err(e) = idb::delete_repo(&db, &language, &key).await {
                        *error.lock().unwrap() = Some(format!("Failed to clear IndexedDB: {}", e));
                    }
                }
                Err(e) => {
                    *error.lock().unwrap() = Some(format!("Failed to open IndexedDB: {}", e));
                }
            }
        });
    }

    pub fn sync_and_store(&self) {
        let repos = Arc::clone(&self.repos);
        let error = Arc::clone(&self.error);
        let is_loading = Arc::clone(&self.is_loading);
        let language = self.get_language();
        let key = format!("latest_{}", language.to_lowercase());
        *is_loading.lock().unwrap() = true;
        let request = ehttp::Request {
            method: String::from("GET"),
            url: format!("https://api.github.com/search/repositories?q=language:{}&sort=stars&order=desc&per_page=100", language),
            body: vec![],
            headers: ehttp::Headers::new(&[("User-Agent", "rust-egui-ehttp-app")]),
            #[cfg(target_arch = "wasm32")]
            mode:  Mode::Cors,
        };
        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            *is_loading.lock().unwrap() = false;
            match result {
                Ok(response) => {
                    if response.ok {
                        match response.json::<crate::db::github::SearchResponse>() {
                            Ok(search_response) => {
                                let filtered_repos = search_response
                                    .items
                                    .into_iter()
                                    .filter(|repo| repo.license.is_some())
                                    .collect::<Vec<_>>();
                                let filtered_repos_for_mutex = filtered_repos.clone();
                                let filtered_repos_for_async = filtered_repos_for_mutex.clone();
                                let key = key.clone();
                                *repos.lock().unwrap() = filtered_repos_for_mutex;
                                wasm_bindgen_futures::spawn_local(async move {
                                    match idb::open_waffle_db_with_languages(&[&language]).await {
                                        Ok(db) => {
                                            if let Err(e) = idb::add_repo(&db, &language, &key, &filtered_repos_for_async).await {
                                                *error.lock().unwrap() = Some(format!("Failed to store in IndexedDB: {}", e));
                                            }
                                        }
                                        Err(e) => {
                                            *error.lock().unwrap() = Some(format!("Failed to open IndexedDB: {}", e));
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                *error.lock().unwrap() = Some(format!("Failed to parse JSON: {}", e));
                            }
                        }
                    } else {
                        *error.lock().unwrap() = Some(format!("HTTP Error: {} - {}", response.status, response.status_text));
                    }
                }
                Err(e) => {
                    *error.lock().unwrap() = Some(format!("Request failed: {}", e));
                }
            }
        });
    }

    pub fn load_from_indexeddb(&self) {
        let repos = Arc::clone(&self.repos);
        let error = Arc::clone(&self.error);
        let language = self.get_language();
        let key = format!("latest_{}", language.to_lowercase());
        wasm_bindgen_futures::spawn_local(async move {
            match idb::open_waffle_db_with_languages(&[&language]).await {
                Ok(db) => {
                    match idb::get_repo::<Vec<crate::db::github::Repository>>(&db, &language, &key).await {
                        Ok(Some(cached_repos)) => {
                            *repos.lock().unwrap() = cached_repos;
                        }
                        Ok(None) => {
                            *repos.lock().unwrap() = vec![];
                        }
                        Err(e) => {
                            *error.lock().unwrap() = Some(format!("Failed to load from IndexedDB: {}", e));
                        }
                    }
                }
                Err(e) => {
                    *error.lock().unwrap() = Some(format!("Failed to open IndexedDB: {}", e));
                }
            }
        });
    }

    pub fn fetch_repositories(&self) {
        if *self.is_loading.lock().unwrap() {
            return;
        }
        *self.is_loading.lock().unwrap() = true;
        let repos = Arc::clone(&self.repos);
        let error = Arc::clone(&self.error);
        let is_loading = Arc::clone(&self.is_loading);

        // Build request, with mode only on wasm32
        let request = {
                ehttp::Request {
                    method: String::from("GET"),
                    url: String::from("https://api.github.com/search/repositories?q=language:rust&sort=stars&order=desc&per_page=100"),
                    body: vec![],
                    headers: ehttp::Headers::new(&[("User-Agent", "rust-egui-ehttp-app")]),
                    #[cfg(target_arch = "wasm32")]
                    mode:  Mode::Cors,
                }
        };

        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            *is_loading.lock().unwrap() = false;
            match result {
                Ok(response) => {
                    if response.ok {
                        match response.json::<SearchResponse>() {
                            Ok(search_response) => {
                                let filtered_repos = search_response
                                    .items
                                    .into_iter()
                                    .filter(|repo| repo.license.is_some())
                                    .collect::<Vec<_>>();

                                *repos.lock().unwrap() = filtered_repos.clone();

                                // Store in IndexedDB asynchronously (WASM only)
                                {
                                    let filtered_repos_for_async = filtered_repos.clone();
                                    let key = "latest_rust".to_string();
                                    let language = "rust".to_string();
                                    spawn_local(async move {
                                        // Use idb.rs public API for storing repos
                                        match idb::open_waffle_db_with_languages(&[&language]).await {
                                            Ok(db) => {
                                                if let Err(e) = idb::add_repo(&db, &language, &key, &filtered_repos_for_async).await {
                                                    *error.lock().unwrap() = Some(format!("Failed to store in IndexedDB: {}", e));
                                                }
                                            }
                                            Err(e) => {
                                                *error.lock().unwrap() = Some(format!("Failed to open IndexedDB: {}", e));
                                            }
                                        }
                                    });
                                }
                            }
                            Err(e) => {
                                *error.lock().unwrap() = Some(format!("Failed to parse JSON: {}", e));
                            }
                        }
                    } else {
                        *error.lock().unwrap() = Some(format!("HTTP Error: {} - {}", response.status, response.status_text));
                    }
                }
                Err(e) => {
                    *error.lock().unwrap() = Some(format!("Request failed: {}", e));
                }
            }
        });
    }

    pub fn get_repos(&self) -> Arc<Mutex<Vec<Repository>>> {
        Arc::clone(&self.repos)
    }

    pub fn is_loading(&self) -> bool {
        *self.is_loading.lock().unwrap()
    }
}