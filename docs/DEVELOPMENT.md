# Development guide

## Adding a new native feature

Keep the dependency direction one-way:

```text
React -> Tauri command adapter -> cosmify-core
```

1. Add/extend DTOs in `cosmify-core/src/models.rs`.
2. Implement the use case in `cosmify-core/src/service.rs` or a focused module.
3. Add Rust fixture tests before exposing the feature.
4. Add a thin `#[tauri::command]` adapter in `src-tauri/src/lib.rs`.
5. Add the typed wrapper to `src/lib/tauri.ts`.
6. Build the UI using reusable `components/ui` primitives.

Do not move filesystem, crypto or Bedrock format logic into React.

## Adding a page

1. Create `src/pages/<Name>Page.tsx`.
2. Add its route in `src/App.tsx`.
3. Add the navigation item in `AppShell.tsx`.
4. Keep data access through `api` from `src/lib/tauri.ts`.

## Import pipeline changes

Any change to the mutation pipeline should answer all of these questions:

- What is validated before the write?
- Is the selected path restricted to the intended directory?
- What is backed up?
- Can the operation be retried safely?
- What happens if the process crashes immediately before commit?
- What happens if Minecraft changes the host between preview and install?
- How is the rebuilt archive verified?
- Which test fixture proves the new assumption?

## Bedrock compatibility

Treat the premium-cache format as version-sensitive. Prefer adding compatibility adapters rather than scattering version checks throughout the service layer.

A future versioning design can introduce:

```text
BedrockFormatAdapter
├─ CurrentGdkAdapter
├─ LegacyUwpAdapter
└─ FutureAdapter
```

without changing the Tauri command API or UI workflow.
