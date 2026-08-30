use std::{collections::{HashMap, HashSet}, fs, path::Path};

use aes::{cipher::KeyIvInit, Aes256};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::{CosmifyError, Result};

pub(crate) const MAGIC: u32 = 0x9BCFB9FC;
pub(crate) const HEADER_SIZE: usize = 0x100;
const FALLBACK_MASTER_KEY: &[u8; 32] = b"s5s5ejuDru4uchuF2drUFuthaspAbepE";
const SKIP_ENCRYPT: &[&str] = &[
    "manifest.json",
    "contents.json",
    "content.zipe",
    "texts",
    "pack_icon.png",
    "signatures.json",
];

type Aes256Cfb8Enc = cfb8::Encryptor<Aes256>;
type Aes256Cfb8Dec = cfb8::Decryptor<Aes256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContentsJson {
    version: u32,
    content: Vec<ContentEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContentEntry {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
}

fn normalize_32(input: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let len = input.len().min(32);
    out[..len].copy_from_slice(&input[..len]);
    out
}

pub(crate) fn encrypt_bytes(key: &[u8], iv: &[u8], input: &[u8]) -> Vec<u8> {
    let key = normalize_32(key);
    let mut iv16 = [0u8; 16];
    let iv_len = iv.len().min(16);
    iv16[..iv_len].copy_from_slice(&iv[..iv_len]);
    let mut output = input.to_vec();
    Aes256Cfb8Enc::new(&key.into(), &iv16.into()).encrypt(&mut output);
    output
}

pub(crate) fn decrypt_bytes(key: &[u8], iv: &[u8], input: &[u8]) -> Vec<u8> {
    let key = normalize_32(key);
    let mut iv16 = [0u8; 16];
    let iv_len = iv.len().min(16);
    iv16[..iv_len].copy_from_slice(&iv[..iv_len]);
    let mut output = input.to_vec();
    Aes256Cfb8Dec::new(&key.into(), &iv16.into()).decrypt(&mut output);
    output
}

pub(crate) fn create_header(uuid: &str) -> Result<Vec<u8>> {
    let uuid_bytes = uuid.as_bytes();
    if uuid_bytes.len() > HEADER_SIZE.saturating_sub(17) || uuid_bytes.len() > u8::MAX as usize {
        return Err(CosmifyError::Internal(
            "Manifest UUID is too long for the content header".to_string(),
        ));
    }
    let mut header = vec![0u8; HEADER_SIZE];
    header[4..8].copy_from_slice(&MAGIC.to_le_bytes());
    header[16] = uuid_bytes.len() as u8;
    header[17..17 + uuid_bytes.len()].copy_from_slice(uuid_bytes);
    Ok(header)
}

fn should_skip_encryption(relative: &str) -> bool {
    relative
        .replace('\\', "/")
        .split('/')
        .any(|part| SKIP_ENCRYPT.iter().any(|skip| part.eq_ignore_ascii_case(skip)))
}

fn random_file_key() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

fn content_key_for(uuid: &str, keys: &HashMap<String, Vec<u8>>) -> [u8; 32] {
    let env_key = std::env::var("COSMIFY_MASTER_KEY")
        .ok()
        .or_else(|| std::env::var("PERSONAFORGE_MASTER_KEY").ok());
    let source = keys
        .get(uuid)
        .map(Vec::as_slice)
        .or_else(|| env_key.as_deref().map(str::as_bytes))
        .unwrap_or(FALLBACK_MASTER_KEY);
    normalize_32(source)
}

fn read_existing_contents(pack_dir: &Path, content_key: &[u8; 32]) -> Result<Option<HashMap<String, String>>> {
    let path = pack_dir.join("contents.json");
    if !path.exists() {
        return Ok(Some(HashMap::new()));
    }
    let bytes = fs::read(&path).map_err(|e| CosmifyError::io(&path, e))?;
    if bytes.len() <= HEADER_SIZE {
        return Ok(None);
    }
    let decrypted = decrypt_bytes(content_key, &content_key[..16], &bytes[HEADER_SIZE..]);
    let text = String::from_utf8_lossy(&decrypted)
        .trim_end_matches('\0')
        .trim()
        .to_string();
    let parsed: ContentsJson = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let map = parsed
        .content
        .into_iter()
        .filter_map(|entry| entry.key.map(|key| (entry.path.replace('\\', "/"), key)))
        .collect();
    Ok(Some(map))
}

fn sign_manifest(pack_dir: &Path) -> Result<()> {
    let manifest_path = pack_dir.join("manifest.json");
    let bytes = fs::read(&manifest_path).map_err(|e| CosmifyError::io(&manifest_path, e))?;
    let hash = Sha256::digest(bytes);
    let payload = serde_json::json!([{
        "hash": BASE64.encode(hash),
        "path": "manifest.json"
    }]);
    let signature_path = pack_dir.join("signatures.json");
    fs::write(&signature_path, serde_json::to_vec_pretty(&payload)?)
        .map_err(|e| CosmifyError::io(&signature_path, e))?;
    Ok(())
}

pub(crate) fn encrypt_pack(
    pack_dir: &Path,
    keys: &HashMap<String, Vec<u8>>,
    modified_paths: &HashSet<String>,
    require_existing_contents: bool,
) -> Result<String> {
    let manifest_path = pack_dir.join("manifest.json");
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(&manifest_path).map_err(|e| CosmifyError::io(&manifest_path, e))?,
    )?;
    let uuid = manifest
        .pointer("/header/uuid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| CosmifyError::InvalidHostPack("manifest.header.uuid is missing".to_string()))?
        .to_string();

    let content_key = content_key_for(&uuid, keys);
    let existing = read_existing_contents(pack_dir, &content_key)?;
    if require_existing_contents && existing.is_none() {
        return Err(CosmifyError::ContentKeyUnavailable);
    }
    let existing = existing.unwrap_or_default();

    let mut entries = Vec::new();
    let mut paths = WalkDir::new(pack_dir)
        .min_depth(1)
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()?;
    paths.sort_by_key(|entry| entry.path().to_path_buf());

    for entry in paths {
        let path = entry.path();
        let rel = path
            .strip_prefix(pack_dir)
            .map_err(|_| CosmifyError::Internal("Failed to build a relative path".to_string()))?
            .to_string_lossy()
            .replace('\\', "/");

        if entry.file_type().is_dir() {
            entries.push(ContentEntry {
                path: format!("{rel}/"),
                key: None,
            });
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }

        if should_skip_encryption(&rel) {
            entries.push(ContentEntry { path: rel, key: None });
            continue;
        }

        let old_key = existing.get(&rel).cloned();
        let must_encrypt = old_key.is_none() || modified_paths.contains(&rel);
        let file_key = old_key.unwrap_or_else(random_file_key);

        if must_encrypt {
            let file_key_bytes = file_key.as_bytes();
            let plain = fs::read(path).map_err(|e| CosmifyError::io(path, e))?;
            let encrypted = encrypt_bytes(file_key_bytes, &file_key_bytes[..16.min(file_key_bytes.len())], &plain);
            fs::write(path, encrypted).map_err(|e| CosmifyError::io(path, e))?;
        }

        entries.push(ContentEntry {
            path: rel,
            key: Some(file_key),
        });
    }

    let contents = ContentsJson {
        version: 1,
        content: entries,
    };
    let raw = serde_json::to_vec(&contents)?;
    let encrypted = encrypt_bytes(&content_key, &content_key[..16], &raw);
    let mut output = create_header(&uuid)?;
    output.extend_from_slice(&encrypted);
    let contents_path = pack_dir.join("contents.json");
    fs::write(&contents_path, output).map_err(|e| CosmifyError::io(&contents_path, e))?;

    let signature_path = pack_dir.join("signatures.json");
    if signature_path.exists() {
        fs::remove_file(&signature_path).map_err(|e| CosmifyError::io(&signature_path, e))?;
    }
    sign_manifest(pack_dir)?;
    Ok(uuid)
}

pub(crate) fn verify_encrypted_pack(pack_dir: &Path, keys: &HashMap<String, Vec<u8>>) -> Result<()> {
    let manifest_path = pack_dir.join("manifest.json");
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(&manifest_path).map_err(|e| CosmifyError::io(&manifest_path, e))?,
    )?;
    let uuid = manifest
        .pointer("/header/uuid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| CosmifyError::InvalidHostPack("manifest.header.uuid is missing".to_string()))?;
    let content_key = content_key_for(uuid, keys);
    let contents_path = pack_dir.join("contents.json");
    let bytes = fs::read(&contents_path).map_err(|e| CosmifyError::io(&contents_path, e))?;
    if bytes.len() <= HEADER_SIZE {
        return Err(CosmifyError::InvalidHostPack(
            "contents.json is shorter than its required header".to_string(),
        ));
    }
    let decoded = decrypt_bytes(&content_key, &content_key[..16], &bytes[HEADER_SIZE..]);
    let text = String::from_utf8_lossy(&decoded)
        .trim_end_matches('\0')
        .trim()
        .to_string();
    let contents: ContentsJson = serde_json::from_str(&text)
        .map_err(|_| CosmifyError::ContentKeyUnavailable)?;

    let mut encrypted_count = 0usize;
    for entry in contents.content {
        let Some(file_key) = entry.key else {
            continue;
        };
        encrypted_count += 1;
        if file_key.as_bytes().len() != 32 {
            return Err(CosmifyError::InvalidHostPack(format!(
                "Invalid per-file key length for {}",
                entry.path
            )));
        }
        let file_path = pack_dir.join(entry.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !file_path.is_file() {
            return Err(CosmifyError::InvalidHostPack(format!(
                "contents.json references a missing file: {}",
                entry.path
            )));
        }
        let encrypted = fs::read(&file_path).map_err(|e| CosmifyError::io(&file_path, e))?;
        let key = file_key.as_bytes();
        let _ = decrypt_bytes(key, &key[..16], &encrypted);
    }

    if encrypted_count == 0 {
        return Err(CosmifyError::InvalidHostPack(
            "contents.json contains no encrypted assets".to_string(),
        ));
    }
    if !pack_dir.join("signatures.json").is_file() {
        return Err(CosmifyError::InvalidHostPack(
            "signatures.json was not generated".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_cfb8_round_trip() {
        let key = b"12345678901234567890123456789012";
        let plain = b"bonjour steve avec sa cape";
        let encrypted = encrypt_bytes(key, &key[..16], plain);
        assert_ne!(encrypted, plain);
        assert_eq!(decrypt_bytes(key, &key[..16], &encrypted), plain);
    }

    #[test]
    fn header_contains_magic_and_uuid() {
        let uuid = "11111111-2222-3333-4444-555555555555";
        let header = create_header(uuid).unwrap();
        assert_eq!(header.len(), HEADER_SIZE);
        assert_eq!(u32::from_le_bytes(header[4..8].try_into().unwrap()), MAGIC);
        assert_eq!(header[16] as usize, uuid.len());
        assert_eq!(&header[17..17 + uuid.len()], uuid.as_bytes());
    }
}
