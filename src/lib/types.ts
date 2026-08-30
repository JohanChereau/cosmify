export interface HostPack {
  uuid: string;
  name: string;
  filename: string;
  filepath: string;
  sizeBytes: number;
  modifiedAt?: string | null;
  sha256: string;
  knownToMinecraft: boolean;
}

export interface ValidationMessage {
  code: string;
  message: string;
}

export interface CustomPackAnalysis {
  path: string;
  valid: boolean;
  fileCount: number;
  pngCount: number;
  geometryCount: number;
  skinCount: number;
  warnings: ValidationMessage[];
}

export interface MinecraftStatus {
  detected: boolean;
  premiumCachePath?: string | null;
  skinPacksPath?: string | null;
  minecraftRunning: boolean;
  minecraftProcesses: string[];
  hostPackCount: number;
}

export interface ImportPreview {
  custom: CustomPackAnalysis;
  host: HostPack;
  hostFingerprint: string;
  filesToCopy: number;
  hostAssetsToRemove: number;
  backupWillBeCreated: boolean;
  warnings: ValidationMessage[];
}

export interface OperationResult {
  success: boolean;
  operationId: string;
  message: string;
  backupId?: string | null;
}

export interface ProgressUpdate {
  operationId: string;
  stage: string;
  label: string;
  progress: number;
}

export interface BackupInfo {
  id: string;
  createdAt: string;
  reason: string;
  originalPath: string;
  filename: string;
  packName: string;
  uuid: string;
  backupFile: string;
  sha256: string;
  sizeBytes: number;
}

export interface ActivityEntry {
  id: string;
  timestamp: string;
  kind: string;
  title: string;
  detail: string;
  success: boolean;
}

export interface AppSettings {
  premiumCacheOverride?: string | null;
  keysDirectory?: string | null;
  autoBackup: boolean;
  requireMinecraftClosed: boolean;
  theme: 'system' | 'light' | 'dark' | string;
}

export interface EncryptPackResult {
  uuid: string;
  outputFile: string;
}
export interface CosmeticPack {
  id: string;
  name: string;
  description: string;
  author: string;
  uuid: string;
  version: string;
  path: string;
  iconDataUrl?: string | null;
  createdAt: string;
  updatedAt: string;
  analysis: CustomPackAnalysis;
}

export interface UpdateCosmeticPackInput {
  id: string;
  name: string;
  description: string;
  author: string;
  uuid: string;
  version: string;
}

