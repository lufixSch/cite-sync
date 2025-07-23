use biblatex::Bibliography;
use eyre::{Result, eyre};
use serde_json::Value;
use std::{collections::HashMap, fs, path::Path};

/// Loads the bibtex file at the given Bibliography
pub fn load_bibtex(path: &Path) -> Result<Bibliography> {
    let content = fs::read_to_string(path)?;

    Bibliography::parse(&content).map_err(|e| eyre!("Unable to parse bibliography: {}", e))
}

/// Loads the Json file at the given path as BetterBibTex Json
pub fn load_json(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Err(eyre!("Bibliography file does not exist"));
    }

    let file = fs::File::open(path)?;
    serde_json::from_reader(file).map_err(|e| eyre!("Unable to parse bibliography: {}", e))
}

/// Loads multiple bibliographies from a directory
/// Loads file from each subdirectory (depth=1)
///
/// # Returns
///
/// HashMap where each item is the parsed bibliopraphy and the key is the directory name
/// (collection)
pub fn load_dir(path: &Path, name: &String) -> Result<HashMap<String, Value>> {
    if !path.exists() {
        return Err(eyre!("Bibliography directory does not exist!"));
    }

    Ok(path
        .read_dir()?
        .flatten()
        .flat_map(|entry| {
            if entry.path().is_file() {
                return None;
            };

            Some::<(String, Value)>((
                entry.path().file_name()?.to_str()?.to_string(),
                load_json(entry.path().join(name).as_path()).ok()?,
            ))
        })
        .collect())
}
