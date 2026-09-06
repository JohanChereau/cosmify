<p align="center">
  <img src="src-tauri/icons/icon.png" alt="Cosmify logo" width="128" height="128">
</p>

<h1 align="center">✦ Cosmify</h1>

<p align="center">
  <strong>A local-first desktop app for managing Minecraft Bedrock Persona cosmetics.</strong>
</p>

<p align="center">
  Manage, organize, customize and safely install Persona cosmetic packs
  through Minecraft Bedrock's premium cache.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.2.1-8b5cf6?style=flat-square" alt="Version">
  <img src="https://img.shields.io/badge/platform-Windows-2563eb?style=flat-square&logo=windows11&logoColor=white" alt="Windows">
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square&logo=tauri&logoColor=white" alt="Tauri 2">
  <img src="https://img.shields.io/badge/Rust-native%20core-f97316?style=flat-square&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/React-19-06b6d4?style=flat-square&logo=react&logoColor=white" alt="React 19">
  <img src="https://img.shields.io/badge/license-MIT-22c55e?style=flat-square" alt="MIT License">
</p>

<p align="center">
  <a href="https://github.com/JohanChereau/cosmify/releases/latest"><strong>Download latest release</strong></a>
  ·
  <a href="docs/COSMIFY_PACK.md">Pack format</a>
  ·
  <a href="docs/RELEASING.md">Releasing</a>
  ·
  <a href="SECURITY.md">Security</a>
</p>

---

Cosmify combines a native Rust core with a React/Tauri interface, keeps reusable cosmetic packs in a managed local library, and treats backup, validation and verification as part of every destructive operation.

> [!NOTE]
> Cosmify is an independent community project.
> It is not affiliated with, endorsed by, or supported by Mojang Studios or Microsoft.

## ✨ Highlights

- **Managed cosmetics library** — import a folder, `.zip`, or `.mcpack` once and reuse it later.
- **Cosmify pack metadata** — optional `.cosmify/pack.json` metadata and private artwork that are never injected into Minecraft.
- **Custom pack appearance** — customize cosmetic banner gradients while keeping backward compatibility with existing packs.
- **Verified installation** — preview host changes, back up the original cache file, rebuild in a temporary workspace, verify, then commit.
- **Safe restore** — checksum-verified backups with pre-restore protection when automatic backups are enabled.
- **Host-pack visibility** — inspect Minecraft's current premium-cache skin packs and fingerprints.
- **Automatic updates** — signed updates distributed through GitHub Releases.
- **Privacy-first runtime** — no telemetry or remote UI content; network access is limited to update checks against GitHub Releases.
- **Author tools** — standalone pack encryption without modifying the plaintext source folder in place.

## 🖥️ Desktop experience

Cosmify is designed as a native-feeling Windows desktop application rather than a browser-style management tool.

The application uses:

- a custom Windows title bar;
- native Windows shadowing and rounded corners;
- local application storage;
- native Rust filesystem operations;
- signed updater artifacts;
- Windows MSI and NSIS installers.

> [!TIP]
> Screenshots of the application will be added here as the interface evolves.

## 🧱 Stack

| Layer              | Technology                       |
| ------------------ | -------------------------------- |
| Desktop shell      | **Tauri 2**                      |
| UI                 | **React 19 + TypeScript + Vite** |
| Native core        | **Rust**                         |
| Package manager    | **pnpm**                         |
| CI / Releases      | **GitHub Actions**               |
| Supported platform | **Windows**                      |

## 🚀 Installation

Download the latest Windows release from:

**[GitHub Releases](https://github.com/JohanChereau/cosmify/releases/latest)**

Available packages include:

- NSIS `.exe` installer;
- MSI installer;
- SHA-256 checksums;
- updater signatures.

> [!WARNING]
> Windows SmartScreen may display a reputation warning because Cosmify is not currently Windows code-signed.
>
> Updater signatures and Windows code signing are separate mechanisms. Cosmify updates are cryptographically signed even when SmartScreen still displays a warning.

## 🔄 Automatic updates

Starting with Cosmify `v0.2.1`, the application includes support for automatic updates through GitHub Releases.

The updater:

1. checks the latest published Cosmify release;
2. compares it with the currently installed version;
3. downloads the appropriate Windows installer;
4. verifies its cryptographic signature;
5. starts the update installation.

Updater artifacts are signed during the GitHub release workflow.

> [!IMPORTANT]
> The updater private signing key is never stored in the repository.
> GitHub Actions receives it through repository secrets during release builds.

## 🛡️ How installation stays conservative

Cosmify does not write directly into a host while rebuilding it.

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

> [!CAUTION]
> Minecraft cache contents are modified only through Cosmify's validated installation pipeline.
> Do not manually interrupt file replacement operations while they are running.

## 🎨 Cosmetics library and Cosmify pack format

Managed packs are copied under Cosmify's application-data directory.

Minecraft-compatible files remain untouched except for the isolated Cosmify metadata directory:

```text
my-pack/
├─ skins.json
├─ skin.png
├─ geometry.custom.json
└─ .cosmify/
   ├─ pack.json
   └─ icon.png       # optional
```

The entire `.cosmify` directory is excluded from:

- Minecraft installation payloads;
- encryption;
- cosmetic analysis counts.

Existing packs without newer optional metadata remain compatible.

See:

- [`docs/COSMIFY_PACK.md`](docs/COSMIFY_PACK.md)
- [`schemas/cosmify-pack.schema.json`](schemas/cosmify-pack.schema.json)

## 💾 Application data

Cosmify stores local application data in the platform-local **Cosmify** data directory.

Typical contents:

```text
settings.json
activity.jsonl
backups/
cosmetics/
keys/
```

On first start after the rename, Cosmify attempts a one-time migration from the former PersonaForge data directory.

If migration cannot be completed safely, Cosmify keeps using the legacy directory rather than making existing backups disappear.

### Premium-cache discovery order

1. Settings override
2. `COSMIFY_PREMIUM_CACHE`
3. legacy `PERSONAFORGE_PREMIUM_CACHE`
4. current GDK `%APPDATA%\Minecraft Bedrock\premium_cache`
5. legacy UWP cache path

## 🔐 Security

The frontend is treated as presentation code, not as a trusted filesystem client.

Paths selected in the UI are validated again in Rust before mutations.

The Tauri CSP blocks remote scripts and remote UI content, and the application does not expose unrestricted filesystem access to the WebView.

Network access is limited to the signed updater workflow.

> [!IMPORTANT]
> Never report private keys, credentials or sensitive cache contents in a public GitHub issue.

Please read [`SECURITY.md`](SECURITY.md) before reporting a vulnerability.

## 🛠️ Development requirements

- Windows 11 or Windows 10
- Node.js 22+
- pnpm 10.34.5+
- Rust 1.98.0 toolchain, pinned by `rust-toolchain.toml`
- Visual Studio Build Tools 2022 with:
  - Desktop development with C++
  - MSVC v143 x64/x86
  - Windows 11 SDK

For Windows, the **x64 Native Tools Command Prompt for VS 2022** is the most reliable shell for local Tauri builds because it initializes the MSVC and Windows SDK paths.

The repository pins pnpm and explicitly allows only the required `esbuild` lifecycle script in `pnpm-workspace.yaml`.

Other dependency build scripts stay blocked unless reviewed.

### Start development

```powershell
pnpm install
pnpm tauri:dev
```

Or run the bootstrap diagnostics first:

```powershell
.\scripts\bootstrap.ps1
```

## ✅ Quality checks

```powershell
pnpm check:version
pnpm format:check
pnpm check

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For complete local release validation:

```powershell
pnpm release:check
```

## ⚙️ CI

Pull requests and pushes to `main` are validated by GitHub Actions.

The CI currently performs:

- frontend formatting, linting, type checking and tests;
- Rust formatting, Clippy and tests;
- Windows desktop compilation;
- dependency review.

Frontend output is reused during Windows validation to avoid unnecessary duplicate builds.

## 📦 GitHub releases

Pushing a version tag such as:

```text
vX.Y.Z
```

triggers `.github/workflows/release.yml`.

The release workflow:

- verifies version consistency;
- verifies updater signing configuration;
- runs frontend and Rust checks;
- builds Windows MSI + NSIS artifacts;
- signs updater artifacts;
- generates `latest.json`;
- creates a **draft** GitHub Release;
- attaches `SHA256SUMS.txt`.

The draft can then be tested before publication.

See [`docs/RELEASING.md`](docs/RELEASING.md).

## 🗂️ Project layout

```text
cosmify/
├─ src/                         React / TypeScript UI
├─ src-tauri/                   Tauri shell and IPC commands
├─ crates/cosmify-core/         UI-independent Rust engine
├─ docs/                        Format, development and release notes
├─ schemas/                     Cosmify metadata schemas
├─ scripts/                     Local setup / validation helpers
└─ .github/                     CI, releases and contribution templates
```

## 🤝 Contributing

Contributions are welcome.

Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) before opening a pull request.

## 📄 License

Cosmify is released under the MIT License.

See [`LICENSE`](LICENSE).
