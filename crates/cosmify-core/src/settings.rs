use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{CosmifyError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub premium_cache_override: Option<String>,
    pub keys_directory: Option<String>,
    pub auto_backup: bool,
    pub require_minecraft_closed: bool,
    pub theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            premium_cache_override: None,
            keys_directory: None,
            auto_backup: true,
            require_minecraft_closed: true,
            theme: "system".to_string(),
        }
    }
}

pub(crate) fn read_settings(path: &Path) -> Result<AppSettings> {
    match fs::read_to_string(path) {
        Ok(value) => Ok(serde_json::from_str(&value)?),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(AppSettings::default()),
        Err(err) => Err(CosmifyError::io(path, err)),
    }
}

pub(crate) fn write_settings(path: &Path, settings: &AppSettings) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| CosmifyError::io(parent, e))?;
    }
    let tmp = path.with_extension("json.tmp");
    let content = serde_json::to_vec_pretty(settings)?;
    fs::write(&tmp, content).map_err(|e| CosmifyError::io(&tmp, e))?;
    let previous = path.with_extension("json.bak");
    if path.exists() {
        if previous.exists() {
            fs::remove_file(&previous).map_err(|e| CosmifyError::io(&previous, e))?;
        }
        fs::rename(path, &previous).map_err(|e| CosmifyError::io(path, e))?;
    }
    match fs::rename(&tmp, path) {
        Ok(()) => {
            if previous.exists() {
                let _ = fs::remove_file(&previous);
            }
            Ok(())
        }
        Err(error) => {
            if previous.exists() {
                let _ = fs::rename(&previous, path);
            }
            let _ = fs::remove_file(&tmp);
            Err(CosmifyError::io(path, error))
        }
    }
}
