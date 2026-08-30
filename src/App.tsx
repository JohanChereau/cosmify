import { HashRouter, Route, Routes } from 'react-router-dom';
import { AppShell } from './components/layout/AppShell';
import { ActivityPage } from './pages/ActivityPage';
import { BackupsPage } from './pages/BackupsPage';
import { CosmeticsPage } from './pages/CosmeticsPage';
import { HostsPage } from './pages/HostsPage';
import { OverviewPage } from './pages/OverviewPage';
import { SettingsPage } from './pages/SettingsPage';
import { ToolsPage } from './pages/ToolsPage';

export default function App() {
  return <HashRouter><Routes><Route element={<AppShell />}><Route index element={<OverviewPage />} /><Route path="cosmetics" element={<CosmeticsPage />} /><Route path="hosts" element={<HostsPage />} /><Route path="backups" element={<BackupsPage />} /><Route path="activity" element={<ActivityPage />} /><Route path="tools" element={<ToolsPage />} /><Route path="settings" element={<SettingsPage />} /></Route></Routes></HashRouter>;
}
