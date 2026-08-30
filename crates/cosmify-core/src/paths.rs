use std::{
    env, fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde_json::Value;

use crate::{AppSettings, CosmifyError, Result};

#[derive(Debug, Clone)]
pub(crate) struct AppPaths {
    pub data_root: PathBuf,
    pub backups_root: PathBuf,
    pub cosmetics_root: PathBuf,
    pub settings_file: PathBuf,
    pub activity_file: PathBuf,
}

impl AppPaths {
    pub fn system() -> Result<Self> {
        let project = ProjectDirs::from("app", "Cosmify", "Cosmify")
            .ok_or(CosmifyError::AppDataUnavailable)?;
        let new_root = project.data_local_dir().to_path_buf();

        let legacy_root = ProjectDirs::from("dev", "PersonaForge", "PersonaForge")
            .map(|legacy| legacy.data_local_dir().to_path_buf());

        if !new_root.exists() {
            if let Some(legacy_root) = legacy_root.as_deref().filter(|path| path.exists()) {
                if migrate_legacy_data(legacy_root, &new_root).is_err() {
                    // Migration should never prevent the app from starting. If Windows has a
                    // transient lock on the old data directory, keep using the legacy root.
                    return Ok(Self::from_root(legacy_root.to_path_buf()));
                }
            }
        }

        Ok(Self::from_root(new_root))
    }

    pub fn from_root(root: PathBuf) -> Self {
        Self {
            backups_root: root.join("backups"),
            cosmetics_root: root.join("cosmetics"),
            settings_file: root.join("settings.json"),
            activity_file: root.join("activity.jsonl"),
            data_root: root,
        }
    }
}

fn migrate_legacy_data(old_root: &Path, new_root: &Path) -> Result<()> {
    if new_root.exists() || !old_root.exists() {
        return Ok(());
    }
    if let Some(parent) = new_root.parent() {
        fs::create_dir_all(parent).map_err(|error| CosmifyError::io(parent, error))?;
    }

    match fs::rename(old_root, new_root) {
        Ok(()) => {}
        Err(_) => {
            copy_directory(old_root, new_root)?;
        }
    }
    rewrite_migrated_json_paths(new_root, old_root, new_root)?;
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).map_err(|error| CosmifyError::io(destination, error))?;
    for entry in walkdir::WalkDir::new(source)
        .min_depth(1)
        .follow_links(false)
    {
        let entry = entry?;
        if entry.file_type().is_symlink() {
            continue;
        }
        let relative = entry.path().strip_prefix(source).map_err(|_| {
            CosmifyError::Internal("Failed to migrate application data".to_string())
        })?;
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|error| CosmifyError::io(&target, error))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|error| CosmifyError::io(parent, error))?;
            }
            fs::copy(entry.path(), &target).map_err(|error| CosmifyError::io(&target, error))?;
        }
    }
    Ok(())
}

fn rewrite_migrated_json_paths(root: &Path, old_root: &Path, new_root: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(root).min_depth(1).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|v| v.to_str()) != Some("json")
        {
            continue;
        }
        let bytes = match fs::read(entry.path()) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        let Ok(mut value) = serde_json::from_slice::<Value>(&bytes) else {
            continue;
        };
        let changed = rewrite_json_value(&mut value, old_root, new_root);
        if changed {
            fs::write(entry.path(), serde_json::to_vec_pretty(&value)?)
                .map_err(|error| CosmifyError::io(entry.path(), error))?;
        }
    }
    Ok(())
}

fn rewrite_json_value(value: &mut Value, old_root: &Path, new_root: &Path) -> bool {
    match value {
        Value::String(text) => {
            let candidate = PathBuf::from(text.as_str());
            if candidate.starts_with(old_root) {
                if let Ok(relative) = candidate.strip_prefix(old_root) {
                    *text = new_root.join(relative).to_string_lossy().into_owned();
                    return true;
                }
            }
            false
        }
        Value::Array(items) => {
            let mut changed = false;
            for item in items {
                changed |= rewrite_json_value(item, old_root, new_root);
            }
            changed
        }
        Value::Object(map) => {
            let mut changed = false;
            for item in map.values_mut() {
                changed |= rewrite_json_value(item, old_root, new_root);
            }
            changed
        }
        _ => false,
    }
}

pub(crate) fn resolve_premium_cache(settings: &AppSettings) -> Option<PathBuf> {
    if let Some(value) = settings.premium_cache_override.as_deref() {
        let path = PathBuf::from(value);
        if path.exists() {
            return Some(path);
        }
    }

    for variable in ["COSMIFY_PREMIUM_CACHE", "PERSONAFORGE_PREMIUM_CACHE"] {
        if let Ok(value) = env::var(variable) {
            let path = PathBuf::from(value);
            if path.exists() {
                return Some(path);
            }
        }
    }

    if let Ok(appdata) = env::var("APPDATA") {
        let gdk = Path::new(&appdata)
            .join("Minecraft Bedrock")
            .join("premium_cache");
        if gdk.exists() {
            return Some(gdk);
        }
    }

    if let Ok(local) = env::var("LOCALAPPDATA") {
        let uwp = Path::new(&local)
            .join("Packages")
            .join("Microsoft.MinecraftUWP_8wekyb3d8bbwe")
            .join("LocalState")
            .join("premium_cache");
        if uwp.exists() {
            return Some(uwp);
        }
    }

    None
}

pub(crate) fn skin_packs_path(premium_cache: &Path) -> PathBuf {
    premium_cache.join("skin_packs")
}
