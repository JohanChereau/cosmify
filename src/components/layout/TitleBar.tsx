import { Minus, Square, WandSparkles, X } from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';

export function TitleBar() {
  return (
    <header className="titlebar">
      <div className="titlebar__identity" data-tauri-drag-region>
        <WandSparkles size={13} strokeWidth={2} />
        <span>Cosmify</span>
      </div>
      <div
        className="titlebar__drag-region"
        data-tauri-drag-region
        onDoubleClick={() => void getCurrentWindow().toggleMaximize()}
      />
      <div className="titlebar__controls">
        <button
          type="button"
          className="titlebar__button"
          aria-label="Minimize"
          title="Minimize"
          onClick={() => void getCurrentWindow().minimize()}
        >
          <Minus size={13} strokeWidth={1.8} />
        </button>
        <button
          type="button"
          className="titlebar__button"
          aria-label="Maximize or restore"
          title="Maximize or restore"
          onClick={() => void getCurrentWindow().toggleMaximize()}
        >
          <Square size={10} strokeWidth={1.7} />
        </button>
        <button
          type="button"
          className="titlebar__button titlebar__button--close"
          aria-label="Close"
          title="Close"
          onClick={() => void getCurrentWindow().close()}
        >
          <X size={14} strokeWidth={1.8} />
        </button>
      </div>
    </header>
  );
}
