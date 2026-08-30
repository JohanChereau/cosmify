use std::{fs, path::{Path, PathBuf}};

use chrono::Utc;

use crate::{
    util::{path_string, safe_filename, sha256_file},
    BackupInfo, HostPack, CosmifyError, Result,
};

pub(crate) fn create_backup(root: &Path, pack: &HostPack, reason: &str) -> Result<BackupInfo> {
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = Utc::now();
    let folder_name = format!(
        "{}_{}_{}",
        timestamp.format("%Y%m%d_%H%M%S"),
        safe_filename(&pack.name),
        &id[..8]
    );
    let dir = root.join(folder_name);
    fs::create_dir_all(&dir).map_err(|e| CosmifyError::io(&dir, e))?;
    let source = PathBuf::from(&pack.filepath);
    let backup_file = dir.join("pack.bin");
    fs::copy(&source, &backup_file).map_err(|e| CosmifyError::io(&backup_file, e))?;
    let size_bytes = fs::metadata(&backup_file)
        .map_err(|e| CosmifyError::io(&backup_file, e))?
        .len();
    let sha256 = sha256_file(&backup_file)?;
    let info = BackupInfo {
        id,
        created_at: timestamp.to_rfc3339(),
        reason: reason.to_string(),
        original_path: pack.filepath.clone(),
        filename: pack.filename.clone(),
        pack_name: pack.name.clone(),
        uuid: pack.uuid.clone(),
        backup_file: path_string(&backup_file),
        sha256,
        size_bytes,
    };
    let metadata = dir.join("backup.json");
    fs::write(&metadata, serde_json::to_vec_pretty(&info)?)
        .map_err(|e| CosmifyError::io(&metadata, e))?;
    Ok(info)
}

pub(crate) fn list_backups(root: &Path) -> Result<Vec<BackupInfo>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut backups = Vec::new();
    for entry in fs::read_dir(root).map_err(|e| CosmifyError::io(root, e))? {
        let entry = entry.map_err(|e| CosmifyError::io(root, e))?;
        if !entry.path().is_dir() {
            continue;
        }
        let metadata = entry.path().join("backup.json");
        if !metadata.exists() {
            continue;
        }
        let info: BackupInfo = match fs::read(&metadata)
            .map_err(|e| CosmifyError::io(&metadata, e))
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(Into::into))
        {
            Ok(info) => info,
            Err(_) => continue,
        };
        let backup_file = Path::new(&info.backup_file);
        let inside_root = backup_file
            .canonicalize()
            .ok()
            .zip(root.canonicalize().ok())
            .is_some_and(|(file, root)| file.starts_with(root));
        if backup_file.is_file() && inside_root {
            backups.push(info);
        }
    }
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(backups)
}

pub(crate) fn find_backup(root: &Path, id: &str) -> Result<BackupInfo> {
    list_backups(root)?
        .into_iter()
        .find(|backup| backup.id == id)
        .ok_or_else(|| CosmifyError::InvalidBackup(format!("Backup {id} was not found")))
}

pub(crate) fn delete_backup(root: &Path, id: &str) -> Result<()> {
    let info = find_backup(root, id)?;
    let metadata_path = Path::new(&info.backup_file)
        .parent()
        .ok_or_else(|| CosmifyError::InvalidBackup("Backup directory is missing".to_string()))?;
    if !metadata_path.starts_with(root) {
        return Err(CosmifyError::InvalidBackup(
            "Backup path escapes Cosmify data directory".to_string(),
        ));
    }
    fs::remove_dir_all(metadata_path).map_err(|e| CosmifyError::io(metadata_path, e))?;
    Ok(())
}
