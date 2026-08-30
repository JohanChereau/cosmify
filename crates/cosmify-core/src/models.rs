use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HostPack {
    pub uuid: String,
    pub name: String,
    pub filename: String,
    pub filepath: String,
    pub size_bytes: u64,
    pub modified_at: Option<String>,
    pub sha256: String,
    pub known_to_minecraft: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CustomPackAnalysis {
    pub path: String,
    pub valid: bool,
    pub file_count: usize,
    pub png_count: usize,
    pub geometry_count: usize,
    pub skin_count: usize,
    pub warnings: Vec<ValidationMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ValidationMessage {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftStatus {
    pub detected: bool,
    pub premium_cache_path: Option<String>,
    pub skin_packs_path: Option<String>,
    pub minecraft_running: bool,
    pub minecraft_processes: Vec<String>,
    pub host_pack_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewRequest {
    pub custom_folder: String,
    pub host_filepath: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub custom: CustomPackAnalysis,
    pub host: HostPack,
    pub host_fingerprint: String,
    pub files_to_copy: usize,
    pub host_assets_to_remove: usize,
    pub backup_will_be_created: bool,
    pub warnings: Vec<ValidationMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstallRequest {
    pub custom_folder: String,
    pub host_filepath: String,
    pub expected_host_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub success: bool,
    pub operation_id: String,
    pub message: String,
    pub backup_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProgressUpdate {
    pub operation_id: String,
    pub stage: String,
    pub label: String,
    pub progress: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub id: String,
    pub created_at: String,
    pub reason: String,
    pub original_path: String,
    pub filename: String,
    pub pack_name: String,
    pub uuid: String,
    pub backup_file: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntry {
    pub id: String,
    pub timestamp: String,
    pub kind: String,
    pub title: String,
    pub detail: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemovePackRequest {
    pub host_filepath: String,
    pub expected_host_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RestoreBackupRequest {
    pub backup_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EncryptPackRequest {
    pub input_folder: String,
    pub output_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EncryptPackResult {
    pub uuid: String,
    pub output_file: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CosmeticPackMetadata {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub uuid: String,
    pub version: String,
    pub icon: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CosmeticPack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub uuid: String,
    pub version: String,
    pub path: String,
    pub icon_data_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub analysis: CustomPackAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportCosmeticPackRequest {
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCosmeticPackRequest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub uuid: String,
    pub version: String,
}
