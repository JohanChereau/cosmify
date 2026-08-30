import { useState } from 'react';
import { ArchiveRestore, RotateCcw, Trash2 } from 'lucide-react';
import { Page } from '../components/layout/Page';
import { Button } from '../components/ui/Button';
import { Card } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { useAsync } from '../hooks/useAsync';
import { api } from '../lib/tauri';
import { formatBytes, formatDate } from '../lib/format';

export function BackupsPage() {
  const backups = useAsync(api.backups, []);
  const [undoBackupId, setUndoBackupId] = useState<string | null>(null);
  const [restoreMessage, setRestoreMessage] = useState('');

  async function restore(id: string) {
    if (!window.confirm('Restore this snapshot? Cosmify will save the current state first when possible.')) return;
    const result = await api.restoreBackup(id);
    setUndoBackupId(result.backupId ?? null);
    setRestoreMessage(
      result.backupId
        ? 'Snapshot restored. The state that was active just before the restore was saved automatically.'
        : 'Snapshot restored.'
    );
    await backups.refresh();
  }

  async function undoLastRestore() {
    if (!undoBackupId) return;
    const result = await api.restoreBackup(undoBackupId);
    setUndoBackupId(result.backupId ?? null);
    setRestoreMessage('Last restore undone. Cosmify kept the previous state as another snapshot.');
    await backups.refresh();
  }

  async function remove(id: string) {
    if (!window.confirm('Delete this backup permanently?')) return;
    await api.deleteBackup(id);
    if (undoBackupId === id) setUndoBackupId(null);
    await backups.refresh();
  }

  return (
    <Page title="Backups" description="Every destructive operation can be rolled back from here.">
      {restoreMessage ? (
        <Card className="list-card">
          <div className="list-card__main">
            <div className="list-card__icon"><RotateCcw size={18} /></div>
            <div>
              <strong>Restore completed safely</strong>
              <span>{restoreMessage}</span>
            </div>
          </div>
          {undoBackupId ? (
            <Button variant="secondary" size="sm" icon={<RotateCcw size={14} />} onClick={() => void undoLastRestore()}>
              Undo last restore
            </Button>
          ) : null}
        </Card>
      ) : null}

      {!backups.data?.length ? (
        <Card>
          <EmptyState
            icon={<ArchiveRestore size={24} />}
            title="No backups yet"
            description="Cosmify will create one automatically before the first installation or removal."
          />
        </Card>
      ) : (
        <div className="list-stack">
          {backups.data.map((backup) => (
            <Card key={backup.id} className="list-card">
              <div className="list-card__main">
                <div className="list-card__icon"><ArchiveRestore size={18} /></div>
                <div>
                  <strong>{backup.packName}</strong>
                  <span>{formatDate(backup.createdAt)} · {formatBytes(backup.sizeBytes)} · {reasonLabel(backup.reason)}</span>
                  <code>{backup.originalPath}</code>
                </div>
              </div>
              <div className="inline-actions">
                <Button variant="secondary" size="sm" icon={<RotateCcw size={14} />} onClick={() => void restore(backup.id)}>
                  Restore snapshot
                </Button>
                <Button variant="ghost" size="sm" icon={<Trash2 size={14} />} onClick={() => void remove(backup.id)}>
                  Delete
                </Button>
              </div>
            </Card>
          ))}
        </div>
      )}
    </Page>
  );
}

function reasonLabel(reason: string) {
  switch (reason) {
    case 'install': return 'Before install';
    case 'remove': return 'Before removal';
    case 'pre-restore': return 'Before restore · undo snapshot';
    default: return reason;
  }
}
