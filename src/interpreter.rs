// Menjalankan AST ID++

use std::collections::HashMap;
use std::io::{self, Write};

use crate::ast::*;
use crate::builtin;
use crate::network;
use crate::environment::{Environment, Nilai};
use crate::error::{ControlFlow, IdppError};

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: Environment::new(),
        }
    }

    pub fn run(&mut self, stmts: Vec<Stmt>) -> Result<(), IdppError> {
        for stmt in stmts {
            match self.exec_stmt(&stmt)? {
                ControlFlow::Normal => {}
                ControlFlow::Return(_) => return Err(IdppError::Runtime { line: 0, pesan: "Kata 'kembalikan' hanya boleh di dalam fungsi".into() }),
                ControlFlow::Break => return Err(IdppError::Runtime { line: 0, pesan: "Kata 'hentikan' hanya boleh di dalam loop".into() }),
                ControlFlow::Continue => return Err(IdppError::Runtime { line: 0, pesan: "Kata 'lanjut' hanya boleh di dalam loop".into() }),
                ControlFlow::Throw(msg, _line) => return Err(IdppError::LemparUser(msg)),
            }
        }
        Ok(())
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<ControlFlow, IdppError> {
        match stmt {
            Stmt::Tulis(exprs) => self.exec_tulis(exprs),
            Stmt::Tanya { pertanyaan, variabel } => self.exec_tanya(pertanyaan, variabel),
            Stmt::Simpan { nama, nilai, konstanta, line } => self.exec_simpan(nama, nilai, *konstanta, *line),
            Stmt::UbahVar { nama, operasi, nilai, line } => self.exec_ubah_var(nama, operasi, nilai, *line),
            Stmt::Jika { kondisi, tubuh, lainnya_jika, lainnya, line } => self.exec_jika(kondisi, tubuh, lainnya_jika, lainnya, *line),
            Stmt::Selama { kondisi, tubuh, line } => self.exec_selama(kondisi, tubuh, *line),
            Stmt::Ulangi { kali, tubuh, line } => self.exec_ulangi(kali, tubuh, *line),
            Stmt::Untuk { variabel, iterable, tubuh, line } => self.exec_untuk(variabel, iterable, tubuh, *line),
            Stmt::BuatFungsi { nama, params, tubuh, line } => self.exec_buat_fungsi(nama, params, tubuh, *line),
            Stmt::JalankanFungsiStmt { nama, args, line } => { self.exec_jalankan(nama, args, *line)?; Ok(ControlFlow::Normal) }
            Stmt::Kembalikan(expr, _line) => Ok(ControlFlow::Return(self.eval_expr(expr)?)),
            Stmt::Hentikan(_) => Ok(ControlFlow::Break),
            Stmt::Lanjut(_) => Ok(ControlFlow::Continue),
            Stmt::Coba { tubuh, tangkap, akhirnya, line } => self.exec_coba(tubuh, tangkap, akhirnya, *line),
            Stmt::Lempar(expr, line) => {
                let msg = self.eval_expr(expr)?.ke_teks();
                Ok(ControlFlow::Throw(msg, *line))
            }
            Stmt::TambahkanKe { nilai, daftar, line } => self.exec_tambahkan_ke(nilai, daftar, *line),
            Stmt::HapusItem { daftar, posisi, line } => self.exec_hapus_item(daftar, posisi, *line),
            Stmt::UbahDaftar { nama, index, nilai, line } => self.exec_ubah_daftar(nama, index, nilai, *line),
            Stmt::UbahKamus { nama, kunci, nilai, line } => self.exec_ubah_kamus(nama, kunci, nilai, *line),
            Stmt::TambahKamus { nama, kunci, nilai, line } => self.exec_tambah_kamus(nama, kunci, nilai, *line),
            Stmt::EksporFungsi { nama, params, tubuh, line } => self.exec_buat_fungsi(nama, params, tubuh, *line), // Sementara sama dengan BuatFungsi
            Stmt::AmbilModul(path, line) => self.exec_impor(path, *line),
        }
    }

    fn exec_impor(&mut self, path: &str, line: usize) -> Result<ControlFlow, IdppError> {
        use std::path::Path;

        // Cari file relatif terhadap current working directory
        let file_path = Path::new(path);

        // Coba cari dengan ekstensi .idpp jika tidak ada ekstensi
        let resolved = if file_path.extension().is_some() {
            file_path.to_path_buf()
        } else {
            file_path.with_extension("idpp")
        };

        let source = std::fs::read_to_string(&resolved).map_err(|_| IdppError::Runtime {
            line,
            pesan: format!("Tidak bisa membuka file impor: '{}'", resolved.display()),
        })?;

        // Tokenisasi
        let mut lexer = crate::lexer::Lexer::new(&source);
        let tokens = lexer.tokenize().map_err(|e| IdppError::Runtime {
            line,
            pesan: format!("Error saat tokenisasi '{}': {}", resolved.display(), e),
        })?;

        // Parse menjadi AST
        let mut parser = crate::parser::Parser::new(tokens);
        let stmts = parser.parse().map_err(|e| IdppError::Runtime {
            line,
            pesan: format!("Error saat parsing '{}': {}", resolved.display(), e),
        })?;

        // Eksekusi langsung dalam environment saat ini
        // Sehingga fungsi dan variabel dari modul tersedia di program pemanggil
        for stmt in &stmts {
            match self.exec_stmt(stmt)? {
                ControlFlow::Normal => {}
                ControlFlow::Return(_) => break,
                other => return Ok(other),
            }
        }

        Ok(ControlFlow::Normal)
    }

    fn exec_tulis(&mut self, exprs: &[Expr]) -> Result<ControlFlow, IdppError> {
        let mut out = String::new();
        for expr in exprs {
            out.push_str(&self.eval_expr(expr)?.ke_teks());
        }
        println!("{}", out);
        Ok(ControlFlow::Normal)
    }

    fn exec_tanya(&mut self, pertanyaan: &str, var: &str) -> Result<ControlFlow, IdppError> {
        print!("{} ", pertanyaan);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim_end().to_string();
        self.env.set(var, Nilai::Teks(input), false)?;
        Ok(ControlFlow::Normal)
    }

    fn exec_simpan(&mut self, nama: &str, expr: &Expr, konstan: bool, line: usize) -> Result<ControlFlow, IdppError> {
        let nilai = self.eval_expr(expr)?;
        self.env.set_dengan_baris(nama, nilai, konstan, line)?;
        Ok(ControlFlow::Normal)
    }

    fn exec_ubah_var(&mut self, nama: &str, operasi: &OpMatematika, expr: &Expr, line: usize) -> Result<ControlFlow, IdppError> {
        let nilai_lama = self.env.get_atau_error(nama, line)?.ke_angka(line)?;
        let nilai_baru = self.eval_expr(expr)?.ke_angka(line)?;
        let hasil = match operasi {
            OpMatematika::Tambah => nilai_lama + nilai_baru,
            OpMatematika::Kurang => nilai_lama - nilai_baru,
            OpMatematika::Kali => nilai_lama * nilai_baru,
            OpMatematika::Bagi => {
                if nilai_baru == 0.0 { return Err(IdppError::BagiNol { line }); }
                nilai_lama / nilai_baru
            }
        };
        self.env.update(nama, Nilai::Angka(hasil), line)?;
        Ok(ControlFlow::Normal)
    }

    fn exec_jika(&mut self, kondisi: &Expr, tubuh: &[Stmt], lainnya_jika: &[(Expr, Vec<Stmt>)], lainnya: &Option<Vec<Stmt>>, _line: usize) -> Result<ControlFlow, IdppError> {
        if self.eval_expr(kondisi)?.is_truthy() {
            return self.exec_block(tubuh);
        }
        for (cond, body) in lainnya_jika {
            if self.eval_expr(cond)?.is_truthy() {
                return self.exec_block(body);
            }
        }
        if let Some(body) = lainnya {
            return self.exec_block(body);
        }
        Ok(ControlFlow::Normal)
    }

    fn exec_selama(&mut self, kondisi: &Expr, tubuh: &[Stmt], _line: usize) -> Result<ControlFlow, IdppError> {
        while self.eval_expr(kondisi)?.is_truthy() {
            match self.exec_block(tubuh)? {
                ControlFlow::Normal | ControlFlow::Continue => {}
                ControlFlow::Break => break,
                other => return Ok(other),
            }
        }
        Ok(ControlFlow::Normal)
    }

    fn exec_ulangi(&mut self, kali: &Expr, tubuh: &[Stmt], line: usize) -> Result<ControlFlow, IdppError> {
        let n = self.eval_expr(kali)?.ke_angka(line)? as i64;
        for _ in 0..n {
            match self.exec_block(tubuh)? {
                ControlFlow::Normal | ControlFlow::Continue => {}
                ControlFlow::Break => break,
                other => return Ok(other),
            }
        }
        Ok(ControlFlow::Normal)
    }

    fn exec_untuk(&mut self, var: &str, iterable: &Expr, tubuh: &[Stmt], line: usize) -> Result<ControlFlow, IdppError> {
        let iter_val = self.eval_expr(iterable)?;
        let items = match iter_val {
            Nilai::Daftar(list) => list,
            Nilai::Teks(s) => s.chars().map(|c| Nilai::Teks(c.to_string())).collect(),
            other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "daftar atau teks".into(), dapat: other.tipe_string().into() }),
        };

        for item in items {
            self.env.set(var, item, false)?;
            match self.exec_block(tubuh)? {
                ControlFlow::Normal | ControlFlow::Continue => {}
                ControlFlow::Break => break,
                other => return Ok(other),
            }
        }
        Ok(ControlFlow::Normal)
    }

    fn exec_buat_fungsi(&mut self, nama: &str, params: &[String], tubuh: &[Stmt], line: usize) -> Result<ControlFlow, IdppError> {
        let fungsi = Nilai::Fungsi { params: params.to_vec(), tubuh: tubuh.to_vec() };
        self.env.set_dengan_baris(nama, fungsi, true, line)?;
        Ok(ControlFlow::Normal)
    }

    fn exec_jalankan(&mut self, nama: &str, args: &[Expr], line: usize) -> Result<Nilai, IdppError> {
        let func = self.env.get_atau_error(nama, line)?;
        let mut eval_args = Vec::new();
        for arg in args {
            eval_args.push(self.eval_expr(arg)?);
        }

        if let Nilai::Fungsi { params, tubuh } = func {
            if params.len() != eval_args.len() {
                return Err(IdppError::JumlahArgumenSalah { line, nama: nama.into(), diharapkan: params.len(), dapat: eval_args.len() });
            }

            let mut env_baru = Environment::new_child(self.env.clone());
            for (param, arg) in params.into_iter().zip(eval_args.into_iter()) {
                env_baru.set(&param, arg, false)?;
            }

            let env_lama = std::mem::replace(&mut self.env, env_baru);
            let hasil = self.exec_block(&tubuh);
            self.env = env_lama;

            match hasil? {
                ControlFlow::Return(nilai) => Ok(nilai),
                ControlFlow::Normal => Ok(Nilai::Kosong),
                ControlFlow::Break | ControlFlow::Continue => Err(IdppError::Runtime { line, pesan: "Break/Continue di luar loop".into() }),
                ControlFlow::Throw(msg, _l) => Err(IdppError::LemparUser(msg)),
            }
        } else {
            Err(IdppError::FungsiTidakAda { line, nama: nama.into() })
        }
    }

    fn exec_coba(&mut self, tubuh: &[Stmt], tangkap: &Option<(String, Vec<Stmt>)>, akhirnya: &Option<Vec<Stmt>>, _line: usize) -> Result<ControlFlow, IdppError> {
        let hasil = self.exec_block(tubuh);
        
        let mut flow = ControlFlow::Normal;
        
        match hasil {
            Ok(f) => { flow = f; }
            Err(e) => {
                if let Some((var_galat, blok_tangkap)) = tangkap {
                    let mut env_baru = Environment::new_child(self.env.clone());
                    let msg = match e {
                        IdppError::LemparUser(m) => m,
                        other => other.to_string(),
                    };
                    env_baru.set(var_galat, Nilai::Teks(msg), false)?;
                    let env_lama = std::mem::replace(&mut self.env, env_baru);
                    flow = self.exec_block(blok_tangkap)?;
                    self.env = env_lama;
                } else {
                    return Err(e);
                }
            }
        }

        if let Some(blok_akhirnya) = akhirnya {
            let flow_akhirnya = self.exec_block(blok_akhirnya)?;
            if let ControlFlow::Normal = flow_akhirnya {
                // let it pass
            } else {
                flow = flow_akhirnya; // override
            }
        }

        Ok(flow)
    }

    fn exec_tambahkan_ke(&mut self, nilai: &Expr, daftar: &str, line: usize) -> Result<ControlFlow, IdppError> {
        let val = self.eval_expr(nilai)?;
        let list_val = self.env.get_atau_error(daftar, line)?;
        if let Nilai::Daftar(mut items) = list_val {
            items.push(val);
            self.env.update(daftar, Nilai::Daftar(items), line)?;
            Ok(ControlFlow::Normal)
        } else {
            Err(IdppError::TipeTidakCocok { line, diharapkan: "daftar".into(), dapat: list_val.tipe_string().into() })
        }
    }

    fn exec_hapus_item(&mut self, daftar: &str, posisi: &PosisiItem, line: usize) -> Result<ControlFlow, IdppError> {
        let list_val = self.env.get_atau_error(daftar, line)?;
        if let Nilai::Daftar(mut items) = list_val {
            if items.is_empty() {
                return Err(IdppError::Runtime { line, pesan: format!("Daftar '{}' kosong", daftar) });
            }
            match posisi {
                PosisiItem::Terakhir => { items.pop(); }
                PosisiItem::Pertama => { items.remove(0); }
                PosisiItem::Index(_) => { /* TODO */ }
            }
            self.env.update(daftar, Nilai::Daftar(items), line)?;
            Ok(ControlFlow::Normal)
        } else {
            Err(IdppError::TipeTidakCocok { line, diharapkan: "daftar".into(), dapat: list_val.tipe_string().into() })
        }
    }

    fn exec_ubah_daftar(&mut self, nama: &str, index: &Expr, nilai: &Expr, line: usize) -> Result<ControlFlow, IdppError> {
        let idx = self.eval_expr(index)?.ke_angka(line)? as usize;
        let val = self.eval_expr(nilai)?;
        let list_val = self.env.get_atau_error(nama, line)?;
        if let Nilai::Daftar(mut items) = list_val {
            if idx >= items.len() {
                return Err(IdppError::IndexDiluarBatas { line, index: idx as i64, panjang: items.len() });
            }
            items[idx] = val;
            self.env.update(nama, Nilai::Daftar(items), line)?;
            Ok(ControlFlow::Normal)
        } else {
            Err(IdppError::TipeTidakCocok { line, diharapkan: "daftar".into(), dapat: list_val.tipe_string().into() })
        }
    }

    fn exec_ubah_kamus(&mut self, nama: &str, kunci: &str, nilai: &Expr, line: usize) -> Result<ControlFlow, IdppError> {
        let val = self.eval_expr(nilai)?;
        let dict_val = self.env.get_atau_error(nama, line)?;
        if let Nilai::Kamus(mut map) = dict_val {
            if !map.contains_key(kunci) {
                return Err(IdppError::KunciTidakAda { line, kunci: kunci.into() });
            }
            map.insert(kunci.into(), val);
            self.env.update(nama, Nilai::Kamus(map), line)?;
            Ok(ControlFlow::Normal)
        } else {
            Err(IdppError::TipeTidakCocok { line, diharapkan: "kamus".into(), dapat: dict_val.tipe_string().into() })
        }
    }

    fn exec_tambah_kamus(&mut self, nama: &str, kunci: &str, nilai: &Expr, line: usize) -> Result<ControlFlow, IdppError> {
        let val = self.eval_expr(nilai)?;
        let dict_val = self.env.get_atau_error(nama, line)?;
        if let Nilai::Kamus(mut map) = dict_val {
            map.insert(kunci.into(), val);
            self.env.update(nama, Nilai::Kamus(map), line)?;
            Ok(ControlFlow::Normal)
        } else {
            Err(IdppError::TipeTidakCocok { line, diharapkan: "kamus".into(), dapat: dict_val.tipe_string().into() })
        }
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> Result<ControlFlow, IdppError> {
        for stmt in stmts {
            let flow = self.exec_stmt(stmt)?;
            if let ControlFlow::Normal = flow {
                continue;
            } else {
                return Ok(flow);
            }
        }
        Ok(ControlFlow::Normal)
    }

    // --- Expressions Evaluator ---

    fn eval_expr(&mut self, expr: &Expr) -> Result<Nilai, IdppError> {
        match expr {
            Expr::String(s) => Ok(Nilai::Teks(s.clone())),
            Expr::Number(n) => Ok(Nilai::Angka(*n)),
            Expr::Boolean(b) => Ok(Nilai::Boolean(*b)),
            Expr::Kosong => Ok(Nilai::Kosong),
            Expr::Identifier(nama, line) => self.env.get_atau_error(nama, *line),
            Expr::Binary { kiri, op, kanan, line } => self.eval_binary(kiri, op, kanan, *line),
            Expr::Unary { op, operand, line } => self.eval_unary(op, operand, *line),
            Expr::JalankanFungsi { nama, args, line } => self.exec_jalankan(nama, args, *line),
            Expr::AksesDaftar { nama, index, line } => {
                let list_val = self.env.get_atau_error(nama, *line)?;
                let idx = self.eval_expr(index)?.ke_angka(*line)? as usize;
                if let Nilai::Daftar(items) = list_val {
                    if idx >= items.len() {
                        return Err(IdppError::IndexDiluarBatas { line: *line, index: idx as i64, panjang: items.len() });
                    }
                    Ok(items[idx].clone())
                } else {
                    Err(IdppError::TipeTidakCocok { line: *line, diharapkan: "daftar".into(), dapat: list_val.tipe_string().into() })
                }
            }
            Expr::AksesKamus { nama, kunci, line } => {
                let dict_val = self.env.get_atau_error(nama, *line)?;
                if let Nilai::Kamus(map) = dict_val {
                    if let Some(val) = map.get(kunci) {
                        Ok(val.clone())
                    } else {
                        Err(IdppError::KunciTidakAda { line: *line, kunci: kunci.clone() })
                    }
                } else {
                    Err(IdppError::TipeTidakCocok { line: *line, diharapkan: "kamus".into(), dapat: dict_val.tipe_string().into() })
                }
            }
            Expr::PunyaKunci { nama, kunci, line } => {
                let dict_val = self.env.get_atau_error(nama, *line)?;
                if let Nilai::Kamus(map) = dict_val {
                    Ok(Nilai::Boolean(map.contains_key(kunci)))
                } else {
                    Err(IdppError::TipeTidakCocok { line: *line, diharapkan: "kamus".into(), dapat: dict_val.tipe_string().into() })
                }
            }
            Expr::Daftar(items, _) => {
                let mut vals = Vec::new();
                for item in items {
                    vals.push(self.eval_expr(item)?);
                }
                Ok(Nilai::Daftar(vals))
            }
            Expr::Kamus(pairs, _) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    map.insert(k.clone(), self.eval_expr(v)?);
                }
                Ok(Nilai::Kamus(map))
            }
            Expr::Rentang { mulai, selesai, line } => {
                let m = self.eval_expr(mulai)?.ke_angka(*line)? as i64;
                let s = self.eval_expr(selesai)?.ke_angka(*line)? as i64;
                let mut vals = Vec::new();
                if m <= s {
                    for i in m..=s { vals.push(Nilai::Angka(i as f64)); }
                } else {
                    let mut i = m;
                    while i >= s { vals.push(Nilai::Angka(i as f64)); i -= 1; }
                }
                Ok(Nilai::Daftar(vals))
            }
            Expr::FungsiBawaan { nama, args, line } => {
                let mut eval_args = Vec::new();
                for a in args { eval_args.push(self.eval_expr(a)?); }
                match nama {
                    NamaBuiltin::Panjang => builtin::panjang(&eval_args, *line),
                    NamaBuiltin::HurufBesar => builtin::huruf_besar(&eval_args, *line),
                    NamaBuiltin::HurufKecil => builtin::huruf_kecil(&eval_args, *line),
                    NamaBuiltin::Potong => builtin::potong(&eval_args, *line),
                    NamaBuiltin::Ganti => builtin::ganti(&eval_args, *line),
                    NamaBuiltin::Mengandung => builtin::mengandung(&eval_args, *line),
                    NamaBuiltin::Bulatkan => builtin::bulatkan(&eval_args, *line),
                    NamaBuiltin::Lantai => builtin::lantai(&eval_args, *line),
                    NamaBuiltin::Langit => builtin::langit(&eval_args, *line),
                    NamaBuiltin::Mutlak => builtin::mutlak(&eval_args, *line),
                    NamaBuiltin::Acak => builtin::acak(&eval_args, *line),
                    NamaBuiltin::Maks => builtin::maks(&eval_args, *line),
                    NamaBuiltin::Min => builtin::min(&eval_args, *line),
                    NamaBuiltin::Akar => builtin::akar(&eval_args, *line),
                    NamaBuiltin::AnkgaDari => builtin::angka_dari(&eval_args, *line),
                    NamaBuiltin::TeksDari => builtin::teks_dari(&eval_args, *line),
                    NamaBuiltin::DesimalDari => builtin::desimal_dari(&eval_args, *line),
                    NamaBuiltin::TipeDari => builtin::tipe_dari(&eval_args, *line),
                }
            }
            Expr::HttpCall { metode, args, line } => {
                let mut eval_args = Vec::new();
                for a in args { eval_args.push(self.eval_expr(a)?); }
                match metode {
                    MetodeHttp::Ambil  => network::http_ambil(&eval_args, *line),
                    MetodeHttp::Kirim  => network::http_kirim(&eval_args, *line),
                    MetodeHttp::Ubah   => network::http_ubah(&eval_args, *line),
                    MetodeHttp::Hapus  => network::http_hapus(&eval_args, *line),
                    MetodeHttp::Perbarui => network::http_perbarui(&eval_args, *line),
                }
            }
        }
    }

    fn eval_binary(&mut self, kiri: &Expr, op: &Op, kanan: &Expr, line: usize) -> Result<Nilai, IdppError> {
        let left_val = self.eval_expr(kiri)?;
        let right_val = self.eval_expr(kanan)?;

        match op {
            Op::Tambah => {
                match (left_val, right_val) {
                    (Nilai::Angka(a), Nilai::Angka(b)) => Ok(Nilai::Angka(a + b)),
                    (Nilai::Teks(a), b) => Ok(Nilai::Teks(format!("{}{}", a, b.ke_teks()))),
                    (a, Nilai::Teks(b)) => Ok(Nilai::Teks(format!("{}{}", a.ke_teks(), b))),
                    _ => Err(IdppError::Runtime { line, pesan: "Operasi tambah hanya untuk angka atau teks".into() }),
                }
            }
            Op::Kurang => Ok(Nilai::Angka(left_val.ke_angka(line)? - right_val.ke_angka(line)?)),
            Op::Kali => Ok(Nilai::Angka(left_val.ke_angka(line)? * right_val.ke_angka(line)?)),
            Op::Bagi => {
                let r = right_val.ke_angka(line)?;
                if r == 0.0 { Err(IdppError::BagiNol { line }) } else { Ok(Nilai::Angka(left_val.ke_angka(line)? / r)) }
            }
            Op::Sisa => {
                let r = right_val.ke_angka(line)?;
                if r == 0.0 { Err(IdppError::BagiNol { line }) } else { Ok(Nilai::Angka(left_val.ke_angka(line)? % r)) }
            }
            Op::Pangkat => Ok(Nilai::Angka(left_val.ke_angka(line)?.powf(right_val.ke_angka(line)?))),
            Op::LebihDari => Ok(Nilai::Boolean(left_val.ke_angka(line)? > right_val.ke_angka(line)?)),
            Op::KurangDari => Ok(Nilai::Boolean(left_val.ke_angka(line)? < right_val.ke_angka(line)?)),
            Op::LebihDariSama => Ok(Nilai::Boolean(left_val.ke_angka(line)? >= right_val.ke_angka(line)?)),
            Op::KurangDariSama => Ok(Nilai::Boolean(left_val.ke_angka(line)? <= right_val.ke_angka(line)?)),
            Op::SamaDengan => Ok(Nilai::Boolean(left_val == right_val)),
            Op::TidakSamaDengan => Ok(Nilai::Boolean(left_val != right_val)),
            Op::Dan => Ok(Nilai::Boolean(left_val.is_truthy() && right_val.is_truthy())),
            Op::Atau => Ok(Nilai::Boolean(left_val.is_truthy() || right_val.is_truthy())),
        }
    }

    fn eval_unary(&mut self, op: &OpUnary, operand: &Expr, line: usize) -> Result<Nilai, IdppError> {
        let val = self.eval_expr(operand)?;
        match op {
            OpUnary::Bukan => Ok(Nilai::Boolean(!val.is_truthy())),
            OpUnary::Negatif => Ok(Nilai::Angka(-val.ke_angka(line)?)),
        }
    }
}
