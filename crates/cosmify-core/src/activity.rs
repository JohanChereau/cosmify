use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
};

use chrono::Utc;

use crate::{ActivityEntry, CosmifyError, Result};

const MAX_ACTIVITY_ENTRIES: usize = 500;

pub(crate) fn append_activity(
    path: &Path,
    kind: &str,
    title: &str,
    detail: &str,
    success: bool,
) -> Result<ActivityEntry> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| CosmifyError::io(parent, e))?;
    }
    let entry = ActivityEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        kind: kind.to_string(),
        title: title.to_string(),
        detail: detail.to_string(),
        success,
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| CosmifyError::io(path, e))?;
    writeln!(file, "{}", serde_json::to_string(&entry)?).map_err(|e| CosmifyError::io(path, e))?;
    Ok(entry)
}

pub(crate) fn list_activity(path: &Path) -> Result<Vec<ActivityEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path).map_err(|e| CosmifyError::io(path, e))?;
    let reader = BufReader::new(file);

    let mut entries = reader
        .lines()
        .map_while(|line| line.ok())
        .filter_map(|line| serde_json::from_str::<ActivityEntry>(&line).ok())
        .collect::<Vec<_>>();

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries.truncate(MAX_ACTIVITY_ENTRIES);

    Ok(entries)
}
