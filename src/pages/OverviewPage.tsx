import {
  ArchiveRestore,
  Boxes,
  RefreshCw,
  ShieldCheck,
  Sparkles,
  TriangleAlert,
  WandSparkles
} from 'lucide-react';
import { Link } from 'react-router-dom';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card, CardHeader } from '../components/ui/Card';
import { Page } from '../components/layout/Page';
import { useAsync } from '../hooks/useAsync';
import { api } from '../lib/tauri';

export function OverviewPage() {
  const status = useAsync(api.status, []);
  const backups = useAsync(api.backups, []);
  const cosmetics = useAsync(api.cosmeticPacks, []);

  function refresh() {
    void Promise.all([status.refresh(), backups.refresh(), cosmetics.refresh()]);
  }

  return (
    <Page
      title="Overview"
      description="A safe, local workspace for Minecraft Bedrock Persona cosmetics."
      action={
        <Button variant="secondary" icon={<RefreshCw size={15} />} onClick={refresh}>
          Refresh
        </Button>
      }
    >
      <div className="hero-card hero-card--cosmic">
        <div className="hero-card__content">
          <Badge tone={status.data?.detected ? 'success' : 'warning'}>
            {status.data?.detected ? 'Minecraft detected' : 'Minecraft not detected'}
          </Badge>
          <h2>
            Make Bedrock cosmetics <span className="gradient-text">feel effortless.</span>
          </h2>
          <p>
            Cosmify keeps your custom packs reusable, creates safety backups, rebuilds premium-cache
            hosts and verifies the result before committing a single Minecraft file.
          </p>
          <div className="hero-card__actions">
            <Link className="button button--primary button--lg" to="/cosmetics">
              <Sparkles size={16} />
              Manage cosmetics
            </Link>
            <Link className="button button--secondary button--lg" to="/backups">
              <ArchiveRestore size={16} />
              View backups
            </Link>
          </div>
        </div>
        <div className="cosmic-emblem" aria-hidden="true">
          <div className="cosmic-emblem__ring cosmic-emblem__ring--one" />
          <div className="cosmic-emblem__ring cosmic-emblem__ring--two" />
          <div className="cosmic-emblem__core">
            <WandSparkles size={40} />
          </div>
          <span className="cosmic-star cosmic-star--one">✦</span>
          <span className="cosmic-star cosmic-star--two">✧</span>
        </div>
      </div>

      <div className="stat-grid stat-grid--four">
        <Card className="stat-card">
          <Sparkles size={18} />
          <strong>{cosmetics.data?.length ?? '—'}</strong>
          <span>Managed cosmetics</span>
        </Card>
        <Card className="stat-card">
          <Boxes size={18} />
          <strong>{status.data?.hostPackCount ?? '—'}</strong>
          <span>Host packs detected</span>
        </Card>
        <Card className="stat-card">
          <ArchiveRestore size={18} />
          <strong>{backups.data?.length ?? '—'}</strong>
          <span>Safety backups</span>
        </Card>
        <Card className="stat-card">
          {status.data?.minecraftRunning ? <TriangleAlert size={18} /> : <ShieldCheck size={18} />}
          <strong>{status.data?.minecraftRunning ? 'Running' : 'Closed'}</strong>
          <span>Minecraft process</span>
        </Card>
      </div>

      <Card>
        <CardHeader title="Environment" description="What Cosmify currently sees on this PC." />
        <div className="detail-list">
          <div>
            <span>Premium cache</span>
            <code>{status.data?.premiumCachePath ?? 'Not found'}</code>
          </div>
          <div>
            <span>Skin packs</span>
            <code>{status.data?.skinPacksPath ?? 'Not found'}</code>
          </div>
          <div>
            <span>Runtime safety</span>
            <strong>
              {status.data?.minecraftRunning
                ? 'Close Minecraft before installing'
                : 'Ready for changes'}
            </strong>
          </div>
        </div>
      </Card>
    </Page>
  );
}
