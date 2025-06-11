use crate::db::github::Repository;
use crate::db::idb;
use egui::{Context, Id};

pub struct SearchWidget {
    pub query: String,
    pub results: Vec<Repository>,
    pub loading: bool,
}

impl SearchWidget {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            loading: false,
        }
    }

    pub fn search(&mut self, language: &str, ctx: &Context) {
        let query = self.query.clone();
        let language = language.to_string();
        let ctx = ctx.clone();
        self.loading = true;
        wasm_bindgen_futures::spawn_local(async move {
            let result = match idb::open_waffle_db().await {
                Ok(db_conn) => idb::filter_repos_in_idb::<Repository>(&db_conn, &language, &query).await.unwrap_or_default(),
                Err(_) => vec![],
            };
            ctx.data_mut(|d| d.insert_temp(Id::new("waffle_search_results"), result));
            ctx.request_repaint();
        });
    }

    pub fn update_results_from_ctx(&mut self, ctx: &Context) {
        if let Some(results) = ctx.data(|d| d.get_temp::<Vec<Repository>>(Id::new("waffle_search_results"))) {
            self.results = results.clone();
            self.loading = false;
        }
    }
}
