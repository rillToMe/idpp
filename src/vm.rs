// VM stack-based untuk eksekusi bytecode ID++

use std::collections::HashMap;
use std::io::{self, Write};
use crate::bytecode::*;
use crate::builtin;
use crate::network;
use crate::environment::Nilai;
use crate::error::IdppError;

struct CallFrame {
    vars: HashMap<u32, (Nilai, bool)>,
    return_ip: usize,
    return_code: FrameCode,
}

enum FrameCode { Main, Func(Vec<OpCode>) }

struct TryHandler {
    catch_ip: usize,
    finally_ip: usize,
}

struct LoopHandler {
    start_ip: usize,
    end_ip: usize,
}

struct IterState {
    items: Vec<Nilai>,
    index: usize,
}

pub struct VM {
    stack: Vec<Nilai>,
    frames: Vec<CallFrame>,
    current_vars: HashMap<u32, (Nilai, bool)>,
    try_stack: Vec<TryHandler>,
    loop_stack: Vec<LoopHandler>,
    iters: HashMap<u32, IterState>,
    line: usize,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::new(),
            frames: Vec::new(),
            current_vars: HashMap::new(),
            try_stack: Vec::new(),
            loop_stack: Vec::new(),
            iters: HashMap::new(),
            line: 0,
        }
    }

    pub fn run(&mut self, program: &ProgramBytecode) -> Result<(), IdppError> {
        self.exec_code(&program.instruksi, &program.teks_pool, &program.simbol, &program.fungsi_tabel)
    }

    fn exec_code(&mut self, code: &[OpCode], pool: &[String], sym: &[String], ftbl: &[FungsiInfo]) -> Result<(), IdppError> {
        let mut ip: usize = 0;
        while ip < code.len() {
            match &code[ip] {
                OpCode::Halt => return Ok(()),
                OpCode::Line(l) => { self.line = *l as usize; }

                // Konstanta
                OpCode::PushAngka(n) => self.stack.push(Nilai::Angka(*n)),
                OpCode::PushTeks(id) => self.stack.push(Nilai::Teks(pool[*id as usize].clone())),
                OpCode::PushBool(b) => self.stack.push(Nilai::Boolean(*b)),
                OpCode::PushKosong => self.stack.push(Nilai::Kosong),
                OpCode::PushKamusKey(id) => self.stack.push(Nilai::Teks(pool[*id as usize].clone())),

                // Variabel
                OpCode::LoadVar(id) => {
                    let val = self.get_var(*id, sym)?;
                    self.stack.push(val);
                }
                OpCode::StoreVar(id) => {
                    let val = self.pop()?;
                    self.set_var(*id, val, false, sym)?;
                }
                OpCode::StoreConst(id) => {
                    let val = self.pop()?;
                    self.set_var(*id, val, true, sym)?;
                }

                // Aritmatika
                OpCode::Tambah => {
                    let b = self.pop()?; let a = self.pop()?;
                    match (a, b) {
                        (Nilai::Angka(x), Nilai::Angka(y)) => self.stack.push(Nilai::Angka(x + y)),
                        (Nilai::Teks(x), b) => self.stack.push(Nilai::Teks(format!("{}{}", x, b.ke_teks()))),
                        (a, Nilai::Teks(y)) => self.stack.push(Nilai::Teks(format!("{}{}", a.ke_teks(), y))),
                        _ => return Err(self.err("Operasi tambah hanya untuk angka atau teks")),
                    }
                }
                OpCode::Kurang => { let b = self.pop_angka()?; let a = self.pop_angka()?; self.stack.push(Nilai::Angka(a - b)); }
                OpCode::Kali => { let b = self.pop_angka()?; let a = self.pop_angka()?; self.stack.push(Nilai::Angka(a * b)); }
                OpCode::Bagi => {
                    let b = self.pop_angka()?; let a = self.pop_angka()?;
                    if b == 0.0 { return Err(IdppError::BagiNol { line: self.line }); }
                    self.stack.push(Nilai::Angka(a / b));
                }
                OpCode::Sisa => {
                    let b = self.pop_angka()?; let a = self.pop_angka()?;
                    if b == 0.0 { return Err(IdppError::BagiNol { line: self.line }); }
                    self.stack.push(Nilai::Angka(a % b));
                }
                OpCode::Pangkat => { let b = self.pop_angka()?; let a = self.pop_angka()?; self.stack.push(Nilai::Angka(a.powf(b))); }

                // Unary
                OpCode::Negatif => { let a = self.pop_angka()?; self.stack.push(Nilai::Angka(-a)); }
                OpCode::Bukan => { let a = self.pop()?; self.stack.push(Nilai::Boolean(!a.is_truthy())); }

                // Perbandingan
                OpCode::CmpGt => { let b = self.pop_angka()?; let a = self.pop_angka()?; self.stack.push(Nilai::Boolean(a > b)); }
                OpCode::CmpLt => { let b = self.pop_angka()?; let a = self.pop_angka()?; self.stack.push(Nilai::Boolean(a < b)); }
                OpCode::CmpGe => { let b = self.pop_angka()?; let a = self.pop_angka()?; self.stack.push(Nilai::Boolean(a >= b)); }
                OpCode::CmpLe => { let b = self.pop_angka()?; let a = self.pop_angka()?; self.stack.push(Nilai::Boolean(a <= b)); }
                OpCode::CmpEq => { let b = self.pop()?; let a = self.pop()?; self.stack.push(Nilai::Boolean(a == b)); }
                OpCode::CmpNe => { let b = self.pop()?; let a = self.pop()?; self.stack.push(Nilai::Boolean(a != b)); }

                // Logika
                OpCode::Dan => { let b = self.pop()?; let a = self.pop()?; self.stack.push(Nilai::Boolean(a.is_truthy() && b.is_truthy())); }
                OpCode::Atau => { let b = self.pop()?; let a = self.pop()?; self.stack.push(Nilai::Boolean(a.is_truthy() || b.is_truthy())); }

                // Control Flow
                OpCode::Jump(offset) => { ip = (ip as i32 + offset) as usize; continue; }
                OpCode::JumpIfFalse(offset) => {
                    let val = self.pop()?;
                    if !val.is_truthy() { ip = (ip as i32 + offset) as usize; continue; }
                }

                // Stack
                OpCode::Pop => { self.pop()?; }

                // I/O
                OpCode::Cetak(n) => {
                    let start = self.stack.len() - *n as usize;
                    let items: Vec<String> = self.stack.drain(start..).map(|v| v.ke_teks()).collect();
                    println!("{}", items.join(""));
                }
                OpCode::Tanya(qid, vid) => {
                    print!("{} ", pool[*qid as usize]);
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    self.set_var(*vid, Nilai::Teks(input.trim_end().to_string()), false, sym)?;
                }

                // Fungsi
                OpCode::DefFunc(idx) => {
                    let fi = &ftbl[*idx as usize];
                    let func = Nilai::Fungsi {
                        params: fi.params.iter().map(|id| sym[*id as usize].clone()).collect(),
                        tubuh: Vec::new(), // AST tubuh tidak dipakai di VM
                    };
                    // simpan bytecode info juga
                    self.current_vars.insert(fi.nama_id, (func, true));
                    // simpan instruksi fungsi di iter map slot (hack: reuse storage)
                    self.iters.insert(fi.nama_id, IterState {
                        items: vec![Nilai::Angka(*idx as f64)], // store func table index
                        index: 0,
                    });
                }
                OpCode::Call(name_id, argc) => {
                    // cari fungsi di func table via stored index
                    let func_idx = if let Some(iter) = self.iters.get(name_id) {
                        iter.items[0].ke_angka(self.line)? as usize
                    } else {
                        return Err(IdppError::FungsiTidakAda { line: self.line, nama: sym[*name_id as usize].clone() });
                    };
                    let fi = &ftbl[func_idx];
                    if fi.params.len() != *argc as usize {
                        return Err(IdppError::JumlahArgumenSalah {
                            line: self.line,
                            nama: sym[fi.nama_id as usize].clone(),
                            diharapkan: fi.params.len(),
                            dapat: *argc as usize,
                        });
                    }
                    // pop args
                    let start = self.stack.len() - *argc as usize;
                    let args: Vec<Nilai> = self.stack.drain(start..).collect();

                    // push new frame
                    let old_vars = std::mem::take(&mut self.current_vars);
                    self.frames.push(CallFrame {
                        vars: old_vars,
                        return_ip: ip + 1,
                        return_code: FrameCode::Main,
                    });

                    // copy parent vars for closure-like access
                    if let Some(frame) = self.frames.last() {
                        self.current_vars = frame.vars.clone();
                    }

                    // bind params
                    for (pid, val) in fi.params.iter().zip(args) {
                        self.current_vars.insert(*pid, (val, false));
                    }

                    // also copy func table refs
                    for (k, v) in self.frames.last().unwrap().vars.iter() {
                        if !self.current_vars.contains_key(k) {
                            self.current_vars.insert(*k, v.clone());
                        }
                    }

                    // copy iter state (func defs)
                    let result = self.exec_code(&fi.instruksi, pool, sym, ftbl);

                    // restore frame
                    let frame = self.frames.pop().unwrap();
                    self.current_vars = frame.vars;

                    match result {
                        Ok(()) => self.stack.push(Nilai::Kosong),
                        Err(IdppError::LemparUser(ref msg)) if msg.starts_with("__RETURN__:") => {
                            // return value is on stack already
                        }
                        Err(e) => return Err(e),
                    }
                }
                OpCode::Return => {
                    // return value sudah di stack, signal ke caller
                    return Err(IdppError::LemparUser("__RETURN__:".into()));
                }

                // Koleksi
                OpCode::BuatDaftar(n) => {
                    let start = self.stack.len() - *n as usize;
                    let items: Vec<Nilai> = self.stack.drain(start..).collect();
                    self.stack.push(Nilai::Daftar(items));
                }
                OpCode::BuatKamus(n) => {
                    let count = *n as usize;
                    let start = self.stack.len() - count * 2;
                    let pairs: Vec<Nilai> = self.stack.drain(start..).collect();
                    let mut map = HashMap::new();
                    for i in 0..count {
                        let key = pairs[i * 2].ke_teks();
                        let val = pairs[i * 2 + 1].clone();
                        map.insert(key, val);
                    }
                    self.stack.push(Nilai::Kamus(map));
                }
                OpCode::AksesDaftar => {
                    let idx = self.pop_angka()? as usize;
                    let list = self.pop()?;
                    match list {
                        Nilai::Daftar(items) => {
                            if idx >= items.len() { return Err(IdppError::IndexDiluarBatas { line: self.line, index: idx as i64, panjang: items.len() }); }
                            self.stack.push(items[idx].clone());
                        }
                        _ => return Err(self.err("Bukan daftar")),
                    }
                }
                OpCode::AksesKamus => {
                    let key = self.pop()?.ke_teks();
                    let dict = self.pop()?;
                    match dict {
                        Nilai::Kamus(map) => {
                            if let Some(v) = map.get(&key) { self.stack.push(v.clone()); }
                            else { return Err(IdppError::KunciTidakAda { line: self.line, kunci: key }); }
                        }
                        _ => return Err(self.err("Bukan kamus")),
                    }
                }
                OpCode::PunyaKunci => {
                    let key = self.pop()?.ke_teks();
                    let dict = self.pop()?;
                    match dict {
                        Nilai::Kamus(map) => self.stack.push(Nilai::Boolean(map.contains_key(&key))),
                        _ => return Err(self.err("Bukan kamus")),
                    }
                }

                // Mutasi
                OpCode::AppendDaftar(vid) => {
                    let val = self.pop()?;
                    let line = self.line;
                    if let Some((ref mut list_val, _)) = self.current_vars.get_mut(vid) {
                        if let Nilai::Daftar(ref mut items) = list_val { items.push(val); }
                        else { return Err(IdppError::Runtime { line, pesan: "Bukan daftar".into() }); }
                    } else { return Err(IdppError::Runtime { line, pesan: "Variabel tidak ada".into() }); }
                }
                OpCode::HapusPertama(vid) => {
                    if let Some((ref mut v, _)) = self.current_vars.get_mut(vid) {
                        if let Nilai::Daftar(ref mut items) = v { if !items.is_empty() { items.remove(0); } }
                    }
                }
                OpCode::HapusTerakhir(vid) => {
                    if let Some((ref mut v, _)) = self.current_vars.get_mut(vid) {
                        if let Nilai::Daftar(ref mut items) = v { items.pop(); }
                    }
                }
                OpCode::SetDaftarIdx(vid) => {
                    let val = self.pop()?;
                    let idx = self.pop_angka()? as usize;
                    let line = self.line;
                    if let Some((ref mut v, _)) = self.current_vars.get_mut(vid) {
                        if let Nilai::Daftar(ref mut items) = v {
                            if idx >= items.len() { return Err(IdppError::IndexDiluarBatas { line, index: idx as i64, panjang: items.len() }); }
                            items[idx] = val;
                        }
                    }
                }
                OpCode::SetKamusKey(vid, kid) => {
                    let val = self.pop()?;
                    let key = pool[*kid as usize].clone();
                    let line = self.line;
                    if let Some((ref mut v, _)) = self.current_vars.get_mut(vid) {
                        if let Nilai::Kamus(ref mut map) = v {
                            if !map.contains_key(&key) { return Err(IdppError::KunciTidakAda { line, kunci: key }); }
                            map.insert(key, val);
                        }
                    }
                }
                OpCode::InsertKamusKey(vid, kid) => {
                    let val = self.pop()?;
                    let key = pool[*kid as usize].clone();
                    if let Some((ref mut v, _)) = self.current_vars.get_mut(vid) {
                        if let Nilai::Kamus(ref mut map) = v { map.insert(key, val); }
                    }
                }

                // UbahVar
                OpCode::AddVar(vid) => { let v = self.pop_angka()?; let o = self.get_angka(*vid, sym)?; self.update_var(*vid, Nilai::Angka(o + v), sym)?; }
                OpCode::SubVar(vid) => { let v = self.pop_angka()?; let o = self.get_angka(*vid, sym)?; self.update_var(*vid, Nilai::Angka(o - v), sym)?; }
                OpCode::MulVar(vid) => { let v = self.pop_angka()?; let o = self.get_angka(*vid, sym)?; self.update_var(*vid, Nilai::Angka(o * v), sym)?; }
                OpCode::DivVar(vid) => {
                    let v = self.pop_angka()?;
                    if v == 0.0 { return Err(IdppError::BagiNol { line: self.line }); }
                    let o = self.get_angka(*vid, sym)?;
                    self.update_var(*vid, Nilai::Angka(o / v), sym)?;
                }

                // Iterasi
                OpCode::SetupIter(slot) => {
                    let iterable = self.pop()?;
                    let items = match iterable {
                        Nilai::Daftar(v) => v,
                        Nilai::Teks(s) => s.chars().map(|c| Nilai::Teks(c.to_string())).collect(),
                        _ => return Err(self.err("Tidak bisa iterasi tipe ini")),
                    };
                    self.iters.insert(*slot, IterState { items, index: 0 });
                }
                OpCode::IterNext(var_id, jump_offset) => {
                    let slot_key = *var_id; // use var_id as lookup hack - find closest iter
                    // find iter by scanning iters for any with matching temp slot
                    // Actually, the compiler uses a temp slot ID stored before the loop var
                    // Let's search for the iter by checking all iters
                    let mut found = false;
                    // The setup stores iter at temp slot, we need to find it
                    // Use a simpler approach: iter slot is (var_id - 1) since temp is allocated right before
                    for (_, state) in self.iters.iter_mut() {
                        if state.index <= state.items.len() && !found {
                            if state.index < state.items.len() {
                                let val = state.items[state.index].clone();
                                self.current_vars.insert(*var_id, (val, false));
                                state.index += 1;
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found {
                        ip = (ip as i32 + jump_offset) as usize;
                        continue;
                    }
                }

                // Loop Control
                OpCode::EnterLoop(_, _) | OpCode::ExitLoop => { /* handled by compiler jumps */ }
                OpCode::Hentikan | OpCode::Lanjut => { /* compiled to Jump by compiler */ }

                // Error Handling
                OpCode::SetupTry(catch_off, finally_off) => {
                    self.try_stack.push(TryHandler {
                        catch_ip: (ip as i32 + catch_off) as usize,
                        finally_ip: (ip as i32 + finally_off) as usize,
                    });
                }
                OpCode::EndTry => { self.try_stack.pop(); }
                OpCode::SetCatchVar(vid) => {
                    // error message should be on stack from throw handling
                    if let Some(val) = self.stack.pop() {
                        self.current_vars.insert(*vid, (val, false));
                    }
                }
                OpCode::Lempar => {
                    let msg = self.pop()?.ke_teks();
                    if let Some(handler) = self.try_stack.pop() {
                        self.stack.push(Nilai::Teks(msg));
                        ip = handler.catch_ip;
                        continue;
                    }
                    return Err(IdppError::LemparUser(msg));
                }

                // Builtin
                OpCode::CallBuiltin(bid, argc) => {
                    let start = self.stack.len() - *argc as usize;
                    let args: Vec<Nilai> = self.stack.drain(start..).collect();
                    let result = match bid {
                        0  => builtin::panjang(&args, self.line),
                        1  => builtin::huruf_besar(&args, self.line),
                        2  => builtin::huruf_kecil(&args, self.line),
                        3  => builtin::potong(&args, self.line),
                        4  => builtin::ganti(&args, self.line),
                        5  => builtin::mengandung(&args, self.line),
                        6  => builtin::bulatkan(&args, self.line),
                        7  => builtin::lantai(&args, self.line),
                        8  => builtin::langit(&args, self.line),
                        9  => builtin::mutlak(&args, self.line),
                        10 => builtin::acak(&args, self.line),
                        11 => builtin::maks(&args, self.line),
                        12 => builtin::min(&args, self.line),
                        13 => builtin::akar(&args, self.line),
                        14 => builtin::angka_dari(&args, self.line),
                        15 => builtin::teks_dari(&args, self.line),
                        16 => builtin::desimal_dari(&args, self.line),
                        17 => builtin::tipe_dari(&args, self.line),
                        _  => Err(self.err("Builtin tidak dikenal")),
                    }?;
                    self.stack.push(result);
                }

                // HTTP
                OpCode::HttpReq(metode, argc) => {
                    let start = self.stack.len() - *argc as usize;
                    let args: Vec<Nilai> = self.stack.drain(start..).collect();
                    let result = match metode {
                        0 => network::http_ambil(&args, self.line),
                        1 => network::http_kirim(&args, self.line),
                        2 => network::http_ubah(&args, self.line),
                        3 => network::http_hapus(&args, self.line),
                        4 => network::http_perbarui(&args, self.line),
                        _ => Err(self.err("HTTP metode tidak dikenal")),
                    }?;
                    self.stack.push(result);
                }

                // Rentang
                OpCode::BuatRentang => {
                    let end = self.pop_angka()? as i64;
                    let start = self.pop_angka()? as i64;
                    let mut vals = Vec::new();
                    if start <= end { for i in start..=end { vals.push(Nilai::Angka(i as f64)); } }
                    else { let mut i = start; while i >= end { vals.push(Nilai::Angka(i as f64)); i -= 1; } }
                    self.stack.push(Nilai::Daftar(vals));
                }

                // Scope
                OpCode::PushScope | OpCode::PopScope => { /* handled by frame system */ }

                OpCode::Nop => {}
            }
            ip += 1;
        }
        Ok(())
    }

    // Helpers

    fn pop(&mut self) -> Result<Nilai, IdppError> {
        self.stack.pop().ok_or_else(|| self.err("Stack kosong"))
    }

    fn pop_angka(&mut self) -> Result<f64, IdppError> {
        self.pop()?.ke_angka(self.line)
    }

    fn err(&self, msg: &str) -> IdppError {
        IdppError::Runtime { line: self.line, pesan: msg.to_string() }
    }

    fn get_var(&self, id: u32, sym: &[String]) -> Result<Nilai, IdppError> {
        if let Some((val, _)) = self.current_vars.get(&id) {
            return Ok(val.clone());
        }
        for frame in self.frames.iter().rev() {
            if let Some((val, _)) = frame.vars.get(&id) {
                return Ok(val.clone());
            }
        }
        let nama = sym.get(id as usize).cloned().unwrap_or_else(|| format!("#{}", id));
        Err(IdppError::VariabelTidakAda { line: self.line, nama })
    }

    fn get_angka(&self, id: u32, sym: &[String]) -> Result<f64, IdppError> {
        self.get_var(id, sym)?.ke_angka(self.line)
    }

    fn set_var(&mut self, id: u32, val: Nilai, konst: bool, sym: &[String]) -> Result<(), IdppError> {
        if let Some((_, is_const)) = self.current_vars.get(&id) {
            if *is_const {
                let nama = sym.get(id as usize).cloned().unwrap_or_default();
                return Err(IdppError::KonstantaTidakBisaDiubah { line: self.line, nama });
            }
        }
        self.current_vars.insert(id, (val, konst));
        Ok(())
    }

    fn update_var(&mut self, id: u32, val: Nilai, sym: &[String]) -> Result<(), IdppError> {
        if let Some((ref mut v, is_const)) = self.current_vars.get_mut(&id) {
            if *is_const {
                let nama = sym.get(id as usize).cloned().unwrap_or_default();
                return Err(IdppError::KonstantaTidakBisaDiubah { line: self.line, nama });
            }
            *v = val;
            return Ok(());
        }
        let nama = sym.get(id as usize).cloned().unwrap_or_default();
        Err(IdppError::VariabelTidakAda { line: self.line, nama })
    }

    fn get_var_mut(&mut self, id: u32, sym: &[String]) -> Result<&mut Nilai, IdppError> {
        if self.current_vars.contains_key(&id) {
            return Ok(&mut self.current_vars.get_mut(&id).unwrap().0);
        }
        let nama = sym.get(id as usize).cloned().unwrap_or_default();
        Err(IdppError::VariabelTidakAda { line: self.line, nama })
    }
}
