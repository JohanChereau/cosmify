# Cosmify architecture

## Principles

1. **Preserve the proven Minecraft mutation path.** Library and UI features wrap the existing install pipeline rather than replacing it.
2. **Core owns trust decisions.** React selects intent; Rust validates paths, formats and mutations.
3. **Transactional filesystem work.** Rebuild in temporary storage, verify, then commit.
4. **Metadata isolation.** Cosmify-only metadata is never copied into Minecraft.
5. **UI-independent core.** `cosmify-core` has no Tauri dependency.

## Layers

### React / TypeScript

Presentation, navigation, forms and progress display. It can request folder/file pickers through Tauri's dialog plugin but cannot directly mutate Minecraft files.

### Tauri command adapter

`src-tauri/src/lib.rs` contains a narrow command surface. Expensive filesystem work runs through `spawn_blocking`; mutation commands share a process-wide lock.

### `cosmify-core`

Focused modules:

- `service.rs` — application use cases and proven host install pipeline
- `cosmetics.rs` — managed library, metadata, artwork and safe archive import
- `validation.rs` — Persona custom-pack validation and copy filtering
- `archive.rs` — host ZIP read/write and verification
- `crypto.rs` — Bedrock content encryption / verification
- `backup.rs` — backup metadata, checksums and restore source validation
- `paths.rs` — app data, legacy migration and Minecraft discovery
- `process.rs` — Minecraft process safety check
- `activity.rs` — local audit log

## Install transaction

```text
custom folder / managed library pack
        ↓
validate
        ↓
selected premium-cache host + fingerprint check
        ↓
backup original
        ↓
temporary extraction
        ↓
replace cosmetic assets
        ↓
encrypt + contents.json
        ↓
verify content
        ↓
rebuild archive
        ↓
verify host UUID
        ↓
same-directory safe replace
```

The managed library simply supplies a stable custom-folder path to this existing pipeline.

## Managed library

Each managed pack is stored under `cosmetics/<internal-id>/`. The internal id is a UUID and is validated before lookup/deletion. `.cosmify/pack.json` and `.cosmify/icon.*` are metadata only; `.cosmify` is explicitly filtered from pack analysis and installation.

## Data migration

The rename from PersonaForge to Cosmify changes the platform application-data identity. At startup, the core attempts a one-time migration of old local data and rewrites absolute app-data paths inside JSON metadata. If migration fails, it falls back to the legacy root instead of losing access to existing backups.
