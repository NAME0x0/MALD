# MALD installer for Windows — downloads the latest release and installs to %LOCALAPPDATA%\mald
#   powershell -c "irm https://raw.githubusercontent.com/NAME0x0/MALD/main/install.ps1 | iex"
$ErrorActionPreference = 'Stop'

$Repo = "NAME0x0/MALD"
$InstallDir = "$env:LOCALAPPDATA\mald"
$Binary = "mald-windows-x64.exe"

# --- Check for conflicting mald.exe in PATH ---
$existing = Get-Command mald -ErrorAction SilentlyContinue
if ($existing) {
    $loc = $existing.Source
    if ($loc -ne "$InstallDir\mald.exe") {
        Write-Host "WARNING: Found existing mald at $loc" -ForegroundColor Yellow
        # Remove stale Python installs automatically
        if ($loc -match 'Python.*Scripts') {
            Write-Host "  This looks like a stale Python install. Removing it..." -ForegroundColor Yellow
            Remove-Item $loc -Force
            Write-Host "  Removed $loc" -ForegroundColor Green
        } else {
            Write-Host "  This may shadow the new install. Consider removing it." -ForegroundColor Yellow
        }
    }
}

# --- Get latest release version ---
Write-Host "Fetching latest release..."
$release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Version = $release.tag_name
Write-Host "Latest version: $Version"

# --- Download binary and checksums ---
$downloadUrl = "https://github.com/$Repo/releases/download/$Version/$Binary"
$checksumUrl = "https://github.com/$Repo/releases/download/$Version/SHA256SUMS"

$tmpDir = New-Item -ItemType Directory -Path (Join-Path $env:TEMP "mald-install-$(Get-Random)")
try {
    $binPath = Join-Path $tmpDir "mald.exe"
    Write-Host "Downloading mald $Version..."
    Invoke-WebRequest -Uri $downloadUrl -OutFile $binPath -UseBasicParsing

    # --- Verify checksum ---
    try {
        $sumPath = Join-Path $tmpDir "SHA256SUMS"
        Invoke-WebRequest -Uri $checksumUrl -OutFile $sumPath -UseBasicParsing
        $expected = (Get-Content $sumPath | Where-Object { $_ -match $Binary }) -replace '\s+.*', ''
        if ($expected) {
            $actual = (Get-FileHash $binPath -Algorithm SHA256).Hash.ToLower()
            if ($expected.ToLower() -ne $actual) {
                Write-Host "Checksum verification FAILED" -ForegroundColor Red
                Write-Host "  Expected: $expected"
                Write-Host "  Got:      $actual"
                exit 1
            }
            Write-Host "Checksum verified." -ForegroundColor Green
        }
    } catch {
        Write-Host "Checksum file not available, skipping verification." -ForegroundColor Yellow
    }

    # --- Install ---
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item $binPath "$InstallDir\mald.exe" -Force
    Write-Host "Installed to $InstallDir\mald.exe" -ForegroundColor Green
} finally {
    Remove-Item $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
}

# --- Add to user PATH if needed ---
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable('Path', "$InstallDir;$userPath", 'User')
    Write-Host "Added $InstallDir to your PATH." -ForegroundColor Green
}

Write-Host ""
Write-Host "Done! Restart your terminal, then run 'mald' to get started." -ForegroundColor Cyan
