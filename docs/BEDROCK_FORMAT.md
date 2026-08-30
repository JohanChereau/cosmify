# Bedrock premium-cache notes

This document records only the behavior Cosmify currently depends on.

## Host packs

Minecraft Bedrock GDK stores premium-cache skin packs under:

```text
%APPDATA%\Minecraft Bedrock\premium_cache\skin_packs
```

The pack files may have no extension but are ZIP-compatible archives.

Cosmify keeps the host `manifest.json` and replaces cosmetic assets such as:

- PNG textures (except `pack_icon.png`)
- root `skins.json`
- geometry JSON files whose filename contains `geometry`

## `contents.json`

The observed format uses:

- a 0x100-byte header
- magic value `0x9BCFB9FC`
- host UUID embedded in the header
- AES-256-CFB8 encrypted JSON payload
- per-file 32-character keys for encrypted assets

Files such as the manifest, `contents.json`, `signatures.json`, `pack_icon.png` and `texts` are not encrypted by this workflow.

## Compatibility warning

This is an implementation detail of current Bedrock behavior, not a stable public API. Minecraft updates may invalidate assumptions. Cosmify therefore treats verification and backup as mandatory architectural concerns rather than optional conveniences.
