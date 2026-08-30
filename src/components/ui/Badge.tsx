import type { ReactNode } from 'react';

export function Badge({
  tone = 'neutral',
  children
}: {
  tone?: 'neutral' | 'success' | 'warning' | 'danger' | 'accent';
  children: ReactNode;
}) {
  return <span className={`badge badge--${tone}`}>{children}</span>;
}
