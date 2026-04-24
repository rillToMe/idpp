# Changelog

Semua perubahan penting pada proyek ID++ akan didokumentasikan di sini.

Format mengacu pada [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
dan proyek ini mengikuti [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.1] - 2026-04-24

### Ditambahkan
- **Kata kunci `impor`**: Memuat dan mengeksekusi file `.idpp` lain ke dalam program. Fungsi dan variabel dari file yang diimpor langsung tersedia di program pemanggil.
  ```idpp
  impor "lib/matematika.idpp".
  simpan hasil = jalankan kuadrat dengan 5.
  tulis hasil.
  ```
- **Contoh modul**: Ditambahkan folder `contoh/lib/` beserta `matematika.idpp` sebagai contoh modul yang bisa diimpor.
- **Contoh penggunaan impor**: Ditambahkan `contoh/13_impor.idpp`.

### Diperbaiki
- **Bug parsing argumen fungsi**: Pemanggilan fungsi dengan lebih dari satu argumen (`jalankan f dengan a dan b`) sebelumnya salah mem-parse `dan` sebagai operator logika AND, sehingga argumen kedua tidak terdeteksi. Sekarang `dan` di antara argumen berfungsi sebagai pemisah dengan benar.

---

## [0.1.0] - 2026-04-23

### Pertama Kali Rilis

- **Stack-based Virtual Machine (VM)**: Kode sumber dikompilasi menjadi *flat bytecode* dan dieksekusi oleh VM yang efisien.
- **Sistem Bytecode Caching (`.idppc`)**: File cache otomatis dibuat saat menjalankan skrip. Eksekusi berikutnya langsung memuat cache tanpa proses parsing ulang.
- **Flag `--no-cache`**: Opsi CLI untuk memaksa eksekusi tanpa cache.
- **Tipe Data Dasar**: `Angka`, `Teks`, `Boolean` (`benar`/`salah`), `Kosong`.
- **Struktur Data Koleksi**: `Daftar` (Array) dan `Kamus` (Dictionary/Map).
- **Control Flow**: `jika`/`atau jika`/`lainnya`, `selama`, `ulangi N kali`, `untuk setiap`.
- **Fungsi**: Definisi (`buat fungsi`), pemanggilan (`jalankan`), dan nilai kembalian (`kembalikan`).
- **Penanganan Error**: `coba`, `tangkap galat`, `akhirnya`, `lempar`.
- **HTTP Client Bawaan**: `http ambil` (GET), `http kirim` (POST), `http ubah` (PUT), `http hapus` (DELETE), `http perbarui` (PATCH).
- **REPL Interaktif**: Masuk mode interaktif tanpa argumen (`idpp`).
- **Fungsi Bawaan**: `panjang`, `huruf besar dari`, `huruf kecil dari`, `potong`, `ganti`, `mengandung`, `bulatkan`, `lantai`, `langit`, `mutlak`, `acak`, `maks`, `min`, `akar`, `angka dari`, `teks dari`, `desimal dari`, `tipe dari`.
- **Registrasi Tipe File Windows**: File `.idpp` mendapat ikon dan asosiasi program otomatis di Windows Explorer.
- **Installer Windows**: Skrip Inno Setup (`idpp.iss`) untuk distribusi ke pengguna Windows.
- **Ekstensi VS Code**: Syntax highlighting, snippets, auto-indent, dan deteksi error sederhana untuk editor VS Code.
- **Lisensi MIT** atas nama KyuzenStudio.

---

[0.1.1]: https://github.com/rillToMe/idpp/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/rillToMe/idpp/releases/tag/v0.1.0
