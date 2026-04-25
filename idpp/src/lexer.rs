// Tokenizer untuk bahasa ID++
// Mengubah source code menjadi daftar token

use crate::token::{Token, TokenKind};
use crate::error::IdppError;

/// Struktur lexer yang memproses source code karakter per karakter
pub struct Lexer {
    source: Vec<char>,
    pos: usize,   // posisi saat ini
    line: usize,  // baris saat ini (1-indexed)
    column: usize, // kolom saat ini (1-indexed)
}

impl Lexer {
    /// Buat lexer baru dari source code
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenisasi seluruh source code
    pub fn tokenize(&mut self) -> Result<Vec<Token>, IdppError> {
        let mut tokens = Vec::new();

        loop {
            // Lewati spasi (kecuali newline)
            self.skip_whitespace();

            if self.is_at_end() {
                tokens.push(Token::new(TokenKind::EOF, self.line, self.column, String::new()));
                break;
            }

            let ch = match self.peek() {
                Some(c) => *c,
                None => break,
            };

            match ch {
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                '.' => {
                    let col = self.column;
                    self.advance();
                    tokens.push(Token::new(TokenKind::Titik, self.line, col, ".".to_string()));
                }
                ',' => {
                    let col = self.column;
                    self.advance();
                    tokens.push(Token::new(TokenKind::Koma, self.line, col, ",".to_string()));
                }
                ':' => {
                    let col = self.column;
                    self.advance();
                    tokens.push(Token::new(TokenKind::TitikDua, self.line, col, ":".to_string()));
                }
                '=' => {
                    let col = self.column;
                    self.advance();
                    if self.peek().copied() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::SamaDengan, self.line, col, "==".to_string()));
                    } else {
                        tokens.push(Token::new(TokenKind::Assign, self.line, col, "=".to_string()));
                    }
                }
                '!' => {
                    let col = self.column;
                    self.advance();
                    if self.peek().copied() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::TidakSamaDengan, self.line, col, "!=".to_string()));
                    } else {
                        return Err(IdppError::Sintaks {
                            line: self.line,
                            pesan: "Karakter '!' harus diikuti '=' (contoh: !=)".to_string(),
                        });
                    }
                }
                '>' => {
                    let col = self.column;
                    self.advance();
                    if self.peek().copied() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::LebihDariSama, self.line, col, ">=".to_string()));
                    } else {
                        tokens.push(Token::new(TokenKind::LebihDari, self.line, col, ">".to_string()));
                    }
                }
                '<' => {
                    let col = self.column;
                    self.advance();
                    if self.peek().copied() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::KurangDariSama, self.line, col, "<=".to_string()));
                    } else {
                        tokens.push(Token::new(TokenKind::KurangDari, self.line, col, "<".to_string()));
                    }
                }
                '+' => {
                    let col = self.column;
                    self.advance();
                    tokens.push(Token::new(TokenKind::Tambah, self.line, col, "+".to_string()));
                }
                '-' => {
                    // Cek: apakah ini operator kurang (infix) atau angka negatif?
                    // Jika token sebelumnya adalah nilai (angka, identifier, dll), ini infix minus
                    let is_infix = tokens.last().map_or(false, |t| matches!(t.kind,
                        TokenKind::Number(_) | TokenKind::Identifier(_) |
                        TokenKind::String(_) | TokenKind::Boolean(_)
                    ));
                    if is_infix {
                        let col = self.column;
                        self.advance();
                        tokens.push(Token::new(TokenKind::Kurang, self.line, col, "-".to_string()));
                    } else if self.peek_next().map_or(false, |n| n.is_ascii_digit()) {
                        // Angka negatif: -5, -3.14
                        let tok = self.scan_number();
                        tokens.push(tok);
                    } else {
                        let col = self.column;
                        self.advance();
                        tokens.push(Token::new(TokenKind::Kurang, self.line, col, "-".to_string()));
                    }
                }
                '*' => {
                    if self.peek_next() == Some('/') {
                        return Err(IdppError::Sintaks {
                            line: self.line,
                            pesan: "Penutup komentar '*/' tanpa pembuka '/*'".to_string(),
                        });
                    }
                    let col = self.column;
                    self.advance();
                    tokens.push(Token::new(TokenKind::Kali, self.line, col, "*".to_string()));
                }
                '/' => {
                    if self.peek_next() == Some('/') {
                        self.skip_line_comment();
                    } else if self.peek_next() == Some('*') {
                        self.skip_block_comment()?;
                    } else {
                        let col = self.column;
                        self.advance();
                        tokens.push(Token::new(TokenKind::Bagi, self.line, col, "/".to_string()));
                    }
                }
                '"' => {
                    let tok = self.scan_string()?;
                    tokens.push(tok);
                }
                '#' => {
                    self.skip_line_comment();
                }
                c if c.is_ascii_digit() => {
                    let tok = self.scan_number();
                    tokens.push(tok);
                }
                c if c.is_alphabetic() || c == '_' => {
                    let tok = self.scan_identifier_or_keyword()?;
                    tokens.push(tok);
                }
                c => {
                    return Err(IdppError::Sintaks {
                        line: self.line,
                        pesan: format!("Karakter tidak dikenal: '{}'", c),
                    });
                }
            }
        }

        Ok(tokens)
    }

    // Metode internal

    /// Ambil karakter saat ini tanpa advance
    fn peek(&self) -> Option<&char> {
        self.source.get(self.pos)
    }

    /// Lihat karakter berikutnya tanpa advance
    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    /// Ambil dan maju ke karakter berikutnya
    fn advance(&mut self) -> Option<char> {
        if self.pos < self.source.len() {
            let c = self.source[self.pos];
            self.pos += 1;
            self.column += 1;
            Some(c)
        } else {
            None
        }
    }

    /// Apakah sudah sampai akhir source?
    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    /// Lewati spasi dan tab (bukan newline)
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Lewati komentar satu baris (// atau #)
    fn skip_line_comment(&mut self) {
        while let Some(&c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Lewati komentar blok /* ... */
    fn skip_block_comment(&mut self) -> Result<(), IdppError> {
        let start_line = self.line;
        // Lewati /*
        self.advance(); // /
        self.advance(); // *
        loop {
            if self.is_at_end() {
                return Err(IdppError::Sintaks {
                    line: start_line,
                    pesan: "Komentar blok '/*' tidak ditutup".to_string(),
                });
            }
            let c = self.advance().unwrap();
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else if c == '*' {
                if self.peek() == Some(&'/') {
                    self.advance(); // lewati '/'
                    break;
                }
            }
        }
        Ok(())
    }

    /// Scan literal string dalam tanda kutip dua
    fn scan_string(&mut self) -> Result<Token, IdppError> {
        let col = self.column;
        let start_line = self.line;
        self.advance(); // lewati '"' pembuka
        let mut result = String::new();

        loop {
            if self.is_at_end() {
                return Err(IdppError::Sintaks {
                    line: start_line,
                    pesan: "String tidak ditutup dengan tanda kutip".to_string(),
                });
            }
            let c = self.advance().unwrap();
            match c {
                '"' => break,
                '\n' => {
                    // String boleh multiline
                    result.push('\n');
                    self.line += 1;
                    self.column = 1;
                }
                '\\' => {
                    // Escape sequences
                    match self.advance() {
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('"') => result.push('"'),
                        Some('\\') => result.push('\\'),
                        Some(c) => {
                            result.push('\\');
                            result.push(c);
                        }
                        None => return Err(IdppError::Sintaks {
                            line: self.line,
                            pesan: "Escape sequence tidak lengkap".to_string(),
                        }),
                    }
                }
                c => result.push(c),
            }
        }

        Ok(Token::new(
            TokenKind::String(result.clone()),
            start_line,
            col,
            format!("\"{}\"", result),
        ))
    }

    /// Scan literal angka (integer atau float)
    fn scan_number(&mut self) -> Token {
        let col = self.column;
        let start_line = self.line;
        let mut num_str = String::new();

        // Angka negatif
        if self.peek() == Some(&'-') {
            num_str.push('-');
            self.advance();
        }

        // Bagian integer
        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Bagian desimal
        if self.peek() == Some(&'.') && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            num_str.push('.');
            self.advance();
            while let Some(&c) = self.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let value: f64 = num_str.parse().unwrap_or(0.0);
        Token::new(TokenKind::Number(value), start_line, col, num_str)
    }

    /// Scan identifier atau keyword (termasuk keyword multi-kata)
    fn scan_identifier_or_keyword(&mut self) -> Result<Token, IdppError> {
        let col = self.column;
        let start_line = self.line;
        let mut word = String::new();

        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                word.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Coba cocokkan keyword multi-kata dengan lookahead
        let kind = self.resolve_keyword(&word, start_line)?;
        Ok(Token::new(kind, start_line, col, word))
    }

    /// Peeking satu atau lebih kata berikutnya untuk multi-word keywords
    fn peek_next_word(&self) -> Option<String> {
        let mut i = self.pos;
        // lewati spasi
        while i < self.source.len() && (self.source[i] == ' ' || self.source[i] == '\t') {
            i += 1;
        }
        if i >= self.source.len() { return None; }

        let mut word = String::new();
        while i < self.source.len() {
            let c = self.source[i];
            if c.is_alphanumeric() || c == '_' {
                word.push(c);
                i += 1;
            } else {
                break;
            }
        }
        if word.is_empty() { None } else { Some(word) }
    }

    /// Lewati spasi dan konsumsi kata berikutnya
    fn consume_next_word(&mut self) -> String {
        // Lewati spasi
        while let Some(&c) = self.peek() {
            if c == ' ' || c == '\t' { self.advance(); } else { break; }
        }
        let mut word = String::new();
        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                word.push(c);
                self.advance();
            } else {
                break;
            }
        }
        word
    }

    /// Resolusi keyword: ubah kata (atau beberapa kata) menjadi TokenKind
    fn resolve_keyword(&mut self, word: &str, line: usize) -> Result<TokenKind, IdppError> {
        match word {
            // Output & Input
            "tulis" => Ok(TokenKind::Tulis),
            "tanya" => Ok(TokenKind::Tanya),

            // Variabel
            "simpan" => {
                // Cek "simpan ke" (untuk tanya...simpan ke)
                if self.peek_next_word().as_deref() == Some("ke") {
                    self.consume_next_word();
                    Ok(TokenKind::SimpanKe)
                } else {
                    Ok(TokenKind::Simpan)
                }
            }
            "sebagai" => Ok(TokenKind::Sebagai),
            "tetap" => Ok(TokenKind::Tetap),

            // Matematika
            "tambah" => Ok(TokenKind::Tambah),
            "tambahkan" => Ok(TokenKind::Tambahkan),
            "kurang" | "kurangi" => {
                // "kurangi" adalah prefix mutasi, "kurang" bisa jadi operator atau KurangDari
                if word == "kurangi" {
                    Ok(TokenKind::KurangiVar)
                } else {
                    // "kurang" bisa diikuti "dari" atau "dari sama"
                    if self.peek_next_word().as_deref() == Some("dari") {
                        self.consume_next_word(); // "dari"
                        if self.peek_next_word().as_deref() == Some("sama") {
                            self.consume_next_word(); // "sama"
                            Ok(TokenKind::KurangDariSama)
                        } else {
                            Ok(TokenKind::KurangDari)
                        }
                    } else {
                        Ok(TokenKind::Kurang)
                    }
                }
            }
            "kali" => Ok(TokenKind::Kali),
            "bagi" => Ok(TokenKind::Bagi),
            "sisa" => Ok(TokenKind::Sisa),
            "pangkat" => Ok(TokenKind::Pangkat),
            "dengan" => Ok(TokenKind::Dengan),
            "ke" => Ok(TokenKind::Ke),

            // Kondisi
            "jika" => Ok(TokenKind::Jika),
            "maka" => Ok(TokenKind::Maka),
            "lainnya" => Ok(TokenKind::Lainnya),
            "atau" => {
                // "atau jika" → else-if
                if self.peek_next_word().as_deref() == Some("jika") {
                    self.consume_next_word();
                    Ok(TokenKind::LainnyaJika)
                } else {
                    Ok(TokenKind::Identifier("atau".to_string()))
                }
            }
            "selesai" => Ok(TokenKind::Selesai),

            // Perbandingan
            "lebih" => {
                if self.peek_next_word().as_deref() == Some("dari") {
                    self.consume_next_word(); // "dari"
                    if self.peek_next_word().as_deref() == Some("sama") {
                        self.consume_next_word(); // "sama"
                        Ok(TokenKind::LebihDariSama)
                    } else {
                        Ok(TokenKind::LebihDari)
                    }
                } else {
                    Ok(TokenKind::Identifier("lebih".to_string()))
                }
            }
            "sama" => {
                if self.peek_next_word().as_deref() == Some("dengan") {
                    self.consume_next_word();
                    Ok(TokenKind::SamaDengan)
                } else {
                    Ok(TokenKind::Identifier("sama".to_string()))
                }
            }
            "tidak" => {
                if self.peek_next_word().as_deref() == Some("sama") {
                    self.consume_next_word(); // "sama"
                    if self.peek_next_word().as_deref() == Some("dengan") {
                        self.consume_next_word(); // "dengan"
                        Ok(TokenKind::TidakSamaDengan)
                    } else {
                        Err(IdppError::Sintaks {
                            line,
                            pesan: "Setelah 'tidak sama' harus diikuti 'dengan'".to_string(),
                        })
                    }
                } else {
                    Err(IdppError::Sintaks {
                        line,
                        pesan: "Kata 'tidak' hanya valid dalam 'tidak sama dengan'".to_string(),
                    })
                }
            }

            // Logika
            "dan" => Ok(TokenKind::Dan),
            "atau" => Ok(TokenKind::Atau),
            "bukan" => Ok(TokenKind::Bukan),

            // Loop
            "selama" => Ok(TokenKind::Selama),
            "lakukan" => Ok(TokenKind::Lakukan),
            "ulangi" => Ok(TokenKind::Ulangi),
            "untuk" => {
                if self.peek_next_word().as_deref() == Some("setiap") {
                    self.consume_next_word();
                    Ok(TokenKind::Untuk)
                } else {
                    Ok(TokenKind::Untuk)
                }
            }
            "setiap" => Ok(TokenKind::Setiap),
            "dalam" => Ok(TokenKind::Dalam),
            "rentang" => Ok(TokenKind::Rentang),
            "sampai" => Ok(TokenKind::Sampai),
            "hentikan" => Ok(TokenKind::Hentikan),
            "lanjut" => Ok(TokenKind::Lanjut),

            // Fungsi
            "buat" => Ok(TokenKind::Buat),
            "fungsi" => Ok(TokenKind::Fungsi),
            "jalankan" => Ok(TokenKind::Jalankan),
            "kembalikan" => Ok(TokenKind::Kembalikan),

            // Daftar
            "daftar" => Ok(TokenKind::Daftar),
            "di" => Ok(TokenKind::Di),
            "ubah" => Ok(TokenKind::Ubah),
            "menjadi" => Ok(TokenKind::Menjadi),
            "hapus" => Ok(TokenKind::Hapus),
            "item" => {
                if self.peek_next_word().as_deref() == Some("terakhir") {
                    self.consume_next_word();
                    Ok(TokenKind::Terakhir) // digunakan sebagai marker hapus terakhir
                } else if self.peek_next_word().as_deref() == Some("pertama") {
                    self.consume_next_word();
                    Ok(TokenKind::Pertama)
                } else {
                    Ok(TokenKind::Item)
                }
            }
            "terakhir" => Ok(TokenKind::Terakhir),
            "pertama" => Ok(TokenKind::Pertama),
            "dari" => Ok(TokenKind::Dari),

            // Kamus
            "kamus" => Ok(TokenKind::Kamus),
            "ambil" => Ok(TokenKind::Ambil),
            "punya" => Ok(TokenKind::Punya),
            "bernilai" => Ok(TokenKind::Bernilai),

            // Error Handling
            "coba" => Ok(TokenKind::Coba),
            "tangkap" => Ok(TokenKind::Tangkap),
            "galat" => Ok(TokenKind::Galat),
            "akhirnya" => Ok(TokenKind::Akhirnya),
            "lempar" => Ok(TokenKind::Lempar),

            // Modul
            "ekspor" => Ok(TokenKind::Ekspor),
            "impor"  => Ok(TokenKind::AmbilModul),

            // Network / HTTP
            "http" => {
                match self.peek_next_word().as_deref() {
                    Some("ambil")  => { self.consume_next_word(); Ok(TokenKind::HttpAmbil) }
                    Some("kirim")  => { self.consume_next_word(); Ok(TokenKind::HttpKirim) }
                    Some("ubah")   => { self.consume_next_word(); Ok(TokenKind::HttpUbah) }
                    Some("hapus")  => { self.consume_next_word(); Ok(TokenKind::HttpHapus) }
                    Some("perbarui") => { self.consume_next_word(); Ok(TokenKind::HttpPerbarui) }
                    _ => Ok(TokenKind::Http),
                }
            }

            // Boolean literal
            "benar" => Ok(TokenKind::Boolean(true)),
            "salah" => Ok(TokenKind::Boolean(false)),
            "kosong" => Ok(TokenKind::Kosong),

            // Fungsi Bawaan
            "panjang" => Ok(TokenKind::Panjang),
            "huruf" => {
                let next = self.peek_next_word().unwrap_or_default();
                if next == "besar" {
                    self.consume_next_word(); // "besar"
                    if self.peek_next_word().as_deref() == Some("dari") {
                        self.consume_next_word(); // "dari"
                    }
                    Ok(TokenKind::HurufBesar)
                } else if next == "kecil" {
                    self.consume_next_word(); // "kecil"
                    if self.peek_next_word().as_deref() == Some("dari") {
                        self.consume_next_word(); // "dari"
                    }
                    Ok(TokenKind::HurufKecil)
                } else {
                    Ok(TokenKind::Identifier("huruf".to_string()))
                }
            }
            "potong" => Ok(TokenKind::Potong),
            "ganti" => Ok(TokenKind::Ganti),
            "mengandung" => Ok(TokenKind::Mengandung),
            "cek" => Ok(TokenKind::Cek),
            "bulatkan" => Ok(TokenKind::Bulatkan),
            "lantai" => Ok(TokenKind::Lantai),
            "langit" => Ok(TokenKind::Langit),
            "mutlak" => Ok(TokenKind::Mutlak),
            "acak" => Ok(TokenKind::Acak),
            "maks" => Ok(TokenKind::Maks),
            "min" => Ok(TokenKind::Min),
            "akar" => Ok(TokenKind::Akar),
            "angka" => {
                if self.peek_next_word().as_deref() == Some("dari") {
                    self.consume_next_word();
                    Ok(TokenKind::AnkgaDari)
                } else {
                    Ok(TokenKind::Identifier("angka".to_string()))
                }
            }
            "teks" => {
                if self.peek_next_word().as_deref() == Some("dari") {
                    self.consume_next_word();
                    Ok(TokenKind::TeksDari)
                } else {
                    Ok(TokenKind::Identifier("teks".to_string()))
                }
            }
            "desimal" => {
                if self.peek_next_word().as_deref() == Some("dari") {
                    self.consume_next_word();
                    Ok(TokenKind::DesimalDari)
                } else {
                    Ok(TokenKind::Identifier("desimal".to_string()))
                }
            }
            "tipe" => {
                if self.peek_next_word().as_deref() == Some("dari") {
                    self.consume_next_word();
                    Ok(TokenKind::TipeDari)
                } else {
                    Ok(TokenKind::Identifier("tipe".to_string()))
                }
            }

            // Identifier biasa
            other => Ok(TokenKind::Identifier(other.to_string())),
        }
    }
}
