# Contributing to Cosmify

Thanks for contributing.

## Core rules

- Keep `cosmify-core` independent from Tauri and React.
- Do not weaken host-path validation, fingerprint checks, backup behavior or archive verification for convenience.
- Treat all imported pack files and metadata as untrusted.
- Keep `.cosmify` metadata isolated from the Minecraft payload.
- Add tests for new filesystem or format behavior.
- Avoid adding runtime network access without an explicit design/security review.

## Before opening a pull request

```powershell
pnpm release:check
```

On Windows, use the x64 Native Tools environment if Cargo cannot find MSVC / Windows SDK libraries.

## Style

The UI intentionally favors calm spacing, restrained surfaces and a small cosmic accent palette over a heavily themed gaming interface. New UI should remain accessible in both light and dark appearance.
