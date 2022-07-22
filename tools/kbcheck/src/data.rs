use crate::entry::Entry;
use miette::Diagnostic;
use std::collections::BTreeMap;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use thiserror;
use walkdir::{self, DirEntry, WalkDir};

pub type EntriesMap = BTreeMap<PathBuf, Entry>;

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum DataError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    JsonParseError(#[from] serde_json::Error),
    #[error(transparent)]
    YamlLoadError(#[from] serde_yaml::Error),
    #[error(transparent)]
    WalkDirError(#[from] walkdir::Error),
}

/// Read a path into a serde_json::Value
pub fn read_json(path: &Path) -> Result<serde_json::Value, DataError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

/// Iterator over DirEntry values corresponding to knowledge base data
/// files
pub fn iter_data_files(root_path: &Path) -> impl Iterator<Item = Result<DirEntry, DataError>> {
    let data_path = root_path.to_owned().join("data");

    WalkDir::new(&data_path)
        .into_iter()
        .filter(|maybe_entry| {
            if let Ok(entry) = maybe_entry {
                let path = entry.path();
                let ext = path.extension().unwrap_or_default();
                ext == "yml" || ext == "yaml"
            } else {
                // Always return errors
                true
            }
        })
        .map(|item| item.map_err(DataError::from))
}

/// Read a YAML file to an Entry
pub fn load_data_file(path: &Path) -> Result<Entry, DataError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_yaml::from_reader(reader)?)
}

/// Load all YAML files in the data directory and get a mapping
/// between PathBuf and Entry
pub fn load_all(root_path: &Path) -> Result<EntriesMap, DataError> {
    let mut entries = BTreeMap::new();

    for maybe_dir_entry in iter_data_files(root_path) {
        let dir_entry = maybe_dir_entry?;
        let path = dir_entry.path();
        let entry = load_data_file(path)?;
        entries.insert(path.to_path_buf(), entry);
    }
    Ok(entries)
}
