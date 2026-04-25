// Definisi semua node AST (Abstract Syntax Tree) untuk ID++

/// Representasi pernyataan (statement) dalam program ID++
#[derive(Debug, Clone)]
pub enum Stmt {
    /// tulis expr1, expr2, ...
    Tulis(Vec<Expr>),

    /// tanya "pertanyaan" simpan ke nama
    Tanya {
        pertanyaan: String,
        variabel: String,
    },

    /// simpan nama sebagai nilai  /  tetap NAMA sebagai nilai
    Simpan {
        nama: String,
        nilai: Expr,
        konstanta: bool, // true = tetap, false = simpan
        line: usize,
    },

    /// tambah/kurangi/kali/bagi variabel dengan nilai
    UbahVar {
        nama: String,
        operasi: OpMatematika,
        nilai: Expr,
        line: usize,
    },

    /// jika ... maka ... lainnya jika ... lainnya ... selesai
    Jika {
        kondisi: Expr,
        tubuh: Vec<Stmt>,
        lainnya_jika: Vec<(Expr, Vec<Stmt>)>,
        lainnya: Option<Vec<Stmt>>,
        line: usize,
    },

    /// selama kondisi lakukan ... selesai
    Selama {
        kondisi: Expr,
        tubuh: Vec<Stmt>,
        line: usize,
    },

    /// ulangi N kali ... selesai
    Ulangi {
        kali: Expr,
        tubuh: Vec<Stmt>,
        line: usize,
    },

    /// untuk setiap var dalam iterable lakukan ... selesai
    Untuk {
        variabel: String,
        iterable: Expr,
        tubuh: Vec<Stmt>,
        line: usize,
    },

    /// buat fungsi nama dengan param1 dan param2 ... selesai
    BuatFungsi {
        nama: String,
        params: Vec<String>,
        tubuh: Vec<Stmt>,
        line: usize,
    },

    /// jalankan fungsi ... (sebagai statement, bukan ekspresi)
    JalankanFungsiStmt {
        nama: String,
        args: Vec<Expr>,
        line: usize,
    },

    /// kembalikan nilai
    Kembalikan(Expr, usize),

    /// hentikan (break)
    Hentikan(usize),

    /// lanjut (continue)
    Lanjut(usize),

    /// coba ... tangkap galat ... akhirnya ... selesai
    Coba {
        tubuh: Vec<Stmt>,
        tangkap: Option<(String, Vec<Stmt>)>, // (nama variabel galat, body)
        akhirnya: Option<Vec<Stmt>>,
        line: usize,
    },

    /// lempar "pesan error"
    Lempar(Expr, usize),

    /// ekspor fungsi nama dengan ...
    EksporFungsi {
        nama: String,
        params: Vec<String>,
        tubuh: Vec<Stmt>,
        line: usize,
    },

    /// ambil "nama-modul"
    AmbilModul(String, usize),

    /// tambahkan nilai ke daftar
    TambahkanKe {
        nilai: Expr,
        daftar: String,
        line: usize,
    },

    /// hapus item terakhir/pertama dari daftar
    HapusItem {
        daftar: String,
        posisi: PosisiItem,
        line: usize,
    },

    /// ubah daftar di index menjadi nilai
    UbahDaftar {
        nama: String,
        index: Expr,
        nilai: Expr,
        line: usize,
    },

    /// ubah kamus kunci menjadi nilai
    UbahKamus {
        nama: String,
        kunci: String,
        nilai: Expr,
        line: usize,
    },

    /// tambah kamus dengan kunci bernilai nilai
    TambahKamus {
        nama: String,
        kunci: String,
        nilai: Expr,
        line: usize,
    },
}

/// Representasi ekspresi dalam program ID++
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal string
    String(String),
    /// Literal angka
    Number(f64),
    /// Literal boolean (benar/salah)
    Boolean(bool),
    /// Nilai kosong
    Kosong,
    /// Referensi ke variabel
    Identifier(String, usize), // (nama, line)
    /// Operasi biner: kiri op kanan
    Binary {
        kiri: Box<Expr>,
        op: Op,
        kanan: Box<Expr>,
        line: usize,
    },
    /// Operasi unary: op operand
    Unary {
        op: OpUnary,
        operand: Box<Expr>,
        line: usize,
    },
    /// Panggilan fungsi sebagai ekspresi
    JalankanFungsi {
        nama: String,
        args: Vec<Expr>,
        line: usize,
    },
    /// Akses elemen daftar: nama di index
    AksesDaftar {
        nama: String,
        index: Box<Expr>,
        line: usize,
    },
    /// Akses nilai kamus: nama ambil kunci
    AksesKamus {
        nama: String,
        kunci: String,
        line: usize,
    },
    /// Cek apakah kamus punya kunci: nama punya kunci
    PunyaKunci {
        nama: String,
        kunci: String,
        line: usize,
    },
    /// Fungsi bawaan
    FungsiBawaan {
        nama: NamaBuiltin,
        args: Vec<Expr>,
        line: usize,
    },
    /// Literal daftar: daftar 1, 2, 3
    Daftar(Vec<Expr>, usize),
    /// Literal kamus
    Kamus(Vec<(String, Expr)>, usize),
    /// Rentang angka: rentang 1 sampai 10
    Rentang {
        mulai: Box<Expr>,
        selesai: Box<Expr>,
        line: usize,
    },
    /// HTTP request: http ambil/kirim/ubah/hapus/perbarui
    HttpCall {
        metode: MetodeHttp,
        args: Vec<Expr>,
        line: usize,
    },
}

/// Operator biner (dua operand)
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Tambah,
    Kurang,
    Kali,
    Bagi,
    Sisa,
    Pangkat,
    LebihDari,
    KurangDari,
    SamaDengan,
    TidakSamaDengan,
    LebihDariSama,
    KurangDariSama,
    Dan,
    Atau,
}

/// Operator unary (satu operand)
#[derive(Debug, Clone)]
pub enum OpUnary {
    Bukan,
    Negatif,
}

/// Jenis operasi untuk perubahan variabel
#[derive(Debug, Clone)]
pub enum OpMatematika {
    Tambah,
    Kurang,
    Kali,
    Bagi,
}

/// Posisi item dalam daftar untuk operasi hapus
#[derive(Debug, Clone)]
pub enum PosisiItem {
    Pertama,
    Terakhir,
    Index(Box<Expr>),
}

/// Nama-nama fungsi bawaan yang tersedia
#[derive(Debug, Clone, PartialEq)]
pub enum NamaBuiltin {
    Panjang,
    HurufBesar,
    HurufKecil,
    Potong,
    Ganti,
    Mengandung,
    Bulatkan,
    Lantai,
    Langit,
    Mutlak,
    Acak,
    Maks,
    Min,
    Akar,
    AnkgaDari,
    TeksDari,
    DesimalDari,
    TipeDari,
}

/// Metode HTTP
#[derive(Debug, Clone, PartialEq)]
pub enum MetodeHttp {
    Ambil,   // GET
    Kirim,   // POST
    Ubah,    // PUT
    Hapus,   // DELETE
    Perbarui, // PATCH
}
