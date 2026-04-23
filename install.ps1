# installer untuk Windows PowerShell

$ErrorActionPreference = "Stop"

$TARGET = "idpp-x86_64-pc-windows-msvc.exe"
$URL = "https://github.com/rillToMe/idpp/releases/latest/download/$TARGET"

$InstallDir = "$env:LOCALAPPDATA\idpp"
$ExePath = "$InstallDir\idpp.exe"

Write-Host "Mengunduh ID++ untuk Windows..."
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Invoke-WebRequest -Uri $URL -OutFile $ExePath

Write-Host "Menambahkan ID++ ke PATH pengguna..."
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    Write-Host "PATH telah diperbarui. Silakan restart terminal Anda agar efeknya bekerja."
}

Write-Host "Instalasi selesai!"
& $ExePath --versi
