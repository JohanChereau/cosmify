import type { CustomPackAnalysis, HostPack, ImportPreview, ProgressUpdate } from './types';

export type ImportPhase = 'select-custom' | 'select-host' | 'review' | 'installing' | 'complete';

export interface ImportState {
  phase: ImportPhase;
  custom: CustomPackAnalysis | null;
  host: HostPack | null;
  preview: ImportPreview | null;
  progress: ProgressUpdate | null;
  error: string | null;
}

export type ImportAction =
  | { type: 'custom-selected'; custom: CustomPackAnalysis }
  | { type: 'host-selected'; host: HostPack }
  | { type: 'preview-ready'; preview: ImportPreview }
  | { type: 'installation-started' }
  | { type: 'progress'; progress: ProgressUpdate }
  | { type: 'installation-complete' }
  | { type: 'error'; message: string }
  | { type: 'reset' }
  | { type: 'back' };

export const initialImportState: ImportState = {
  phase: 'select-custom',
  custom: null,
  host: null,
  preview: null,
  progress: null,
  error: null
};

export function importReducer(state: ImportState, action: ImportAction): ImportState {
  switch (action.type) {
    case 'custom-selected':
      return {
        ...initialImportState,
        phase: 'select-host',
        custom: action.custom
      };
    case 'host-selected':
      return { ...state, host: action.host, preview: null, error: null };
    case 'preview-ready':
      return { ...state, phase: 'review', preview: action.preview, error: null };
    case 'installation-started':
      return { ...state, phase: 'installing', progress: null, error: null };
    case 'progress':
      return { ...state, progress: action.progress };
    case 'installation-complete':
      return { ...state, phase: 'complete', error: null };
    case 'error':
      return { ...state, error: action.message };
    case 'reset':
      return initialImportState;
    case 'back':
      if (state.phase === 'select-host') return initialImportState;
      if (state.phase === 'review') {
        return { ...state, phase: 'select-host', preview: null, error: null };
      }
      return state;
  }
}
