# ✦ Cosmify

![Version](https://img.shields.io/badge/version-0.2.0-8b5cf6?style=flat-square)
![Platform](https://img.shields.io/badge/platform-Windows-2563eb?style=flat-square&logo=windows11&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square&logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-native%20core-f97316?style=flat-square&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React-19-06b6d4?style=flat-square&logo=react&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-22c55e?style=flat-square)

**Cosmify** is a local-first Windows desktop app for managing, organizing and installing custom Minecraft Bedrock Persona cosmetics through premium-cache skin packs.

It combines a native Rust core with a React/Tauri interface, keeps reusable cosmetic packs in a managed library, and treats backup + verification as part of every destructive workflow.

> Cosmify is an independent community project. It is not affiliated with, endorsed by, or supported by Mojang Studios or Microsoft.

## Highlights

- **Managed cosmetics library** — import a folder, `.zip`, or `.mcpack` once and reuse it later.
- **Cosmify pack metadata** — optional `.cosmify/pack.json` metadata and private artwork that are never injected into Minecraft.
- **Verified installation** — preview host changes, back up the original cache file, rebuild in a temporary workspace, verify, then commit.
- **Safe restore** — checksum-verified backups with pre-restore protection when automatic backups are enabled.
- **Host-pack visibility** — inspect Minecraft's current premium-cache skin packs and fingerprints.
- **Local-only runtime** — no telemetry, remote content, or background network client.
- **Author tools** — standalone pack encryption without modifying the plaintext source folder in place.

## Stack

- **Tauri 2** — native desktop shell and narrow IPC boundary
- **React 19 + TypeScript + Vite** — UI
- **Rust** — Minecraft discovery, ZIP handling, validation, crypto, backups and filesystem mutations
- **pnpm** — frontend package manager

## Development requirements

- Windows 11 or Windows 10
- Node.js 22+
- pnpm 10.34.5+
- Rust 1.98.0 toolchain (pinned by `rust-toolchain.toml`)
- Visual Studio Build Tools 2022 with:
  - Desktop development with C++
  - MSVC v143 x64/x86
  - Windows 11 SDK

For Windows, the **x64 Native Tools Command Prompt for VS 2022** is the most reliable shell for local Tauri builds because it initializes MSVC and Windows SDK library paths.

The repository pins pnpm and explicitly allows only the required `esbuild` lifecycle script in `pnpm-workspace.yaml`; other dependency build scripts stay blocked unless reviewed.

```powershell
pnpm install
pnpm tauri:dev
```

Or run the bootstrap diagnostics first:

```powershell
.\scripts\bootstrap.ps1
```

## Quality checks

```powershell
pnpm check:version
pnpm format:check
pnpm check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The shortcut for release preparation is:

```powershell
pnpm release:check
```

## How installation stays conservative

Cosmify does not write directly into a host while it is rebuilding it.

1. Validate the custom pack and selected host.
2. Verify that the host fingerprint has not changed since preview.
3. Require Minecraft to be closed by default.
4. Create a safety backup.
5. Extract and rebuild in a temporary directory.
6. Replace only the cosmetic assets used by the known workflow.
7. Encrypt / rebuild `contents.json`.
8. Verify encrypted content and the rebuilt archive.
9. Commit via a same-directory swap with rollback behavior on failure.

The existing host `manifest.json` and UUID are preserved.

## Cosmetics library and the Cosmify pack format

Managed packs are copied under Cosmify's application-data directory. Their Minecraft-compatible files remain untouched except for the addition of an isolated metadata directory:

```text
my-pack/
├─ skins.json
├─ skin.png
├─ geometry.custom.json
└─ .cosmify/
   ├─ pack.json
   └─ icon.png       # optional
```

The entire `.cosmify` directory is excluded from pack analysis counts and from the Minecraft installation pipeline. See [`docs/COSMIFY_PACK.md`](docs/COSMIFY_PACK.md) and the checked-in [`schemas/cosmify-pack.schema.json`](schemas/cosmify-pack.schema.json).

## Application data

New versions use the platform-local **Cosmify** data directory for:

- `settings.json`
- `activity.jsonl`
- `backups/`
- `cosmetics/`
- optional `keys/`

On first start after the rename, Cosmify attempts a one-time migration from the former PersonaForge data directory. If migration cannot be completed safely, the legacy directory is used rather than making existing backups disappear.

Premium-cache discovery order:

1. Settings override
2. `COSMIFY_PREMIUM_CACHE`
3. legacy `PERSONAFORGE_PREMIUM_CACHE`
4. current GDK `%APPDATA%\Minecraft Bedrock\premium_cache`
5. legacy UWP cache path

## GitHub releases

Pushing a version tag such as `v0.2.0` triggers `.github/workflows/release.yml`.

The workflow:

- verifies version consistency;
- runs frontend and Rust checks;
- builds Windows MSI + NSIS artifacts with Tauri;
- creates a **draft** GitHub Release;
- attaches `SHA256SUMS.txt` for installer verification.

Unsigned Windows binaries can trigger SmartScreen reputation warnings even when the source is clean. Code signing is intentionally left as a repository-owner step; see [`docs/RELEASING.md`](docs/RELEASING.md).

## Security

The frontend is treated as presentation code, not as a trusted filesystem client. Paths selected in the UI are validated again in Rust before mutations. The Tauri CSP blocks remote scripts/content and the app does not expose the filesystem plugin to the WebView.

Please read [`SECURITY.md`](SECURITY.md) before reporting a vulnerability.

## Project layout

```text
cosmify/
├─ src/                         React / TypeScript UI
├─ src-tauri/                   Tauri shell and IPC commands
├─ crates/cosmify-core/         UI-independent Rust engine
├─ docs/                        Format, development and release notes
├─ scripts/                     Local setup / validation helpers
└─ .github/                     CI, releases and contribution templates
```

## License

See [`LICENSE`](LICENSE).
