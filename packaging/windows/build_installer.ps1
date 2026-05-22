$ErrorActionPreference = "Stop"

$Root = Resolve-Path "$PSScriptRoot\..\.."
Push-Location "$Root\rust_core"
cargo build --release
Pop-Location

Push-Location "$Root\flutter_app"
flutter build windows --release
Pop-Location

$Output = "$Root\flutter_app\build\windows\x64\runner\Release"
Copy-Item "$Root\rust_core\target\release\keylesspass_core.dll" "$Output\" -Force

Write-Host "Windows output: flutter_app\build\windows\x64\runner\Release"
Write-Host "Use WiX Toolset or Inno Setup to produce MSI/EXE installers."
