import { readFile } from 'node:fs/promises';

const packageJson = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8'));
const tauriConfig = JSON.parse(await readFile(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8'));
const tauriCargo = await readFile(new URL('../src-tauri/Cargo.toml', import.meta.url), 'utf8');
const coreCargo = await readFile(new URL('../crates/cosmify-core/Cargo.toml', import.meta.url), 'utf8');

function cargoVersion(source, label) {
  const packageSection = source.split('[package]')[1]?.split(/\n\[/)[0] ?? '';
  const match = packageSection.match(/^version\s*=\s*"([^"]+)"/m);
  if (!match) throw new Error(`Could not read version from ${label}`);
  return match[1];
}

const versions = new Map([
  ['package.json', packageJson.version],
  ['src-tauri/tauri.conf.json', tauriConfig.version],
  ['src-tauri/Cargo.toml', cargoVersion(tauriCargo, 'src-tauri/Cargo.toml')],
  ['crates/cosmify-core/Cargo.toml', cargoVersion(coreCargo, 'crates/cosmify-core/Cargo.toml')]
]);

const expected = packageJson.version;
const mismatches = [...versions].filter(([, version]) => version !== expected);
if (mismatches.length) {
  console.error(`Version mismatch; expected ${expected}:`);
  for (const [file, version] of mismatches) console.error(`- ${file}: ${version}`);
  process.exit(1);
}

const tag = process.env.COSMIFY_RELEASE_TAG?.trim();
if (tag && tag !== `v${expected}`) {
  console.error(`Release tag ${tag} does not match application version v${expected}`);
  process.exit(1);
}

console.log(`Cosmify version ${expected} is consistent across manifests.`);
