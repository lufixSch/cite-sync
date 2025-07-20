use serde_derive::{Deserialize, Serialize};

use eyre::eyre;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CiteSyncClientConfig {
    /// Base URL of the CiteSync Server
    pub api_url: String,

    /// List of bibtex files to sync
    pub bib_files: Vec<String>
}

impl Default for CiteSyncClientConfig {
    fn default() -> Self {
        Self {
            api_url: "http://127.0.0.1:3000".into(),
            bib_files: vec![]
        }
    }
}

impl CiteSyncClientConfig {
    /// Load CiteSync configuration
    pub fn load() -> Result<Self, eyre::Error> {
        let conf: CiteSyncClientConfig = confy::load("cite_sync", "config").map_err(|e| eyre!(format!("Unable to load Iris config: {}", e)))?;
        Ok(conf)
    }
}
