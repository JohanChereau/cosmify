import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { FileArchive, FolderOpen, LockKeyhole } from 'lucide-react';
import { Page } from '../components/layout/Page';
import { Button } from '../components/ui/Button';
import { Card, CardHeader } from '../components/ui/Card';
import { api } from '../lib/tauri';

export function ToolsPage() {
  const [input, setInput] = useState('');
  const [output, setOutput] = useState('');
  const [result, setResult] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

  async function pickInput() {
    const value = await open({
      directory: true,
      multiple: false,
      title: 'Choose a plaintext skin pack'
    });
    if (value && !Array.isArray(value)) setInput(value);
  }
  async function pickOutput() {
    const value = await open({ directory: true, multiple: false, title: 'Choose output folder' });
    if (value && !Array.isArray(value)) setOutput(value);
  }
  async function encrypt() {
    try {
      setBusy(true);
      setError('');
      setResult('');
      const value = await api.encryptPackDirectory(input, output);
      setResult(value.outputFile);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Page title="Tools" description="Low-level utilities for pack authors and troubleshooting.">
      <Card>
        <CardHeader
          title="Encrypt a standalone skin pack"
          description="Encrypt a plaintext pack in an isolated temporary workspace; the source folder is never modified in place."
          action={<LockKeyhole size={18} />}
        />
        <div className="tool-fields">
          <ToolPath label="Plaintext pack" value={input} onBrowse={() => void pickInput()} />
          <ToolPath label="Output directory" value={output} onBrowse={() => void pickOutput()} />
        </div>
        {error ? <div className="alert alert--danger">{error}</div> : null}
        {result ? (
          <div className="result-box">
            <FileArchive size={17} />
            <div>
              <strong>Encrypted pack created</strong>
              <code>{result}</code>
            </div>
          </div>
        ) : null}
        <div className="action-row">
          <Button
            disabled={!input || !output || busy}
            icon={<LockKeyhole size={15} />}
            onClick={() => void encrypt()}
          >
            {busy ? 'Encrypting…' : 'Encrypt pack'}
          </Button>
        </div>
      </Card>
    </Page>
  );
}

function ToolPath({
  label,
  value,
  onBrowse
}: {
  label: string;
  value: string;
  onBrowse: () => void;
}) {
  return (
    <div className="path-field">
      <label>{label}</label>
      <div>
        <code>{value || 'Not selected'}</code>
        <Button variant="secondary" size="sm" icon={<FolderOpen size={14} />} onClick={onBrowse}>
          Browse
        </Button>
      </div>
    </div>
  );
}
