use std::{collections::HashSet, fs, path::{Path, PathBuf}};

use serde_json::Value;
use walkdir::{DirEntry, WalkDir};

use crate::{CustomPackAnalysis, CosmifyError, Result, ValidationMessage};

const IGNORE_CUSTOM: &[&str] = &[
    "desktop.ini",
    "thumbs.db",
    ".ds_store",
    "folder.jpg",
    "manifest.json",
    "contents.json",
    "signatures.json",
    ".cosmify",
];

pub(crate) fn analyze_custom_pack(path: &Path) -> Result<CustomPackAnalysis> {
    if !path.is_dir() {
        return Err(CosmifyError::InvalidCustomPack(
            "Selected custom pack path is not a directory".to_string(),
        ));
    }
    let skins_path = path.join("skins.json");
    if !skins_path.is_file() {
        return Err(CosmifyError::InvalidCustomPack(
            "skins.json must exist at the root of the selected folder".to_string(),
        ));
    }

    let skins_value: Value = serde_json::from_slice(
        &fs::read(&skins_path).map_err(|error| CosmifyError::io(&skins_path, error))?,
    )
    .map_err(|error| CosmifyError::InvalidCustomPack(format!("skins.json: {error}")))?;

    let skin_count = skins_value
        .get("skins")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let mut file_count = 0;
    let mut png_count = 0;
    let mut geometry_count = 0;
    let mut warnings = Vec::new();
    let mut names = HashSet::new();

    for entry in custom_walk(path) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        file_count += 1;
        let rel = entry
            .path()
            .strip_prefix(path)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        let lower = rel.to_ascii_lowercase();
        if lower.ends_with(".png") {
            png_count += 1;
        }
        if lower.contains("geometry") && lower.ends_with(".json") {
            geometry_count += count_geometry_definitions(entry.path())?;
        }
        if !names.insert(lower) {
            warnings.push(ValidationMessage {
                code: "case-collision".to_string(),
                message: format!("Duplicate path ignoring case: {rel}"),
            });
        }
    }

    if skin_count == 0 {
        warnings.push(ValidationMessage {
            code: "no-skins".to_string(),
            message: "skins.json contains no skin entries".to_string(),
        });
    }
    if png_count == 0 {
        warnings.push(ValidationMessage {
            code: "no-png".to_string(),
            message: "No PNG textures were found".to_string(),
        });
    }

    Ok(CustomPackAnalysis {
        path: path.to_string_lossy().into_owned(),
        valid: true,
        file_count,
        png_count,
        geometry_count,
        skin_count,
        warnings,
    })
}

fn count_geometry_definitions(path: &Path) -> Result<usize> {
    let bytes = fs::read(path).map_err(|error| CosmifyError::io(path, error))?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
        CosmifyError::InvalidCustomPack(format!(
            "{}: {error}",
            path.file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("geometry JSON")
        ))
    })?;

    // Modern Bedrock geometry files store definitions in a minecraft:geometry array.
    if let Some(definitions) = value.get("minecraft:geometry").and_then(Value::as_array) {
        return Ok(definitions.len());
    }

    // Legacy / Persona geometry.json files store one definition per top-level
    // key, e.g. "geometry.BeachPartySkinPack.ClassyBeachwear".
    Ok(value
        .as_object()
        .map(|object| {
            object
                .keys()
                .filter(|key| key.to_ascii_lowercase().starts_with("geometry."))
                .count()
        })
        .unwrap_or(0))
}

pub(crate) fn is_ignored_custom_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    IGNORE_CUSTOM.iter().any(|ignore| lower == *ignore)
}

pub(crate) fn custom_files_to_copy(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in custom_walk(path) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let filename = entry.file_name().to_string_lossy();
        if is_ignored_custom_name(&filename) {
            continue;
        }
        files.push(entry.path().to_path_buf());
    }
    files.sort();
    Ok(files)
}

fn custom_walk(path: &Path) -> impl Iterator<Item = walkdir::Result<DirEntry>> {
    WalkDir::new(path)
        .min_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            entry.depth() == 0 || !is_ignored_custom_name(&entry.file_name().to_string_lossy())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cosmify_metadata_is_not_counted_or_copied_into_minecraft() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("skins.json"), br#"{"skins":[{}]}"#).unwrap();
        fs::write(temp.path().join("skin.png"), b"texture").unwrap();
        fs::create_dir_all(temp.path().join(".cosmify")).unwrap();
        fs::write(temp.path().join(".cosmify/pack.json"), b"{}").unwrap();
        fs::write(temp.path().join(".cosmify/icon.png"), b"artwork").unwrap();

        let analysis = analyze_custom_pack(temp.path()).unwrap();
        assert_eq!(analysis.file_count, 2);
        let copied = custom_files_to_copy(temp.path()).unwrap();
        assert_eq!(copied.len(), 2);
        assert!(copied.iter().any(|path| path.ends_with("skin.png")));
        assert!(copied.iter().any(|path| path.ends_with("skins.json")));
        assert!(copied.iter().all(|path| !path.to_string_lossy().contains(".cosmify")));
    }

    #[test]
    fn legacy_geometry_file_counts_definitions_not_files() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("skins.json"), br#"{"skins":[{},{}]}"#).unwrap();
        fs::write(temp.path().join("skin.png"), b"texture").unwrap();
        fs::write(
            temp.path().join("geometry.json"),
            br#"{
                "geometry.example.one": {"bones": []},
                "geometry.example.two": {"bones": []},
                "geometry.example.three": {"bones": []}
            }"#,
        )
        .unwrap();

        let analysis = analyze_custom_pack(temp.path()).unwrap();
        assert_eq!(analysis.geometry_count, 3);
    }

    #[test]
    fn modern_geometry_file_counts_array_entries() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("skins.json"), br#"{"skins":[{}]}"#).unwrap();
        fs::write(temp.path().join("skin.png"), b"texture").unwrap();
        fs::write(
            temp.path().join("geometry.custom.json"),
            br#"{"format_version":"1.12.0","minecraft:geometry":[{},{}]}"#,
        )
        .unwrap();

        let analysis = analyze_custom_pack(temp.path()).unwrap();
        assert_eq!(analysis.geometry_count, 2);
    }
}
