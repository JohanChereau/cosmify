import { check, type Update } from '@tauri-apps/plugin-updater';
import { Download, LoaderCircle, X } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { Button } from '../ui/Button';

type UpdatePhase = 'available' | 'downloading' | 'error';

export function UpdateBanner() {
  const updateRef = useRef<Update | null>(null);
  const [phase, setPhase] = useState<UpdatePhase | null>(null);
  const [version, setVersion] = useState('');
  const [progress, setProgress] = useState<number | null>(null);
  const [message, setMessage] = useState('');

  useEffect(() => {
    if (import.meta.env.DEV) return;

    let cancelled = false;
    void check({ timeout: 8_000 })
      .then((update) => {
        if (!update) return;
        if (cancelled) {
          void update.close();
          return;
        }
        updateRef.current = update;
        setVersion(update.version);
        setPhase('available');
      })
      .catch(() => undefined);

    return () => {
      cancelled = true;
      if (updateRef.current) void updateRef.current.close();
      updateRef.current = null;
    };
  }, []);

  function dismiss() {
    if (phase === 'downloading') return;
    if (updateRef.current) void updateRef.current.close();
    updateRef.current = null;
    setPhase(null);
  }

  async function install() {
    const update = updateRef.current;
    if (!update) return;

    let downloaded = 0;
    let contentLength = 0;
    setPhase('downloading');
    setMessage('Downloading update…');
    setProgress(null);

    try {
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength ?? 0;
            downloaded = 0;
            setProgress(contentLength > 0 ? 0 : null);
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            if (contentLength > 0) {
              setProgress(Math.min(100, Math.round((downloaded / contentLength) * 100)));
            }
            break;
          case 'Finished':
            setProgress(100);
            setMessage('Installing update…');
            break;
        }
      });
    } catch (error) {
      setPhase('error');
      setProgress(null);
      setMessage(String(error));
    }
  }

  if (!phase) return null;

  return (
    <aside className={`update-banner ${phase === 'error' ? 'update-banner--error' : ''}`}>
      <div className="update-banner__icon">
        {phase === 'downloading' ? (
          <LoaderCircle className="spin" size={17} />
        ) : (
          <Download size={17} />
        )}
      </div>
      <div className="update-banner__copy">
        <strong>{phase === 'error' ? 'Update failed' : `Cosmify v${version} is available`}</strong>
        <span>
          {phase === 'available'
            ? 'A signed update is ready to install.'
            : phase === 'downloading'
              ? message
              : message || 'Try again later.'}
        </span>
        {phase === 'downloading' && progress !== null ? (
          <div className="update-banner__progress" aria-label={`Update download ${progress}%`}>
            <span style={{ width: `${progress}%` }} />
          </div>
        ) : null}
      </div>
      {phase === 'available' ? (
        <Button size="sm" onClick={() => void install()}>
          Update now
        </Button>
      ) : null}
      {phase !== 'downloading' ? (
        <button
          type="button"
          className="update-banner__dismiss"
          aria-label="Dismiss"
          onClick={dismiss}
        >
          <X size={14} />
        </button>
      ) : null}
    </aside>
  );
}
