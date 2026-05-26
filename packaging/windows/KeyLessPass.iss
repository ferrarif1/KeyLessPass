#define AppName "KeyLessPass"
#define AppPublisher "KeyLessPass Project Contributors"
#define AppExeName "KeyLessPass.exe"

#define EnvAppVersion GetEnv("KEYLESSPASS_APP_VERSION")
#if EnvAppVersion == ""
#define AppVersion "0.1.0"
#else
#define AppVersion EnvAppVersion
#endif

#define EnvReleaseDir GetEnv("KEYLESSPASS_RELEASE_DIR")
#if EnvReleaseDir == ""
#define ReleaseDir "..\..\flutter_app\build\windows\x64\runner\Release"
#else
#define ReleaseDir EnvReleaseDir
#endif

#define EnvOutputDir GetEnv("KEYLESSPASS_OUTPUT_DIR")
#if EnvOutputDir == ""
#define OutputDir "..\..\dist\windows"
#else
#define OutputDir EnvOutputDir
#endif

#define EnvIconFile GetEnv("KEYLESSPASS_ICON_FILE")
#if EnvIconFile == ""
#define IconFile "..\..\flutter_app\windows\runner\resources\app_icon.ico"
#else
#define IconFile EnvIconFile
#endif

#define EnvLicenseFile GetEnv("KEYLESSPASS_LICENSE_FILE")
#if EnvLicenseFile == ""
#define LicenseFilePath "..\..\LICENSE"
#else
#define LicenseFilePath EnvLicenseFile
#endif

[Setup]
AppId={{30D45E65-7D7A-4C76-9C85-3A8A2A08B9C4}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
OutputDir={#OutputDir}
OutputBaseFilename=KeyLessPass-Setup-{#AppVersion}
SetupIconFile={#IconFile}
UninstallDisplayIcon={app}\{#AppExeName}
LicenseFile={#LicenseFilePath}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "{#ReleaseDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExeName}"; Description: "Launch {#AppName}"; Flags: nowait postinstall skipifsilent
