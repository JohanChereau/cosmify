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
git tag -a vX.Y.Z -m "Cosmify vX.Y.Z"
git push origin vX.Y.Z
```

The `Release` GitHub Actions workflow creates a **draft** release with Windows MSI and NSIS installers, updater signatures, `latest.json`, then attaches SHA-256 checksums.

Always test at least one installer from the draft release on a clean or disposable Windows account before publishing the draft.

## 3. Updater signing

Tauri requires updater artifacts to be cryptographically signed. This signature is separate from Windows Authenticode code signing.

Generate the updater keypair once and keep the private key permanently:

```powershell
pnpm tauri signer generate -w "$env:USERPROFILE\.tauri\cosmify.key"
Get-Content -Raw "$env:USERPROFILE\.tauri\cosmify.key.pub"
```

Then:

1. copy the generated **public key content** into `plugins.updater.pubkey` in `src-tauri/tauri.conf.json`, replacing `__COSMIFY_UPDATER_PUBLIC_KEY__`;
2. create the GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY` containing either the private-key content or its supported value;
3. create the GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` containing the password used to protect the updater private key.

Never commit the private key. Losing it prevents future releases from updating already-installed updater-enabled builds.

Release builds merge `src-tauri/tauri.updater.conf.json`, which enables updater artifact generation only for the release workflow. The application checks the latest published GitHub release at startup and offers the update in-app when a newer version is available.

## 4. Windows code signing

The default workflow does not embed a private signing credential. Unsigned installers may trigger Microsoft Defender SmartScreen reputation prompts.

For a broadly distributed public release, configure a trusted Windows code-signing certificate according to the current Tauri Windows signing documentation, store credentials only in GitHub Actions secrets, and never commit certificate material to the repository.

## 5. Publish

After testing:

1. confirm the release version and changelog;
2. verify `SHA256SUMS.txt` matches the uploaded installers;
3. confirm CI is green for the release commit;
4. publish the draft release.

After publishing, confirm the release contains `latest.json` and the updater signature assets in addition to the normal installers and `SHA256SUMS.txt`.
