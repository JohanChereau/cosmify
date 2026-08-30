use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use uuid::Uuid;
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::{
    validation::analyze_custom_pack, CosmeticPack, CosmeticPackMetadata, CosmifyError,
    ImportCosmeticPackRequest, Result, UpdateCosmeticPackRequest, DEFAULT_BANNER_GRADIENT_END,
    DEFAULT_BANNER_GRADIENT_START,
};

pub const COSMIFY_DIR: &str = ".cosmify";
pub const COSMIFY_MANIFEST: &str = "pack.json";
const MAX_ARCHIVE_ENTRIES: usize = 20_000;
const MAX_ARCHIVE_UNCOMPRESSED_BYTES: u64 = 768 * 1024 * 1024;
const MAX_ICON_BYTES: u64 = 1024 * 1024;

#[derive(Default)]
struct SourcePackDefaults {
    name: Option<String>,
    uuid: Option<String>,
    version: Option<String>,
}

fn discover_pack_defaults(path: &Path) -> SourcePackDefaults {
    let mut defaults = SourcePackDefaults::default();

    let manifest_path = path.join("manifest.json");
    if let Ok(bytes) = fs::read(&manifest_path) {
        if let Ok(manifest) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            defaults.name = manifest
                .pointer("/header/name")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);

            defaults.uuid = manifest
                .pointer("/header/uuid")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| Uuid::parse_str(value).is_ok())
                .map(str::to_string);

            defaults.version = manifest
                .pointer("/header/version")
                .and_then(manifest_version_string);
        }
    }

    // Some Persona packs do not have a useful manifest name. skins.json carries
    // the same user-facing identity in localization_name / serialize_name, so it
    // makes a safe secondary source without changing Minecraft pack contents.
    if defaults.name.is_none() {
        let skins_path = path.join("skins.json");
        if let Ok(bytes) = fs::read(&skins_path) {
            if let Ok(skins) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                defaults.name = ["localization_name", "serialize_name"]
                    .into_iter()
                    .find_map(|key| {
                        skins
                            .get(key)
                            .and_then(serde_json::Value::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(str::to_string)
                    });
            }
        }
    }

    defaults
}

fn manifest_version_string(value: &serde_json::Value) -> Option<String> {
    let parts = value
        .as_array()?
        .iter()
        .map(|part| part.as_u64().map(|value| value.to_string()))
        .collect::<Option<Vec<_>>>()?;
    (!parts.is_empty()).then(|| parts.join("."))
}

pub(crate) fn list_cosmetic_packs(root: &Path) -> Result<Vec<CosmeticPack>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut packs = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| CosmifyError::io(root, error))? {
        let entry = entry.map_err(|error| CosmifyError::io(root, error))?;
        if !entry.path().is_dir() {
            continue;
        }
        if let Ok(pack) = load_cosmetic_pack(&entry.path()) {
            packs.push(pack);
        }
    }
    packs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(packs)
}

pub(crate) fn import_cosmetic_pack(
    root: &Path,
    request: ImportCosmeticPackRequest,
) -> Result<CosmeticPack> {
    let source = PathBuf::from(&request.source_path);
    if !source.exists() {
        return Err(CosmifyError::InvalidCosmeticPack(
            "The selected source does not exist".to_string(),
        ));
    }

    let staging = tempfile::Builder::new()
        .prefix("cosmify-import-")
        .tempdir()
        .map_err(|error| CosmifyError::io(std::env::temp_dir(), error))?;

    let pack_source = if source.is_dir() {
        source.clone()
    } else if source.is_file() {
        let extracted = staging.path().join("archive");
        extract_cosmetic_archive(&source, &extracted)?;
        resolve_pack_root(&extracted)?
    } else {
        return Err(CosmifyError::InvalidCosmeticPack(
            "The selected source must be a folder or ZIP-compatible archive".to_string(),
        ));
    };

    let analysis = analyze_custom_pack(&pack_source)?;
    if !analysis.valid {
        return Err(CosmifyError::InvalidCosmeticPack(
            "The selected pack did not pass validation".to_string(),
        ));
    }

    fs::create_dir_all(root).map_err(|error| CosmifyError::io(root, error))?;

    let embedded = if metadata_path(&pack_source).is_file() {
        Some(read_metadata(&pack_source)?)
    } else {
        None
    };
    let id = unique_internal_id(root, embedded.as_ref().map(|value| value.id.as_str()));
    let target = root.join(&id);
    copy_pack_directory(&pack_source, &target)?;

    let now = Utc::now().to_rfc3339();
    let source_defaults = discover_pack_defaults(&pack_source);
    let fallback_name = source_defaults.name.clone().unwrap_or_else(|| {
        source
            .file_stem()
            .or_else(|| pack_source.file_name())
            .and_then(|value| value.to_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("Cosmetic pack")
            .to_string()
    });

    let default_uuid = source_defaults
        .uuid
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let default_version = source_defaults
        .version
        .clone()
        .unwrap_or_else(|| "1.0.0".to_string());

    let mut metadata = embedded.unwrap_or_else(|| CosmeticPackMetadata {
        schema_version: 1,
        id: id.clone(),
        name: fallback_name.clone(),
        description: String::new(),
        author: String::new(),
        uuid: default_uuid.clone(),
        version: default_version.clone(),
        icon: None,
        banner_gradient_start: DEFAULT_BANNER_GRADIENT_START.to_string(),
        banner_gradient_end: DEFAULT_BANNER_GRADIENT_END.to_string(),
        created_at: now.clone(),
        updated_at: now.clone(),
    });

    metadata.schema_version = 1;
    metadata.id = id;
    if metadata.name.trim().is_empty() {
        metadata.name = fallback_name;
    } else {
        metadata.name = metadata.name.trim().to_string();
    }
    metadata.description = metadata.description.trim().to_string();
    metadata.author = metadata.author.trim().to_string();
    if metadata.version.trim().is_empty() {
        metadata.version = default_version;
    } else {
        metadata.version = metadata.version.trim().to_string();
    }
    if Uuid::parse_str(&metadata.uuid).is_err() {
        metadata.uuid = default_uuid;
    }
    if metadata.created_at.trim().is_empty() {
        metadata.created_at = now.clone();
    }
    metadata.updated_at = now;

    copy_embedded_icon(&pack_source, &target, &mut metadata)?;
    adopt_existing_icon(&target, &mut metadata)?;
    write_metadata(&target, &metadata)?;
    load_cosmetic_pack(&target)
}

pub(crate) fn update_cosmetic_pack(
    root: &Path,
    request: UpdateCosmeticPackRequest,
) -> Result<CosmeticPack> {
    let path = pack_path(root, &request.id)?;
    let mut metadata = read_metadata(&path)?;

    let name = request.name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(CosmifyError::InvalidCosmeticPack(
            "Name must contain between 1 and 80 characters".to_string(),
        ));
    }
    if request.description.chars().count() > 280 {
        return Err(CosmifyError::InvalidCosmeticPack(
            "Description cannot exceed 280 characters".to_string(),
        ));
    }
    if request.author.chars().count() > 80 {
        return Err(CosmifyError::InvalidCosmeticPack(
            "Author cannot exceed 80 characters".to_string(),
        ));
    }
    Uuid::parse_str(request.uuid.trim())
        .map_err(|_| CosmifyError::InvalidCosmeticPack("UUID must be a valid UUID".to_string()))?;
    if request.version.trim().is_empty() || request.version.chars().count() > 32 {
        return Err(CosmifyError::InvalidCosmeticPack(
            "Version must contain between 1 and 32 characters".to_string(),
        ));
    }

    metadata.name = name.to_string();
    metadata.description = request.description.trim().to_string();
    metadata.author = request.author.trim().to_string();
    metadata.uuid = request.uuid.trim().to_string();
    metadata.version = request.version.trim().to_string();
    metadata.banner_gradient_start = normalize_hex_color(
        &request.banner_gradient_start,
        "Banner gradient start color",
    )?;
    metadata.banner_gradient_end =
        normalize_hex_color(&request.banner_gradient_end, "Banner gradient end color")?;
    metadata.updated_at = Utc::now().to_rfc3339();
    write_metadata(&path, &metadata)?;
    load_cosmetic_pack(&path)
}

pub(crate) fn set_cosmetic_pack_icon(root: &Path, id: &str, source: &Path) -> Result<CosmeticPack> {
    let path = pack_path(root, id)?;
    if !source.is_file() {
        return Err(CosmifyError::InvalidCosmeticPack(
            "Icon must point to an existing image file".to_string(),
        ));
    }
    let metadata_fs = fs::metadata(source).map_err(|error| CosmifyError::io(source, error))?;
    if metadata_fs.len() > MAX_ICON_BYTES {
        return Err(CosmifyError::InvalidCosmeticPack(
            "Icon is too large (maximum 1 MiB)".to_string(),
        ));
    }
    let extension = validated_image_extension(source)?;
    let meta_dir = path.join(COSMIFY_DIR);
    fs::create_dir_all(&meta_dir).map_err(|error| CosmifyError::io(&meta_dir, error))?;
    remove_managed_icons(&meta_dir)?;
    let filename = format!("icon.{extension}");
    let target = meta_dir.join(&filename);
    fs::copy(source, &target).map_err(|error| CosmifyError::io(&target, error))?;

    let mut metadata = read_metadata(&path)?;
    metadata.icon = Some(filename);
    metadata.updated_at = Utc::now().to_rfc3339();
    write_metadata(&path, &metadata)?;
    load_cosmetic_pack(&path)
}

pub(crate) fn delete_cosmetic_pack(root: &Path, id: &str) -> Result<()> {
    let path = pack_path(root, id)?;
    fs::remove_dir_all(&path).map_err(|error| CosmifyError::io(&path, error))
}

pub(crate) fn load_cosmetic_pack(path: &Path) -> Result<CosmeticPack> {
    let metadata = read_metadata(path)?;
    if path.file_name().and_then(|value| value.to_str()) != Some(metadata.id.as_str()) {
        return Err(CosmifyError::InvalidCosmeticPack(
            "Managed pack metadata id does not match its library directory".to_string(),
        ));
    }
    let analysis = analyze_custom_pack(path)?;
    let icon_data_url = read_icon_data_url(path, &metadata)?;
    Ok(CosmeticPack {
        id: metadata.id,
        name: metadata.name,
        description: metadata.description,
        author: metadata.author,
        uuid: metadata.uuid,
        version: metadata.version,
        path: path.to_string_lossy().into_owned(),
        icon_data_url,
        banner_gradient_start: metadata.banner_gradient_start,
        banner_gradient_end: metadata.banner_gradient_end,
        created_at: metadata.created_at,
        updated_at: metadata.updated_at,
        analysis,
    })
}

fn pack_path(root: &Path, id: &str) -> Result<PathBuf> {
    Uuid::parse_str(id)
        .map_err(|_| CosmifyError::InvalidCosmeticPack("Invalid managed pack id".to_string()))?;
    let candidate = root.join(id);
    if !candidate.is_dir() {
        return Err(CosmifyError::CosmeticPackNotFound(id.to_string()));
    }
    let canonical_root = fs::canonicalize(root).map_err(|error| CosmifyError::io(root, error))?;
    let canonical_candidate =
        fs::canonicalize(&candidate).map_err(|error| CosmifyError::io(&candidate, error))?;
    if !canonical_candidate.starts_with(canonical_root) {
        return Err(CosmifyError::InvalidCosmeticPack(
            "Managed pack path escapes the Cosmify library".to_string(),
        ));
    }
    Ok(candidate)
}

fn unique_internal_id(root: &Path, preferred: Option<&str>) -> String {
    if let Some(preferred) = preferred {
        if Uuid::parse_str(preferred).is_ok() && !root.join(preferred).exists() {
            return preferred.to_string();
        }
    }
    loop {
        let id = Uuid::new_v4().to_string();
        if !root.join(&id).exists() {
            return id;
        }
    }
}

fn resolve_pack_root(extracted: &Path) -> Result<PathBuf> {
    if extracted.join("skins.json").is_file() {
        return Ok(extracted.to_path_buf());
    }
    let directories = fs::read_dir(extracted)
        .map_err(|error| CosmifyError::io(extracted, error))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    if directories.len() == 1 && directories[0].join("skins.json").is_file() {
        return Ok(directories[0].clone());
    }
    Err(CosmifyError::InvalidCosmeticPack(
        "Archive must contain skins.json at its root (or inside a single top-level folder)"
            .to_string(),
    ))
}

fn extract_cosmetic_archive(source: &Path, destination: &Path) -> Result<()> {
    let file = File::open(source).map_err(|error| CosmifyError::io(source, error))?;
    let mut archive = ZipArchive::new(file)?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(CosmifyError::InvalidCosmeticPack(format!(
            "Archive contains too many entries (maximum {MAX_ARCHIVE_ENTRIES})"
        )));
    }
    fs::create_dir_all(destination).map_err(|error| CosmifyError::io(destination, error))?;

    let mut expanded = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(enclosed) = entry.enclosed_name() else {
            return Err(CosmifyError::UnsafeArchiveEntry(entry.name().to_string()));
        };
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(CosmifyError::UnsafeArchiveEntry(entry.name().to_string()));
        }
        expanded = expanded.saturating_add(entry.size());
        if expanded > MAX_ARCHIVE_UNCOMPRESSED_BYTES {
            return Err(CosmifyError::InvalidCosmeticPack(
                "Archive is too large when extracted (maximum 768 MiB)".to_string(),
            ));
        }
        let target = destination.join(enclosed);
        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(|error| CosmifyError::io(&target, error))?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| CosmifyError::io(parent, error))?;
        }
        let mut output = File::create(&target).map_err(|error| CosmifyError::io(&target, error))?;
        std::io::copy(&mut entry, &mut output).map_err(|error| CosmifyError::io(&target, error))?;
    }
    Ok(())
}

fn copy_pack_directory(source: &Path, destination: &Path) -> Result<()> {
    if destination.exists() {
        return Err(CosmifyError::InvalidCosmeticPack(
            "A managed pack with this internal id already exists".to_string(),
        ));
    }
    fs::create_dir_all(destination).map_err(|error| CosmifyError::io(destination, error))?;
    for entry in WalkDir::new(source)
        .min_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            entry.depth() == 0 || entry.file_name().to_string_lossy() != COSMIFY_DIR
        })
    {
        let entry = entry?;
        if entry.file_type().is_symlink() {
            return Err(CosmifyError::InvalidCosmeticPack(format!(
                "Symbolic links are not allowed in cosmetic packs: {}",
                entry.path().display()
            )));
        }
        let relative = entry.path().strip_prefix(source).map_err(|_| {
            CosmifyError::Internal("Failed to resolve imported pack path".to_string())
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

fn metadata_path(pack: &Path) -> PathBuf {
    pack.join(COSMIFY_DIR).join(COSMIFY_MANIFEST)
}

fn read_metadata(pack: &Path) -> Result<CosmeticPackMetadata> {
    let path = metadata_path(pack);
    let bytes = fs::read(&path).map_err(|error| CosmifyError::io(&path, error))?;
    let metadata: CosmeticPackMetadata = serde_json::from_slice(&bytes)?;
    if metadata.schema_version != 1 {
        return Err(CosmifyError::InvalidCosmeticPack(format!(
            "Unsupported Cosmify pack schema version {}",
            metadata.schema_version
        )));
    }
    if metadata.name.chars().count() > 80
        || metadata.description.chars().count() > 280
        || metadata.author.chars().count() > 80
        || metadata.version.chars().count() > 32
    {
        return Err(CosmifyError::InvalidCosmeticPack(
            "Cosmify metadata exceeds supported field lengths".to_string(),
        ));
    }
    validate_hex_color(
        &metadata.banner_gradient_start,
        "Banner gradient start color",
    )?;
    validate_hex_color(&metadata.banner_gradient_end, "Banner gradient end color")?;
    if let Some(icon) = metadata.icon.as_deref() {
        let icon_path = Path::new(icon);
        if icon_path.components().count() != 1
            || icon_path.file_name().and_then(|value| value.to_str()) != Some(icon)
        {
            return Err(CosmifyError::InvalidCosmeticPack(
                "Cosmify icon must be a simple file name".to_string(),
            ));
        }
    }
    Ok(metadata)
}

fn validate_hex_color(value: &str, label: &str) -> Result<()> {
    let value = value.trim();
    let valid = value.len() == 7
        && value.starts_with('#')
        && value[1..]
            .chars()
            .all(|character| character.is_ascii_hexdigit());
    if valid {
        Ok(())
    } else {
        Err(CosmifyError::InvalidCosmeticPack(format!(
            "{label} must use the #RRGGBB format"
        )))
    }
}

fn normalize_hex_color(value: &str, label: &str) -> Result<String> {
    validate_hex_color(value, label)?;
    Ok(value.trim().to_ascii_uppercase())
}

fn write_metadata(pack: &Path, metadata: &CosmeticPackMetadata) -> Result<()> {
    let directory = pack.join(COSMIFY_DIR);
    fs::create_dir_all(&directory).map_err(|error| CosmifyError::io(&directory, error))?;
    let path = directory.join(COSMIFY_MANIFEST);
    let tmp = directory.join("pack.json.tmp");
    let previous = directory.join("pack.json.bak");
    fs::write(&tmp, serde_json::to_vec_pretty(metadata)?)
        .map_err(|error| CosmifyError::io(&tmp, error))?;
    if path.exists() {
        if previous.exists() {
            fs::remove_file(&previous).map_err(|error| CosmifyError::io(&previous, error))?;
        }
        fs::rename(&path, &previous).map_err(|error| CosmifyError::io(&path, error))?;
    }
    match fs::rename(&tmp, &path) {
        Ok(()) => {
            if previous.exists() {
                let _ = fs::remove_file(&previous);
            }
            Ok(())
        }
        Err(error) => {
            if previous.exists() {
                let _ = fs::rename(&previous, &path);
            }
            let _ = fs::remove_file(&tmp);
            Err(CosmifyError::io(&path, error))
        }
    }
}

fn copy_embedded_icon(
    source: &Path,
    target: &Path,
    metadata: &mut CosmeticPackMetadata,
) -> Result<()> {
    let Some(icon) = metadata.icon.clone() else {
        return Ok(());
    };
    let source_icon = source.join(COSMIFY_DIR).join(&icon);
    if !source_icon.is_file() {
        metadata.icon = None;
        return Ok(());
    }
    let info = fs::metadata(&source_icon).map_err(|error| CosmifyError::io(&source_icon, error))?;
    if info.len() > MAX_ICON_BYTES {
        metadata.icon = None;
        return Ok(());
    }
    let extension = validated_image_extension(&source_icon)?;
    let meta_dir = target.join(COSMIFY_DIR);
    fs::create_dir_all(&meta_dir).map_err(|error| CosmifyError::io(&meta_dir, error))?;
    let filename = format!("icon.{extension}");
    let target_icon = meta_dir.join(&filename);
    fs::copy(&source_icon, &target_icon).map_err(|error| CosmifyError::io(&target_icon, error))?;
    metadata.icon = Some(filename);
    Ok(())
}

fn adopt_existing_icon(pack: &Path, metadata: &mut CosmeticPackMetadata) -> Result<()> {
    let meta_dir = pack.join(COSMIFY_DIR);
    if let Some(existing) = metadata.icon.as_deref() {
        let candidate = meta_dir.join(existing);
        if candidate.is_file()
            && fs::metadata(&candidate)
                .map(|value| value.len())
                .unwrap_or(u64::MAX)
                <= MAX_ICON_BYTES
        {
            return Ok(());
        }
        metadata.icon = None;
    }

    for candidate in [
        "pack_icon.png",
        "pack_icon.jpg",
        "pack_icon.jpeg",
        "pack_icon.webp",
    ] {
        let source = pack.join(candidate);
        if !source.is_file() {
            continue;
        }
        if fs::metadata(&source)
            .map_err(|error| CosmifyError::io(&source, error))?
            .len()
            > MAX_ICON_BYTES
        {
            continue;
        }
        let Ok(extension) = validated_image_extension(&source) else {
            continue;
        };
        fs::create_dir_all(&meta_dir).map_err(|error| CosmifyError::io(&meta_dir, error))?;
        let filename = format!("icon.{extension}");
        let target = meta_dir.join(&filename);
        fs::copy(&source, &target).map_err(|error| CosmifyError::io(&target, error))?;
        metadata.icon = Some(filename);
        break;
    }
    Ok(())
}

fn validated_image_extension(source: &Path) -> Result<&'static str> {
    let mut file = File::open(source).map_err(|error| CosmifyError::io(source, error))?;
    let mut head = [0u8; 16];
    let read = file
        .read(&mut head)
        .map_err(|error| CosmifyError::io(source, error))?;
    let head = &head[..read];
    if head.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Ok("png");
    }
    if head.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok("jpg");
    }
    if head.len() >= 12 && &head[..4] == b"RIFF" && &head[8..12] == b"WEBP" {
        return Ok("webp");
    }
    Err(CosmifyError::InvalidCosmeticPack(
        "Icon must be a PNG, JPEG or WebP image".to_string(),
    ))
}

fn remove_managed_icons(meta_dir: &Path) -> Result<()> {
    if !meta_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(meta_dir).map_err(|error| CosmifyError::io(meta_dir, error))? {
        let entry = entry.map_err(|error| CosmifyError::io(meta_dir, error))?;
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if entry.path().is_file() && name.starts_with("icon.") {
            fs::remove_file(entry.path()).map_err(|error| CosmifyError::io(entry.path(), error))?;
        }
    }
    Ok(())
}

fn read_icon_data_url(pack: &Path, metadata: &CosmeticPackMetadata) -> Result<Option<String>> {
    let Some(icon) = metadata.icon.as_deref() else {
        return Ok(None);
    };
    let path = pack.join(COSMIFY_DIR).join(icon);
    if !path.is_file() {
        return Ok(None);
    }
    let info = fs::metadata(&path).map_err(|error| CosmifyError::io(&path, error))?;
    if info.len() > MAX_ICON_BYTES {
        return Ok(None);
    }
    let extension = validated_image_extension(&path)?;
    let mime = match extension {
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "webp" => "image/webp",
        _ => return Ok(None),
    };
    let bytes = fs::read(&path).map_err(|error| CosmifyError::io(&path, error))?;
    Ok(Some(format!("data:{mime};base64,{}", BASE64.encode(bytes))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_pack(path: &Path) {
        fs::create_dir_all(path).unwrap();
        fs::write(
            path.join("skins.json"),
            br#"{"serialize_name":"fixture","skins":[{"localization_name":"One"}]}"#,
        )
        .unwrap();
        fs::write(path.join("skin.png"), b"png fixture").unwrap();
    }

    #[test]
    fn folder_import_creates_managed_metadata_and_can_be_edited() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("My Pack");
        let library = temp.path().join("library");
        create_pack(&source);

        let imported = import_cosmetic_pack(
            &library,
            ImportCosmeticPackRequest {
                source_path: source.to_string_lossy().into_owned(),
            },
        )
        .unwrap();
        assert_eq!(imported.name, "fixture");
        assert_eq!(imported.analysis.skin_count, 1);
        assert!(Path::new(&imported.path)
            .join(COSMIFY_DIR)
            .join(COSMIFY_MANIFEST)
            .is_file());

        let edited = update_cosmetic_pack(
            &library,
            UpdateCosmeticPackRequest {
                id: imported.id.clone(),
                name: "Night Pack".to_string(),
                description: "A managed fixture".to_string(),
                author: "Cosmify".to_string(),
                uuid: "9d661f42-5214-45e3-8b2b-b91f9a19fa8e".to_string(),
                version: "2.0.0".to_string(),
                banner_gradient_start: "#123456".to_string(),
                banner_gradient_end: "#ABCDEF".to_string(),
            },
        )
        .unwrap();
        assert_eq!(edited.name, "Night Pack");
        assert_eq!(edited.banner_gradient_start, "#123456");
        assert_eq!(edited.banner_gradient_end, "#ABCDEF");
        assert_eq!(list_cosmetic_packs(&library).unwrap().len(), 1);

        delete_cosmetic_pack(&library, &imported.id).unwrap();
        assert!(list_cosmetic_packs(&library).unwrap().is_empty());
    }

    #[test]
    fn legacy_metadata_without_banner_colors_uses_cosmify_defaults() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("Legacy Pack");
        let library = temp.path().join("library");
        create_pack(&source);

        let imported = import_cosmetic_pack(
            &library,
            ImportCosmeticPackRequest {
                source_path: source.to_string_lossy().into_owned(),
            },
        )
        .unwrap();
        let metadata_file = Path::new(&imported.path)
            .join(COSMIFY_DIR)
            .join(COSMIFY_MANIFEST);
        let mut metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(&metadata_file).unwrap()).unwrap();
        let object = metadata.as_object_mut().unwrap();
        object.remove("bannerGradientStart");
        object.remove("bannerGradientEnd");
        fs::write(
            &metadata_file,
            serde_json::to_vec_pretty(&metadata).unwrap(),
        )
        .unwrap();

        let loaded = load_cosmetic_pack(Path::new(&imported.path)).unwrap();
        assert_eq!(loaded.banner_gradient_start, DEFAULT_BANNER_GRADIENT_START);
        assert_eq!(loaded.banner_gradient_end, DEFAULT_BANNER_GRADIENT_END);
    }

    #[test]
    fn folder_import_uses_manifest_identity_as_default_metadata() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("Folder Name Should Not Win");
        let library = temp.path().join("library");
        create_pack(&source);
        fs::write(
            source.join("manifest.json"),
            br#"{
                "format_version": 2,
                "header": {
                    "name": "Imported Persona Pack",
                    "uuid": "637f229d-b09b-4b3d-990e-b20e674452a2",
                    "version": [3, 4, 5]
                },
                "modules": []
            }"#,
        )
        .unwrap();

        let imported = import_cosmetic_pack(
            &library,
            ImportCosmeticPackRequest {
                source_path: source.to_string_lossy().into_owned(),
            },
        )
        .unwrap();

        assert_eq!(imported.name, "Imported Persona Pack");
        assert_eq!(imported.uuid, "637f229d-b09b-4b3d-990e-b20e674452a2");
        assert_eq!(imported.version, "3.4.5");
    }

    #[test]
    fn invalid_root_pack_icon_is_ignored_instead_of_breaking_import() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("Pack");
        let library = temp.path().join("library");
        create_pack(&source);
        fs::write(source.join("pack_icon.png"), b"not an actual png").unwrap();

        let imported = import_cosmetic_pack(
            &library,
            ImportCosmeticPackRequest {
                source_path: source.to_string_lossy().into_owned(),
            },
        )
        .unwrap();

        assert!(imported.icon_data_url.is_none());
    }

    #[test]
    fn archive_import_rejects_parent_traversal() {
        let temp = tempdir().unwrap();
        let archive_path = temp.path().join("bad.zip");
        let file = File::create(&archive_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("../escape.txt", zip::write::SimpleFileOptions::default())
            .unwrap();
        use std::io::Write;
        zip.write_all(b"nope").unwrap();
        zip.finish().unwrap();

        let error = import_cosmetic_pack(
            &temp.path().join("library"),
            ImportCosmeticPackRequest {
                source_path: archive_path.to_string_lossy().into_owned(),
            },
        )
        .unwrap_err();
        assert!(matches!(error, CosmifyError::UnsafeArchiveEntry(_)));
    }
}
