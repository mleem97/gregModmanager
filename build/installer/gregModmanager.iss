; Build with build-windows.ps1 against the staged Rust distribution.
#ifndef MyAppVersion
  #error MyAppVersion must be supplied by the build script
#endif
#ifndef SourceDir
  #error SourceDir must point to the staged Rust distribution
#endif
#ifndef MarkerFile
  #error MarkerFile must identify the installer type
#endif

[Setup]
AppId={{7A2F9E1B-4C3D-5E6F-7890-ABCDEF123401}
AppName=gregModmanager
AppVersion={#MyAppVersion}
AppPublisher=GregFramework
AppPublisherURL=https://github.com/mleem97/gregModmanager
AppUpdatesURL=https://github.com/mleem97/gregModmanager/releases
DefaultDirName={autopf64}\gregModmanager
UsePreviousAppDir=yes
DefaultGroupName=gregModmanager
CloseApplications=yes
RestartApplications=no
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\gregmodmanager.exe
MinVersion=10.0.17763

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: checkedonce

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#MarkerFile}"; DestDir: "{app}"; DestName: ".greg-install-kind"; Flags: ignoreversion

[Icons]
Name: "{group}\gregModmanager"; Filename: "{app}\gregmodmanager.exe"; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,gregModmanager}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\gregModmanager"; Filename: "{app}\gregmodmanager.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\gregmodmanager.exe"; WorkingDir: "{app}"; Description: "{cm:LaunchProgram,gregModmanager}"; Flags: nowait postinstall skipifsilent runasoriginaluser

[Registry]
Root: HKA; Subkey: "Software\Classes\greg"; ValueType: string; ValueName: ""; ValueData: "URL:greg Protocol"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\greg"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\greg\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\gregmodmanager.exe,0"
Root: HKA; Subkey: "Software\Classes\greg\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\gregmodmanager.exe"" ""%1"""
