## What changed

Describe the user-visible and technical changes.

## Safety checklist

- [ ] Existing install / backup / restore behavior is unchanged unless intentionally modified.
- [ ] New filesystem input is validated in the Rust core.
- [ ] Tests cover the new behavior or explain why a test is not practical.
- [ ] `pnpm release:check` passes locally (or CI is green).
- [ ] No secrets, content keys, personal cache files, or generated build artifacts are committed.
