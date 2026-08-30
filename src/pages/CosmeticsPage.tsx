import { useCallback, useEffect, useReducer, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import {
  Archive,
  CheckCircle2,
  ChevronLeft,
  FolderOpen,
  ImagePlus,
  LoaderCircle,
  PackageCheck,
  Pencil,
  Plus,
  ShieldCheck,
  Sparkles,
  Trash2,
  TriangleAlert,
  Upload
} from 'lucide-react';
import { Page } from '../components/layout/Page';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card, CardHeader } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { Modal } from '../components/ui/Modal';
import { TextAreaField, TextField } from '../components/ui/TextField';
import { api } from '../lib/tauri';
import { formatBytes } from '../lib/format';
import { importReducer, initialImportState } from '../lib/import-machine';
import type { CosmeticPack, HostPack, UpdateCosmeticPackInput } from '../lib/types';

export function CosmeticsPage() {
  const [state, dispatch] = useReducer(importReducer, initialImportState);
  const [hosts, setHosts] = useState<HostPack[]>([]);
  const [library, setLibrary] = useState<CosmeticPack[]>([]);
  const [selectedLibraryPack, setSelectedLibraryPack] = useState<CosmeticPack | null>(null);
  const [editing, setEditing] = useState<CosmeticPack | null>(null);
  const [busy, setBusy] = useState(false);
  const [libraryBusy, setLibraryBusy] = useState(false);
  const [libraryError, setLibraryError] = useState('');
  const [notice, setNotice] = useState('');

  const refreshLibrary = useCallback(async () => {
    setLibrary(await api.cosmeticPacks());
  }, []);

  useEffect(() => {
    void refreshLibrary().catch((error) => setLibraryError(String(error)));
  }, [refreshLibrary]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void api
      .onProgress((progress) => dispatch({ type: 'progress', progress }))
      .then((fn) => {
        unlisten = fn;
      });
    return () => unlisten?.();
  }, []);

  useEffect(() => {
    if (!notice) return;
    const timer = window.setTimeout(() => setNotice(''), 2200);
    return () => window.clearTimeout(timer);
  }, [notice]);

  async function selectPackPath(path: string, managed: CosmeticPack | null) {
    try {
      setBusy(true);
      const [custom, hostPacks] = await Promise.all([api.analyzeCustomPack(path), api.hostPacks()]);
      setHosts(hostPacks);
      setSelectedLibraryPack(managed);
      dispatch({ type: 'custom-selected', custom });
      window.requestAnimationFrame(() =>
        document.querySelector('#installer')?.scrollIntoView({ behavior: 'smooth', block: 'start' })
      );
    } catch (error) {
      dispatch({ type: 'error', message: String(error) });
    } finally {
      setBusy(false);
    }
  }

  async function chooseCustom() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Choose your Persona cosmetics folder'
    });
    if (!selected || Array.isArray(selected)) return;
    await selectPackPath(selected, null);
  }

  async function importToLibrary(kind: 'folder' | 'archive') {
    const selected = await open(
      kind === 'folder'
        ? { directory: true, multiple: false, title: 'Import a cosmetics folder into Cosmify' }
        : {
            directory: false,
            multiple: false,
            title: 'Import a cosmetics archive into Cosmify',
            filters: [{ name: 'Cosmetics archives', extensions: ['zip', 'mcpack'] }]
          }
    );
    if (!selected || Array.isArray(selected)) return;
    try {
      setLibraryBusy(true);
      setLibraryError('');
      const imported = await api.importCosmeticPack(selected);
      await refreshLibrary();
      setNotice(`${imported.name} added to your library`);
    } catch (error) {
      setLibraryError(String(error));
    } finally {
      setLibraryBusy(false);
    }
  }

  async function preview() {
    if (!state.custom || !state.host) return;
    try {
      setBusy(true);
      const value = await api.previewImport(state.custom.path, state.host.filepath);
      dispatch({ type: 'preview-ready', preview: value });
    } catch (error) {
      dispatch({ type: 'error', message: String(error) });
    } finally {
      setBusy(false);
    }
  }

  async function install() {
    if (!state.preview) return;
    try {
      dispatch({ type: 'installation-started' });
      await api.installPack(
        state.preview.custom.path,
        state.preview.host.filepath,
        state.preview.hostFingerprint
      );
      dispatch({ type: 'installation-complete' });
    } catch (error) {
      dispatch({ type: 'error', message: String(error) });
    }
  }

  async function savePack(values: UpdateCosmeticPackInput) {
    try {
      setLibraryBusy(true);
      setLibraryError('');
      const updated = await api.updateCosmeticPack(values);
      setEditing(updated);
      await refreshLibrary();
      setNotice('Pack details updated');
    } catch (error) {
      setLibraryError(String(error));
      throw error;
    } finally {
      setLibraryBusy(false);
    }
  }

  async function choosePackIcon(pack: CosmeticPack) {
    const selected = await open({
      directory: false,
      multiple: false,
      title: 'Choose pack artwork',
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] }]
    });
    if (!selected || Array.isArray(selected)) return;
    try {
      setLibraryBusy(true);
      const updated = await api.setCosmeticPackIcon(pack.id, selected);
      setEditing(updated);
      await refreshLibrary();
      setNotice('Pack artwork updated');
    } catch (error) {
      setLibraryError(String(error));
    } finally {
      setLibraryBusy(false);
    }
  }

  async function deletePack(pack: CosmeticPack) {
    if (
      !window.confirm(
        `Delete “${pack.name}” from your Cosmify library?\n\nThis only removes Cosmify's managed copy. It does not touch Minecraft or your original source folder.`
      )
    )
      return;
    try {
      setLibraryBusy(true);
      await api.deleteCosmeticPack(pack.id);
      setEditing(null);
      await refreshLibrary();
      setNotice(`${pack.name} removed from your library`);
    } catch (error) {
      setLibraryError(String(error));
    } finally {
      setLibraryBusy(false);
    }
  }

  return (
    <Page
      title="Cosmetics"
      description="Keep reusable Persona packs in one place, then install them through the same verified workflow."
      action={
        <div className="header-actions">
          <Button
            variant="secondary"
            icon={<FolderOpen size={15} />}
            disabled={libraryBusy}
            onClick={() => void importToLibrary('folder')}
          >
            Import folder
          </Button>
          <Button
            icon={<Upload size={15} />}
            disabled={libraryBusy}
            onClick={() => void importToLibrary('archive')}
          >
            Import archive
          </Button>
        </div>
      }
    >
      {notice ? (
        <div className="toast-inline">
          <CheckCircle2 size={15} />
          {notice}
        </div>
      ) : null}
      {libraryError ? (
        <div className="alert alert--danger">
          <TriangleAlert size={17} />
          <div>
            <strong>Library operation failed</strong>
            <span>{libraryError}</span>
          </div>
        </div>
      ) : null}

      <section className="library-section">
        <div className="section-heading">
          <div>
            <h2>Your cosmetics</h2>
            <p>
              Managed copies live inside Cosmify, so you can reinstall them without browsing for the
              source every time.
            </p>
          </div>
          <Badge tone="accent">
            {library.length} {library.length === 1 ? 'pack' : 'packs'}
          </Badge>
        </div>

        {library.length ? (
          <div className="cosmetic-grid">
            {library.map((pack) => (
              <article className="cosmetic-card" key={pack.id}>
                <div className="cosmetic-card__visual">
                  {pack.iconDataUrl ? (
                    <img src={pack.iconDataUrl} alt="" />
                  ) : (
                    <div className="cosmetic-card__fallback">
                      <Sparkles size={34} />
                      <span>Cosmify</span>
                    </div>
                  )}
                  <div className="cosmetic-card__glow" />
                  <Badge tone="accent">v{pack.version}</Badge>
                </div>
                <div className="cosmetic-card__body">
                  <div className="cosmetic-card__title">
                    <div>
                      <h3>{pack.name}</h3>
                      <p>{pack.description || 'Custom Bedrock Persona cosmetics'}</p>
                    </div>
                    <button
                      className="icon-button"
                      aria-label={`Edit ${pack.name}`}
                      onClick={() => setEditing(pack)}
                    >
                      <Pencil size={15} />
                    </button>
                  </div>
                  <div className="cosmetic-card__stats">
                    <span>
                      <strong>{pack.analysis.skinCount}</strong> skins
                    </span>
                    <span>
                      <strong>{pack.analysis.pngCount}</strong> textures
                    </span>
                    <span>
                      <strong>{pack.analysis.geometryCount}</strong>{' '}
                      {pack.analysis.geometryCount === 1 ? 'geometry' : 'geometries'}
                    </span>
                  </div>
                  <div className="cosmetic-card__meta">
                    <code>{shortUuid(pack.uuid)}</code>
                    {pack.author ? <span>{pack.author}</span> : <span>Local pack</span>}
                  </div>
                  <div className="cosmetic-card__actions">
                    <Button
                      size="sm"
                      icon={<Sparkles size={14} />}
                      disabled={busy}
                      onClick={() => void selectPackPath(pack.path, pack)}
                    >
                      Install
                    </Button>
                    <Button
                      variant="secondary"
                      size="sm"
                      icon={<Pencil size={13} />}
                      onClick={() => setEditing(pack)}
                    >
                      Edit
                    </Button>
                  </div>
                </div>
              </article>
            ))}
            <button
              className="cosmetic-card cosmetic-card--add"
              onClick={() => void importToLibrary('folder')}
              disabled={libraryBusy}
            >
              <div className="cosmetic-card--add__icon">
                <Plus size={23} />
              </div>
              <strong>Add another pack</strong>
              <span>Import a folder or archive</span>
            </button>
          </div>
        ) : (
          <Card className="library-empty">
            <EmptyState
              icon={<Archive size={25} />}
              title="Build your cosmetics library"
              description="Import a folder or ZIP/MCpack once. Cosmify stores a private managed copy with optional metadata and artwork; the original source stays untouched."
              action={
                <div className="empty-actions">
                  <Button
                    icon={<FolderOpen size={15} />}
                    onClick={() => void importToLibrary('folder')}
                  >
                    Import folder
                  </Button>
                  <Button
                    variant="secondary"
                    icon={<Archive size={15} />}
                    onClick={() => void importToLibrary('archive')}
                  >
                    Import archive
                  </Button>
                </div>
              }
            />
          </Card>
        )}
      </section>

      <section id="installer" className="installer-section">
        <div className="section-heading section-heading--installer">
          <div>
            <h2>Installer</h2>
            <p>
              Use a managed pack above or pick any compatible folder directly. The proven host-pack
              workflow is unchanged.
            </p>
          </div>
          <Badge tone="success">
            <ShieldCheck size={12} /> Verified workflow
          </Badge>
        </div>

        <div className="stepper">
          {['Choose pack', 'Choose host', 'Review', 'Install'].map((label, index) => (
            <div
              key={label}
              className={`step ${phaseIndex(state.phase) >= index ? 'step--active' : ''}`}
            >
              <span>{index + 1}</span>
              {label}
            </div>
          ))}
        </div>

        {state.error ? (
          <div className="alert alert--danger">
            <TriangleAlert size={17} />
            <div>
              <strong>Something went wrong</strong>
              <span>{state.error}</span>
            </div>
          </div>
        ) : null}

        {state.phase === 'select-custom' ? (
          <Card className="picker-card">
            <EmptyState
              icon={<Sparkles size={26} />}
              title="Choose a cosmetics pack"
              description="Select a managed pack above, or choose a folder containing root skins.json, textures and geometry files."
              action={
                <Button
                  icon={
                    busy ? <LoaderCircle className="spin" size={16} /> : <FolderOpen size={16} />
                  }
                  onClick={() => void chooseCustom()}
                  disabled={busy}
                >
                  Choose folder
                </Button>
              }
            />
          </Card>
        ) : null}

        {state.phase === 'select-host' && state.custom ? (
          <div className="two-column">
            <Card>
              <CardHeader title="Custom pack" action={<Badge tone="success">Valid</Badge>} />
              <div className="pack-summary">
                <div className="pack-summary__icon">
                  {selectedLibraryPack?.iconDataUrl ? (
                    <img src={selectedLibraryPack.iconDataUrl} alt="" />
                  ) : (
                    <Sparkles size={20} />
                  )}
                </div>
                <div>
                  <strong>{selectedLibraryPack?.name ?? basename(state.custom.path)}</strong>
                  <span>
                    {state.custom.skinCount} skins · {state.custom.pngCount} textures ·{' '}
                    {state.custom.geometryCount} geometries
                  </span>
                </div>
              </div>
              <Button
                variant="ghost"
                size="sm"
                icon={<ChevronLeft size={15} />}
                onClick={() => {
                  setSelectedLibraryPack(null);
                  dispatch({ type: 'back' });
                }}
              >
                Choose another
              </Button>
            </Card>
            <Card>
              <CardHeader
                title="Choose a host pack"
                description="Cosmify preserves the host manifest and replaces only cosmetic assets."
              />
              <div className="host-list">
                {hosts.map((host) => (
                  <button
                    key={host.filepath}
                    className={`host-row ${state.host?.filepath === host.filepath ? 'host-row--selected' : ''}`}
                    onClick={() => dispatch({ type: 'host-selected', host })}
                  >
                    <div>
                      <strong>{host.name}</strong>
                      <span>
                        {formatBytes(host.sizeBytes)} ·{' '}
                        {host.knownToMinecraft ? 'Known by Minecraft' : 'Cache file'}
                      </span>
                    </div>
                    {state.host?.filepath === host.filepath ? <CheckCircle2 size={18} /> : null}
                  </button>
                ))}
              </div>
              <Button
                onClick={() => void preview()}
                disabled={!state.host || busy}
                icon={
                  busy ? <LoaderCircle className="spin" size={16} /> : <PackageCheck size={16} />
                }
              >
                Review changes
              </Button>
            </Card>
          </div>
        ) : null}

        {state.phase === 'review' && state.preview ? (
          <Card>
            <CardHeader
              title="Ready to install"
              description="No changes have been made yet."
              action={
                <Badge tone="success">
                  <ShieldCheck size={13} /> Safe workflow
                </Badge>
              }
            />
            <div className="review-grid">
              <div>
                <span>Custom files</span>
                <strong>{state.preview.filesToCopy}</strong>
              </div>
              <div>
                <span>Host assets replaced</span>
                <strong>{state.preview.hostAssetsToRemove}</strong>
              </div>
              <div>
                <span>Automatic backup</span>
                <strong>{state.preview.backupWillBeCreated ? 'Enabled' : 'Disabled'}</strong>
              </div>
            </div>
            {state.preview.warnings.map((warning) => (
              <div className="alert alert--warning" key={warning.code}>
                <TriangleAlert size={16} />
                <span>{warning.message}</span>
              </div>
            ))}
            <div className="action-row">
              <Button variant="secondary" onClick={() => dispatch({ type: 'back' })}>
                Back
              </Button>
              <Button icon={<Sparkles size={16} />} onClick={() => void install()}>
                Install cosmetics
              </Button>
            </div>
          </Card>
        ) : null}

        {state.phase === 'installing' ? (
          <Card className="install-card">
            <LoaderCircle className="spin install-icon" size={30} />
            <h3>{state.progress?.label ?? 'Starting installation…'}</h3>
            <div className="progress">
              <span style={{ width: `${state.progress?.progress ?? 3}%` }} />
            </div>
            <p>
              {state.progress?.progress ?? 3}% · Minecraft files are only committed after
              verification.
            </p>
          </Card>
        ) : null}

        {state.phase === 'complete' ? (
          <Card className="install-card install-card--done">
            <CheckCircle2 className="install-icon" size={32} />
            <h3>Installed successfully</h3>
            <p>
              Restart Minecraft Bedrock and equip your cosmetics. A safety backup is available if
              you need to roll back.
            </p>
            <Button
              onClick={() => {
                setSelectedLibraryPack(null);
                dispatch({ type: 'reset' });
              }}
            >
              Install another pack
            </Button>
          </Card>
        ) : null}
      </section>

      <PackEditor
        pack={editing}
        busy={libraryBusy}
        onClose={() => setEditing(null)}
        onSave={savePack}
        onIcon={choosePackIcon}
        onDelete={deletePack}
      />
    </Page>
  );
}

function PackEditor({
  pack,
  busy,
  onClose,
  onSave,
  onIcon,
  onDelete
}: {
  pack: CosmeticPack | null;
  busy: boolean;
  onClose: () => void;
  onSave: (values: UpdateCosmeticPackInput) => Promise<void>;
  onIcon: (pack: CosmeticPack) => Promise<void>;
  onDelete: (pack: CosmeticPack) => Promise<void>;
}) {
  const [form, setForm] = useState<UpdateCosmeticPackInput | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    if (!pack) {
      setForm(null);
      setError('');
      return;
    }
    setForm({
      id: pack.id,
      name: pack.name,
      description: pack.description,
      author: pack.author,
      uuid: pack.uuid,
      version: pack.version
    });
    setError('');
  }, [pack]);

  if (!pack || !form) return null;
  const update = (key: keyof UpdateCosmeticPackInput, value: string) =>
    setForm((current) => (current ? { ...current, [key]: value } : current));

  return (
    <Modal
      open
      title="Edit cosmetic pack"
      description="Cosmify metadata lives in .cosmify/pack.json and is never injected into Minecraft."
      onClose={onClose}
      footer={
        <>
          <Button
            variant="danger"
            size="sm"
            icon={<Trash2 size={14} />}
            disabled={busy}
            onClick={() => void onDelete(pack)}
          >
            Delete pack
          </Button>
          <div className="modal__footer-spacer" />
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button
            disabled={busy}
            onClick={() =>
              void onSave(form)
                .then(onClose)
                .catch((reason) => setError(String(reason)))
            }
          >
            {busy ? 'Saving…' : 'Save changes'}
          </Button>
        </>
      }
    >
      <div className="editor-artwork">
        <div className="editor-artwork__preview">
          {pack.iconDataUrl ? <img src={pack.iconDataUrl} alt="" /> : <Sparkles size={28} />}
        </div>
        <div>
          <strong>Pack artwork</strong>
          <p>PNG, JPEG or WebP up to 1 MiB. Stored privately with the managed pack.</p>
          <Button
            variant="secondary"
            size="sm"
            icon={<ImagePlus size={14} />}
            disabled={busy}
            onClick={() => void onIcon(pack)}
          >
            Choose image
          </Button>
        </div>
      </div>
      <div className="form-grid">
        <TextField
          label="Name"
          value={form.name}
          maxLength={80}
          onChange={(event) => update('name', event.target.value)}
        />
        <TextField
          label="Version"
          value={form.version}
          maxLength={32}
          onChange={(event) => update('version', event.target.value)}
        />
        <TextField
          label="Author"
          value={form.author}
          maxLength={80}
          placeholder="Optional"
          onChange={(event) => update('author', event.target.value)}
        />
        <TextField
          label="UUID"
          value={form.uuid}
          spellCheck={false}
          onChange={(event) => update('uuid', event.target.value)}
          hint="Cosmify identity metadata; the Minecraft host UUID is preserved during installation."
        />
        <div className="form-grid__full">
          <TextAreaField
            label="Description"
            value={form.description}
            maxLength={280}
            rows={3}
            placeholder="Optional description"
            onChange={(event) => update('description', event.target.value)}
          />
        </div>
      </div>
      {error ? (
        <div className="alert alert--danger">
          <TriangleAlert size={15} />
          <span>{error}</span>
        </div>
      ) : null}
    </Modal>
  );
}

function phaseIndex(phase: string) {
  return (
    (
      { 'select-custom': 0, 'select-host': 1, review: 2, installing: 3, complete: 3 } as Record<
        string,
        number
      >
    )[phase] ?? 0
  );
}
function basename(path: string) {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? 'Cosmetic pack';
}
function shortUuid(uuid: string) {
  return uuid.length > 18 ? `${uuid.slice(0, 8)}…${uuid.slice(-6)}` : uuid;
}
