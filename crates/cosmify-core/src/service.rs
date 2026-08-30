use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::{
    activity::{append_activity, list_activity},
    archive::{
        count_replaceable_assets_in_archive, extract_archive, read_manifest_from_archive,
        read_pack_name_from_language, verify_archive, write_archive,
    },
    backup::{create_backup, delete_backup, find_backup, list_backups},
    cosmetics::{
        delete_cosmetic_pack, import_cosmetic_pack, list_cosmetic_packs, set_cosmetic_pack_icon,
        update_cosmetic_pack,
    },
    crypto::{encrypt_pack, verify_encrypted_pack},
    keys::load_keys,
    models::*,
    paths::{resolve_premium_cache, skin_packs_path, AppPaths},
    process::minecraft_processes,
    settings::{read_settings, write_settings},
    util::{modified_at_iso, path_string, sha256_file},
    validation::{analyze_custom_pack, custom_files_to_copy, is_ignored_custom_name},
    AppSettings, CosmifyError, Result,
};

#[derive(Debug, Clone)]
pub struct Cosmify {
    paths: AppPaths,
}

impl Cosmify {
    pub fn new() -> Result<Self> {
        Ok(Self {
            paths: AppPaths::system()?,
        })
    }

    pub fn with_data_root(root: impl Into<PathBuf>) -> Self {
        Self {
            paths: AppPaths::from_root(root.into()),
        }
    }

    pub fn settings(&self) -> Result<AppSettings> {
        read_settings(&self.paths.settings_file)
    }

    pub fn save_settings(&self, settings: AppSettings) -> Result<AppSettings> {
        if !matches!(settings.theme.as_str(), "system" | "light" | "dark") {
            return Err(CosmifyError::InvalidSettings(
                "Theme must be one of: system, light, dark".to_string(),
            ));
        }
        if let Some(value) = settings.premium_cache_override.as_deref() {
            if !value.trim().is_empty() && !Path::new(value).is_dir() {
                return Err(CosmifyError::InvalidCustomPack(
                    "Premium cache override must point to an existing directory".to_string(),
                ));
            }
        }
        if let Some(value) = settings.keys_directory.as_deref() {
            if !value.trim().is_empty() && !Path::new(value).is_dir() {
                return Err(CosmifyError::InvalidCustomPack(
                    "Keys directory must point to an existing directory".to_string(),
                ));
            }
        }
        write_settings(&self.paths.settings_file, &settings)?;
        Ok(settings)
    }

    pub fn system_status(&self) -> Result<MinecraftStatus> {
        let settings = self.settings()?;
        let processes = minecraft_processes();
        let premium = resolve_premium_cache(&settings);
        let skin_path = premium.as_deref().map(skin_packs_path);
        let host_pack_count = match &skin_path {
            Some(_) => self.list_host_packs().map_or(0, |packs| packs.len()),
            None => 0,
        };
        Ok(MinecraftStatus {
            detected: premium.is_some(),
            premium_cache_path: premium.as_deref().map(path_string),
            skin_packs_path: skin_path.as_deref().map(path_string),
            minecraft_running: !processes.is_empty(),
            minecraft_processes: processes,
            host_pack_count,
        })
    }

    pub fn list_host_packs(&self) -> Result<Vec<HostPack>> {
        let settings = self.settings()?;
        let premium = resolve_premium_cache(&settings).ok_or(CosmifyError::PremiumCacheNotFound)?;
        let folder = skin_packs_path(&premium);
        if !folder.is_dir() {
            return Ok(Vec::new());
        }

        let known = read_known_uuids(&folder);
        let mut packs = Vec::new();
        for item in fs::read_dir(&folder).map_err(|e| CosmifyError::io(&folder, e))? {
            let item = item.map_err(|e| CosmifyError::io(&folder, e))?;
            let path = item.path();
            if !path.is_file() {
                continue;
            }
            let filename = item.file_name().to_string_lossy().into_owned();
            if filename.starts_with('.') || filename.eq_ignore_ascii_case("desktop.ini") {
                continue;
            }
            if let Ok(pack) = self.host_pack_from_path(&path, known.contains(&filename)) {
                packs.push(pack);
            }
        }

        for pack in &mut packs {
            if known.contains(&pack.uuid) {
                pack.known_to_minecraft = true;
            }
        }
        packs.sort_by(|a, b| {
            b.known_to_minecraft
                .cmp(&a.known_to_minecraft)
                .then_with(|| {
                    a.name
                        .to_ascii_lowercase()
                        .cmp(&b.name.to_ascii_lowercase())
                })
        });
        Ok(packs)
    }

    pub fn analyze_custom_pack(&self, path: impl AsRef<Path>) -> Result<CustomPackAnalysis> {
        analyze_custom_pack(path.as_ref())
    }

    pub fn list_cosmetic_packs(&self) -> Result<Vec<CosmeticPack>> {
        list_cosmetic_packs(&self.paths.cosmetics_root)
    }

    pub fn import_cosmetic_pack(&self, request: ImportCosmeticPackRequest) -> Result<CosmeticPack> {
        let pack = import_cosmetic_pack(&self.paths.cosmetics_root, request)?;
        append_activity(
            &self.paths.activity_file,
            "library",
            "Cosmetic pack imported",
            &pack.name,
            true,
        )?;
        Ok(pack)
    }

    pub fn update_cosmetic_pack(&self, request: UpdateCosmeticPackRequest) -> Result<CosmeticPack> {
        let pack = update_cosmetic_pack(&self.paths.cosmetics_root, request)?;
        append_activity(
            &self.paths.activity_file,
            "library",
            "Cosmetic pack updated",
            &pack.name,
            true,
        )?;
        Ok(pack)
    }

    pub fn set_cosmetic_pack_icon(&self, id: &str, source_path: &Path) -> Result<CosmeticPack> {
        let pack = set_cosmetic_pack_icon(&self.paths.cosmetics_root, id, source_path)?;
        append_activity(
            &self.paths.activity_file,
            "library",
            "Cosmetic pack artwork updated",
            &pack.name,
            true,
        )?;
        Ok(pack)
    }

    pub fn delete_cosmetic_pack(&self, id: &str) -> Result<()> {
        let name = self
            .list_cosmetic_packs()?
            .into_iter()
            .find(|pack| pack.id == id)
            .map(|pack| pack.name)
            .unwrap_or_else(|| id.to_string());
        delete_cosmetic_pack(&self.paths.cosmetics_root, id)?;
        append_activity(
            &self.paths.activity_file,
            "library",
            "Cosmetic pack deleted",
            &name,
            true,
        )?;
        Ok(())
    }

    pub fn preview_import(&self, request: ImportPreviewRequest) -> Result<ImportPreview> {
        let custom_path = PathBuf::from(&request.custom_folder);
        let custom = analyze_custom_pack(&custom_path)?;
        let host = self.validated_host_pack(Path::new(&request.host_filepath))?;
        let files_to_copy = custom_files_to_copy(&custom_path)?.len();
        let host_assets_to_remove = count_replaceable_assets_in_archive(Path::new(&host.filepath))?;
        let settings = self.settings()?;
        let mut warnings = custom.warnings.clone();
        if !minecraft_processes().is_empty() {
            warnings.push(ValidationMessage {
                code: "minecraft-running".to_string(),
                message: "Minecraft is currently running. It must be closed before installation."
                    .to_string(),
            });
        }
        if !settings.auto_backup {
            warnings.push(ValidationMessage {
                code: "backup-disabled".to_string(),
                message: "Automatic backups are disabled in Settings.".to_string(),
            });
        }
        Ok(ImportPreview {
            custom,
            host_fingerprint: host.sha256.clone(),
            host,
            files_to_copy,
            host_assets_to_remove,
            backup_will_be_created: settings.auto_backup,
            warnings,
        })
    }

    pub fn install<F>(&self, request: InstallRequest, mut progress: F) -> Result<OperationResult>
    where
        F: FnMut(ProgressUpdate),
    {
        let operation_id = uuid::Uuid::new_v4().to_string();
        match self.install_inner(&request, &operation_id, &mut progress) {
            Ok(done) => {
                let _ = append_activity(
                    &self.paths.activity_file,
                    "install",
                    "Cosmetic pack installed",
                    &format!("Host pack: {}", request.host_filepath),
                    true,
                );
                Ok(done)
            }
            Err(error) => {
                let _ = append_activity(
                    &self.paths.activity_file,
                    "install",
                    "Installation failed",
                    &error.to_string(),
                    false,
                );
                Err(error)
            }
        }
    }

    fn install_inner<F>(
        &self,
        request: &InstallRequest,
        operation_id: &str,
        progress: &mut F,
    ) -> Result<OperationResult>
    where
        F: FnMut(ProgressUpdate),
    {
        let settings = self.settings()?;
        self.enforce_minecraft_closed(&settings)?;
        emit(progress, operation_id, "validate", "Validating files", 6);

        let custom_path = PathBuf::from(&request.custom_folder);
        analyze_custom_pack(&custom_path)?;
        let host = self.validated_host_pack(Path::new(&request.host_filepath))?;
        if host.sha256 != request.expected_host_fingerprint {
            return Err(CosmifyError::HostChanged);
        }

        emit(
            progress,
            operation_id,
            "backup",
            "Creating a safety backup",
            14,
        );
        let backup = if settings.auto_backup {
            Some(create_backup(&self.paths.backups_root, &host, "install")?)
        } else {
            None
        };

        let temp = tempfile::Builder::new()
            .prefix("cosmify-")
            .tempdir()
            .map_err(|e| CosmifyError::io(std::env::temp_dir(), e))?;
        let work = temp.path().join("work");
        fs::create_dir_all(&work).map_err(|e| CosmifyError::io(&work, e))?;

        emit(
            progress,
            operation_id,
            "extract",
            "Preparing the host pack",
            26,
        );
        extract_archive(Path::new(&host.filepath), &work)?;

        emit(
            progress,
            operation_id,
            "clean",
            "Removing replaceable host assets",
            38,
        );
        delete_replaceable_assets(&work)?;

        emit(
            progress,
            operation_id,
            "copy",
            "Copying custom cosmetics",
            52,
        );
        let modified_paths = copy_custom_files(&custom_path, &work)?;

        emit(
            progress,
            operation_id,
            "encrypt",
            "Encrypting custom assets",
            68,
        );
        let keys_dir = self.keys_directory(&settings);
        let keys = load_keys(keys_dir.as_deref())?;
        let require_existing = work.join("contents.json").exists();
        let uuid = encrypt_pack(&work, &keys, &modified_paths, require_existing)?;
        if uuid != host.uuid {
            return Err(CosmifyError::InvalidHostPack(
                "The rebuilt pack no longer matches the selected host UUID".to_string(),
            ));
        }

        emit(
            progress,
            operation_id,
            "verify",
            "Verifying encrypted content",
            78,
        );
        verify_encrypted_pack(&work, &keys)?;

        let host_path = PathBuf::from(&host.filepath);
        let output_tmp = sibling_temp_path(&host_path, operation_id);
        emit(
            progress,
            operation_id,
            "archive",
            "Rebuilding the premium-cache archive",
            87,
        );
        let build_result = (|| -> Result<()> {
            write_archive(&work, &output_tmp)?;
            verify_archive(&output_tmp, &host.uuid)?;
            Ok(())
        })();
        if let Err(error) = build_result {
            let _ = fs::remove_file(&output_tmp);
            return Err(error);
        }

        emit(
            progress,
            operation_id,
            "commit",
            "Committing changes safely",
            95,
        );
        if let Err(error) = safe_replace(&output_tmp, &host_path) {
            let _ = fs::remove_file(&output_tmp);
            return Err(error);
        }
        let new_pack = self.host_pack_from_path(&host_path, host.known_to_minecraft)?;
        if new_pack.uuid != host.uuid {
            return Err(CosmifyError::InvalidHostPack(
                "Installed pack changed the host UUID unexpectedly".to_string(),
            ));
        }

        emit(
            progress,
            operation_id,
            "done",
            "Installed successfully",
            100,
        );
        Ok(OperationResult {
            success: true,
            operation_id: operation_id.to_string(),
            message: "Cosmetic pack installed. Restart Minecraft to apply it.".to_string(),
            backup_id: backup.map(|item| item.id),
        })
    }

    pub fn encrypt_pack_directory(&self, request: EncryptPackRequest) -> Result<EncryptPackResult> {
        let input = PathBuf::from(&request.input_folder);
        let output_dir = PathBuf::from(&request.output_directory);
        if !input.is_dir() {
            return Err(CosmifyError::InvalidCustomPack(
                "Input pack directory does not exist".to_string(),
            ));
        }
        let manifest = input.join("manifest.json");
        if !manifest.is_file() {
            return Err(CosmifyError::InvalidCustomPack(
                "Standalone encryption requires manifest.json".to_string(),
            ));
        }
        if !output_dir.is_dir() {
            return Err(CosmifyError::InvalidCustomPack(
                "Output directory does not exist".to_string(),
            ));
        }

        let temp = tempfile::Builder::new()
            .prefix("cosmify-encrypt-")
            .tempdir()
            .map_err(|e| CosmifyError::io(std::env::temp_dir(), e))?;
        let work = temp.path().join("work");
        copy_directory(&input, &work)?;

        let mut modified = HashSet::new();
        for entry in walkdir::WalkDir::new(&work).min_depth(1) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let relative = entry
                    .path()
                    .strip_prefix(&work)
                    .map_err(|_| CosmifyError::Internal("Failed to resolve pack path".to_string()))?
                    .to_string_lossy()
                    .replace('\\', "/");
                modified.insert(relative);
            }
        }

        let settings = self.settings()?;
        let keys_dir = self.keys_directory(&settings);
        let keys = load_keys(keys_dir.as_deref())?;
        let uuid = encrypt_pack(&work, &keys, &modified, false)?;
        verify_encrypted_pack(&work, &keys)?;

        let output = next_available_output(&output_dir, &uuid);
        write_archive(&work, &output)?;
        verify_archive(&output, &uuid)?;
        append_activity(
            &self.paths.activity_file,
            "encrypt",
            "Skin pack encrypted",
            &path_string(&output),
            true,
        )?;
        Ok(EncryptPackResult {
            uuid,
            output_file: path_string(&output),
        })
    }

    pub fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        list_backups(&self.paths.backups_root)
    }

    pub fn restore_backup(&self, request: RestoreBackupRequest) -> Result<OperationResult> {
        let settings = self.settings()?;
        self.enforce_minecraft_closed(&settings)?;
        let backup = find_backup(&self.paths.backups_root, &request.backup_id)?;
        let source = PathBuf::from(&backup.backup_file);
        if sha256_file(&source)? != backup.sha256 {
            return Err(CosmifyError::InvalidBackup(
                "Backup checksum does not match its metadata".to_string(),
            ));
        }
        let target = PathBuf::from(&backup.original_path);
        self.assert_host_path(&target)?;

        let undo_backup = if target.exists() && settings.auto_backup {
            if let Ok(current) = self.host_pack_from_path(&target, true) {
                Some(create_backup(
                    &self.paths.backups_root,
                    &current,
                    "pre-restore",
                )?)
            } else {
                None
            }
        } else {
            None
        };

        let operation_id = uuid::Uuid::new_v4().to_string();
        let temp = sibling_temp_path(&target, &operation_id);
        fs::copy(&source, &temp).map_err(|e| CosmifyError::io(&temp, e))?;
        safe_replace(&temp, &target)?;
        append_activity(
            &self.paths.activity_file,
            "restore",
            "Backup restored",
            &backup.pack_name,
            true,
        )?;
        Ok(OperationResult {
            success: true,
            operation_id,
            message: format!("Restored {}", backup.pack_name),
            // OperationResult.backup_id consistently points to the snapshot created
            // by this operation. For restore, that snapshot is the state that was
            // present immediately before restoring, which makes the restore safely
            // undoable without mutating the selected backup.
            backup_id: undo_backup.map(|backup| backup.id),
        })
    }

    pub fn delete_backup(&self, backup_id: &str) -> Result<()> {
        delete_backup(&self.paths.backups_root, backup_id)?;
        append_activity(
            &self.paths.activity_file,
            "backup",
            "Backup deleted",
            backup_id,
            true,
        )?;
        Ok(())
    }

    pub fn remove_pack(&self, request: RemovePackRequest) -> Result<OperationResult> {
        let settings = self.settings()?;
        self.enforce_minecraft_closed(&settings)?;
        let host = self.validated_host_pack(Path::new(&request.host_filepath))?;
        if host.sha256 != request.expected_host_fingerprint {
            return Err(CosmifyError::HostChanged);
        }
        let backup = if settings.auto_backup {
            Some(create_backup(&self.paths.backups_root, &host, "remove")?)
        } else {
            None
        };
        fs::remove_file(&host.filepath)
            .map_err(|e| CosmifyError::io(Path::new(&host.filepath), e))?;
        append_activity(
            &self.paths.activity_file,
            "remove",
            "Host pack removed",
            &host.name,
            true,
        )?;
        Ok(OperationResult {
            success: true,
            operation_id: uuid::Uuid::new_v4().to_string(),
            message: format!("Removed {}", host.name),
            backup_id: backup.map(|item| item.id),
        })
    }

    pub fn activity(&self) -> Result<Vec<ActivityEntry>> {
        list_activity(&self.paths.activity_file)
    }

    pub fn data_root(&self) -> &Path {
        &self.paths.data_root
    }

    fn enforce_minecraft_closed(&self, settings: &AppSettings) -> Result<()> {
        if settings.require_minecraft_closed && !minecraft_processes().is_empty() {
            return Err(CosmifyError::MinecraftRunning);
        }
        Ok(())
    }

    fn keys_directory(&self, settings: &AppSettings) -> Option<PathBuf> {
        settings
            .keys_directory
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                let default = self.paths.data_root.join("keys");
                default.is_dir().then_some(default)
            })
    }

    fn assert_host_path(&self, host: &Path) -> Result<()> {
        let settings = self.settings()?;
        let premium = resolve_premium_cache(&settings).ok_or(CosmifyError::PremiumCacheNotFound)?;
        let skin_folder = skin_packs_path(&premium);
        let canonical_folder =
            fs::canonicalize(&skin_folder).map_err(|e| CosmifyError::io(&skin_folder, e))?;
        let inside = if host.exists() {
            let canonical_host = fs::canonicalize(host).map_err(|e| CosmifyError::io(host, e))?;
            canonical_host.starts_with(&canonical_folder)
        } else {
            let parent = host
                .parent()
                .ok_or_else(|| CosmifyError::HostOutsideSkinPacks(host.to_path_buf()))?;
            let canonical_parent =
                fs::canonicalize(parent).map_err(|e| CosmifyError::io(parent, e))?;
            canonical_parent == canonical_folder
        };
        if !inside {
            return Err(CosmifyError::HostOutsideSkinPacks(host.to_path_buf()));
        }
        Ok(())
    }

    fn validated_host_pack(&self, host: &Path) -> Result<HostPack> {
        self.assert_host_path(host)?;
        self.host_pack_from_path(host, true)
    }

    fn host_pack_from_path(&self, path: &Path, known_to_minecraft: bool) -> Result<HostPack> {
        let manifest = read_manifest_from_archive(path)?.ok_or_else(|| {
            CosmifyError::InvalidHostPack("No readable manifest.json".to_string())
        })?;
        let uuid = manifest
            .pointer("/header/uuid")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CosmifyError::InvalidHostPack("manifest.header.uuid is missing".to_string())
            })?
            .to_string();
        let mut name = manifest
            .pointer("/header/name")
            .and_then(Value::as_str)
            .unwrap_or("Unknown")
            .to_string();
        if matches!(
            name.to_ascii_lowercase().as_str(),
            "pack.name" | "unknown" | "pack.description"
        ) {
            if let Some(localized) = read_pack_name_from_language(path)? {
                name = localized;
            }
        }
        let metadata = fs::metadata(path).map_err(|e| CosmifyError::io(path, e))?;
        Ok(HostPack {
            uuid,
            name,
            filename: path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("pack")
                .to_string(),
            filepath: path_string(path),
            size_bytes: metadata.len(),
            modified_at: metadata.modified().ok().and_then(modified_at_iso),
            sha256: sha256_file(path)?,
            known_to_minecraft,
        })
    }
}

fn emit<F>(progress: &mut F, operation_id: &str, stage: &str, label: &str, value: u8)
where
    F: FnMut(ProgressUpdate),
{
    progress(ProgressUpdate {
        operation_id: operation_id.to_string(),
        stage: stage.to_string(),
        label: label.to_string(),
        progress: value,
    });
}

fn read_known_uuids(folder: &Path) -> HashSet<String> {
    let path = folder.join(".Iru.json");
    let Ok(bytes) = fs::read(path) else {
        return HashSet::new();
    };
    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
        return HashSet::new();
    };
    value
        .get("entries")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("id").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect()
}

fn delete_replaceable_assets(root: &Path) -> Result<()> {
    let mut to_remove = Vec::new();
    for entry in walkdir::WalkDir::new(root).min_depth(1) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let lower = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if (lower.ends_with(".png") && lower != "pack_icon.png")
            || lower == "skins.json"
            || (lower.contains("geometry") && lower.ends_with(".json"))
        {
            to_remove.push(entry.path().to_path_buf());
        }
    }
    for path in to_remove {
        fs::remove_file(&path).map_err(|e| CosmifyError::io(&path, e))?;
    }
    Ok(())
}

fn copy_custom_files(custom: &Path, destination: &Path) -> Result<HashSet<String>> {
    let mut modified = HashSet::new();
    for entry in fs::read_dir(custom).map_err(|e| CosmifyError::io(custom, e))? {
        let entry = entry.map_err(|e| CosmifyError::io(custom, e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_ignored_custom_name(&name) {
            continue;
        }
        let source = entry.path();
        let target = destination.join(&name);
        let file_type = entry
            .file_type()
            .map_err(|e| CosmifyError::io(&source, e))?;
        if file_type.is_dir() {
            if target.exists() {
                fs::remove_dir_all(&target).map_err(|e| CosmifyError::io(&target, e))?;
            }
            copy_directory(&source, &target)?;
            for copied in walkdir::WalkDir::new(&target).min_depth(1) {
                let copied = copied?;
                if copied.file_type().is_file() {
                    let relative = copied.path().strip_prefix(destination).map_err(|_| {
                        CosmifyError::Internal("Failed to resolve copied asset path".to_string())
                    })?;
                    modified.insert(relative.to_string_lossy().replace('\\', "/"));
                }
            }
        } else if file_type.is_file() {
            fs::copy(&source, &target).map_err(|e| CosmifyError::io(&target, e))?;
            modified.insert(name.replace('\\', "/"));
        }
    }
    Ok(modified)
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).map_err(|e| CosmifyError::io(destination, e))?;
    for entry in walkdir::WalkDir::new(source).min_depth(1) {
        let entry = entry?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .map_err(|_| CosmifyError::Internal("Failed to copy pack directory".to_string()))?;
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|e| CosmifyError::io(&target, e))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| CosmifyError::io(parent, e))?;
            }
            fs::copy(entry.path(), &target).map_err(|e| CosmifyError::io(&target, e))?;
        }
    }
    Ok(())
}

fn next_available_output(directory: &Path, stem: &str) -> PathBuf {
    let first = directory.join(stem);
    if !first.exists() {
        return first;
    }
    for index in 1..10_000 {
        let candidate = directory.join(format!("{stem}-{index}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    directory.join(format!("{stem}-{}", uuid::Uuid::new_v4()))
}

fn sibling_temp_path(target: &Path, operation_id: &str) -> PathBuf {
    let file = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("pack");
    target.with_file_name(format!(".{file}.cosmify.{operation_id}.tmp"))
}

fn safe_replace(temp: &Path, target: &Path) -> Result<()> {
    if !temp.exists() {
        return Err(CosmifyError::Internal(
            "Replacement file does not exist".to_string(),
        ));
    }
    let parent = target
        .parent()
        .ok_or_else(|| CosmifyError::Internal("Target has no parent directory".to_string()))?;
    let swap = parent.join(format!(
        ".{}.cosmify-swap-{}",
        target
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("pack"),
        uuid::Uuid::new_v4()
    ));

    if target.exists() {
        fs::rename(target, &swap).map_err(|e| CosmifyError::io(target, e))?;
    }
    match fs::rename(temp, target) {
        Ok(()) => {
            if swap.exists() {
                let _ = fs::remove_file(&swap);
            }
            Ok(())
        }
        Err(error) => {
            if swap.exists() {
                let _ = fs::rename(&swap, target);
            }
            Err(CosmifyError::io(target, error))
        }
    }
}
