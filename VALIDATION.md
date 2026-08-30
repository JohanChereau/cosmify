# Validation status

## Runtime behavior already validated manually

The original 0.1 install pipeline was successfully tested against a real Minecraft Bedrock GDK premium cache:

- host discovery;
- custom Persona pack analysis;
- install into an existing premium-cache host;
- custom skins, capes and geometry rendering in Minecraft;
- automatic backup creation;
- restoration of the original host pack.

Cosmify 0.2 intentionally preserves that host mutation pipeline and adds the managed library around it.

## Automated coverage

Rust tests cover:

- AES/CFB8 helpers and Bedrock encrypted-content behavior;
- ZIP round trips;
- full import pipeline with backup + restore;
- rejection when a host changes after preview;
- managed cosmetics folder import / metadata update / deletion;
- unsafe archive traversal rejection;
- exclusion of `.cosmify` metadata from Minecraft copy analysis.

Frontend tests cover utility formatting and the installer state machine.

## Required release gates

GitHub CI runs:

- Prettier check;
- ESLint;
- TypeScript build/typecheck;
- Vitest;
- Rustfmt;
- Clippy with warnings denied;
- workspace Rust tests;
- full Windows Tauri build.

Tag releases rerun the relevant checks before producing draft artifacts.
