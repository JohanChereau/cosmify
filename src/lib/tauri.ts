import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  ActivityEntry,
  AppSettings,
  BackupInfo,
  CosmeticPack,
  CustomPackAnalysis,
  EncryptPackResult,
  HostPack,
  ImportPreview,
  MinecraftStatus,
  OperationResult,
  ProgressUpdate,
  UpdateCosmeticPackInput
} from './types';

export const api = {
  status: () => invoke<MinecraftStatus>('get_system_status'),
  hostPacks: () => invoke<HostPack[]>('list_host_packs'),
  analyzeCustomPack: (path: string) =>
    invoke<CustomPackAnalysis>('analyze_custom_pack', { path }),
  cosmeticPacks: () => invoke<CosmeticPack[]>('list_cosmetic_packs'),
  importCosmeticPack: (sourcePath: string) =>
    invoke<CosmeticPack>('import_cosmetic_pack', { request: { sourcePath } }),
  updateCosmeticPack: (request: UpdateCosmeticPackInput) =>
    invoke<CosmeticPack>('update_cosmetic_pack', { request }),
  setCosmeticPackIcon: (id: string, sourcePath: string) =>
    invoke<CosmeticPack>('set_cosmetic_pack_icon', { id, sourcePath }),
  deleteCosmeticPack: (id: string) => invoke<void>('delete_cosmetic_pack', { id }),
  previewImport: (customFolder: string, hostFilepath: string) =>
    invoke<ImportPreview>('preview_import', {
      request: { customFolder, hostFilepath }
    }),
  installPack: (customFolder: string, hostFilepath: string, expectedHostFingerprint: string) =>
    invoke<OperationResult>('install_pack', {
      request: { customFolder, hostFilepath, expectedHostFingerprint }
    }),
  encryptPackDirectory: (inputFolder: string, outputDirectory: string) =>
    invoke<EncryptPackResult>('encrypt_pack_directory', {
      request: { inputFolder, outputDirectory }
    }),
  backups: () => invoke<BackupInfo[]>('list_backups'),
  restoreBackup: (backupId: string) =>
    invoke<OperationResult>('restore_backup', { request: { backupId } }),
  deleteBackup: (backupId: string) => invoke<void>('delete_backup', { backupId }),
  removePack: (hostFilepath: string, expectedHostFingerprint: string) =>
    invoke<OperationResult>('remove_pack', {
      request: { hostFilepath, expectedHostFingerprint }
    }),
  activity: () => invoke<ActivityEntry[]>('get_activity'),
  settings: () => invoke<AppSettings>('get_settings'),
  saveSettings: (settings: AppSettings) => invoke<AppSettings>('save_settings', { settings }),
  appDataPath: () => invoke<string>('get_app_data_path'),
  onProgress: (callback: (progress: ProgressUpdate) => void): Promise<UnlistenFn> =>
    listen<ProgressUpdate>('cosmify://progress', (event) => callback(event.payload))
};
