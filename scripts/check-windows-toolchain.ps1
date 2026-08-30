$ErrorActionPreference = 'Stop'

Write-Host 'Cosmify Windows toolchain check' -ForegroundColor Magenta

foreach ($command in @('node', 'pnpm', 'rustc', 'cargo')) {
  if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
    throw "$command was not found in PATH"
  }
}

Write-Host "Node  : $(node --version)"
Write-Host "pnpm  : $(pnpm --version)"
Write-Host "Rust  : $(rustc --version)"
Write-Host "Cargo : $(cargo --version)"

$link = Get-Command link.exe -ErrorAction SilentlyContinue
if (-not $link) {
  Write-Warning 'MSVC link.exe is not visible. Open x64 Native Tools Command Prompt for VS 2022 or initialize VsDevShell with -Arch amd64 -HostArch amd64.'
} else {
  Write-Host "MSVC  : $($link.Source)"
}

if ($env:WindowsSdkDir -and $env:WindowsSDKVersion) {
  $kernel = Join-Path $env:WindowsSdkDir "Lib\$($env:WindowsSDKVersion)um\x64\kernel32.lib"
  if (Test-Path $kernel) { Write-Host "SDK   : $kernel" }
  else { Write-Warning "Windows SDK is configured but kernel32.lib was not found at $kernel" }
} else {
  Write-Warning 'Windows SDK environment variables are missing. The x64 Native Tools prompt is recommended for local Tauri builds.'
}
