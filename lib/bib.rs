use biblatex::Bibliography;
use eyre::{eyre, Result};
use serde_json::Value;
use std::{fs, path::Path};

pub fn load_bibtex(path: &Path) -> Result<Bibliography> {
    let content = fs::read_to_string(path)?;
    
    Bibliography::parse(&content).map_err(|e| eyre!("Unable to parse bibliography: {}", e))
}

/// Loads the Json file at the given path as CSL Json
pub fn load_json(path: &Path) -> Result<Value> {
    let file = fs::File::open(path)?;
    serde_json::from_reader(file).map_err(|e| eyre!("Unable to parse bibliography: {}", e))
}
