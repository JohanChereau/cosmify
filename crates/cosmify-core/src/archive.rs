use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde_json::Value;
use walkdir::WalkDir;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::{CosmifyError, Result};

pub(crate) fn read_manifest_from_archive(path: &Path) -> Result<Option<Value>> {
    let file = File::open(path).map_err(|e| CosmifyError::io(path, e))?;
    let mut archive = match ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(zip::result::ZipError::InvalidArchive(_)) => return Ok(None),
        Err(error) => return Err(error.into()),
    };

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().replace('\\', "/");
        if name == "manifest.json" || name.ends_with("/manifest.json") {
            let mut content = String::new();
            entry
                .read_to_string(&mut content)
                .map_err(|e| CosmifyError::io(path, e))?;
            return Ok(serde_json::from_str(&content).ok());
        }
    }
    Ok(None)
}

pub(crate) fn read_pack_name_from_language(path: &Path) -> Result<Option<String>> {
    let file = File::open(path).map_err(|e| CosmifyError::io(path, e))?;
    let mut archive = ZipArchive::new(file)?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().replace('\\', "/").to_ascii_lowercase();
        if !(name.ends_with("en_us.lang")) {
            continue;
        }
        let mut content = String::new();
        entry
            .read_to_string(&mut content)
            .map_err(|e| CosmifyError::io(path, e))?;
        for line in content.lines().map(str::trim) {
            if line.starts_with("pack.name=") || line.starts_with("skinpack.") {
                if let Some((_, value)) = line.split_once('=') {
                    let value = value.trim();
                    if !value.is_empty() {
                        return Ok(Some(value.to_string()));
                    }
                }
            }
        }
    }
    Ok(None)
}

pub(crate) fn extract_archive(path: &Path, destination: &Path) -> Result<()> {
    let file = File::open(path).map_err(|e| CosmifyError::io(path, e))?;
    let mut archive = ZipArchive::new(file)?;
    fs::create_dir_all(destination).map_err(|e| CosmifyError::io(destination, e))?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(enclosed) = entry.enclosed_name() else {
            return Err(CosmifyError::UnsafeArchiveEntry(entry.name().to_string()));
        };
        let output = destination.join(enclosed);
        if entry.is_dir() {
            fs::create_dir_all(&output).map_err(|e| CosmifyError::io(&output, e))?;
            continue;
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|e| CosmifyError::io(parent, e))?;
        }
        let mut target = File::create(&output).map_err(|e| CosmifyError::io(&output, e))?;
        std::io::copy(&mut entry, &mut target).map_err(|e| CosmifyError::io(&output, e))?;
    }
    Ok(())
}

pub(crate) fn write_archive(source: &Path, output: &Path) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| CosmifyError::io(parent, e))?;
    }
    let file = File::create(output).map_err(|e| CosmifyError::io(output, e))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(1));

    let mut entries = WalkDir::new(source)
        .min_depth(1)
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path().to_path_buf());

    for entry in entries {
        let relative = entry
            .path()
            .strip_prefix(source)
            .map_err(|_| CosmifyError::Internal("Failed to create archive path".to_string()))?;
        let name = relative.to_string_lossy().replace('\\', "/");
        if entry.file_type().is_dir() {
            zip.add_directory(format!("{name}/"), options)?;
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        zip.start_file(name, options)?;
        let mut source_file = File::open(entry.path())
            .map_err(|e| CosmifyError::io(entry.path(), e))?;
        std::io::copy(&mut source_file, &mut zip)
            .map_err(|e| CosmifyError::io(output, e))?;
    }
    let mut output_file = zip.finish()?;
    output_file.flush().map_err(|e| CosmifyError::io(output, e))?;
    output_file.sync_all().map_err(|e| CosmifyError::io(output, e))?;
    Ok(())
}

pub(crate) fn verify_archive(path: &Path, expected_uuid: &str) -> Result<()> {
    let manifest = read_manifest_from_archive(path)?
        .ok_or_else(|| CosmifyError::InvalidHostPack("Rebuilt archive has no manifest.json".to_string()))?;
    let uuid = manifest
        .pointer("/header/uuid")
        .and_then(Value::as_str)
        .ok_or_else(|| CosmifyError::InvalidHostPack("Rebuilt manifest has no header.uuid".to_string()))?;
    if uuid != expected_uuid {
        return Err(CosmifyError::InvalidHostPack(
            "Rebuilt archive UUID differs from the host pack".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn count_replaceable_assets_in_archive(path: &Path) -> Result<usize> {
    let file = File::open(path).map_err(|e| CosmifyError::io(path, e))?;
    let mut archive = ZipArchive::new(file)?;
    let mut count = 0;
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let name = PathBuf::from(entry.name());
        let lower = name
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if (lower.ends_with(".png") && lower != "pack_icon.png")
            || lower == "skins.json"
            || (lower.contains("geometry") && lower.ends_with(".json"))
        {
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn archive_round_trip() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let archive = temp.path().join("archive.bin");
        let extracted = temp.path().join("extracted");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("hello.txt"), b"hello").unwrap();
        fs::write(source.join("nested/data.bin"), [0u8, 1, 2, 255]).unwrap();

        write_archive(&source, &archive).unwrap();
        extract_archive(&archive, &extracted).unwrap();

        assert_eq!(fs::read(extracted.join("hello.txt")).unwrap(), b"hello");
        assert_eq!(fs::read(extracted.join("nested/data.bin")).unwrap(), [0u8, 1, 2, 255]);
    }
}
