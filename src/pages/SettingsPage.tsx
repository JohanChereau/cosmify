import { useEffect, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { FolderOpen, Save } from 'lucide-react';
import { Page } from '../components/layout/Page';
import { Button } from '../components/ui/Button';
import { Card, CardHeader } from '../components/ui/Card';
import { Toggle } from '../components/ui/Toggle';
import { api } from '../lib/tauri';
import { applyTheme } from '../lib/theme';
import type { AppSettings } from '../lib/types';

const defaults: AppSettings = {
  premiumCacheOverride: null,
  keysDirectory: null,
  autoBackup: true,
  requireMinecraftClosed: true,
  theme: 'system'
};

export function SettingsPage() {
  const [settings, setSettings] = useState<AppSettings>(defaults);
  const [saved, setSaved] = useState(false);
  useEffect(() => {
    void api.settings().then((value) => {
      setSettings(value);
      applyTheme(value.theme);
    });
  }, []);
  async function choose(field: 'premiumCacheOverride' | 'keysDirectory') {
    const value = await open({ directory: true, multiple: false });
    if (value && !Array.isArray(value)) setSettings((current) => ({ ...current, [field]: value }));
  }
  async function save() {
    const next = await api.saveSettings(settings);
    setSettings(next);
    applyTheme(next.theme);
    setSaved(true);
    window.setTimeout(() => setSaved(false), 1600);
  }
  function setTheme(theme: 'system' | 'light' | 'dark') {
    const next = { ...settings, theme };
    setSettings(next);
    applyTheme(theme);
  }
  return (
    <Page
      title="Settings"
      description="Safety-first defaults, with advanced paths when automatic discovery is not enough."
      action={
        <Button icon={<Save size={15} />} onClick={() => void save()}>
          {saved ? 'Saved' : 'Save changes'}
        </Button>
      }
    >
      <Card>
        <CardHeader title="Safety" description="Recommended for normal use." />
        <Toggle
          checked={settings.autoBackup}
          onChange={(autoBackup) => setSettings({ ...settings, autoBackup })}
          label="Automatic backups"
          description="Create a verified copy before every install, removal and restore."
        />
        <Toggle
          checked={settings.requireMinecraftClosed}
          onChange={(requireMinecraftClosed) =>
            setSettings({ ...settings, requireMinecraftClosed })
          }
          label="Require Minecraft to be closed"
          description="Prevents cache corruption caused by concurrent writes."
        />
      </Card>
      <Card>
        <CardHeader
          title="Appearance"
          description="Keep Cosmify calm and neutral, with cosmic accents where they matter."
        />
        <div className="setting-row">
          <div>
            <div className="setting-row__label">Theme</div>
            <div className="setting-row__description">
              Follow Windows or choose a fixed appearance.
            </div>
          </div>
          <div className="segmented">
            {(['system', 'light', 'dark'] as const).map((theme) => (
              <button
                type="button"
                key={theme}
                className={settings.theme === theme ? 'segmented__active' : ''}
                onClick={() => setTheme(theme)}
              >
                {theme[0].toUpperCase() + theme.slice(1)}
              </button>
            ))}
          </div>
        </div>
      </Card>
      <Card>
        <CardHeader
          title="Advanced paths"
          description="Leave these empty to use automatic detection."
        />
        <PathField
          label="Premium cache override"
          value={settings.premiumCacheOverride ?? ''}
          onClear={() => setSettings({ ...settings, premiumCacheOverride: null })}
          onBrowse={() => void choose('premiumCacheOverride')}
        />
        <PathField
          label="Content keys directory"
          value={settings.keysDirectory ?? ''}
          onClear={() => setSettings({ ...settings, keysDirectory: null })}
          onBrowse={() => void choose('keysDirectory')}
        />
      </Card>
      <Card>
        <CardHeader title="Privacy" />
        <p className="muted-copy">
          Cosmify contains no telemetry or remote content. The runtime only reads and writes local
          Minecraft and Cosmify data. Release downloads are handled by GitHub, outside the app.
        </p>
      </Card>
    </Page>
  );
}

function PathField({
  label,
  value,
  onBrowse,
  onClear
}: {
  label: string;
  value: string;
  onBrowse: () => void;
  onClear: () => void;
}) {
  return (
    <div className="path-field">
      <label>{label}</label>
      <div>
        <code>{value || 'Automatic'}</code>
        <Button variant="secondary" size="sm" icon={<FolderOpen size={14} />} onClick={onBrowse}>
          Browse
        </Button>
        {value ? (
          <Button variant="ghost" size="sm" onClick={onClear}>
            Clear
          </Button>
        ) : null}
      </div>
    </div>
  );
}
