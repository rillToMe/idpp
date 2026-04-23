// Mengubah daftar token menjadi AST

use crate::ast::*;
use crate::error::IdppError;
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, IdppError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // Abaikan titik ekstra jika ada
            if self.check(&TokenKind::Titik) {
                self.advance();
                continue;
            }
            statements.push(self.parse_stmt()?);
        }

        Ok(statements)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, IdppError> {
        let token = self.peek().clone();

        match &token.kind {
            TokenKind::Tulis => self.parse_tulis(),
            TokenKind::Tanya => self.parse_tanya(),
            TokenKind::Simpan | TokenKind::Tetap => self.parse_simpan(),
            TokenKind::Jika => self.parse_jika(),
            TokenKind::Selama => self.parse_selama(),
            TokenKind::Ulangi => self.parse_ulangi(),
            TokenKind::Untuk => self.parse_untuk(),
            TokenKind::Buat => self.parse_buat_fungsi(),
            TokenKind::Jalankan => self.parse_jalankan_stmt(),
            TokenKind::Kembalikan => self.parse_kembalikan(),
            TokenKind::Hentikan => {
                let line = self.advance().line;
                self.expect_titik()?;
                Ok(Stmt::Hentikan(line))
            }
            TokenKind::Lanjut => {
                let line = self.advance().line;
                self.expect_titik()?;
                Ok(Stmt::Lanjut(line))
            }
            TokenKind::Coba => self.parse_coba(),
            TokenKind::Lempar => {
                let line = self.advance().line;
                let expr = self.parse_expr()?;
                self.expect_titik()?;
                Ok(Stmt::Lempar(expr, line))
            }
            TokenKind::Ekspor => self.parse_ekspor(),
            TokenKind::AmbilModul => {
                let line = self.advance().line;
                let nama = self.parse_string_literal()?;
                self.expect_titik()?;
                Ok(Stmt::AmbilModul(nama, line))
            }
            TokenKind::Tambahkan => self.parse_tambahkan_ke(),
            TokenKind::Hapus => self.parse_hapus_item(),
            TokenKind::Ubah => self.parse_ubah(),
            TokenKind::TambahVar | TokenKind::KurangiVar | TokenKind::KaliVar | TokenKind::BagiVar => {
                self.parse_ubah_var()
            }
            // "tambah X dengan Y" → mutasi variabel ATAU "tambah kamus dengan kunci bernilai nilai"
            TokenKind::Tambah => self.parse_tambah_stmt(),
            TokenKind::Kurang => self.parse_kurang_stmt(),
            TokenKind::KurangiVar => self.parse_ubah_var(),
            TokenKind::Kali => self.parse_kali_stmt(),
            TokenKind::Bagi => self.parse_bagi_stmt(),
            _ => {
                // Bisa jadi pemanggilan method, dll.
                // Untuk kesederhanaan kita asumsikan error jika bukan statement valid
                Err(IdppError::Sintaks {
                    line: token.line,
                    pesan: format!("Pernyataan tidak valid diawali dengan: {:?}", token.kind),
                })
            }
        }
    }

    /// "tambah X dengan Y" - mutasi variabel atau "tambah kamus dengan kunci bernilai nilai"
    fn parse_tambah_stmt(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line; // tambah
        let nama = self.parse_identifier()?;
        self.expect(TokenKind::Dengan)?;
        
        // Cek apakah ini "tambah kamus dengan kunci bernilai nilai"
        // Setelah "dengan" jika ada identifier diikuti "bernilai" -> TambahKamus
        let saved_pos = self.pos;
        if let Ok(kunci) = self.parse_identifier() {
            if self.check(&TokenKind::Bernilai) {
                self.advance(); // bernilai
                let nilai = self.parse_expr()?;
                self.expect_titik()?;
                return Ok(Stmt::TambahKamus { nama, kunci, nilai, line });
            }
            // Bukan bernilai - kembalikan posisi parser
            self.pos = saved_pos;
        } else {
            self.pos = saved_pos;
        }
        
        let nilai = self.parse_expr()?;
        self.expect_titik()?;
        Ok(Stmt::UbahVar { nama, operasi: OpMatematika::Tambah, nilai, line })
    }
    
    /// "kurang X dengan Y" (tidak dipakai, kurangi sudah ada)
    fn parse_kurang_stmt(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line;
        let nama = self.parse_identifier()?;
        self.expect(TokenKind::Dengan)?;
        let nilai = self.parse_expr()?;
        self.expect_titik()?;
        Ok(Stmt::UbahVar { nama, operasi: OpMatematika::Kurang, nilai, line })
    }
    
    /// "kali X dengan Y"
    fn parse_kali_stmt(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line;
        let nama = self.parse_identifier()?;
        self.expect(TokenKind::Dengan)?;
        let nilai = self.parse_expr()?;
        self.expect_titik()?;
        Ok(Stmt::UbahVar { nama, operasi: OpMatematika::Kali, nilai, line })
    }
    
    /// "bagi X dengan Y"
    fn parse_bagi_stmt(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line;
        let nama = self.parse_identifier()?;
        self.expect(TokenKind::Dengan)?;
        let nilai = self.parse_expr()?;
        self.expect_titik()?;
        Ok(Stmt::UbahVar { nama, operasi: OpMatematika::Bagi, nilai, line })
    }

    fn parse_tulis(&mut self) -> Result<Stmt, IdppError> {
        self.advance(); // tulis
        let mut exprs = Vec::new();
        exprs.push(self.parse_expr()?);

        while self.check(&TokenKind::Koma) {
            self.advance();
            exprs.push(self.parse_expr()?);
        }

        self.expect_titik()?;
        Ok(Stmt::Tulis(exprs))
    }

    fn parse_tanya(&mut self) -> Result<Stmt, IdppError> {
        self.advance(); // tanya
        let pertanyaan = self.parse_string_literal()?;
        
        self.expect(TokenKind::SimpanKe)?;
        let variabel = self.parse_identifier()?;
        
        self.expect_titik()?;
        Ok(Stmt::Tanya { pertanyaan, variabel })
    }

    fn parse_simpan(&mut self) -> Result<Stmt, IdppError> {
        let is_const = self.advance().kind == TokenKind::Tetap;
        let line = self.peek().line;
        
        let nama = self.parse_identifier()?;
        // Terima "sebagai" atau "=" untuk assignment
        if self.check(&TokenKind::Sebagai) || self.check(&TokenKind::Assign) {
            self.advance();
        } else {
            return Err(IdppError::Sintaks {
                line: self.peek().line,
                pesan: "Diharapkan 'sebagai' atau '=' setelah nama variabel".into(),
            });
        }
        
        let nilai = self.parse_expr()?;
        self.expect_titik()?;
        
        Ok(Stmt::Simpan { nama, nilai, konstanta: is_const, line })
    }

    fn parse_ubah_var(&mut self) -> Result<Stmt, IdppError> {
        let op_token = self.advance().clone();
        let operasi = match op_token.kind {
            TokenKind::TambahVar => OpMatematika::Tambah,
            TokenKind::KurangiVar => OpMatematika::Kurang,
            TokenKind::KaliVar => OpMatematika::Kali,
            TokenKind::BagiVar => OpMatematika::Bagi,
            _ => unreachable!(),
        };
        let nama = self.parse_identifier()?;
        self.expect(TokenKind::Dengan)?;
        let nilai = self.parse_expr()?;
        self.expect_titik()?;
        Ok(Stmt::UbahVar { nama, operasi, nilai, line: op_token.line })
    }

    fn parse_jika(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line; // jika
        let kondisi = self.parse_expr()?;
        self.expect(TokenKind::Maka)?;
        
        let mut tubuh = Vec::new();
        while !self.check(&TokenKind::LainnyaJika) && !self.check(&TokenKind::Lainnya) && !self.check(&TokenKind::Selesai) && !self.is_at_end() {
            tubuh.push(self.parse_stmt()?);
        }
        
        let mut lainnya_jika = Vec::new();
        while self.check(&TokenKind::LainnyaJika) {
            self.advance();
            let cond = self.parse_expr()?;
            self.expect(TokenKind::Maka)?;
            let mut body = Vec::new();
            while !self.check(&TokenKind::LainnyaJika) && !self.check(&TokenKind::Lainnya) && !self.check(&TokenKind::Selesai) && !self.is_at_end() {
                body.push(self.parse_stmt()?);
            }
            lainnya_jika.push((cond, body));
        }
        
        let mut lainnya = None;
        if self.check(&TokenKind::Lainnya) {
            self.advance();
            let mut body = Vec::new();
            while !self.check(&TokenKind::Selesai) && !self.is_at_end() {
                body.push(self.parse_stmt()?);
            }
            lainnya = Some(body);
        }
        
        self.expect(TokenKind::Selesai)?;
        self.expect_titik()?;
        
        Ok(Stmt::Jika { kondisi, tubuh, lainnya_jika, lainnya, line })
    }

    fn parse_selama(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line; // selama
        let kondisi = self.parse_expr()?;
        self.expect(TokenKind::Lakukan)?;
        
        let mut tubuh = Vec::new();
        while !self.check(&TokenKind::Selesai) && !self.is_at_end() {
            tubuh.push(self.parse_stmt()?);
        }
        
        self.expect(TokenKind::Selesai)?;
        self.expect_titik()?;
        
        Ok(Stmt::Selama { kondisi, tubuh, line })
    }

    fn parse_ulangi(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line; // ulangi
        let kali = self.parse_expr()?;
        self.expect(TokenKind::Kali)?;
        
        let mut tubuh = Vec::new();
        while !self.check(&TokenKind::Selesai) && !self.is_at_end() {
            tubuh.push(self.parse_stmt()?);
        }
        
        self.expect(TokenKind::Selesai)?;
        self.expect_titik()?;
        
        Ok(Stmt::Ulangi { kali, tubuh, line })
    }

    fn parse_untuk(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line; // untuk
        let variabel = self.parse_identifier()?;
        self.expect(TokenKind::Dalam)?;
        let iterable = self.parse_expr()?;
        self.expect(TokenKind::Lakukan)?;
        
        let mut tubuh = Vec::new();
        while !self.check(&TokenKind::Selesai) && !self.is_at_end() {
            tubuh.push(self.parse_stmt()?);
        }
        
        self.expect(TokenKind::Selesai)?;
        self.expect_titik()?;
        
        Ok(Stmt::Untuk { variabel, iterable, tubuh, line })
    }

    fn parse_buat_fungsi(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line; // buat
        self.expect(TokenKind::Fungsi)?;
        let nama = self.parse_identifier()?;
        
        let mut params = Vec::new();
        if self.check(&TokenKind::Dengan) {
            self.advance();
            params.push(self.parse_identifier()?);
            while self.check(&TokenKind::Dan) {
                self.advance();
                params.push(self.parse_identifier()?);
            }
        }
        
        let mut tubuh = Vec::new();
        while !self.check(&TokenKind::Selesai) && !self.is_at_end() {
            tubuh.push(self.parse_stmt()?);
        }
        
        self.expect(TokenKind::Selesai)?;
        self.expect_titik()?;
        
        Ok(Stmt::BuatFungsi { nama, params, tubuh, line })
    }

    fn parse_jalankan_stmt(&mut self) -> Result<Stmt, IdppError> {
        let expr = self.parse_jalankan_expr()?;
        self.expect_titik()?;
        if let Expr::JalankanFungsi { nama, args, line } = expr {
            Ok(Stmt::JalankanFungsiStmt { nama, args, line })
        } else {
            unreachable!()
        }
    }

    fn parse_kembalikan(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line; // kembalikan
        let expr = self.parse_expr()?;
        self.expect_titik()?;
        Ok(Stmt::Kembalikan(expr, line))
    }

    fn parse_coba(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line; // coba
        
        let mut tubuh = Vec::new();
        while !self.check(&TokenKind::Tangkap) && !self.check(&TokenKind::Akhirnya) && !self.check(&TokenKind::Selesai) && !self.is_at_end() {
            tubuh.push(self.parse_stmt()?);
        }
        
        let mut tangkap = None;
        if self.check(&TokenKind::Tangkap) {
            self.advance();
            let var_galat = self.parse_identifier()?; // e.g. tangkap galat
            let mut tangkap_tubuh = Vec::new();
            while !self.check(&TokenKind::Akhirnya) && !self.check(&TokenKind::Selesai) && !self.is_at_end() {
                tangkap_tubuh.push(self.parse_stmt()?);
            }
            tangkap = Some((var_galat, tangkap_tubuh));
        }
        
        let mut akhirnya = None;
        if self.check(&TokenKind::Akhirnya) {
            self.advance();
            let mut akhirnya_tubuh = Vec::new();
            while !self.check(&TokenKind::Selesai) && !self.is_at_end() {
                akhirnya_tubuh.push(self.parse_stmt()?);
            }
            akhirnya = Some(akhirnya_tubuh);
        }
        
        self.expect(TokenKind::Selesai)?;
        self.expect_titik()?;
        
        Ok(Stmt::Coba { tubuh, tangkap, akhirnya, line })
    }

    fn parse_ekspor(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line; // ekspor
        self.expect(TokenKind::Fungsi)?;
        let nama = self.parse_identifier()?;
        
        let mut params = Vec::new();
        if self.check(&TokenKind::Dengan) {
            self.advance();
            params.push(self.parse_identifier()?);
            while self.check(&TokenKind::Dan) {
                self.advance();
                params.push(self.parse_identifier()?);
            }
        }
        
        let mut tubuh = Vec::new();
        while !self.check(&TokenKind::Selesai) && !self.is_at_end() {
            tubuh.push(self.parse_stmt()?);
        }
        
        self.expect(TokenKind::Selesai)?;
        self.expect_titik()?;
        
        Ok(Stmt::EksporFungsi { nama, params, tubuh, line })
    }

    fn parse_tambahkan_ke(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line; // tambahkan
        let nilai = self.parse_expr()?;
        self.expect(TokenKind::Ke)?;
        let daftar = self.parse_identifier()?;
        self.expect_titik()?;
        Ok(Stmt::TambahkanKe { nilai, daftar, line })
    }

    fn parse_hapus_item(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line; // hapus
        self.expect(TokenKind::Item)?;
        let posisi = if self.check(&TokenKind::Terakhir) {
            self.advance();
            PosisiItem::Terakhir
        } else if self.check(&TokenKind::Pertama) {
            self.advance();
            PosisiItem::Pertama
        } else {
            return Err(IdppError::Sintaks { line, pesan: "Diharapkan 'terakhir' atau 'pertama' setelah hapus item".into() });
        };
        self.expect(TokenKind::Dari)?;
        let daftar = self.parse_identifier()?;
        self.expect_titik()?;
        Ok(Stmt::HapusItem { daftar, posisi, line })
    }

    fn parse_ubah(&mut self) -> Result<Stmt, IdppError> {
        let line = self.advance().line; // ubah
        let nama = self.parse_identifier()?;
        
        if self.check(&TokenKind::Di) {
            self.advance();
            let index = self.parse_expr()?;
            self.expect(TokenKind::Menjadi)?;
            let nilai = self.parse_expr()?;
            self.expect_titik()?;
            Ok(Stmt::UbahDaftar { nama, index, nilai, line })
        } else {
            // asumsikan ubah kamus
            let kunci = self.parse_identifier()?;
            self.expect(TokenKind::Menjadi)?;
            let nilai = self.parse_expr()?;
            self.expect_titik()?;
            Ok(Stmt::UbahKamus { nama, kunci, nilai, line })
        }
    }

    // --- Expressions ---

    fn parse_expr(&mut self) -> Result<Expr, IdppError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, IdppError> {
        let mut expr = self.parse_and()?;
        while self.check(&TokenKind::Atau) {
            let line = self.advance().line;
            let kanan = self.parse_and()?;
            expr = Expr::Binary { kiri: Box::new(expr), op: Op::Atau, kanan: Box::new(kanan), line };
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, IdppError> {
        let mut expr = self.parse_equality()?;
        while self.check(&TokenKind::Dan) {
            let line = self.advance().line;
            let kanan = self.parse_equality()?;
            expr = Expr::Binary { kiri: Box::new(expr), op: Op::Dan, kanan: Box::new(kanan), line };
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, IdppError> {
        let mut expr = self.parse_comparison()?;
        while self.check(&TokenKind::SamaDengan) || self.check(&TokenKind::TidakSamaDengan) {
            let token = self.advance().clone();
            let op = match token.kind {
                TokenKind::SamaDengan => Op::SamaDengan,
                TokenKind::TidakSamaDengan => Op::TidakSamaDengan,
                _ => unreachable!(),
            };
            let kanan = self.parse_comparison()?;
            expr = Expr::Binary { kiri: Box::new(expr), op, kanan: Box::new(kanan), line: token.line };
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, IdppError> {
        let mut expr = self.parse_term()?;
        while self.check(&TokenKind::LebihDari) || self.check(&TokenKind::LebihDariSama) ||
              self.check(&TokenKind::KurangDari) || self.check(&TokenKind::KurangDariSama) {
            let token = self.advance().clone();
            let op = match token.kind {
                TokenKind::LebihDari => Op::LebihDari,
                TokenKind::LebihDariSama => Op::LebihDariSama,
                TokenKind::KurangDari => Op::KurangDari,
                TokenKind::KurangDariSama => Op::KurangDariSama,
                _ => unreachable!(),
            };
            let kanan = self.parse_term()?;
            expr = Expr::Binary { kiri: Box::new(expr), op, kanan: Box::new(kanan), line: token.line };
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, IdppError> {
        let mut expr = self.parse_factor()?;
        while self.check(&TokenKind::Tambah) || self.check(&TokenKind::Kurang) {
            let token = self.advance().clone();
            let op = match token.kind {
                TokenKind::Tambah => Op::Tambah,
                TokenKind::Kurang => Op::Kurang,
                _ => unreachable!(),
            };
            let kanan = self.parse_factor()?;
            expr = Expr::Binary { kiri: Box::new(expr), op, kanan: Box::new(kanan), line: token.line };
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, IdppError> {
        let mut expr = self.parse_unary()?;
        while self.check(&TokenKind::Kali) || self.check(&TokenKind::Bagi) || self.check(&TokenKind::Sisa) || self.check(&TokenKind::Pangkat) {
            let token = self.advance().clone();
            let op = match token.kind {
                TokenKind::Kali => Op::Kali,
                TokenKind::Bagi => Op::Bagi,
                TokenKind::Sisa => Op::Sisa,
                TokenKind::Pangkat => Op::Pangkat,
                _ => unreachable!(),
            };
            let kanan = self.parse_unary()?;
            expr = Expr::Binary { kiri: Box::new(expr), op, kanan: Box::new(kanan), line: token.line };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, IdppError> {
        if self.check(&TokenKind::Bukan) || self.check(&TokenKind::Kurang) {
            let token = self.advance().clone();
            let op = match token.kind {
                TokenKind::Bukan => OpUnary::Bukan,
                TokenKind::Kurang => OpUnary::Negatif,
                _ => unreachable!(),
            };
            let operand = self.parse_unary()?;
            return Ok(Expr::Unary { op, operand: Box::new(operand), line: token.line });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, IdppError> {
        let token = self.peek().clone();
        
        match &token.kind {
            TokenKind::Number(n) => { self.advance(); Ok(Expr::Number(*n)) }
            TokenKind::String(s) => { self.advance(); Ok(Expr::String(s.clone())) }
            TokenKind::Boolean(b) => { self.advance(); Ok(Expr::Boolean(*b)) }
            TokenKind::Kosong => { self.advance(); Ok(Expr::Kosong) }
            
            TokenKind::Jalankan => self.parse_jalankan_expr(),
            
            TokenKind::Panjang | TokenKind::HurufBesar | TokenKind::HurufKecil |
            TokenKind::Potong | TokenKind::Ganti | TokenKind::Mengandung |
            TokenKind::Bulatkan | TokenKind::Lantai | TokenKind::Langit |
            TokenKind::Mutlak | TokenKind::Acak | TokenKind::Maks | TokenKind::Min |
            TokenKind::Akar | TokenKind::AnkgaDari | TokenKind::TeksDari |
            TokenKind::DesimalDari | TokenKind::TipeDari => {
                self.parse_fungsi_bawaan()
            }

            // HTTP: http ambil/kirim/ubah/hapus/perbarui "url" [, opsi]
            TokenKind::HttpAmbil | TokenKind::HttpKirim | TokenKind::HttpUbah |
            TokenKind::HttpHapus | TokenKind::HttpPerbarui => {
                self.parse_http_call()
            }
            
            TokenKind::Daftar => {
                let line = self.advance().line; // daftar
                let mut items = Vec::new();
                if !self.check(&TokenKind::Titik) && !self.is_at_end() {
                    items.push(self.parse_expr()?);
                    while self.check(&TokenKind::Koma) {
                        self.advance();
                        items.push(self.parse_expr()?);
                    }
                }
                Ok(Expr::Daftar(items, line))
            }
            
            TokenKind::Kamus => {
                let line = self.advance().line; // kamus
                let mut pairs = Vec::new();
                while !self.check(&TokenKind::Selesai) && !self.is_at_end() {
                    let k = self.parse_identifier()?;
                    self.expect(TokenKind::TitikDua)?;
                    let v = self.parse_expr()?;
                    pairs.push((k, v));
                    if self.check(&TokenKind::Koma) {
                        self.advance();
                    }
                }
                self.expect(TokenKind::Selesai)?;
                Ok(Expr::Kamus(pairs, line))
            }
            
            TokenKind::Rentang => {
                let line = self.advance().line; // rentang
                let mulai = self.parse_expr()?;
                self.expect(TokenKind::Sampai)?;
                let selesai = self.parse_expr()?;
                Ok(Expr::Rentang { mulai: Box::new(mulai), selesai: Box::new(selesai), line })
            }
            
            TokenKind::Identifier(nama) => {
                self.advance(); // consume identifier
                let nama_str = nama.clone();
                let line = token.line;
                
                // Cek apakah ini akses daftar atau kamus
                if self.check(&TokenKind::Di) {
                    self.advance();
                    let index = self.parse_expr()?;
                    Ok(Expr::AksesDaftar { nama: nama_str, index: Box::new(index), line })
                } else if self.check(&TokenKind::Ambil) {
                    self.advance();
                    let kunci = self.parse_identifier()?;
                    Ok(Expr::AksesKamus { nama: nama_str, kunci, line })
                } else if self.check(&TokenKind::Punya) {
                    self.advance();
                    let kunci = self.parse_identifier()?;
                    Ok(Expr::PunyaKunci { nama: nama_str, kunci, line })
                } else {
                    Ok(Expr::Identifier(nama_str, line))
                }
            }
            
            _ => Err(IdppError::Sintaks { line: token.line, pesan: format!("Ekspresi tidak valid: {:?}", token.kind) })
        }
    }

    fn parse_jalankan_expr(&mut self) -> Result<Expr, IdppError> {
        let line = self.advance().line; // jalankan
        let nama = self.parse_identifier()?;
        
        let mut args = Vec::new();
        if self.check(&TokenKind::Dengan) {
            self.advance();
            args.push(self.parse_expr()?);
            while self.check(&TokenKind::Dan) {
                self.advance();
                args.push(self.parse_expr()?);
            }
        }
        
        Ok(Expr::JalankanFungsi { nama, args, line })
    }

    fn parse_fungsi_bawaan(&mut self) -> Result<Expr, IdppError> {
        let token = self.advance().clone();
        let nama = match token.kind {
            TokenKind::Panjang => NamaBuiltin::Panjang,
            TokenKind::HurufBesar => NamaBuiltin::HurufBesar,
            TokenKind::HurufKecil => NamaBuiltin::HurufKecil,
            TokenKind::Potong => NamaBuiltin::Potong,
            TokenKind::Ganti => NamaBuiltin::Ganti,
            TokenKind::Mengandung => NamaBuiltin::Mengandung,
            TokenKind::Bulatkan => NamaBuiltin::Bulatkan,
            TokenKind::Lantai => NamaBuiltin::Lantai,
            TokenKind::Langit => NamaBuiltin::Langit,
            TokenKind::Mutlak => NamaBuiltin::Mutlak,
            TokenKind::Acak => NamaBuiltin::Acak,
            TokenKind::Maks => NamaBuiltin::Maks,
            TokenKind::Min => NamaBuiltin::Min,
            TokenKind::Akar => NamaBuiltin::Akar,
            TokenKind::AnkgaDari => NamaBuiltin::AnkgaDari,
            TokenKind::TeksDari => NamaBuiltin::TeksDari,
            TokenKind::DesimalDari => NamaBuiltin::DesimalDari,
            TokenKind::TipeDari => NamaBuiltin::TipeDari,
            _ => unreachable!(),
        };
        
        let mut args = Vec::new();
        
        // beberapa fungsi bawaan punya sintaks khusus
        match nama {
            NamaBuiltin::Potong => {
                args.push(self.parse_expr()?);
                self.expect(TokenKind::Dari)?;
                args.push(self.parse_expr()?);
                self.expect(TokenKind::Ke)?;
                args.push(self.parse_expr()?);
            }
            NamaBuiltin::Ganti => {
                args.push(self.parse_expr()?);
                self.expect(TokenKind::Dari)?;
                args.push(self.parse_expr()?);
                self.expect(TokenKind::Ke)?;
                args.push(self.parse_expr()?);
            }
            NamaBuiltin::Mengandung => {
                args.push(self.parse_expr()?);
                self.expect(TokenKind::Cek)?;
                args.push(self.parse_expr()?);
            }
            NamaBuiltin::Maks | NamaBuiltin::Min => {
                args.push(self.parse_expr()?);
                while self.check(&TokenKind::Koma) {
                    self.advance();
                    args.push(self.parse_expr()?);
                }
            }
            NamaBuiltin::Acak => {
                // acak tidak butuh argumen
            }
            _ => {
                // rata-rata butuh 1 argumen
                args.push(self.parse_expr()?);
            }
        }
        
        Ok(Expr::FungsiBawaan { nama, args, line: token.line })
    }

    // --- Helpers ---

    fn peek(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &self.tokens.last().unwrap()
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.peek_previous()
    }

    fn peek_previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::EOF
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() { return false; }
        &self.peek().kind == kind
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&Token, IdppError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(IdppError::Sintaks {
                line: self.peek().line,
                pesan: format!("Diharapkan {:?}, tapi dapat {:?}", kind, self.peek().kind),
            })
        }
    }

    fn expect_titik(&mut self) -> Result<(), IdppError> {
        self.expect(TokenKind::Titik).map(|_| ())
    }

    fn parse_identifier(&mut self) -> Result<String, IdppError> {
        let token = self.advance().clone();
        if let TokenKind::Identifier(nama) = token.kind {
            Ok(nama)
        } else {
            Err(IdppError::Sintaks {
                line: token.line,
                pesan: format!("Diharapkan nama (identifier), tapi dapat {:?}", token.kind),
            })
        }
    }

    fn parse_string_literal(&mut self) -> Result<String, IdppError> {
        let token = self.advance().clone();
        if let TokenKind::String(s) = token.kind {
            Ok(s)
        } else {
            Err(IdppError::Sintaks {
                line: token.line,
                pesan: format!("Diharapkan teks (string), tapi dapat {:?}", token.kind),
            })
        }
    }
    fn parse_http_call(&mut self) -> Result<Expr, IdppError> {
        use crate::ast::MetodeHttp;
        let tok = self.advance();
        let line = tok.line;
        let metode = match &tok.kind {
            TokenKind::HttpAmbil  => MetodeHttp::Ambil,
            TokenKind::HttpKirim  => MetodeHttp::Kirim,
            TokenKind::HttpUbah   => MetodeHttp::Ubah,
            TokenKind::HttpHapus  => MetodeHttp::Hapus,
            TokenKind::HttpPerbarui => MetodeHttp::Perbarui,
            _ => return Err(IdppError::Sintaks { line, pesan: "Metode HTTP tidak dikenal".into() }),
        };

        // Kumpulkan argumen: url [, body] [, opsi_kamus]
        let mut args = Vec::new();
        args.push(self.parse_expr()?);
        while self.check(&TokenKind::Koma) {
            self.advance();
            args.push(self.parse_expr()?);
        }

        Ok(Expr::HttpCall { metode, args, line })
    }
}
