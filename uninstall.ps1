# MALD uninstaller for Windows
#   powershell -c "irm https://raw.githubusercontent.com/NAME0x0/MALD/main/uninstall.ps1 | iex"
$ErrorActionPreference = 'Stop'

$InstallDir = "$env:LOCALAPPDATA\mald"

# --- Remove binary and directory ---
if (Test-Path "$InstallDir\mald.exe") {
    Remove-Item "$InstallDir\mald.exe" -Force
    Write-Host "Removed $InstallDir\mald.exe" -ForegroundColor Green
} else {
    Write-Host "mald.exe not found at $InstallDir" -ForegroundColor Yellow
}

if ((Test-Path $InstallDir) -and -not (Get-ChildItem $InstallDir)) {
    Remove-Item $InstallDir -Force
    Write-Host "Removed $InstallDir directory" -ForegroundColor Green
}

# --- Remove from user PATH ---
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($userPath -like "*$InstallDir*") {
    $newPath = ($userPath -split ';' | Where-Object { $_ -ne $InstallDir }) -join ';'
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    Write-Host "Removed $InstallDir from PATH." -ForegroundColor Green
}

Write-Host ""
Write-Host "MALD has been uninstalled. Restart your terminal for PATH changes to take effect." -ForegroundColor Cyan
