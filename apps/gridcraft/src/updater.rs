//! Windows only: automatic updates from GitHub Releases (`uaidenys/uni-excel`).
//!
//! At startup a hidden PowerShell process asks the GitHub API for the latest release. When its
//! version is newer than this build, it downloads the release's `*-windows-<arch>.msi`, waits for
//! the app to close and then runs the installer (`msiexec /passive`). Every failure (no network,
//! no release, rate limit) is silent: the app is never blocked or interrupted by the check.
//! `UNI_EXCEL_NO_UPDATE=1` turns it off.

const REPO: &str = "uaidenys/uni-excel";

#[cfg(target_arch = "aarch64")]
const ASSET_SUFFIX: &str = "-windows-arm64.msi";
#[cfg(target_arch = "x86")]
const ASSET_SUFFIX: &str = "-windows-x86.msi";
#[cfg(not(any(target_arch = "aarch64", target_arch = "x86")))]
const ASSET_SUFFIX: &str = "-windows-x64.msi";

const SCRIPT: &str = r#"param([string] $Repo, [string] $Version, [string] $Suffix, [int] $AppPid)
$ErrorActionPreference = 'Stop'
try {
  [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
  $headers = @{ 'User-Agent' = 'uni-excel-updater'; 'Accept' = 'application/vnd.github+json' }
  $rel = Invoke-RestMethod -UseBasicParsing -Headers $headers "https://api.github.com/repos/$Repo/releases/latest"
  $latest = [version]((($rel.tag_name -replace '^v', '') -split '-')[0])
  $current = [version](($Version -split '-')[0])
  if ($latest -le $current) { exit 0 }
  $asset = $rel.assets | Where-Object { $_.name -like "*$Suffix" } | Select-Object -First 1
  if (-not $asset) { exit 0 }
  $dir = Join-Path $env:TEMP 'uni-excel-update'
  New-Item -ItemType Directory -Force -Path $dir | Out-Null
  $msi = Join-Path $dir $asset.name
  Invoke-WebRequest -UseBasicParsing -Headers @{ 'User-Agent' = 'uni-excel-updater' } $asset.browser_download_url -OutFile $msi
  Wait-Process -Id $AppPid -ErrorAction SilentlyContinue
  Start-Process -FilePath 'msiexec.exe' -ArgumentList @('/i', "`"$msi`"", '/passive', '/norestart')
} catch { exit 0 }
"#;

/// Starts the background update check. Never fails: problems are only logged.
pub fn spawn() {
    if std::env::var_os("UNI_EXCEL_NO_UPDATE").is_some() {
        return;
    }
    if let Err(e) = try_spawn() {
        log::warn!("update check not started: {e}");
    }
}

fn try_spawn() -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let dir = std::env::temp_dir().join("uni-excel-update");
    std::fs::create_dir_all(&dir)?;
    let script = dir.join("update.ps1");
    std::fs::write(&script, SCRIPT)?;
    std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-WindowStyle", "Hidden", "-File"])
        .arg(&script)
        .args(["-Repo", REPO, "-Version", env!("CARGO_PKG_VERSION"), "-Suffix", ASSET_SUFFIX, "-AppPid"])
        .arg(std::process::id().to_string())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    log::info!("update check started ({REPO})");
    Ok(())
}
