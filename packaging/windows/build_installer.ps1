$ErrorActionPreference = "Stop"
$FlutterBin = if ($env:FLUTTER_BIN) { $env:FLUTTER_BIN } else { "flutter" }

function Sign-KeyLessPassArtifact([string]$Path) {
    if (!$env:KEYLESSPASS_WINDOWS_SIGN_CERT_SHA1) { return }
    $SignTool = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if (!$SignTool) { throw "signtool.exe was not found" }
    $TimestampUrl = if ($env:KEYLESSPASS_TIMESTAMP_URL) { $env:KEYLESSPASS_TIMESTAMP_URL } else { "http://timestamp.digicert.com" }
    & $SignTool.Source sign /sha1 $env:KEYLESSPASS_WINDOWS_SIGN_CERT_SHA1 /fd SHA256 /tr $TimestampUrl /td SHA256 $Path
    if ($LASTEXITCODE -ne 0) { throw "Authenticode signing failed: $Path" }
}

$Root = Resolve-Path "$PSScriptRoot\..\.."
$Pubspec = Join-Path $Root "flutter_app\pubspec.yaml"
$VersionLine = Get-Content $Pubspec | Where-Object { $_ -match "^version:\s*(.+)$" } | Select-Object -First 1
$AppVersion = "0.1.0"
if ($VersionLine -match "^version:\s*([^\+]+)") {
    $AppVersion = $Matches[1].Trim()
}

$Iscc = $env:ISCC
if (!$Iscc) {
    $Candidates = @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
    )
    foreach ($Candidate in $Candidates) {
        if ($Candidate -and (Test-Path $Candidate)) {
            $Iscc = $Candidate
            break
        }
    }
}

if (!$Iscc) {
    throw "Inno Setup compiler was not found. Install Inno Setup 6 or set ISCC to ISCC.exe, then rerun this script."
}

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
Sign-KeyLessPassArtifact (Join-Path $Output "keylesspass_core.dll")
$AppExe = Join-Path $Output "KeyLessPass.exe"
if (!(Test-Path $AppExe)) { throw "Windows application executable was not created: $AppExe" }
Sign-KeyLessPassArtifact $AppExe

$InstallerOutput = "$Root\dist\windows"
New-Item -ItemType Directory -Force -Path $InstallerOutput | Out-Null

$env:KEYLESSPASS_APP_VERSION = $AppVersion
$env:KEYLESSPASS_RELEASE_DIR = $Output
$env:KEYLESSPASS_OUTPUT_DIR = $InstallerOutput
$env:KEYLESSPASS_ICON_FILE = "$Root\flutter_app\windows\runner\resources\app_icon.ico"
$IssFile = "$Root\packaging\windows\KeyLessPass.iss"
& $Iscc $IssFile
$IsccExitCode = $LASTEXITCODE
if ($IsccExitCode -ne 0) {
    throw "Inno Setup installer build failed"
}

$Installer = "$InstallerOutput\KeyLessPass-Setup-$AppVersion.exe"
if (!(Test-Path $Installer)) {
    throw "Windows installer was not created: $Installer"
}
Sign-KeyLessPassArtifact $Installer

Write-Host "Windows release directory: flutter_app\build\windows\x64\runner\Release"
Write-Host "Windows installer output: dist\windows\KeyLessPass-Setup-$AppVersion.exe"
