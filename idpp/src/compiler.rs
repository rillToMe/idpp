// Compiler: AST → flat bytecode dengan jump offset dan u32 variable IDs

use std::collections::HashMap;
use crate::ast::*;
use crate::bytecode::*;

pub struct Compiler {
    pub instruksi: Vec<OpCode>,
    pub teks_pool: Vec<String>,
    pub simbol: Vec<String>,           // id → nama
    pub fungsi_tabel: Vec<FungsiInfo>,
    simbol_map: HashMap<String, u32>,   // nama → id
    teks_map: HashMap<String, u32>,     // string → pool index
    loop_stack: Vec<LoopCtx>,
    temp_counter: u32,
}

struct LoopCtx {
    start_ip: usize,
    break_patches: Vec<usize>,     // posisi Jump yang perlu di-patch
    continue_patches: Vec<usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instruksi: Vec::new(),
            teks_pool: Vec::new(),
            simbol: Vec::new(),
            fungsi_tabel: Vec::new(),
            simbol_map: HashMap::new(),
            teks_map: HashMap::new(),
            loop_stack: Vec::new(),
            temp_counter: 0,
        }
    }

    pub fn compile(mut self, stmts: &[Stmt]) -> ProgramBytecode {
        for s in stmts {
            self.compile_stmt(s);
        }
        self.emit(OpCode::Halt);
        ProgramBytecode {
            versi: VERSI_BYTECODE,
            teks_pool: self.teks_pool,
            simbol: self.simbol,
            fungsi_tabel: self.fungsi_tabel,
            instruksi: self.instruksi,
        }
    }

    // Helpers

    fn emit(&mut self, op: OpCode) -> usize {
        let pos = self.instruksi.len();
        self.instruksi.push(op);
        pos
    }

    fn ip(&self) -> usize { self.instruksi.len() }

    fn patch_jump(&mut self, pos: usize, target: usize) {
        let offset = target as i32 - pos as i32;
        match &mut self.instruksi[pos] {
            OpCode::Jump(ref mut o) | OpCode::JumpIfFalse(ref mut o) => *o = offset,
            OpCode::IterNext(_, ref mut o) => *o = offset,
            _ => {}
        }
    }

    fn var_id(&mut self, nama: &str) -> u32 {
        if let Some(&id) = self.simbol_map.get(nama) {
            return id;
        }
        let id = self.simbol.len() as u32;
        self.simbol.push(nama.to_string());
        self.simbol_map.insert(nama.to_string(), id);
        id
    }

    fn str_id(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.teks_map.get(s) {
            return id;
        }
        let id = self.teks_pool.len() as u32;
        self.teks_pool.push(s.to_string());
        self.teks_map.insert(s.to_string(), id);
        id
    }

    fn temp_var(&mut self) -> u32 {
        self.temp_counter += 1;
        self.var_id(&format!("__tmp_{}", self.temp_counter))
    }

    // Statement Compilation

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Tulis(exprs) => {
                let n = exprs.len();
                for e in exprs { self.compile_expr(e); }
                self.emit(OpCode::Cetak(n as u16));
            }
            Stmt::Tanya { pertanyaan, variabel } => {
                let q = self.str_id(pertanyaan);
                let v = self.var_id(variabel);
                self.emit(OpCode::Tanya(q, v));
            }
            Stmt::Simpan { nama, nilai, konstanta, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_expr(nilai);
                let id = self.var_id(nama);
                if *konstanta {
                    self.emit(OpCode::StoreConst(id));
                } else {
                    self.emit(OpCode::StoreVar(id));
                }
            }
            Stmt::UbahVar { nama, operasi, nilai, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_expr(nilai);
                let id = self.var_id(nama);
                match operasi {
                    OpMatematika::Tambah => self.emit(OpCode::AddVar(id)),
                    OpMatematika::Kurang => self.emit(OpCode::SubVar(id)),
                    OpMatematika::Kali   => self.emit(OpCode::MulVar(id)),
                    OpMatematika::Bagi   => self.emit(OpCode::DivVar(id)),
                };
            }
            Stmt::Jika { kondisi, tubuh, lainnya_jika, lainnya, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_jika(kondisi, tubuh, lainnya_jika, lainnya);
            }
            Stmt::Selama { kondisi, tubuh, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_selama(kondisi, tubuh);
            }
            Stmt::Ulangi { kali, tubuh, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_ulangi(kali, tubuh);
            }
            Stmt::Untuk { variabel, iterable, tubuh, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_untuk(variabel, iterable, tubuh);
            }
            Stmt::BuatFungsi { nama, params, tubuh, line } |
            Stmt::EksporFungsi { nama, params, tubuh, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_fungsi(nama, params, tubuh);
            }
            Stmt::JalankanFungsiStmt { nama, args, line } => {
                self.emit(OpCode::Line(*line as u32));
                for a in args { self.compile_expr(a); }
                let id = self.var_id(nama);
                self.emit(OpCode::Call(id, args.len() as u16));
                self.emit(OpCode::Pop); // buang return value
            }
            Stmt::Kembalikan(expr, line) => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_expr(expr);
                self.emit(OpCode::Return);
            }
            Stmt::Hentikan(line) => {
                self.emit(OpCode::Line(*line as u32));
                let pos = self.emit(OpCode::Jump(0)); // placeholder
                if let Some(ctx) = self.loop_stack.last_mut() {
                    ctx.break_patches.push(pos);
                }
            }
            Stmt::Lanjut(line) => {
                self.emit(OpCode::Line(*line as u32));
                let pos = self.emit(OpCode::Jump(0)); // placeholder
                if let Some(ctx) = self.loop_stack.last_mut() {
                    ctx.continue_patches.push(pos);
                }
            }
            Stmt::Coba { tubuh, tangkap, akhirnya, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_coba(tubuh, tangkap, akhirnya);
            }
            Stmt::Lempar(expr, line) => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_expr(expr);
                self.emit(OpCode::Lempar);
            }
            Stmt::TambahkanKe { nilai, daftar, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_expr(nilai);
                let id = self.var_id(daftar);
                self.emit(OpCode::AppendDaftar(id));
            }
            Stmt::HapusItem { daftar, posisi, line } => {
                self.emit(OpCode::Line(*line as u32));
                let id = self.var_id(daftar);
                match posisi {
                    PosisiItem::Pertama => self.emit(OpCode::HapusPertama(id)),
                    PosisiItem::Terakhir => self.emit(OpCode::HapusTerakhir(id)),
                    PosisiItem::Index(_) => self.emit(OpCode::HapusTerakhir(id)), // TODO
                };
            }
            Stmt::UbahDaftar { nama, index, nilai, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_expr(index);
                self.compile_expr(nilai);
                let id = self.var_id(nama);
                self.emit(OpCode::SetDaftarIdx(id));
            }
            Stmt::UbahKamus { nama, kunci, nilai, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_expr(nilai);
                let vid = self.var_id(nama);
                let kid = self.str_id(kunci);
                self.emit(OpCode::SetKamusKey(vid, kid));
            }
            Stmt::TambahKamus { nama, kunci, nilai, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_expr(nilai);
                let vid = self.var_id(nama);
                let kid = self.str_id(kunci);
                self.emit(OpCode::InsertKamusKey(vid, kid));
            }
            Stmt::AmbilModul(path, _line) => {
                use std::path::Path;

                // Resolusi path: tambahkan .idpp jika tidak ada ekstensi
                let file_path = Path::new(path.as_str());
                let resolved = if file_path.extension().is_some() {
                    file_path.to_path_buf()
                } else {
                    file_path.with_extension("idpp")
                };

                // Load dan parse file impor
                if let Ok(source) = std::fs::read_to_string(&resolved) {
                    if let Ok(tokens) = crate::lexer::Lexer::new(&source).tokenize() {
                        if let Ok(stmts) = crate::parser::Parser::new(tokens).parse() {
                            // Inline-compile semua statement dari file yang diimpor
                            for s in &stmts {
                                self.compile_stmt(s);
                            }
                        }
                    }
                }
                // Jika file tidak ditemukan, runtime error akan ditangani oleh interpreter
                // (compiler tidak bisa return error dari sini karena compile_stmt → ())
            }
        }
    }

    // Control Flow

    fn compile_jika(&mut self, kondisi: &Expr, tubuh: &[Stmt], lainnya_jika: &[(Expr, Vec<Stmt>)], lainnya: &Option<Vec<Stmt>>) {
        // if kondisi → body
        self.compile_expr(kondisi);
        let jmp_false = self.emit(OpCode::JumpIfFalse(0));

        for s in tubuh { self.compile_stmt(s); }
        let jmp_end = self.emit(OpCode::Jump(0)); // skip else

        self.patch_jump(jmp_false, self.ip());

        // else-if chains
        let mut end_patches = vec![jmp_end];
        for (cond, body) in lainnya_jika {
            self.compile_expr(cond);
            let jf = self.emit(OpCode::JumpIfFalse(0));
            for s in body { self.compile_stmt(s); }
            let je = self.emit(OpCode::Jump(0));
            end_patches.push(je);
            self.patch_jump(jf, self.ip());
        }

        // else
        if let Some(body) = lainnya {
            for s in body { self.compile_stmt(s); }
        }

        // patch semua jump-to-end
        let end = self.ip();
        for p in end_patches { self.patch_jump(p, end); }
    }

    fn compile_selama(&mut self, kondisi: &Expr, tubuh: &[Stmt]) {
        let loop_start = self.ip();

        self.loop_stack.push(LoopCtx {
            start_ip: loop_start,
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
        });

        // kondisi
        self.compile_expr(kondisi);
        let jmp_exit = self.emit(OpCode::JumpIfFalse(0));

        // body
        for s in tubuh { self.compile_stmt(s); }

        // jump back
        let back = self.emit(OpCode::Jump(0));
        self.patch_jump(back, loop_start);

        let loop_end = self.ip();
        self.patch_jump(jmp_exit, loop_end);

        // backpatch break/continue
        let ctx = self.loop_stack.pop().unwrap();
        for p in ctx.break_patches { self.patch_jump(p, loop_end); }
        for p in ctx.continue_patches { self.patch_jump(p, loop_start); }
    }

    fn compile_ulangi(&mut self, kali: &Expr, tubuh: &[Stmt]) {
        // simpan counter dan limit ke temp var
        let counter = self.temp_var();
        let limit = self.temp_var();

        self.compile_expr(kali);
        self.emit(OpCode::StoreVar(limit));
        self.emit(OpCode::PushAngka(0.0));
        self.emit(OpCode::StoreVar(counter));

        let loop_start = self.ip();
        self.loop_stack.push(LoopCtx {
            start_ip: loop_start,
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
        });

        // counter < limit?
        self.emit(OpCode::LoadVar(counter));
        self.emit(OpCode::LoadVar(limit));
        self.emit(OpCode::CmpLt);
        let jmp_exit = self.emit(OpCode::JumpIfFalse(0));

        for s in tubuh { self.compile_stmt(s); }

        // counter++
        let inc_ip = self.ip();
        self.emit(OpCode::PushAngka(1.0));
        self.emit(OpCode::AddVar(counter));

        let back = self.emit(OpCode::Jump(0));
        self.patch_jump(back, loop_start);

        let loop_end = self.ip();
        self.patch_jump(jmp_exit, loop_end);

        let ctx = self.loop_stack.pop().unwrap();
        for p in ctx.break_patches { self.patch_jump(p, loop_end); }
        for p in ctx.continue_patches { self.patch_jump(p, inc_ip); }
    }

    fn compile_untuk(&mut self, variabel: &str, iterable: &Expr, tubuh: &[Stmt]) {
        let iter_slot = self.temp_var();
        let loop_var = self.var_id(variabel);

        self.compile_expr(iterable);
        self.emit(OpCode::SetupIter(iter_slot));

        let loop_start = self.ip();
        self.loop_stack.push(LoopCtx {
            start_ip: loop_start,
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
        });

        let iter_next = self.emit(OpCode::IterNext(loop_var, 0));

        for s in tubuh { self.compile_stmt(s); }

        let back = self.emit(OpCode::Jump(0));
        self.patch_jump(back, loop_start);

        let loop_end = self.ip();
        self.patch_jump(iter_next, loop_end);

        let ctx = self.loop_stack.pop().unwrap();
        for p in ctx.break_patches { self.patch_jump(p, loop_end); }
        for p in ctx.continue_patches { self.patch_jump(p, loop_start); }
    }

    fn compile_fungsi(&mut self, nama: &str, params: &[String], tubuh: &[Stmt]) {
        let nama_id = self.var_id(nama);
        let param_ids: Vec<u32> = params.iter().map(|p| self.var_id(p)).collect();

        // Compile function body in separate compiler context
        let mut func_compiler = Compiler::new();
        // share symbol table
        func_compiler.simbol = self.simbol.clone();
        func_compiler.simbol_map = self.simbol_map.clone();
        func_compiler.teks_pool = self.teks_pool.clone();
        func_compiler.teks_map = self.teks_map.clone();

        for s in tubuh { func_compiler.compile_stmt(s); }
        func_compiler.emit(OpCode::PushKosong);
        func_compiler.emit(OpCode::Return);

        // merge symbol/string tables back
        self.simbol = func_compiler.simbol;
        self.simbol_map = func_compiler.simbol_map;
        self.teks_pool = func_compiler.teks_pool;
        self.teks_map = func_compiler.teks_map;

        let fi = FungsiInfo {
            nama_id,
            params: param_ids,
            instruksi: func_compiler.instruksi,
        };
        let idx = self.fungsi_tabel.len() as u32;
        self.fungsi_tabel.push(fi);
        self.emit(OpCode::DefFunc(idx));
    }

    fn compile_coba(&mut self, tubuh: &[Stmt], tangkap: &Option<(String, Vec<Stmt>)>, akhirnya: &Option<Vec<Stmt>>) {
        let setup = self.emit(OpCode::SetupTry(0, 0));

        // try body
        for s in tubuh { self.compile_stmt(s); }
        self.emit(OpCode::EndTry);
        let jmp_finally = self.emit(OpCode::Jump(0));

        // catch
        let catch_ip = self.ip();
        if let Some((var_name, catch_body)) = tangkap {
            let vid = self.var_id(var_name);
            self.emit(OpCode::SetCatchVar(vid));
            for s in catch_body { self.compile_stmt(s); }
        }
        let jmp_finally2 = self.emit(OpCode::Jump(0));

        // finally
        let finally_ip = self.ip();
        if let Some(finally_body) = akhirnya {
            for s in finally_body { self.compile_stmt(s); }
        }
        let end_ip = self.ip();

        // patch SetupTry
        let catch_off = catch_ip as i32 - setup as i32;
        let finally_off = finally_ip as i32 - setup as i32;
        self.instruksi[setup] = OpCode::SetupTry(catch_off, finally_off);

        self.patch_jump(jmp_finally, finally_ip);
        self.patch_jump(jmp_finally2, finally_ip);
        let _ = end_ip;
    }

    // Expression Compilation

    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n) => { self.emit(OpCode::PushAngka(*n)); }
            Expr::String(s) => {
                let id = self.str_id(s);
                self.emit(OpCode::PushTeks(id));
            }
            Expr::Boolean(b) => { self.emit(OpCode::PushBool(*b)); }
            Expr::Kosong => { self.emit(OpCode::PushKosong); }
            Expr::Identifier(nama, line) => {
                self.emit(OpCode::Line(*line as u32));
                let id = self.var_id(nama);
                self.emit(OpCode::LoadVar(id));
            }
            Expr::Binary { kiri, op, kanan, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_expr(kiri);
                self.compile_expr(kanan);
                match op {
                    Op::Tambah => self.emit(OpCode::Tambah),
                    Op::Kurang => self.emit(OpCode::Kurang),
                    Op::Kali => self.emit(OpCode::Kali),
                    Op::Bagi => self.emit(OpCode::Bagi),
                    Op::Sisa => self.emit(OpCode::Sisa),
                    Op::Pangkat => self.emit(OpCode::Pangkat),
                    Op::LebihDari => self.emit(OpCode::CmpGt),
                    Op::KurangDari => self.emit(OpCode::CmpLt),
                    Op::LebihDariSama => self.emit(OpCode::CmpGe),
                    Op::KurangDariSama => self.emit(OpCode::CmpLe),
                    Op::SamaDengan => self.emit(OpCode::CmpEq),
                    Op::TidakSamaDengan => self.emit(OpCode::CmpNe),
                    Op::Dan => self.emit(OpCode::Dan),
                    Op::Atau => self.emit(OpCode::Atau),
                };
            }
            Expr::Unary { op, operand, line } => {
                self.emit(OpCode::Line(*line as u32));
                self.compile_expr(operand);
                match op {
                    OpUnary::Negatif => self.emit(OpCode::Negatif),
                    OpUnary::Bukan => self.emit(OpCode::Bukan),
                };
            }
            Expr::JalankanFungsi { nama, args, line } => {
                self.emit(OpCode::Line(*line as u32));
                for a in args { self.compile_expr(a); }
                let id = self.var_id(nama);
                self.emit(OpCode::Call(id, args.len() as u16));
            }
            Expr::AksesDaftar { nama, index, line } => {
                self.emit(OpCode::Line(*line as u32));
                let id = self.var_id(nama);
                self.emit(OpCode::LoadVar(id));
                self.compile_expr(index);
                self.emit(OpCode::AksesDaftar);
            }
            Expr::AksesKamus { nama, kunci, line } => {
                self.emit(OpCode::Line(*line as u32));
                let id = self.var_id(nama);
                self.emit(OpCode::LoadVar(id));
                let kid = self.str_id(kunci);
                self.emit(OpCode::PushTeks(kid));
                self.emit(OpCode::AksesKamus);
            }
            Expr::PunyaKunci { nama, kunci, line } => {
                self.emit(OpCode::Line(*line as u32));
                let id = self.var_id(nama);
                self.emit(OpCode::LoadVar(id));
                let kid = self.str_id(kunci);
                self.emit(OpCode::PushTeks(kid));
                self.emit(OpCode::PunyaKunci);
            }
            Expr::FungsiBawaan { nama, args, line } => {
                self.emit(OpCode::Line(*line as u32));
                for a in args { self.compile_expr(a); }
                let bid = match nama {
                    NamaBuiltin::Panjang     => 0,
                    NamaBuiltin::HurufBesar  => 1,
                    NamaBuiltin::HurufKecil  => 2,
                    NamaBuiltin::Potong      => 3,
                    NamaBuiltin::Ganti       => 4,
                    NamaBuiltin::Mengandung  => 5,
                    NamaBuiltin::Bulatkan    => 6,
                    NamaBuiltin::Lantai      => 7,
                    NamaBuiltin::Langit      => 8,
                    NamaBuiltin::Mutlak      => 9,
                    NamaBuiltin::Acak        => 10,
                    NamaBuiltin::Maks        => 11,
                    NamaBuiltin::Min         => 12,
                    NamaBuiltin::Akar        => 13,
                    NamaBuiltin::AnkgaDari   => 14,
                    NamaBuiltin::TeksDari    => 15,
                    NamaBuiltin::DesimalDari => 16,
                    NamaBuiltin::TipeDari    => 17,
                };
                self.emit(OpCode::CallBuiltin(bid, args.len() as u16));
            }
            Expr::Daftar(items, _) => {
                let n = items.len();
                for item in items { self.compile_expr(item); }
                self.emit(OpCode::BuatDaftar(n as u16));
            }
            Expr::Kamus(pairs, _) => {
                let n = pairs.len();
                for (key, val) in pairs {
                    let kid = self.str_id(key);
                    self.emit(OpCode::PushKamusKey(kid));
                    self.compile_expr(val);
                }
                self.emit(OpCode::BuatKamus(n as u16));
            }
            Expr::Rentang { mulai, selesai, .. } => {
                self.compile_expr(mulai);
                self.compile_expr(selesai);
                self.emit(OpCode::BuatRentang);
            }
            Expr::HttpCall { metode, args, line } => {
                self.emit(OpCode::Line(*line as u32));
                for a in args { self.compile_expr(a); }
                let mid = match metode {
                    MetodeHttp::Ambil    => 0,
                    MetodeHttp::Kirim    => 1,
                    MetodeHttp::Ubah     => 2,
                    MetodeHttp::Hapus    => 3,
                    MetodeHttp::Perbarui => 4,
                };
                self.emit(OpCode::HttpReq(mid, args.len() as u16));
            }
        }
    }
}
