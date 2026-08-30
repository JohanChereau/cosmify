# Cosmify pack metadata v1

Cosmify can manage ordinary compatible Persona cosmetic folders without any metadata. The optional `.cosmify` directory adds library-only information without changing what gets installed into Minecraft.

## Layout

```text
pack-root/
├─ skins.json
├─ *.png
├─ *geometry*.json
└─ .cosmify/
   ├─ pack.json
   └─ icon.png | icon.jpg | icon.webp
```

`.cosmify/` is always excluded from the Minecraft copy/encryption pipeline.

## `pack.json`

Schema version 1. A machine-readable JSON Schema is available at [`schemas/cosmify-pack.schema.json`](../schemas/cosmify-pack.schema.json).

```json
{
  "schemaVersion": 1,
  "id": "5cb29f35-1734-4d7c-9ecf-13bd3db32b37",
  "name": "Night Drive",
  "description": "Neon Persona cosmetics.",
  "author": "Example",
  "uuid": "7617db64-b974-4e25-b429-e6a6a92f8668",
  "version": "1.0.0",
  "icon": "icon.png",
  "createdAt": "2026-08-30T10:00:00Z",
  "updatedAt": "2026-08-30T10:00:00Z"
}
```

### Identity fields

- `id` is an immutable internal library identifier used as the managed directory name.
- `uuid` is editable pack metadata for authors and future Cosmify interoperability. It does **not** replace the selected Minecraft host UUID during installation.
- `version` is intentionally a string so authors can use semantic versions or simple labels.

## Artwork

Artwork is optional and limited to PNG, JPEG or WebP up to 1 MiB. Cosmify displays it as a library thumbnail through a local data URL. It is not exposed through a broad filesystem asset scope.

When an imported source has a root `pack_icon.png`, Cosmify may copy it into `.cosmify/` as initial artwork. The source `pack_icon.png` itself remains part of the source pack and follows the existing installation behavior.

## Import behavior

Cosmify accepts:

- a folder containing root `skins.json`;
- a ZIP-compatible archive containing root `skins.json`;
- an archive containing exactly one top-level directory with `skins.json` inside it.

Archive extraction rejects path traversal and symbolic links and applies entry-count / expanded-size limits.
