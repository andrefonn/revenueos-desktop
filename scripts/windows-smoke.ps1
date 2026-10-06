param(
  [Parameter(Mandatory=$true)][string]$Installer,
  [Parameter(Mandatory=$true)][string]$Version,
  [switch]$SkipInstall,
  [string]$EvidenceDirectory = 'evidence'
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$installerPath = (Resolve-Path -LiteralPath $Installer).Path
$setupExitCode = $null
if (!$SkipInstall) {
  $setupProcess = Start-Process -FilePath $installerPath -ArgumentList '/S' -WindowStyle Hidden -Wait -PassThru
  $setupExitCode = $setupProcess.ExitCode
  if ($setupExitCode -ne 0) { throw "Installer failed: $setupExitCode" }
}
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Revenue OS'
$installation = Get-ItemProperty -LiteralPath $uninstallKey
if ($installation.DisplayVersion -ne $Version) { throw 'Installed version does not match release.' }
$installDirectory = $installation.InstallLocation
if (!$installDirectory) { $installDirectory = $installation.'(default)' }
if (!$installDirectory) { $installDirectory = Join-Path $env:LOCALAPPDATA 'Revenue OS' }
$installDirectory = $installDirectory.Trim('"')
$applicationPath = Join-Path $installDirectory 'revenue-os-desktop.exe'
if (!(Test-Path -LiteralPath $applicationPath)) { throw 'Installed application executable is missing.' }
$desktopShortcut = Join-Path ([Environment]::GetFolderPath('Desktop')) 'Revenue OS.lnk'
if (!(Test-Path -LiteralPath $desktopShortcut)) { throw 'Desktop shortcut is missing.' }
$menuShortcut = Join-Path ([Environment]::GetFolderPath('StartMenu')) 'Programs\Revenue OS\Revenue OS.lnk'
if (!(Test-Path -LiteralPath $menuShortcut)) { throw 'Start menu shortcut is missing.' }
$shell = New-Object -ComObject WScript.Shell
if ($shell.CreateShortcut($desktopShortcut).TargetPath -ne $applicationPath) { throw 'Desktop shortcut points to the wrong executable.' }
$application = Get-Process -Name 'revenue-os-desktop' -ErrorAction SilentlyContinue | Select-Object -First 1
if (!$application) { $application = Start-Process -FilePath $applicationPath -WindowStyle Hidden -PassThru }
$deadline = [DateTime]::UtcNow.AddSeconds(45)
$window = $null
$names = @()
do {
  Start-Sleep -Milliseconds 500
  $application.Refresh()
  if ($application.HasExited) { throw 'Application exited during startup.' }
  if ($application.MainWindowHandle -eq [IntPtr]::Zero) { continue }
  $window = [System.Windows.Automation.AutomationElement]::FromHandle($application.MainWindowHandle)
  $controls = $window.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
  $names = @($controls | ForEach-Object { $_.Current.Name } | Where-Object { $_ })
  if ($names -contains 'E-mail' -and $names -contains 'Senha' -and $names -contains 'Entrar') { break }
} while ([DateTime]::UtcNow -lt $deadline)
if (!($names -contains 'E-mail' -and $names -contains 'Senha' -and $names -contains 'Entrar')) { throw ('Live login screen did not render. Controls: ' + ($names -join ', ')) }
New-Item -ItemType Directory -Force -Path $EvidenceDirectory | Out-Null
$evidence = [ordered]@{
  version = $installation.DisplayVersion
  application = $applicationPath
  installerExitCode = $setupExitCode
  desktopShortcut = $desktopShortcut
  startMenuShortcut = $menuShortcut
  processId = $application.Id
  loginRendered = $true
  capturedAt = [DateTime]::UtcNow.ToString('o')
}
$evidence | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $EvidenceDirectory 'windows-smoke.json')
$evidence | ConvertTo-Json
