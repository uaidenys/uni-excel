//! Windows only: automatic updates from GitHub Releases (`uaidenys/uni-excel`).
//!
//! At startup a hidden PowerShell process asks the GitHub API for the latest release. When its
//! version is newer than this build, it downloads the release's `*-windows-<arch>.msi`, checks it
//! against the release's `SHA256SUMS.txt`, waits until every Uni Excel window has closed and then
//! runs the installer (`msiexec /passive`). Every failure (no network, no release, rate limit, a
//! bad checksum) is silent for the user and noted in `%TEMP%\uni-excel-update\update.log`: the app
//! is never blocked or interrupted by the check.
//!
//! Only the copy the MSI installed updates itself: debug builds, the portable zip and any other
//! copy are skipped, so a dev run never installs the per-machine package. Setting the environment
//! variable `UNI_EXCEL_NO_UPDATE` (to any value) turns updates off.

const REPO: &str = "uaidenys/uni-excel";

#[cfg(target_arch = "aarch64")]
const ASSET_SUFFIX: &str = "-windows-arm64.msi";
#[cfg(target_arch = "x86")]
const ASSET_SUFFIX: &str = "-windows-x86.msi";
#[cfg(not(any(target_arch = "aarch64", target_arch = "x86")))]
const ASSET_SUFFIX: &str = "-windows-x64.msi";

const SCRIPT: &str = r#"param([string] $Repo, [string] $Version, [string] $Suffix, [string] $AppPath)
$ErrorActionPreference = 'Stop'
# The progress bar makes Invoke-WebRequest many times slower on Windows PowerShell 5.1.
$ProgressPreference = 'SilentlyContinue'
$dir = Join-Path $env:TEMP 'uni-excel-update'
$log = Join-Path $dir 'update.log'
function Write-Log([string] $Message) {
  try { Add-Content -Path $log -Value ('{0:yyyy-MM-dd HH:mm:ss} {1}' -f (Get-Date), $Message) } catch {}
}

# 'v1.2.3', '1.2', 'uni-excel-v1.2.3', '1.2.3-rc.1', '1.2.3+build' -> number and pre-release tag.
function Read-Version([string] $Text) {
  if ($Text -notmatch '^(?:[A-Za-z][A-Za-z0-9_-]*?[-_])?[vV]?(\d+)(?:\.(\d+))?(?:\.(\d+))?(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]*)?$') { return $null }
  $number = New-Object Version ([int]$Matches[1]), ([int]('0' + $Matches[2])), ([int]('0' + $Matches[3]))
  return @{ Number = $number; Pre = $Matches[4] }
}

function Test-Newer([string] $Tag, [string] $Current) {
  $a = Read-Version $Tag
  $b = Read-Version $Current
  if (-not $a -or -not $b) { return $false }
  if ($a.Number -ne $b.Number) { return $a.Number -gt $b.Number }
  # Same number: the final release is newer than any pre-release of it.
  return (-not $a.Pre) -and [bool] $b.Pre
}

function Test-InstalledCopy([string] $Path) {
  try {
    $key = Get-Item 'HKLM:\Software\Microsoft\Windows\CurrentVersion\App Paths\uni-excel.exe'
    $installed = [string] $key.GetValue('')
    return $installed -and ([IO.Path]::GetFullPath($installed) -eq [IO.Path]::GetFullPath($Path))
  } catch { return $false }
}

New-Item -ItemType Directory -Force -Path $dir | Out-Null
# One update at a time, however many Uni Excel windows are open.
$mutex = New-Object System.Threading.Mutex($false, 'Local\UniExcelUpdater')
try { $owned = $mutex.WaitOne(0) } catch [System.Threading.AbandonedMutexException] { $owned = $true }
if (-not $owned) { exit 0 }
try {
  if (-not (Test-InstalledCopy $AppPath)) { exit 0 }
  [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
  $headers = @{ 'User-Agent' = 'uni-excel-updater'; 'Accept' = 'application/vnd.github+json' }
  $rel = Invoke-RestMethod -UseBasicParsing -Headers $headers "https://api.github.com/repos/$Repo/releases/latest"
  if (-not (Test-Newer $rel.tag_name $Version)) { exit 0 }

  # A user who said no to the installer is asked again after three days, not at every close.
  $declined = Join-Path $dir 'declined.txt'
  if (Test-Path $declined) {
    $last = (Get-Content $declined -Raw).Trim() -split '\|'
    if ($last[0] -eq $rel.tag_name -and ((Get-Date) - [datetime]::Parse($last[1], [Globalization.CultureInfo]::InvariantCulture)).TotalDays -lt 3) { exit 0 }
  }

  $asset = $rel.assets | Where-Object { $_.name -like "*$Suffix" } | Select-Object -First 1
  $sumsAsset = $rel.assets | Where-Object { $_.name -eq 'SHA256SUMS.txt' } | Select-Object -First 1
  if (-not $asset -or -not $sumsAsset) { Write-Log "$($rel.tag_name): no $Suffix installer or no SHA256SUMS.txt"; exit 0 }
  $download = @{ 'User-Agent' = 'uni-excel-updater' }
  $sumsFile = Join-Path $dir 'SHA256SUMS.txt'
  Invoke-WebRequest -UseBasicParsing -Headers $download $sumsAsset.browser_download_url -OutFile $sumsFile
  $expected = $null
  foreach ($line in Get-Content $sumsFile) {
    if ($line -match '^([0-9a-fA-F]{64})\s+\*?(.+?)\s*$' -and $Matches[2] -eq $asset.name) { $expected = $Matches[1].ToLowerInvariant() }
  }
  if (-not $expected) { Write-Log "$($rel.tag_name): SHA256SUMS.txt has no line for $($asset.name)"; exit 0 }

  $msi = Join-Path $dir $asset.name
  $hash = { (Get-FileHash -Algorithm SHA256 -Path $msi).Hash.ToLowerInvariant() }
  if (-not (Test-Path $msi) -or (& $hash) -ne $expected) {
    Invoke-WebRequest -UseBasicParsing -Headers $download $asset.browser_download_url -OutFile $msi
  }
  if ((& $hash) -ne $expected) {
    Write-Log "$($asset.name): checksum does not match SHA256SUMS.txt; not installing"
    Remove-Item -Force $msi -ErrorAction SilentlyContinue
    exit 0
  }

  # Install only once no Uni Excel window is left open, so no file is in use.
  $name = [IO.Path]::GetFileNameWithoutExtension($AppPath)
  while (@(Get-Process -Name $name -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $AppPath }).Count -gt 0) {
    Start-Sleep -Seconds 3
  }
  $msiLog = Join-Path $dir 'install.log'
  $p = Start-Process -FilePath 'msiexec.exe' -ArgumentList @('/i', "`"$msi`"", '/passive', '/norestart', '/l*v', "`"$msiLog`"") -Wait -PassThru
  Write-Log "$($asset.name): msiexec exit code $($p.ExitCode) (log: $msiLog)"
  if ($p.ExitCode -eq 0 -or $p.ExitCode -eq 3010) {
    Remove-Item -Force $msi, $declined -ErrorAction SilentlyContinue
  } elseif ($p.ExitCode -eq 1602 -or $p.ExitCode -eq 1625) {
    # 1602: the user cancelled (or declined the UAC prompt); 1625: blocked by policy.
    Set-Content -Path $declined -Value ('{0}|{1}' -f $rel.tag_name, (Get-Date).ToString('o', [Globalization.CultureInfo]::InvariantCulture))
  }
} catch {
  Write-Log "update check failed: $($_.Exception.Message)"
  exit 0
} finally {
  $mutex.ReleaseMutex()
}
"#;

/// Starts the background update check. Never fails: problems are only logged.
pub fn spawn() {
    if cfg!(debug_assertions) || std::env::var_os("UNI_EXCEL_NO_UPDATE").is_some() {
        return;
    }
    if let Err(e) = try_spawn() {
        log::warn!("update check not started: {e}");
    }
}

fn try_spawn() -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let exe = std::env::current_exe()?;
    let dir = std::env::temp_dir().join("uni-excel-update");
    std::fs::create_dir_all(&dir)?;
    let script = dir.join("update.ps1");
    std::fs::write(&script, SCRIPT)?;
    std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-WindowStyle", "Hidden", "-File"])
        .arg(&script)
        .args(["-Repo", REPO, "-Version", env!("CARGO_PKG_VERSION"), "-Suffix", ASSET_SUFFIX, "-AppPath"])
        .arg(&exe)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    log::info!("update check started ({REPO})");
    Ok(())
}
