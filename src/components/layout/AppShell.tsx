import {
  ArchiveRestore,
  Boxes,
  LayoutDashboard,
  ScrollText,
  Settings,
  Sparkles,
  WandSparkles,
  Wrench
} from 'lucide-react';
import { useEffect } from 'react';
import { NavLink, Outlet } from 'react-router-dom';
import { api } from '../../lib/tauri';
import { applyTheme } from '../../lib/theme';
import { TitleBar } from './TitleBar';
import { UpdateBanner } from '../updates/UpdateBanner';

const navigation = [
  { to: '/', label: 'Overview', icon: LayoutDashboard, end: true },
  { to: '/cosmetics', label: 'Cosmetics', icon: Sparkles },
  { to: '/hosts', label: 'Host packs', icon: Boxes },
  { to: '/backups', label: 'Backups', icon: ArchiveRestore },
  { to: '/activity', label: 'Activity', icon: ScrollText },
  { to: '/tools', label: 'Tools', icon: Wrench },
  { to: '/settings', label: 'Settings', icon: Settings }
];

export function AppShell() {
  useEffect(() => {
    void api
      .settings()
      .then((settings) => applyTheme(settings.theme))
      .catch(() => undefined);
  }, []);

  return (
    <div className="desktop-frame">
      <TitleBar />
      <div className="app-shell">
        <aside className="sidebar">
          <div className="brand">
            <div className="brand__mark">
              <WandSparkles size={17} />
            </div>
            <div>
              <strong>Cosmify</strong>
              <span>Bedrock cosmetics</span>
            </div>
          </div>
          <nav className="sidebar__nav">
            {navigation.map(({ to, label, icon: Icon, end }) => (
              <NavLink
                key={to}
                to={to}
                end={end}
                className={({ isActive }) => `nav-item ${isActive ? 'nav-item--active' : ''}`}
              >
                <Icon size={17} strokeWidth={1.9} />
                <span>{label}</span>
              </NavLink>
            ))}
          </nav>
          <div className="sidebar__footer">
            <span className="status-dot" />
            <span>Local only · no telemetry</span>
          </div>
        </aside>
        <main className="content">
          <Outlet />
        </main>
      </div>
      <UpdateBanner />
    </div>
  );
}
