import { Boxes, RefreshCw, Trash2 } from 'lucide-react';
import { Page } from '../components/layout/Page';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { useAsync } from '../hooks/useAsync';
import { api } from '../lib/tauri';
import { formatBytes, formatDate, shortHash } from '../lib/format';

export function HostsPage() {
  const hosts = useAsync(api.hostPacks, []);
  async function remove(filepath: string, sha256: string) {
    if (!window.confirm('Remove this host pack from the premium cache? A backup will be created first when backups are enabled.')) return;
    await api.removePack(filepath, sha256); await hosts.refresh();
  }
  return <Page title="Host packs" description="ZIP-like skin packs currently available in Minecraft's premium cache." action={<Button variant="secondary" icon={<RefreshCw size={15} />} onClick={() => void hosts.refresh()}>Refresh</Button>}>
    {!hosts.data?.length ? <Card><EmptyState icon={<Boxes size={24} />} title="No host packs found" description="Open Minecraft and load cosmetics so the premium cache contains a usable host pack." /></Card> : <div className="list-stack">{hosts.data.map((host) => <Card key={host.filepath} className="list-card"><div className="list-card__main"><div className="list-card__icon"><Boxes size={18} /></div><div><div className="list-card__title"><strong>{host.name}</strong>{host.knownToMinecraft ? <Badge tone="success">Known</Badge> : <Badge>Cache</Badge>}</div><span>{formatBytes(host.sizeBytes)} · {formatDate(host.modifiedAt)} · SHA {shortHash(host.sha256)}</span><code>{host.filepath}</code></div></div><Button variant="ghost" size="sm" icon={<Trash2 size={15} />} onClick={() => void remove(host.filepath, host.sha256)}>Remove</Button></Card>)}</div>}
  </Page>;
}
