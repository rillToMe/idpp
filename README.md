<div align="center">
  <img src="assets/idpp.png" alt="Logo ID++" width="200" />

  # ID++ - Bahasa Pemrograman Indonesia

  **Bahasa pemrograman yang sintaksnya seperti kalimat Indonesia sehari-hari.**

  [![Versi](https://img.shields.io/badge/versi-1.0.0-blue?style=flat-square)](https://github.com/rillToMe/idpp/releases)
  [![Lisensi](https://img.shields.io/badge/lisensi-MIT-green?style=flat-square)](LICENSE)
  [![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square)](https://github.com/rillToMe/idpp/releases)
  [![Dibuat dengan Rust](https://img.shields.io/badge/dibuat%20dengan-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
</div>

---

ID++ adalah bahasa pemrograman sederhana yang menggunakan bahasa Indonesia sebagai sintaksisnya. Tujuan utamanya adalah untuk mempermudah siapa saja, khususnya pemula, dalam belajar logika pemrograman tanpa terkendala oleh hambatan bahasa Inggris.

## Instalasi

ID++ tersedia untuk Windows, macOS, dan Linux.

### Cara 1: Menggunakan Cargo (Rust)
Jika Anda sudah memiliki Rust dan Cargo terpasang:
```sh
cargo install idpp
```

### Cara 2: Skrip Instalasi Mac / Linux
Jalankan perintah ini di terminal:
```sh
curl -fsSL https://raw.githubusercontent.com/rillToMe/idpp/main/install.sh | sh
```

### Cara 3: Skrip Instalasi Windows PowerShell
Buka PowerShell dan jalankan:
```powershell
iwr https://raw.githubusercontent.com/rillToMe/idpp/main/install.ps1 | iex
```

### Cara 4: Unduh Langsung
Kunjungi halaman [Releases](https://github.com/rillToMe/idpp/releases) dan unduh versi yang sesuai dengan sistem operasi Anda.

## Penggunaan Cepat

Setelah terpasang, Anda bisa menggunakan ID++ langsung di terminal.

Untuk menjalankan file:
```sh
idpp program.idpp
```

Untuk masuk ke mode interaktif (REPL):
```sh
idpp
```

## Ekstensi VS Code

Untuk pengalaman penulisan kode yang lebih baik dengan syntax highlighting, gunakan ekstensi VS Code resmi dari ID++!

1. Buka folder `vscode-extension`
2. Jalankan `npm install` dan `npm run package` (menggunakan vsce)
3. Instal ekstensi: `code --install-extension idpp-language-1.0.0.vsix`

## Sintaks Dasar

ID++ membaca seperti bahasa Indonesia biasa. Akhiri setiap baris dengan titik `.`.

### Output & Input
```idpp
tulis "Halo Dunia!".
tulis "Nilai kamu adalah ", 100, ".".

tanya "Siapa namamu?" simpan ke nama.
```

### Variabel
```idpp
simpan nama = "Budi".
simpan umur = 17.
tetap PI = 3.14.    // tetap = konstanta, tidak bisa diubah
```

### Operasi Matematika

Gunakan simbol `+`, `-`, `*`, `/` untuk ekspresi:
```idpp
simpan hasil = 10 + 5.
simpan selisih = 10 - 3.
simpan kali_dua = 4 * 2.
simpan bagi_tiga = 9 / 3.
simpan sisa = 10 sisa 3.    // modulo
simpan pangkat = 2 pangkat 8.
```

### Kondisi

| Simbol | Kata | Arti |
|---|---|---|
| `>` | `lebih dari` | lebih besar |
| `<` | `kurang dari` | lebih kecil |
| `>=` | `lebih dari sama` | lebih besar atau sama |
| `<=` | `kurang dari sama` | lebih kecil atau sama |
| `==` | `sama dengan` | sama |
| `!=` | `tidak sama dengan` | tidak sama |

```idpp
simpan nilai = 85.
jika nilai >= 90 maka
    tulis "Predikat A".
atau jika nilai >= 80 maka
    tulis "Predikat B".
atau jika nilai >= 70 maka
    tulis "Predikat C".
lainnya
    tulis "Predikat D".
selesai.
```

### Loop
```idpp
// Loop selama kondisi terpenuhi
simpan i = 1.
selama i kurang dari sama 5 lakukan
    tulis i.
    tambah i dengan 1.
selesai.

// Loop N kali
ulangi 5 kali
    tulis "Halo".
selesai.

// Loop untuk setiap item
simpan buah = daftar "apel", "mangga", "jeruk".
untuk setiap b dalam buah lakukan
    tulis b.
selesai.
```

### Fungsi
```idpp
buat fungsi jumlah dengan a dan b
    kembalikan a + b.
selesai.

simpan hasil = jalankan jumlah dengan 5 dan 10.
```

### Daftar (Array)
```idpp
simpan angka = daftar 1, 2, 3.
tambahkan 4 ke angka.
tulis angka di 0.
```

### Kamus (Dictionary)
```idpp
simpan siswa = kamus
    nama: "Budi",
    umur: 17.
selesai.

tulis siswa ambil nama.
ubah siswa umur menjadi 18.
```

### HTTP / Network

ID++ punya library HTTP bawaan. Respons berupa kamus dengan field `status`, `ok`, `teks`, `json`, `header`, `url`.

```idpp
// GET - ambil data dari API
simpan resp = http ambil "https://api.example.com/users/1".
tulis "Status: ", resp ambil status.
tulis "OK: ", resp ambil ok.
simpan data = resp ambil json.
tulis "Nama: ", data ambil name.

// GET dengan query string dan header
simpan opsi = kamus
    param: kamus { q: "rust", page: 1 },
    header: kamus { Authorization: "Bearer token123" }.
selesai.
simpan resp = http ambil "https://api.example.com/search", opsi.

// POST - kirim data JSON
simpan payload = kamus
    nama: "Budi",
    umur: 17.
selesai.
simpan resp = http kirim "https://api.example.com/users", payload.

// PUT - update data
simpan resp = http ubah "https://api.example.com/users/1", payload.

// DELETE
simpan resp = http hapus "https://api.example.com/users/1".

// PATCH - update sebagian data
simpan resp = http perbarui "https://api.example.com/users/1", payload.

// Basic Auth
simpan opsi = kamus
    auth: daftar "username", "password",
    timeout: 10.
selesai.
simpan resp = http ambil "https://api.example.com/private", opsi.
```

### Error Handling
```idpp
coba
    lempar "Ada masalah!".
tangkap galat
    tulis "Error: ", galat, ".".
akhirnya
    tulis "Selesai.".
selesai.
```

### Komentar
```idpp
// Ini komentar satu baris
# Ini juga komentar satu baris
/* Ini komentar
   beberapa baris */
```

## Fungsi Bawaan

| Fungsi | Deskripsi | Contoh Penggunaan |
|---|---|---|
| `panjang` | Panjang teks atau daftar | `panjang "halo"` |
| `huruf besar dari` | Ubah teks jadi kapital | `huruf besar dari "halo"` |
| `huruf kecil dari` | Ubah teks jadi huruf kecil | `huruf kecil dari "HALO"` |
| `potong` | Memotong string dari index ke index | `potong "halo" dari 0 ke 2` |
| `ganti` | Ganti substring | `ganti "budi" dari "b" ke "p"` |
| `mengandung` | Cek ketersediaan substring | `mengandung "budi" cek "ud"` |
| `bulatkan` | Pembulatan angka | `bulatkan 3.5` |
| `lantai` | Pembulatan ke bawah | `lantai 3.9` |
| `langit` | Pembulatan ke atas | `langit 3.1` |
| `mutlak` | Nilai absolut | `mutlak -5` |
| `acak` | Angka acak 0.0 sampai 1.0 | `acak` |
| `maks` | Angka terbesar | `maks 1, 2, 3` |
| `min` | Angka terkecil | `min 1, 2, 3` |
| `akar` | Akar kuadrat | `akar 16` |
| `angka dari` | Konversi teks ke integer | `angka dari "10"` |
| `teks dari` | Konversi angka ke teks | `teks dari 10` |
| `desimal dari`| Konversi teks ke desimal | `desimal dari "10.5"` |
| `tipe dari` | Cek tipe variabel | `tipe dari umur` |

## Kontribusi
Kami sangat terbuka untuk kontribusi! Silakan buka Issue atau buat Pull Request di repository GitHub kami.

## Lisensi
Proyek ini dilisensikan di bawah [MIT License](LICENSE).
