$ErrorActionPreference = 'Stop'
Write-Host 'Cosmify bootstrap' -ForegroundColor Magenta

& "$PSScriptRoot\check-windows-toolchain.ps1"

pnpm install --frozen-lockfile
pnpm ensure:dist
pnpm check:version
pnpm check
cargo test --workspace

Write-Host 'Bootstrap complete. Run: pnpm tauri:dev' -ForegroundColor Green
