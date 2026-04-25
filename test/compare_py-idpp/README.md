# 🚀 Benchmark: ID++ vs Python (Fibonacci Rekursif)

Folder ini berisi skrip perbandingan performa antara **ID++ (v0.1.1)** dan **Python 3**. Pengujian menggunakan algoritma **Fibonacci Rekursif**, sebuah metode yang sangat intensif dalam menguji kecepatan eksekusi pemanggilan fungsi (*function call*) dan manajemen *stack frame* pada Virtual Machine.

## 📄 Skrip Pengujian
Kedua skrip menjalankan logika yang identik untuk menghitung angka Fibonacci ke-25.

1. **`fib.idpp`**: Menggunakan sintaks asli ID++ dengan optimasi *bytecode caching*.
2. **`fib.py`**: Menggunakan implementasi standar Python (CPython).

---

## 💻 Cara Menjalankan Benchmark

Gunakan perintah berikut di terminal untuk mendapatkan durasi eksekusi yang akurat.

### 1. Di Windows (PowerShell)
Gunakan perintah `Measure-Command` untuk melihat statistik waktu hingga milidetik:

```powershell
# Jalankan ID++
Measure-Command { idpp fib.idpp | Out-Default }

# Jalankan Python
Measure-Command { python fib.py | Out-Default }
```

### 2. Di Linux / macOS (Terminal)
Gunakan utilitas `time`:

```bash
# Jalankan ID++
time idpp fib.idpp

# Jalankan Python
time python3 fib.py
```

---

## 📊 Hasil Pengujian (Referensi)

Berdasarkan pengujian pada mesin pengembangan (Windows 11), berikut adalah perbandingan durasi eksekusinya:

| Bahasa | Versi | Rata-rata Waktu (TotalMilliseconds) |
| :--- | :--- | :--- |
| **Python** | 3.x (CPython) | **94.2 ms** |
| **ID++** | 0.1.1 (Rust VM) | **155.6 ms** |

---

## 🔍 Penjelasan Teknis

Mengapa ID++ sudah sangat kompetitif meskipun baru dirilis?

### ⚡ Bytecode Caching
ID++ tidak melakukan *parsing* ulang setiap kali dijalankan. Melalui `cache.rs`, ID++ mengonversi kode sumber menjadi format biner `.idppc` menggunakan **Bincode**. Pada eksekusi kedua, VM langsung memuat bytecode ini, menghemat waktu inisialisasi secara signifikan.

### 🛠️ Arsitektur VM
ID++ menggunakan **Stack-based Virtual Machine** yang ditulis dalam **Rust**. 
- **Keunggulan:** Manajemen memori yang sangat ketat dan cepat.
- **Analisis Performa:** Saat ini ID++ v0.1.1 masih menggunakan `HashMap` untuk pencarian variabel (`LoadVar`/`StoreVar`) dan melakukan *clone* pada *context variables* saat pemanggilan fungsi. Hal inilah yang menyebabkan selisih ~60ms dengan Python. 



### 📈 Potensi Optimasi
 Dengan transisi dari `HashMap` ke *Index-based Stack* (menggunakan array/Vec untuk variabel), ID++ memiliki potensi teknis untuk melampaui kecepatan Python dalam tugas-tugas komputasi murni di versi mendatang.

---
**Catatan**: Jalankan skrip ID++ minimal dua kali untuk memastikan sistem *cache* aktif dan memberikan hasil yang adil.