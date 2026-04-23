[Setup]
AppName=ID++
AppVersion=0.1.0
AppPublisher=KyuzenStudio
AppPublisherURL=https://github.com/rillToMe/idpp
AppSupportURL=https://github.com/rillToMe/idpp/issues
AppUpdatesURL=https://github.com/rillToMe/idpp/releases

DefaultDirName={autopf}\ID++
DisableProgramGroupPage=yes
LicenseFile=LICENSE
OutputBaseFilename=IDPP_Setup_v0.1.0
OutputDir=installer_build
SetupIconFile=assets\idpp.ico
Compression=lzma2/ultra64
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64

ChangesEnvironment=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "target\release\idpp.exe"; DestDir: "{app}"; Flags: ignoreversion

Source: "examples\*"; DestDir: "{app}\examples"; Excludes: "*.idppc"; Flags: ignoreversion recursesubdirs createallsubdirs

Source: "README.md"; DestDir: "{app}"; Flags: isreadme
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Code]

const
  EnvironmentKey = 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';

function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := (Pos(';' + UpperCase(Param) + ';', ';' + UpperCase(OrigPath) + ';') = 0);
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  OrigPath: string;
begin
  if CurStep = ssPostInstall then
  begin
    if NeedsAddPath(ExpandConstant('{app}')) then
    begin
      if RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', OrigPath) then
      begin
        RegWriteStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', OrigPath + ';' + ExpandConstant('{app}'));
      end;
    end;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  OrigPath, AppPath: string;
  PosAppPath: Integer;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    AppPath := ExpandConstant('{app}');
    if RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', OrigPath) then
    begin
      PosAppPath := Pos(';' + UpperCase(AppPath) + ';', ';' + UpperCase(OrigPath) + ';');
      if PosAppPath > 0 then
      begin
        StringChangeEx(OrigPath, ';' + AppPath, '', True);
        StringChangeEx(OrigPath, AppPath + ';', '', True);
        StringChangeEx(OrigPath, AppPath, '', True);
        RegWriteStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', OrigPath);
      end;
    end;
  end;
end;
