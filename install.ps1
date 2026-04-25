# Installer ID++ untuk Windows PowerShell
# Mengunduh binary langsung dari GitHub Releases

$ErrorActionPreference = "Stop"

$VERSION = "latest"
$REPO = "rillToMe/idpp"
$BINARY_NAME = "idpp-x86_64-pc-windows-msvc.zip"

# Resolusi URL terbaru
if ($VERSION -eq "latest") {
    $API_URL = "https://api.github.com/repos/$REPO/releases/latest"
    try {
        $Release = Invoke-RestMethod -Uri $API_URL -Headers @{ "User-Agent" = "idpp-installer" }
        $DownloadURL = ($Release.assets | Where-Object { $_.name -eq $BINARY_NAME }).browser_download_url
        if (-not $DownloadURL) {
            Write-Error "Asset '$BINARY_NAME' tidak ditemukan di release terbaru."
            exit 1
        }
        $ReleaseTag = $Release.tag_name
    } catch {
        Write-Error "Gagal mengambil informasi release: $_"
        exit 1
    }
} else {
    $DownloadURL = "https://github.com/$REPO/releases/download/$VERSION/$BINARY_NAME"
    $ReleaseTag = $VERSION
}

# Lokasi instalasi: AppData\Local\Programs\ID++
$InstallDir = "$env:LOCALAPPDATA\Programs\ID++"
$ZipPath = "$env:TEMP\idpp_temp.zip"

Write-Host ""
Write-Host "  ID++ Installer" -ForegroundColor Cyan
Write-Host "  Versi: $ReleaseTag" -ForegroundColor Gray
Write-Host ""

Write-Host "Mengunduh ID++ dan Rak dari:" -ForegroundColor Yellow
Write-Host "  $DownloadURL" -ForegroundColor Gray
Invoke-WebRequest -Uri $DownloadURL -OutFile $ZipPath

# Buat direktori jika belum ada
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Write-Host "Mengekstrak file..." -ForegroundColor Yellow
Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force
Remove-Item $ZipPath

Write-Host "Menambahkan ID++ ke PATH pengguna..." -ForegroundColor Yellow
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    $NewPath = if ($UserPath) { "$UserPath;$InstallDir" } else { $InstallDir }
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    Write-Host "PATH diperbarui. Restart terminal agar perubahan aktif." -ForegroundColor Green
} else {
    Write-Host "PATH sudah mengandung direktori instalasi." -ForegroundColor Gray
}

Write-Host ""
Write-Host "Instalasi selesai! ID++ dan Rak dipasang di:" -ForegroundColor Green
Write-Host "  $InstallDir" -ForegroundColor Cyan
Write-Host ""

# Verifikasi instalasi
try {
    & "$InstallDir\idpp.exe" --versi
    & "$InstallDir\rak.exe" --help
} catch {
    Write-Host "Catatan: Jalankan 'idpp --versi' dan 'rak --help' setelah restart terminal." -ForegroundColor Yellow
}
