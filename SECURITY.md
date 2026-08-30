# Security policy

Cosmify is a local-first utility that intentionally modifies Minecraft Bedrock premium-cache files. The project treats every path arriving from the UI or an imported archive as untrusted input.

## Supported versions

Security fixes are provided for the latest published release.

## Reporting a vulnerability

Please use the repository's **GitHub Security Advisory / private vulnerability reporting** feature when available. Do not publish exploitable details in a public issue before maintainers have had time to investigate.

Useful reports include:

- affected Cosmify version;
- the smallest reproducible input or archive structure;
- expected vs actual security boundary;
- whether arbitrary file read/write, path traversal, code execution, or data exposure is possible.

Do **not** send Microsoft account credentials, session tokens, private content keys, or unrelated Minecraft cache data.

## Security boundaries

- The WebView has no generic filesystem plugin permission.
- User-selected paths are validated again by the Rust core.
- Host mutations are restricted to Minecraft's detected `premium_cache/skin_packs` directory.
- Managed library deletions are restricted to UUID-named directories inside Cosmify's own library root.
- Imported archives reject unsafe paths and symbolic links and enforce extraction limits.
- Backups are checksum-verified before restore.
- The default Content Security Policy disallows remote scripts and remote content.
- Cosmify ships without telemetry or an in-app network updater.

## Release trust

Source availability and CI do not replace platform code signing. Public Windows installers should be signed when a suitable certificate is available. GitHub release artifacts include SHA-256 checksums so users can verify downloaded installers.
