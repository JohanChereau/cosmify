# Changelog

All notable changes to Cosmify are documented here.

## [0.2.1] - 2026-08-30

### Added

- Automatic application updates through GitHub Releases.
- Customizable two-color gradients for cosmetic pack banners.
- Random and default gradient presets.

### Changed

- Added a custom Windows title bar and improved native Windows 11 window styling.
- Refined the application icon.

### Fixed

- Prevented a console window from opening alongside production builds.
- Fixed rendering artifacts on gradient buttons.

---

## [0.2.0] - 2026-08-30

### Added

- Renamed the application to **Cosmify** with new cosmic visual identity and application icon.
- Managed cosmetics library with reusable local copies.
- Folder, ZIP and MCpack library imports.
- `.cosmify/pack.json` metadata format with editable name, author, description, UUID and version.
- Optional per-pack artwork stored outside the Minecraft installation payload.
- GitHub tag-based draft release workflow with installer checksums.
- Dependency-review workflow and public issue / pull-request templates.
- Version-consistency and Windows toolchain validation scripts.

### Security

- Added archive entry and expanded-size limits for library imports.
- Rejected archive path traversal and symbolic links.
- Kept managed metadata outside the Minecraft copy/encryption pipeline.
- Tightened Tauri CSP to local content and IPC only.
- Added one-time data migration strategy for the former PersonaForge app-data directory.

### Changed

- Updated primary UI accents to a restrained violet / magenta / coral cosmic gradient while retaining the existing clean Notion/shadcn-inspired layout.
- Standardized project and crate names around Cosmify.
- Pin pnpm 10.34.5 and explicitly allow only the required esbuild install script for reproducible public CI installs.
- Pin the development/CI Rust toolchain to 1.98.0 for reproducible formatting and builds.
