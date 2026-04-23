// Definisi semua token type untuk bahasa ID++

/// Jenis-jenis token yang dikenali oleh lexer
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literal
    String(String),
    Number(f64),
    Boolean(bool), // benar / salah
    Kosong,        // nilai null/nil

    // Identifier (nama variabel, fungsi, dsb.)
    Identifier(String),

    // Output & Input 
    Tulis,   // tulis
    Tanya,   // tanya
    SimpanKe, // "simpan ke" - digunakan setelah pertanyaan tanya

    // Variabel 
    Simpan,    // simpan
    Sebagai,   // sebagai
    Tetap,     // tetap (konstanta)

    // Operasi Matematika
    Tambah,    // tambah (operator infix)
    Kurang,    // kurang (operator infix)
    Kali,      // kali
    Bagi,      // bagi
    Sisa,      // sisa (modulo)
    Pangkat,   // pangkat (power)
    Dengan,    // dengan - digunakan pada "tambah X dengan Y"

    // Operator mutasi variabel
    TambahVar,  // "tambah <var>" (prefix perubahan var)
    KurangiVar, // "kurangi <var>"
    KaliVar,    // "kali <var>"  (tidak ada, tapi disiapkan)
    BagiVar,    // "bagi <var>"

    // Kondisi
    Jika,        // jika
    Maka,        // maka
    LainnyaJika, // atau jika
    Lainnya,     // lainnya
    Selesai,     // selesai

    // Perbandingan
    LebihDari,       // lebih dari           >
    KurangDari,      // kurang dari          <
    SamaDengan,      // sama dengan          ==
    TidakSamaDengan, // tidak sama dengan    !=
    LebihDariSama,   // lebih dari sama      >=
    KurangDariSama,  // kurang dari sama     <=

    // Logika
    Dan,   // dan   &&
    Atau,  // atau  ||
    Bukan, // bukan !

    // Loop
    Selama,  // selama
    Lakukan, // lakukan
    Ulangi,  // ulangi
    // Kali sudah didefinisikan di bagian Operasi Matematika (baris 28)
    // Token yang sama digunakan untuk "ulangi N kali"
    Untuk,   // untuk
    Setiap,  // setiap
    Dalam,   // dalam
    Rentang, // rentang
    Sampai,  // sampai
    Hentikan, // hentikan (break)
    Lanjut,  // lanjut (continue)

    // Fungsi
    Buat,       // buat
    Fungsi,     // fungsi
    Jalankan,   // jalankan
    Kembalikan, // kembalikan

    // Daftar (Array) 
    Daftar,    // daftar
    Di,        // di (akses index: "angka di 0")
    Ubah,      // ubah
    Menjadi,   // menjadi
    Tambahkan, // tambahkan
    Ke,        // ke
    Hapus,     // hapus
    Item,      // item
    Terakhir,  // terakhir
    Pertama,   // pertama
    Dari,      // dari

    // Kamus (Dictionary)
    Kamus,   // kamus
    Ambil,   // ambil (akses nilai kamus)
    Punya,   // punya (cek key)
    Bernilai, // bernilai

    // Error Handling 
    Coba,    // coba
    Tangkap, // tangkap
    Galat,   // galat (nama variabel error)
    Akhirnya, // akhirnya
    Lempar,  // lempar (throw)

    // Modu
    Ekspor,     // ekspor
    AmbilModul, // ambil (konteks modul import) - bedakan dengan Ambil kamus

    // Fungsi Bawaan 
    Panjang,    // panjang
    HurufBesar, // huruf besar dari
    HurufKecil, // huruf kecil dari
    Potong,     // potong
    Ganti,      // ganti
    Mengandung, // mengandung
    Cek,        // cek (digunakan dalam "mengandung X cek Y")
    Bulatkan,   // bulatkan
    Lantai,     // lantai
    Langit,     // langit
    Mutlak,     // mutlak
    Acak,       // acak
    Maks,       // maks
    Min,        // min
    Akar,       // akar
    AnkgaDari,  // angka dari
    TeksDari,   // teks dari
    DesimalDari, // desimal dari
    TipeDari,   // tipe dari

    // Network / HTTP 
    Http,       // http (keyword utama)
    HttpAmbil,  // http ambil  → GET
    HttpKirim,  // http kirim  → POST
    HttpUbah,   // http ubah   → PUT
    HttpHapus,  // http hapus  → DELETE
    HttpPerbarui, // http perbarui → PATCH

    // Operator Assignment
    Assign, // = (alternatif dari "sebagai")

    // Tanda Baca
    Titik,    // .
    Koma,     // ,
    TitikDua, // :

    // Kontrol Alur
    Newline,
    EOF,
}

/// Sebuah token dengan informasi posisi untuk pelaporan error
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,    // nomor baris (1-indexed) untuk pesan error
    pub column: usize,  // nomor kolom (1-indexed)
    pub lexeme: String, // teks asli dari source code
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize, lexeme: String) -> Self {
        Token { kind, line, column, lexeme }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::Boolean(b) => write!(f, "{}", if *b { "benar" } else { "salah" }),
            TokenKind::Kosong => write!(f, "kosong"),
            TokenKind::Identifier(name) => write!(f, "{}", name),
            TokenKind::Titik => write!(f, "."),
            TokenKind::Koma => write!(f, ","),
            TokenKind::TitikDua => write!(f, ":"),
            TokenKind::EOF => write!(f, "EOF"),
            other => write!(f, "{:?}", other),
        }
    }
}
