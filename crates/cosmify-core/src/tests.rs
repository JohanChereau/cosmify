use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Write,
    path::Path,
};

use tempfile::tempdir;

use crate::{
    archive::{extract_archive, write_archive},
    crypto::{decrypt_bytes, encrypt_pack, HEADER_SIZE},
    AppSettings, Cosmify, ImportCosmeticPackRequest, ImportPreviewRequest, InstallRequest,
    RestoreBackupRequest,
};

fn manifest(uuid: &str) -> serde_json::Value {
    serde_json::json!({
        "format_version": 2,
        "header": {
            "name": "Host Pack",
            "description": "fixture",
            "uuid": uuid,
            "version": [1, 0, 0],
            "min_engine_version": [1, 20, 0]
        },
        "modules": [{
            "type": "skin_pack",
            "uuid": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
            "version": [1, 0, 0]
        }]
    })
}

#[test]
fn full_import_pipeline_creates_backup_and_keeps_assets_decryptable() {
    let root = tempdir().unwrap();
    let app_data = root.path().join("appdata");
    let premium = root.path().join("premium_cache");
    let skin_packs = premium.join("skin_packs");
    let host_dir = root.path().join("host");
    let custom_dir = root.path().join("custom");
    fs::create_dir_all(&skin_packs).unwrap();
    fs::create_dir_all(&host_dir).unwrap();
    fs::create_dir_all(&custom_dir).unwrap();

    let core = Cosmify::with_data_root(&app_data);
    core.save_settings(AppSettings {
        premium_cache_override: Some(premium.to_string_lossy().into_owned()),
        keys_directory: None,
        auto_backup: true,
        require_minecraft_closed: false,
        theme: "system".to_string(),
    })
    .unwrap();

    let uuid = "11111111-2222-3333-4444-555555555555";
    fs::write(
        host_dir.join("manifest.json"),
        serde_json::to_vec(&manifest(uuid)).unwrap(),
    )
    .unwrap();
    fs::write(
        host_dir.join("skins.json"),
        br#"{"serialize_name":"host","localization_name":"host","skins":[]}"#,
    )
    .unwrap();
    fs::write(host_dir.join("host.png"), b"HOST PNG").unwrap();
    fs::write(host_dir.join("keep.bin"), b"KEEP ME").unwrap();
    fs::write(host_dir.join("pack_icon.png"), b"ICON").unwrap();

    let modified = [
        "manifest.json",
        "skins.json",
        "host.png",
        "keep.bin",
        "pack_icon.png",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<HashSet<_>>();
    encrypt_pack(&host_dir, &HashMap::new(), &modified, false).unwrap();
    let host_file = skin_packs.join(uuid);
    write_archive(&host_dir, &host_file).unwrap();
    fs::write(
        skin_packs.join(".Iru.json"),
        serde_json::to_vec(&serde_json::json!({"entries":[{"id":uuid}]})).unwrap(),
    )
    .unwrap();

    fs::write(
        custom_dir.join("skins.json"),
        br#"{"serialize_name":"custom","localization_name":"custom","skins":[{"localization_name":"Izernix"}]}"#,
    )
    .unwrap();
    fs::write(custom_dir.join("custom.png"), b"CUSTOM PNG").unwrap();
    fs::write(
        custom_dir.join("geometry.custom.json"),
        br#"{"format_version":"1.12.0","minecraft:geometry":[]}"#,
    )
    .unwrap();

    let managed = core
        .import_cosmetic_pack(ImportCosmeticPackRequest {
            source_path: custom_dir.to_string_lossy().into_owned(),
        })
        .unwrap();

    let hosts = core.list_host_packs().unwrap();
    assert_eq!(hosts.len(), 1);
    assert_eq!(hosts[0].uuid, uuid);
    assert!(hosts[0].known_to_minecraft);

    let preview = core
        .preview_import(ImportPreviewRequest {
            custom_folder: managed.path.clone(),
            host_filepath: hosts[0].filepath.clone(),
        })
        .unwrap();
    assert!(preview.files_to_copy >= 3);
    assert!(preview.backup_will_be_created);

    let mut progress = Vec::new();
    let result = core
        .install(
            InstallRequest {
                custom_folder: managed.path.clone(),
                host_filepath: hosts[0].filepath.clone(),
                expected_host_fingerprint: preview.host_fingerprint,
            },
            |event| progress.push(event),
        )
        .unwrap();
    assert!(result.success);
    assert_eq!(progress.last().unwrap().progress, 100);
    assert_eq!(core.list_backups().unwrap().len(), 1);

    let extracted = root.path().join("extracted");
    extract_archive(&host_file, &extracted).unwrap();
    assert_eq!(fs::read(extracted.join("pack_icon.png")).unwrap(), b"ICON");
    assert!(!extracted.join("host.png").exists());
    assert!(extracted.join("custom.png").exists());
    assert!(!extracted.join(".cosmify").exists());

    let contents = fs::read(extracted.join("contents.json")).unwrap();
    let master = b"s5s5ejuDru4uchuF2drUFuthaspAbepE";
    let decoded = decrypt_bytes(master, &master[..16], &contents[HEADER_SIZE..]);
    let parsed: serde_json::Value =
        serde_json::from_slice(trim_zeroes(&decoded)).expect("contents.json should decrypt");
    let entries = parsed["content"].as_array().unwrap();

    assert_decrypts_to(&extracted, entries, "custom.png", b"CUSTOM PNG");
    assert_decrypts_to(&extracted, entries, "keep.bin", b"KEEP ME");

    let first_backup = core.list_backups().unwrap().remove(0);
    let restore_result = core
        .restore_backup(RestoreBackupRequest {
            backup_id: first_backup.id,
        })
        .unwrap();
    let restored = root.path().join("restored");
    extract_archive(&host_file, &restored).unwrap();
    assert!(restored.join("host.png").exists());
    assert!(!restored.join("custom.png").exists());

    // Restoring is snapshot-based, not destructive. Cosmify creates an automatic
    // pre-restore snapshot and returns its id, so the restore itself can be undone.
    let undo_backup_id = restore_result
        .backup_id
        .expect("restore should expose the pre-restore snapshot");
    core.restore_backup(RestoreBackupRequest {
        backup_id: undo_backup_id,
    })
    .unwrap();
    let restored_custom = root.path().join("restored-custom");
    extract_archive(&host_file, &restored_custom).unwrap();
    assert!(!restored_custom.join("host.png").exists());
    assert!(restored_custom.join("custom.png").exists());
}

#[test]
fn install_rejects_a_host_that_changed_after_preview() {
    let root = tempdir().unwrap();
    let app_data = root.path().join("appdata");
    let premium = root.path().join("premium_cache");
    let skin_packs = premium.join("skin_packs");
    fs::create_dir_all(&skin_packs).unwrap();
    let host_dir = root.path().join("host");
    let custom_dir = root.path().join("custom");
    fs::create_dir_all(&host_dir).unwrap();
    fs::create_dir_all(&custom_dir).unwrap();

    let core = Cosmify::with_data_root(&app_data);
    core.save_settings(AppSettings {
        premium_cache_override: Some(premium.to_string_lossy().into_owned()),
        require_minecraft_closed: false,
        ..AppSettings::default()
    })
    .unwrap();

    let uuid = "22222222-2222-3333-4444-555555555555";
    fs::write(
        host_dir.join("manifest.json"),
        serde_json::to_vec(&manifest(uuid)).unwrap(),
    )
    .unwrap();
    fs::write(host_dir.join("skins.json"), br#"{"skins":[]}"#).unwrap();
    fs::write(host_dir.join("old.png"), b"old").unwrap();
    let modified = ["manifest.json", "skins.json", "old.png"]
        .into_iter()
        .map(str::to_string)
        .collect::<HashSet<_>>();
    encrypt_pack(&host_dir, &HashMap::new(), &modified, false).unwrap();
    let host_file = skin_packs.join(uuid);
    write_archive(&host_dir, &host_file).unwrap();

    fs::write(custom_dir.join("skins.json"), br#"{"skins":[]}"#).unwrap();
    fs::write(custom_dir.join("new.png"), b"new").unwrap();

    let host = core.list_host_packs().unwrap().remove(0);
    let preview = core
        .preview_import(ImportPreviewRequest {
            custom_folder: custom_dir.to_string_lossy().into_owned(),
            host_filepath: host.filepath.clone(),
        })
        .unwrap();
    fs::OpenOptions::new()
        .append(true)
        .open(&host_file)
        .unwrap()
        .write_all(b"changed")
        .unwrap();

    let error = core
        .install(
            InstallRequest {
                custom_folder: custom_dir.to_string_lossy().into_owned(),
                host_filepath: host.filepath,
                expected_host_fingerprint: preview.host_fingerprint,
            },
            |_| {},
        )
        .unwrap_err();
    assert!(error.to_string().contains("changed since preview"));
}

#[test]
fn settings_reject_unknown_theme_values() {
    let root = tempdir().unwrap();
    let core = Cosmify::with_data_root(root.path().join("appdata"));
    let error = core
        .save_settings(AppSettings {
            theme: "neon-from-untrusted-ipc".to_string(),
            ..AppSettings::default()
        })
        .unwrap_err();
    assert!(error.to_string().contains("Theme must be one of"));
}

fn trim_zeroes(input: &[u8]) -> &[u8] {
    let len = input
        .iter()
        .rposition(|value| *value != 0)
        .map_or(0, |index| index + 1);
    &input[..len]
}

fn assert_decrypts_to(root: &Path, entries: &[serde_json::Value], name: &str, expected: &[u8]) {
    let entry = entries
        .iter()
        .find(|entry| entry["path"].as_str() == Some(name))
        .expect("encrypted entry missing");
    let key = entry["key"].as_str().unwrap().as_bytes();
    let encrypted = fs::read(root.join(name)).unwrap();
    let plain = decrypt_bytes(key, &key[..16], &encrypted);
    assert_eq!(plain, expected);
}
