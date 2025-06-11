use idb::{Database, DatabaseEvent, Error, Factory, ObjectStoreParams, TransactionMode};
use serde::{Serialize, de::DeserializeOwned};
use wasm_bindgen::JsValue;

const DB_NAME: &str = "WaffleDB";
const DB_VERSION: u32 = 1;

pub async fn open_waffle_db_with_languages(languages: &[&str]) -> Result<Database, Error> {
    let factory = Factory::new()?;
    let mut open_request = factory.open(DB_NAME, Some(DB_VERSION))?;
    let langs = languages.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    open_request.on_upgrade_needed(move |event| {
        let db = event.database().unwrap();
        for lang in &langs {
            if db.store_names().iter().all(|n| n != lang) {
                let mut store_params = ObjectStoreParams::new();
                store_params.auto_increment(false);
                db.create_object_store(lang, store_params).unwrap();
            }
        }
    });
    open_request.await
}

pub async fn add_repo<T: Serialize>(db: &Database, language: &str, key: &str, value: &T) -> Result<(), Error> {
    let tx = db.transaction(&[language], TransactionMode::ReadWrite)?;
    let store = tx.object_store(language).unwrap();
    let js_value = serde_wasm_bindgen::to_value(value).unwrap();
    store.put(&js_value, Some(&JsValue::from_str(key)))?;
    tx.await?;
    Ok(())
}

pub async fn get_repo<T: DeserializeOwned>(db: &Database, language: &str, key: &str) -> Result<Option<T>, Error> {
    let tx = db.transaction(&[language], TransactionMode::ReadOnly)?;
    let store = tx.object_store(language).unwrap();
    let result = store.get(JsValue::from_str(key))?.await?;
    tx.await?;
    if let Some(js_value) = result {
        Ok(Some(serde_wasm_bindgen::from_value(js_value).unwrap()))
    } else {
        Ok(None)
    }
}

pub async fn delete_repo(db: &Database, language: &str, key: &str) -> Result<(), Error> {
    let tx = db.transaction(&[language], TransactionMode::ReadWrite)?;
    let store = tx.object_store(language).unwrap();
    store.delete(JsValue::from_str(key))?.await?;
    tx.await?;
    Ok(())
}

pub async fn get_all_repos<T: DeserializeOwned>(db: &Database, language: &str) -> Result<Vec<T>, Error> {
    let tx = db.transaction(&[language], TransactionMode::ReadOnly)?;
    let store = tx.object_store(language).unwrap();
    let mut results = Vec::new();
    let cursor = store.open_cursor(None, None)?;
    let mut cursor = cursor.await?;
    while let Some(cur) = cursor {
        let value: T = serde_wasm_bindgen::from_value(cur.value()?.clone()).unwrap();
        results.push(value);
        cursor = cur.next(None)?.await?;
    }
    tx.await?;
    Ok(results)
}

pub async fn filter_repos_in_idb<T: DeserializeOwned + Clone>(db: &Database, language: &str, query: &str) -> Result<Vec<T>, Error> {
    let tx = db.transaction(&[language], TransactionMode::ReadOnly)?;
    let store = tx.object_store(language).unwrap();
    let mut results = Vec::new();
    let cursor = store.open_cursor(None, None)?;
    let mut cursor = cursor.await?;
    while let Some(cur) = cursor {
        let value: T = serde_wasm_bindgen::from_value(cur.value()?.clone()).unwrap();
        // Filtering logic: assumes T is crate::db::github::Repository
        let repo = unsafe { &*(std::ptr::addr_of!(value) as *const crate::db::github::Repository) };
        let repo_lang = repo.language.as_deref().unwrap_or("");
        let repo_name = repo.full_name.as_deref().unwrap_or("");
        let repo_desc = repo.description.as_deref().unwrap_or("");
        let matches_language = repo_lang.eq_ignore_ascii_case(language);
        let matches_query = repo_name.to_lowercase().contains(&query.to_lowercase()) || repo_desc.to_lowercase().contains(&query.to_lowercase());
        if matches_language && matches_query {
            results.push(value.clone());
        }
        cursor = cur.next(None)?.await?;
    }
    tx.await?;
    Ok(results)
}
