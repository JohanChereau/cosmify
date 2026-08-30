use std::{fs::File, io::Read, path::Path, time::SystemTime};

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use crate::{CosmifyError, Result};

pub(crate) fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(|e| CosmifyError::io(path, e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| CosmifyError::io(path, e))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub(crate) fn modified_at_iso(time: SystemTime) -> Option<String> {
    let value: DateTime<Utc> = time.into();
    Some(value.to_rfc3339())
}

pub(crate) fn safe_filename(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "pack".to_string()
    } else {
        trimmed.chars().take(80).collect()
    }
}

pub(crate) fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
