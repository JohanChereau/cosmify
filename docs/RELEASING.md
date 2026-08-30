# Releasing Cosmify

## 1. Prepare the version

Keep these versions identical:

- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `crates/cosmify-core/Cargo.toml`

Validate with:

```powershell
pnpm check:version
pnpm release:check
```

Update `CHANGELOG.md` before tagging.

## 2. Tag the release

```bash
git tag v0.2.0
git push origin v0.2.0
```

The `Release` GitHub Actions workflow creates a **draft** release with Windows MSI and NSIS installers, then attaches SHA-256 checksums.

Always test at least one installer from the draft release on a clean or disposable Windows account before publishing the draft.

## 3. Windows code signing

The default workflow does not embed a private signing credential. Unsigned installers may trigger Microsoft Defender SmartScreen reputation prompts.

For a broadly distributed public release, configure a trusted Windows code-signing certificate according to the current Tauri Windows signing documentation, store credentials only in GitHub Actions secrets, and never commit certificate material to the repository.

## 4. Publish

After testing:

1. confirm the release version and changelog;
2. verify `SHA256SUMS.txt` matches the uploaded installers;
3. confirm CI is green for the release commit;
4. publish the draft release.

Cosmify deliberately has no in-app updater in v0.2.0, keeping the runtime network-free. Users obtain updates from GitHub Releases.
