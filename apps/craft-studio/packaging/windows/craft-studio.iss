; Inno Setup 6 script for the Craft Studio installer. ../package.sh passes AppVersion, StageDir
; (the files to install) and OutputDir. Per-user install by default, so no administrator rights
; are needed; the first page offers an all-users install instead.

#ifndef AppVersion
  #error AppVersion is not defined (iscc /DAppVersion=...)
#endif

[Setup]
AppId={{1C90DD02-63FF-4D82-9262-7D5DDCE95C7C}
AppName=Craft Studio
AppVersion={#AppVersion}
AppVerName=Craft Studio {#AppVersion}
AppPublisher=cloudtools
AppPublisherURL=https://github.com/maximemarc/cloudtools
AppSupportURL=https://github.com/maximemarc/cloudtools/issues
DefaultDirName={autopf}\Craft Studio
DefaultGroupName=Craft Studio
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir={#OutputDir}
OutputBaseFilename=craft-studio-{#AppVersion}-windows-x64-setup
SetupIconFile={#StageDir}\craft-studio.ico
UninstallDisplayIcon={app}\craft-studio.ico
UninstallDisplayName=Craft Studio
Compression=lzma2/max
SolidCompression=yes
; The 64-bit compressor process: the 32-bit compiler runs out of memory on ~500 MB with threads.
LZMAUseSeparateProcess=yes
LZMANumBlockThreads=2
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#StageDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Craft Studio"; Filename: "{app}\craft-studio.exe"; IconFilename: "{app}\craft-studio.ico"
Name: "{autodesktop}\Craft Studio"; Filename: "{app}\craft-studio.exe"; IconFilename: "{app}\craft-studio.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\craft-studio.exe"; Description: "{cm:LaunchProgram,Craft Studio}"; Flags: nowait postinstall skipifsilent
