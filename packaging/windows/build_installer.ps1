$ErrorActionPreference = "Stop"
$FlutterBin = if ($env:FLUTTER_BIN) { $env:FLUTTER_BIN } else { "flutter" }

$Root = Resolve-Path "$PSScriptRoot\..\.."
Push-Location "$Root\rust_core"
cargo build --release
$CargoExitCode = $LASTEXITCODE
Pop-Location
if ($CargoExitCode -ne 0) {
    throw "cargo build --release failed"
}

Push-Location "$Root\flutter_app"
$WindowsBuildDir = "$Root\flutter_app\build\windows"
if (Test-Path $WindowsBuildDir) {
    Remove-Item $WindowsBuildDir -Recurse -Force
}
& $FlutterBin build windows --release
$FlutterExitCode = $LASTEXITCODE
Pop-Location
if ($FlutterExitCode -ne 0) {
    throw "flutter build windows --release failed"
}

$Output = "$Root\flutter_app\build\windows\x64\runner\Release"
$CoreDll = "$Root\rust_core\target\release\keylesspass_core.dll"
if (!(Test-Path $Output)) {
    throw "Windows Flutter release output was not created: $Output"
}
if (!(Test-Path $CoreDll)) {
    throw "Rust Core DLL was not created: $CoreDll"
}
Copy-Item $CoreDll "$Output\" -Force

Write-Host "Windows output: flutter_app\build\windows\x64\runner\Release"
Write-Host "Use WiX Toolset or Inno Setup to produce MSI/EXE installers."
