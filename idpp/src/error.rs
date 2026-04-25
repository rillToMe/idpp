// Tipe-tipe error kustom untuk bahasa ID++
use thiserror::Error;

/// Semua jenis error yang bisa terjadi di ID++
#[derive(Debug, Error, Clone)]
pub enum IdppError {
    /// Error saat runtime (eksekusi program)
    #[error("[ID++ Error] Baris {line}: {pesan}")]
    Runtime { line: usize, pesan: String },

    /// Error sintaks saat proses parsing atau lexing
    #[error("[ID++ Error] Sintaks salah di baris {line}: {pesan}")]
    Sintaks { line: usize, pesan: String },

    /// Variabel belum didefinisikan
    #[error("[ID++ Error] Baris {line}: Variabel '{nama}' belum didefinisikan")]
    VariabelTidakAda { line: usize, nama: String },

    /// Fungsi tidak ditemukan
    #[error("[ID++ Error] Baris {line}: Fungsi '{nama}' tidak ditemukan")]
    FungsiTidakAda { line: usize, nama: String },

    /// Pembagian dengan nol
    #[error("[ID++ Error] Baris {line}: Tidak bisa membagi dengan nol")]
    BagiNol { line: usize },

    /// Tipe data tidak sesuai
    #[error("[ID++ Error] Baris {line}: Tipe data tidak cocok - diharapkan {diharapkan}, dapat {dapat}")]
    TipeTidakCocok { line: usize, diharapkan: String, dapat: String },

    /// Konstanta tidak bisa diubah
    #[error("[ID++ Error] Baris {line}: Konstanta '{nama}' tidak bisa diubah")]
    KonstantaTidakBisaDiubah { line: usize, nama: String },

    /// Index di luar batas daftar
    #[error("[ID++ Error] Baris {line}: Index {index} di luar batas daftar (panjang: {panjang})")]
    IndexDiluarBatas { line: usize, index: i64, panjang: usize },

    /// File tidak ditemukan
    #[error("[ID++ Error] File '{path}' tidak ditemukan")]
    FileTidakAda { path: String },

    /// Modul tidak ditemukan
    #[error("[ID++ Error] Modul '{nama}' tidak ditemukan")]
    ModulTidakAda { nama: String },

    /// Error yang dilempar oleh pengguna dengan kata kunci `lempar`
    #[error("{0}")]
    LemparUser(String),

    /// Jumlah argumen tidak sesuai
    #[error("[ID++ Error] Baris {line}: Fungsi '{nama}' membutuhkan {diharapkan} argumen, diberikan {dapat}")]
    JumlahArgumenSalah { line: usize, nama: String, diharapkan: usize, dapat: usize },

    /// Kunci kamus tidak ditemukan
    #[error("[ID++ Error] Baris {line}: Kunci '{kunci}' tidak ditemukan dalam kamus")]
    KunciTidakAda { line: usize, kunci: String },
}

/// Signal khusus untuk control flow internal (bukan error nyata)
/// Digunakan untuk mengimplementasikan return, break, continue
#[derive(Debug, Clone)]
pub enum ControlFlow {
    Normal,
    Return(crate::environment::Nilai),
    Break,
    Continue,
    Throw(String, usize), // pesan error, nomor baris
}
