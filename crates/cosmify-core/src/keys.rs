use std::{collections::HashMap, fs, path::Path};

use crate::{CosmifyError, Result};

pub(crate) fn load_keys(dir: Option<&Path>) -> Result<HashMap<String, Vec<u8>>> {
    let Some(dir) = dir else {
        return Ok(HashMap::new());
    };

    let mut keys = HashMap::new();
    let keys_db = dir.join("keys.db");
    if keys_db.exists() {
        let text = fs::read_to_string(&keys_db).map_err(|e| CosmifyError::io(&keys_db, e))?;
        for raw in text.lines() {
            let line = raw.trim();
            let Some((uuid, key)) = line.split_once('=') else {
                continue;
            };
            let uuid = uuid.trim();
            let key = key.trim();
            if !uuid.is_empty() && !key.is_empty() {
                keys.insert(uuid.to_string(), key.as_bytes().to_vec());
            }
        }
    }

    let content_keys = dir.join("contentkeys.lst");
    if content_keys.exists() {
        let text =
            fs::read_to_string(&content_keys).map_err(|e| CosmifyError::io(&content_keys, e))?;
        let mut lines = text.lines();
        if lines.next().map(str::trim) == Some("!!! Content Key Entries List !!!") {
            for raw in lines {
                let line = raw.trim();
                let Some((uuid, key)) = line.split_once(':') else {
                    continue;
                };
                let uuid = uuid.trim();
                let key = key.trim();
                if !uuid.is_empty() && !key.is_empty() {
                    keys.entry(uuid.to_string())
                        .or_insert_with(|| key.as_bytes().to_vec());
                }
            }
        }
    }

    Ok(keys)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn loads_both_key_formats_without_overwriting_primary() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("keys.db"), "a=primary\n").unwrap();
        fs::write(
            dir.path().join("contentkeys.lst"),
            "!!! Content Key Entries List !!!\na:secondary\nb:other\n",
        )
        .unwrap();

        let keys = load_keys(Some(dir.path())).unwrap();
        assert_eq!(keys.get("a").unwrap(), b"primary");
        assert_eq!(keys.get("b").unwrap(), b"other");
    }
}
