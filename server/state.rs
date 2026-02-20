use libcitesync::{
    bib,
    item::{Bibliography, ResearchItem},
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::SystemTime};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CiteSyncPaths {
    pub data_dir: String,
    pub bib_name: String,
}

pub struct CollectionMetadata {
    pub json_path: String,
    pub storage_path: String,
    pub last_modified: SystemTime,
    pub collections_structure: Option<Value>,
}

pub struct CachedBibliography {
    pub research_items: HashMap<u64, ResearchItem>,
    pub authors: HashMap<String, (String, Vec<u64>)>,
    pub tags: HashMap<String, Vec<u64>>,
    pub collections_metadata: HashMap<String, CollectionMetadata>,
}

impl CachedBibliography {
    pub fn new() -> Self {
        Self {
            research_items: HashMap::new(),
            authors: HashMap::new(),
            tags: HashMap::new(),
            collections_metadata: HashMap::new(),
        }
    }

    pub fn build_indexes(&mut self) {
        for (item_id, item) in &self.research_items {
            for author in &item.authors {
                let author_id = author.get_id().to_string();
                self.authors
                    .entry(author_id.clone())
                    .or_insert_with(|| (author.get_name().to_string(), vec![]))
                    .1
                    .push(*item_id);
            }
            for tag in &item.tags {
                self.tags.entry(tag.clone()).or_default().push(*item_id);
            }
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub paths: CiteSyncPaths,
    pub bibliography: Arc<RwLock<CachedBibliography>>,
}

impl AppState {
    pub async fn new(paths: CiteSyncPaths) -> eyre::Result<Self> {
        let state = Self {
            paths,
            bibliography: Arc::new(RwLock::new(CachedBibliography::new())),
        };

        state.reload_bibliography().await?;

        Ok(state)
    }

    pub async fn get_bibliography(&self) -> eyre::Result<Arc<RwLock<CachedBibliography>>> {
        if self.should_reload().await? {
            self.reload_bibliography().await?;
        }
        Ok(self.bibliography.clone())
    }

    async fn should_reload(&self) -> eyre::Result<bool> {
        let cache = self.bibliography.read().await;

        if cache.collections_metadata.is_empty() {
            return Ok(true);
        }

        for metadata in cache.collections_metadata.values() {
            if let Ok(modified) = std::fs::metadata(&metadata.json_path)?.modified()
                && modified != metadata.last_modified
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn reload_bibliography(&self) -> eyre::Result<()> {
        let path = std::path::Path::new(&self.paths.data_dir);
        let bibliographies = bib::load_dir(path, &self.paths.bib_name)?;

        let mut new_cache = CachedBibliography::new();

        for (collection_name, json_value) in bibliographies {
            let collection_path = path.join(&collection_name);
            let json_path = collection_path.join(&self.paths.bib_name);
            let storage_path = collection_path.join("storage");

            let last_modified = std::fs::metadata(&json_path)?.modified()?;

            let collections_structure = json_value.get("collections").cloned();

            new_cache.collections_metadata.insert(
                collection_name.clone(),
                CollectionMetadata {
                    json_path: json_path.to_string_lossy().to_string(),
                    storage_path: storage_path.to_string_lossy().to_string(),
                    last_modified,
                    collections_structure,
                },
            );

            if let Some(items) = ResearchItem::from_json(&json_value, &collection_name) {
                for (item_id, item) in items {
                    new_cache.research_items.insert(item_id, item);
                }
            }
        }

        new_cache.build_indexes();

        *self.bibliography.write().await = new_cache;

        Ok(())
    }
}
