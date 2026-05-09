; Inno Setup 6 — gregModmanager (GregModmanager.exe)
; Kompilieren: ISCC.exe gregModmanager.iss  (oder ..\build.ps1)
; Version wird per ..\build.ps1 aus dem Avalonia-Projekt als /DMyAppVersion uebergeben.

#ifndef MyAppVersion
#define MyAppVersion "1.5.0"
#endif

#ifndef MyAppNumericVersion
#define MyAppNumericVersion "1.5.0.0"
#endif

#ifndef MyAppOutputBaseFilename
#define MyAppOutputBaseFilename "gregModmanager-{#MyAppVersion}-Windows"
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
OutputBaseFilename={#MyAppOutputBaseFilename}
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
Source: "..\..\src\GregModmanager.Avalonia\bin\Release\net9.0\win-x64\publish\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

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
var
  VCRedistPath: string;

// Prüft ob die VC++ 2015-2022 Runtimes (x64) installiert sind
function IsVCRedistInstalled(): Boolean;
var
  Installed: Cardinal;
begin
  Result := False;
  if RegQueryDWordValue(HKLM, 'SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64', 'Installed', Installed) then
  begin
    Result := (Installed = 1);
  end;
end;

// Prüft ob Steam installiert ist
function IsSteamInstalled(): Boolean;
begin
  Result := RegKeyExists(HKCU, 'Software\Valve\Steam') or RegKeyExists(HKLM, 'SOFTWARE\Valve\Steam');
end;

function InitializeSetup(): Boolean;
var
  ResultCode: Integer;
  UninstallString: String;
begin
  Result := true;

  // 1) Steam Check (Warnung falls nicht vorhanden)
  if not IsSteamInstalled() then
  begin
    if MsgBox('Steam was not detected on this system. gregModmanager requires Steam to manage mods for Data Center. Do you want to continue anyway?', mbConfirmation, MB_YESNO) = IDNO then
    begin
      Result := false;
      Exit;
    end;
  end;

  // 2) Suche nach alter Installation unter anderer AppId (falls vorhanden)
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
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if CurStep = ssInstall then
  begin
    if not IsVCRedistInstalled() then
    begin
      VCRedistPath := ExpandConstant('{tmp}\vc_redist.x64.exe');
      
      WizardForm.StatusLabel.Caption := 'Downloading Microsoft Visual C++ Redistributable (x64)...';
      WizardForm.ProgressGauge.Style := npbstMarquee;
      
      try
        // Online-Installer: Download der Dependency via PowerShell
        if not Exec('powershell.exe', '-NoProfile -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri ''https://aka.ms/vs/17/release/vc_redist.x64.exe'' -OutFile ''' + VCRedistPath + '''"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) or (ResultCode <> 0) then
        begin
          MsgBox('Download of VC++ Redistributable failed. You may need to install it manually to run the application.', mbInformation, MB_OK);
        end
        else
        begin
          WizardForm.StatusLabel.Caption := 'Installing Microsoft Visual C++ Redistributable...';
          if not Exec(VCRedistPath, '/quiet /norestart', '', SW_SHOW, ewWaitUntilTerminated, ResultCode) then
          begin
            MsgBox('Installation of VC++ Redistributable failed. Error: ' + IntToStr(ResultCode), mbError, MB_OK);
          end;
        end;
      finally
        WizardForm.ProgressGauge.Style := npbstNormal;
      end;
    end;
  end;
end;
