use ehttp::{self};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// WASM-only imports
#[cfg(target_arch = "wasm32")]
use ehttp::Mode;
#[cfg(target_arch = "wasm32")]
use idb::{Database, ObjectStoreParams, Factory, DatabaseEvent};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::JsValue;

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

pub struct GithubDb {
    repos: Arc<Mutex<Vec<Repository>>>,
    error: Arc<Mutex<Option<String>>>,
    is_loading: Arc<Mutex<bool>>,
}

impl GithubDb {
    pub fn new() -> Self {
        Self {
            repos: Arc::new(Mutex::new(Vec::new())),
            error: Arc::new(Mutex::new(None)),
            is_loading: Arc::new(Mutex::new(false)),
        }
    }

    pub fn get_repos(&self) -> Arc<Mutex<Vec<Repository>>> {
        Arc::clone(&self.repos)
    }

    pub fn get_error(&self) -> Arc<Mutex<Option<String>>> {
        Arc::clone(&self.error)
    }

    pub fn get_is_loading(&self) -> Arc<Mutex<bool>> {
        Arc::clone(&self.is_loading)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn sync_and_store(&self) {
        let repos = Arc::clone(&self.repos);
        let error = Arc::clone(&self.error);
        let is_loading = Arc::clone(&self.is_loading);
        *is_loading.lock().unwrap() = true;
        let request = ehttp::Request {
            method: String::from("GET"),
            url: String::from("https://api.github.com/search/repositories?q=language:rust&sort=stars&order=desc&per_page=100"),
            body: vec![],
            headers: ehttp::Headers::new(&[("User-Agent", "rust-egui-ehttp-app")]),
            #[cfg(target_arch = "wasm32")]
            mode: Mode::Cors,
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
                                // Only clone if we need to use in async closure
                                let filtered_repos_for_store = filtered_repos.clone();
                                *repos.lock().unwrap() = filtered_repos;
                                #[cfg(target_arch = "wasm32")]
                                {
                                    spawn_local(async move {
                                        if let Err(e) = Self::store_repos_in_indexeddb(&filtered_repos_for_store).await {
                                            *error.lock().unwrap() = Some(format!("Failed to store in IndexedDB: {}", e));
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

    // WASM: Load from IndexedDB
    #[cfg(target_arch = "wasm32")]
    pub fn load_from_indexeddb(&self) {
        let repos = Arc::clone(&self.repos);
        let error = Arc::clone(&self.error);
        spawn_local(async move {
            match Self::read_from_indexeddb().await {
                Ok(cached_repos) => {
                    *repos.lock().unwrap() = cached_repos;
                }
                Err(e) => {
                    *error.lock().unwrap() = Some(format!("Failed to load from IndexedDB: {}", e));
                }
            }
        });
    }

    // Native: No-op for load_from_indexeddb
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_indexeddb(&self) {
        // No-op on native
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
            #[cfg(target_arch = "wasm32")]
            {
                ehttp::Request {
                    method: String::from("GET"),
                    url: String::from("https://api.github.com/search/repositories?q=language:rust&sort=stars&order=desc&per_page=100"),
                    body: vec![],
                    headers: ehttp::Headers::new(&[("User-Agent", "rust-egui-ehttp-app")]),
                    mode: Mode::Cors,
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                ehttp::Request {
                    method: String::from("GET"),
                    url: String::from("https://api.github.com/search/repositories?q=language:rust&sort=stars&order=desc&per_page=100"),
                    body: vec![],
                    headers: ehttp::Headers::new(&[("User-Agent", "rust-egui-ehttp-app")]),
                }
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
                                *repos.lock().unwrap() = filtered_repos;

                                // Store in IndexedDB asynchronously (WASM only)
                                #[cfg(target_arch = "wasm32")]
                                {
                                    spawn_local(async move {
                                        if let Err(e) = Self::store_repos_in_indexeddb(&filtered_repos).await {
                                            *error.lock().unwrap() = Some(format!("Failed to store in IndexedDB: {}", e));
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

    // WASM: IndexedDB helpers
    #[cfg(target_arch = "wasm32")]
    async fn read_from_indexeddb() -> Result<Vec<Repository>, String> {
        let factory = Factory::new().map_err(|e| format!("Failed to create IndexedDB factory: {:?}", e))?;
        let db = factory
            .open("RustProjectsDB", None)
            .map_err(|e| format!("Failed to open database: {:?}", e))?
            .await
            .map_err(|e| format!("Failed to open database: {:?}", e))?;
        let tx = db
            .transaction(&["repositories"], idb::TransactionMode::ReadOnly)
            .map_err(|e| format!("Failed to create transaction: {:?}", e))?;
        let store = tx
            .object_store("repositories")
            .map_err(|e| format!("Failed to access object store: {:?}", e))?;
        let value = store
            .get(JsValue::from_str("latest"))
            .map_err(|e| format!("Failed to get data: {:?}", e))?
            .await
            .map_err(|e| format!("Failed to get data: {:?}", e))?;
        if let Some(value) = value {
            let json = serde_wasm_bindgen::from_value::<serde_json::Value>(value)
                .map_err(|e| format!("Failed to deserialize JSON: {:?}", e))?;
            let search_response = serde_json::from_value::<SearchResponse>(json)
                .map_err(|e| format!("Failed to parse search response: {:?}", e))?;
            Ok(search_response.items.into_iter().filter(|repo| repo.license.is_some()).collect())
        } else {
            Ok(vec![])
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn store_repos_in_indexeddb(repos: &Vec<Repository>) -> Result<(), String> {
        let factory = Factory::new().map_err(|e| format!("Failed to create IndexedDB factory: {:?}", e))?;
        let mut open_request = factory
            .open("RustProjectsDB", Some(1))
            .map_err(|e| format!("Failed to open database: {:?}", e))?;
        open_request.on_upgrade_needed(|event| {
            let db = event.database().expect("Failed to get database");
            let store_params = ObjectStoreParams::new();
            db.create_object_store("repositories", store_params)
                .expect("Failed to create object store");
        });
        let db = open_request
            .await
            .map_err(|e| format!("Failed to open database: {:?}", e))?;
        let tx = db
            .transaction(&["repositories"], idb::TransactionMode::ReadWrite)
            .map_err(|e| format!("Failed to create transaction: {:?}", e))?;
        let store = tx
            .object_store("repositories")
            .map_err(|e| format!("Failed to access object store: {:?}", e))?;
        store
            .put(
                &serde_wasm_bindgen::to_value(&SearchResponse {
                    total_count: Some(repos.len() as u64),
                    incomplete_results: Some(false),
                    items: repos.clone(),
                }).map_err(|e| format!("Failed to convert to JsValue: {:?}", e))?,
                Some(&JsValue::from_str("latest")),
            )
            .map_err(|e| format!("Failed to store data: {:?}", e))?;
        tx.await.map_err(|e| format!("Failed to complete transaction: {:?}", e))?;
        Ok(())
    }

    // Native: Stub implementations for IndexedDB helpers
    #[cfg(not(target_arch = "wasm32"))]
    async fn read_from_indexeddb() -> Result<Vec<Repository>, String> {
        Ok(vec![])
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn store_in_indexeddb(_response: &ehttp::Response) -> Result<(), String> {
        Ok(())
    }
}