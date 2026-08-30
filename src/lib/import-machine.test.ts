import { describe, expect, it } from 'vitest';
import { importReducer, initialImportState } from './import-machine';
import type { CustomPackAnalysis, HostPack, ImportPreview } from './types';

const custom: CustomPackAnalysis = {
  path: 'C:/Izernix',
  valid: true,
  fileCount: 10,
  pngCount: 8,
  geometryCount: 1,
  skinCount: 2,
  warnings: []
};

const host: HostPack = {
  uuid: 'uuid',
  name: 'Host',
  filename: 'uuid',
  filepath: 'C:/cache/uuid',
  sizeBytes: 100,
  modifiedAt: null,
  sha256: 'hash',
  knownToMinecraft: true
};

const preview: ImportPreview = {
  custom,
  host,
  hostFingerprint: 'hash',
  filesToCopy: 10,
  hostAssetsToRemove: 8,
  backupWillBeCreated: true,
  warnings: []
};

describe('importReducer', () => {
  it('follows the guarded import workflow', () => {
    const selected = importReducer(initialImportState, { type: 'custom-selected', custom });
    expect(selected.phase).toBe('select-host');

    const hostSelected = importReducer(selected, { type: 'host-selected', host });
    const reviewed = importReducer(hostSelected, { type: 'preview-ready', preview });
    expect(reviewed.phase).toBe('review');

    const installing = importReducer(reviewed, { type: 'installation-started' });
    expect(installing.phase).toBe('installing');

    const done = importReducer(installing, { type: 'installation-complete' });
    expect(done.phase).toBe('complete');
  });

  it('does not lose the selected custom pack when returning from review', () => {
    const state = importReducer(
      { ...initialImportState, phase: 'review', custom, host, preview },
      { type: 'back' }
    );
    expect(state.phase).toBe('select-host');
    expect(state.custom).toBe(custom);
    expect(state.preview).toBeNull();
  });
});
