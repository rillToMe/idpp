use clap::{Parser, Subcommand};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::{self, Command},
};

// Struktur Manifest (rak.toml)

/// Representasi lengkap dari file `rak.toml`.
#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(rename = "proyek")]
    proyek: InfoProyek,

    #[serde(rename = "dependensi", default)]
    dependensi: HashMap<String, String>,
}

/// Blok `[proyek]` di dalam `rak.toml`.
#[derive(Debug, Deserialize)]
struct InfoProyek {
    nama: String,
    versi: String,
    titik_masuk: String,
}

// Definisi CLI (clap)

#[derive(Debug, Parser)]
#[command(
    name = "rak",
    version = env!("CARGO_PKG_VERSION"),
    about = "🧰 rak - Package Manager untuk bahasa pemrograman ID++",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    perintah: Perintah,
}

#[derive(Debug, Subcommand)]
enum Perintah {
    /// Membuat proyek ID++ baru dengan struktur folder standar.
    Buat {
        /// Nama proyek yang akan dibuat.
        nama_proyek: String,
    },

    /// Mengunduh dan memasang semua dependensi dari rak.toml.
    Pasang,

    /// Menjalankan program ID++ berdasarkan titik_masuk di rak.toml.
    Jalan,
}

// Fungsi Pembantu: Baca Manifest

/// Membaca dan mem-parse `rak.toml` dari direktori kerja saat ini.
fn baca_manifest() -> Result<Manifest, String> {
    let path = Path::new("rak.toml");

    if !path.exists() {
        return Err(
            "❌ Gagal: File 'rak.toml' tidak ditemukan di direktori ini.\n\
             💡 Pastikan Anda berada di dalam folder proyek yang benar, \
             atau jalankan `rak buat <nama>` untuk membuat proyek baru."
                .to_string(),
        );
    }

    let isi = fs::read_to_string(path).map_err(|e| {
        format!("❌ Gagal membaca file 'rak.toml': {e}")
    })?;

    toml::from_str::<Manifest>(&isi).map_err(|e| {
        format!(
            "❌ Format 'rak.toml' tidak valid. Periksa sintaks TOML Anda.\n   Detail: {e}"
        )
    })
}

// Sub-perintah: `rak buat <nama_proyek>`

fn cmd_buat(nama_proyek: &str) -> Result<(), String> {
    let root = PathBuf::from(nama_proyek);

    // Pastikan folder belum ada agar tidak menimpa proyek yang ada.
    if root.exists() {
        return Err(format!(
            "❌ Gagal: Folder '{}' sudah ada. Pilih nama proyek yang berbeda.",
            nama_proyek
        ));
    }

    // Buat struktur folder
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| {
        format!("❌ Gagal membuat folder '{}': {e}", src_dir.display())
    })?;

    // Tulis rak.toml
    let manifest_isi = format!(
        r#"[proyek]
nama = "{nama}"
versi = "0.1.0"
titik_masuk = "src/utama.idpp"

[dependensi]
# Tambahkan dependensi di sini, contoh:
# matematika = "https://raw.githubusercontent.com/user/repo/main/math.idpp"
"#,
        nama = nama_proyek
    );

    let manifest_path = root.join("rak.toml");
    fs::write(&manifest_path, manifest_isi).map_err(|e| {
        format!("❌ Gagal menulis 'rak.toml': {e}")
    })?;

    // Tulis src/utama.idpp
    let entrypoint_isi = r#"tulis "Halo dari ID++!"
"#;

    let entrypoint_path = src_dir.join("utama.idpp");
    fs::write(&entrypoint_path, entrypoint_isi).map_err(|e| {
        format!("❌ Gagal membuat file '{}': {e}", entrypoint_path.display())
    })?;

    println!("✅ Proyek '{}' berhasil dibuat!", nama_proyek);
    println!("   📂 {}/", nama_proyek);
    println!("   ├── rak.toml");
    println!("   └── src/");
    println!("       └── utama.idpp");
    println!();
    println!("💡 Mulai dengan: cd {} && rak jalan", nama_proyek);

    Ok(())
}

// Sub-perintah: `rak pasang`

fn cmd_pasang() -> Result<(), String> {
    let manifest = baca_manifest()?;

    if manifest.dependensi.is_empty() {
        println!("ℹ️  Tidak ada dependensi yang terdaftar di 'rak.toml'. Selesai.");
        return Ok(());
    }

    // Buat folder .rak_modul/ jika belum ada
    let modul_dir = Path::new(".rak_modul");
    fs::create_dir_all(modul_dir).map_err(|e| {
        format!("❌ Gagal membuat folder '.rak_modul/': {e}")
    })?;

    let client = Client::new();

    println!(
        "📦 Memasang {} dependensi...\n",
        manifest.dependensi.len()
    );

    let mut ada_gagal = false;

    for (nama, url) in &manifest.dependensi {
        print!("   ⏬ Mengunduh '{nama}' dari {url} ... ");

        match client.get(url).send() {
            Ok(resp) if resp.status().is_success() => {
                match resp.bytes() {
                    Ok(bytes) => {
                        // Simpan sebagai .rak_modul/<nama>.idpp
                        let tujuan = modul_dir.join(format!("{nama}.idpp"));
                        match fs::write(&tujuan, &bytes) {
                            Ok(_) => {
                                println!("✅ Tersimpan ke '{}'", tujuan.display());
                            }
                            Err(e) => {
                                println!("❌ Gagal menyimpan: {e}");
                                ada_gagal = true;
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Gagal membaca respons: {e}");
                        ada_gagal = true;
                    }
                }
            }
            Ok(resp) => {
                println!(
                    "❌ Server menolak permintaan (status HTTP {})",
                    resp.status()
                );
                ada_gagal = true;
            }
            Err(e) => {
                println!("❌ Gagal terhubung ke URL: {e}");
                ada_gagal = true;
            }
        }
    }

    println!();
    if ada_gagal {
        Err(
            "⚠️  Beberapa dependensi gagal diunduh. Periksa koneksi internet dan URL di 'rak.toml'."
                .to_string(),
        )
    } else {
        println!("🎉 Semua dependensi berhasil dipasang ke folder '.rak_modul/'!");
        Ok(())
    }
}

// Sub-perintah: `rak jalan`

fn cmd_jalan() -> Result<(), String> {
    let manifest = baca_manifest()?;
    let titik_masuk = &manifest.proyek.titik_masuk;

    // Pastikan file titik masuk ada
    if !Path::new(titik_masuk).exists() {
        return Err(format!(
            "❌ Gagal: File titik masuk '{}' tidak ditemukan.\n\
             💡 Periksa field 'titik_masuk' di 'rak.toml' atau buat file tersebut terlebih dahulu.",
            titik_masuk
        ));
    }

    println!(
        "🚀 Menjalankan '{}' (proyek: {} v{})...\n",
        titik_masuk, manifest.proyek.nama, manifest.proyek.versi
    );

    // Spawn proses `idpp <titik_masuk>` dan teruskan I/O ke terminal pengguna
    let status = Command::new("idpp")
        .arg(titik_masuk)
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "❌ Gagal: Interpreter 'idpp' tidak ditemukan.\n\
                 💡 Pastikan 'idpp' sudah terpasang dan tersedia di PATH sistem Anda.\n\
                 📖 Panduan instalasi: https://idpp-lang.dev/pasang"
                    .to_string()
            } else {
                format!("❌ Gagal menjalankan interpreter 'idpp': {e}")
            }
        })?;

    if !status.success() {
        let kode = status.code().unwrap_or(-1);
        return Err(format!(
            "❌ Program berakhir dengan kode kesalahan: {kode}"
        ));
    }

    Ok(())
}

// Main

fn main() {
    let cli = Cli::parse();

    let hasil = match cli.perintah {
        Perintah::Buat { nama_proyek } => cmd_buat(&nama_proyek),
        Perintah::Pasang => cmd_pasang(),
        Perintah::Jalan => cmd_jalan(),
    };

    if let Err(pesan) = hasil {
        eprintln!("\n{pesan}\n");
        process::exit(1);
    }
}
