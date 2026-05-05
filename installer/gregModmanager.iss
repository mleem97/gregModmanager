; Inno Setup 6 — gregModmanager (GregModmanager.exe)
; Kompilieren: ISCC.exe gregModmanager.iss  (oder ..\build.ps1)
; Version wird per ..\build.ps1 aus dem Avalonia-Projekt als /DMyAppVersion uebergeben.

#ifndef MyAppVersion
#define MyAppVersion "1.5.0"
#endif

#ifndef MyAppNumericVersion
#define MyAppNumericVersion "1.5.0.0"
#endif

#define MyAppName "gregModmanager"
#define MyAppPublisher "GregFramework"
#define MyAppExeName "GregModmanager.exe"
#define MyAppURL "https://github.com/mleem97/gregFramework"

[Setup]
AppId={{7A2F9E1B-4C3D-5E6F-7890-ABCDEF123401}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf64}\{#MyAppName}
UsePreviousAppDir=yes
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
CloseApplications=yes
RestartApplications=no
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=Output
OutputBaseFilename=gregModmanager-{#MyAppVersion}-Setup
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
DisableWelcomePage=no
DisableProgramGroupPage=no
UninstallDisplayIcon={app}\{#MyAppExeName}
MinVersion=10.0.17763
VersionInfoVersion={#MyAppNumericVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription={#MyAppName} Setup
VersionInfoProductName={#MyAppName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: checkedonce

[Files]
Source: "..\GregModmanager.Avalonia\bin\Release\net9.0\win-x64\publish\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[InstallDelete]
Type: filesandordirs; Name: "{app}\*"

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent runasoriginaluser

[Registry]
Root: HKCU; Subkey: "Software\Classes\greg"; ValueType: string; ValueName: ""; ValueData: "URL:greg Protocol"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\greg"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\greg\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\greg\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Flags: uninsdeletekey

[Code]
// Deinstalliert aeltere Versionen des Modmanagers (andere AppIds oder alte Installationspfade)
// bevor die neue Avalonia-Version installiert wird.
function InitializeSetup(): Boolean;
var
  ResultCode: Integer;
  UninstallString: String;
begin
  Result := true;

  // 1) Suche nach alter Installation unter anderer AppId (falls vorhanden)
  if RegQueryStringValue(HKLM, 'SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{7A2F9E1B-4C3D-5E6F-7890-ABCDEF123400}_is1', 'UninstallString', UninstallString) then
  begin
    if MsgBox('An older version of gregModmanager was found. It must be uninstalled before the new version can be installed.' + #13#10 + 'Proceed with uninstallation?', mbConfirmation, MB_YESNO) = IDYES then
    begin
      Exec(RemoveQuotes(UninstallString), '/SILENT', '', SW_SHOWNORMAL, ewWaitUntilTerminated, ResultCode);
    end
    else
    begin
      Result := false;
      Exit;
    end;
  end;

  // 2) Suche nach Installation unter HKCU (benutzerspezifisch)
  if RegQueryStringValue(HKCU, 'SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{7A2F9E1B-4C3D-5E6F-7890-ABCDEF123400}_is1', 'UninstallString', UninstallString) then
  begin
    if MsgBox('An older version of gregModmanager was found. It must be uninstalled before the new version can be installed.' + #13#10 + 'Proceed with uninstallation?', mbConfirmation, MB_YESNO) = IDYES then
    begin
      Exec(RemoveQuotes(UninstallString), '/SILENT', '', SW_SHOWNORMAL, ewWaitUntilTerminated, ResultCode);
    end
    else
    begin
      Result := false;
      Exit;
    end;
  end;

  // 3) Optional: Suche nach DisplayName-Eintraegen (robust gegen AppId-Aenderungen)
  // Falls es noch weitere alte gregModmanager-Installationen gibt
end;
