# register-idpp.ps1
# Mendaftarkan file .idpp ke Windows Explorer dengan ikon dan asosiasi program
# Tidak membutuhkan hak Administrator (menggunakan HKCU)

$ErrorActionPreference = "Stop"

# Path ke exe dan ikon
$cmd = Get-Command idpp -ErrorAction SilentlyContinue
if ($cmd) {
    $idppExe = $cmd.Source
} else {
    $idppExe = "$env:USERPROFILE\.cargo\bin\idpp.exe"
}

# Path ke file ICO — gunakan yang ada di folder proyek
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$icoPath   = Join-Path $scriptDir "assets\idpp.ico"

if (-not (Test-Path $icoPath)) {
    Write-Warning "File ikon tidak ditemukan di: $icoPath"
    Write-Warning "Pastikan sudah pernah build proyek (cargo build --release)"
    exit 1
}

Write-Host "Mendaftarkan .idpp ke Windows Explorer..."
Write-Host "  EXE : $idppExe"
Write-Host "  IKON: $icoPath"

# Gunakan HKCU agar tidak perlu admin
$root = "HKCU:\Software\Classes"

# 1. Daftarkan ekstensi .idpp
New-Item -Path "$root\.idpp"          -Force | Out-Null
Set-ItemProperty -Path "$root\.idpp"  -Name "(Default)" -Value "idpp_file"

# 2. Daftarkan tipe file "idpp_file"
New-Item -Path "$root\idpp_file"      -Force | Out-Null
Set-ItemProperty -Path "$root\idpp_file" -Name "(Default)" -Value "ID++ Source File"

# 3. Set ikon kustom
New-Item -Path "$root\idpp_file\DefaultIcon" -Force | Out-Null
Set-ItemProperty -Path "$root\idpp_file\DefaultIcon" -Name "(Default)" -Value "`"$icoPath`",0"

# 4. Set perintah buka file (double-click)
New-Item -Path "$root\idpp_file\shell\open\command" -Force | Out-Null
Set-ItemProperty -Path "$root\idpp_file\shell\open\command" `
    -Name "(Default)" -Value "`"$idppExe`" `"%1`""

# 5. Paksa Windows Explorer refresh ikon cache
Write-Host "Memuat ulang cache ikon Windows Explorer..."
$code = @"
[System.Runtime.InteropServices.DllImport("shell32.dll")]
public static extern void SHChangeNotify(int wEventId, int uFlags, IntPtr dwItem1, IntPtr dwItem2);
"@
$shell = Add-Type -MemberDefinition $code -Name WinShell -Namespace Shell -PassThru
$shell::SHChangeNotify(0x08000000, 0x0000, [IntPtr]::Zero, [IntPtr]::Zero)

Write-Host ""
Write-Host "Selesai! File .idpp sekarang pakai ikon ID++ di Windows Explorer."
Write-Host "Jika ikon belum berubah, tutup dan buka ulang File Explorer."
