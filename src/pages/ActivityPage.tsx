import { CheckCircle2, ScrollText, XCircle } from 'lucide-react';
import { Page } from '../components/layout/Page';
import { Card } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { useAsync } from '../hooks/useAsync';
import { api } from '../lib/tauri';
import { formatDate } from '../lib/format';

export function ActivityPage() {
  const activity = useAsync(api.activity, []);
  return <Page title="Activity" description="A local audit trail of installs, restores and removals.">
    {!activity.data?.length ? <Card><EmptyState icon={<ScrollText size={24} />} title="Nothing to report" description="Successful and failed operations will appear here." /></Card> : <Card><div className="timeline">{activity.data.map((item) => <div className="timeline__item" key={item.id}>{item.success ? <CheckCircle2 size={17} /> : <XCircle size={17} />}<div><strong>{item.title}</strong><span>{item.detail}</span><small>{formatDate(item.timestamp)}</small></div></div>)}</div></Card>}
  </Page>;
}
