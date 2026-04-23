// Manajemen scope dan nilai variabel untuk ID++

use std::collections::HashMap;
use crate::ast::Stmt;
use crate::error::IdppError;

/// Semua tipe nilai yang bisa disimpan dalam variabel ID++
#[derive(Debug, Clone)]
pub enum Nilai {
    Teks(String),
    Angka(f64),
    Boolean(bool),
    Daftar(Vec<Nilai>),
    Kamus(HashMap<String, Nilai>),
    Fungsi {
        params: Vec<String>,
        tubuh: Vec<Stmt>,
    },
    Kosong,
}

impl Nilai {
    /// Kembalikan nama tipe sebagai string Indonesia
    pub fn tipe_string(&self) -> &str {
        match self {
            Nilai::Teks(_) => "teks",
            Nilai::Angka(_) => "angka",
            Nilai::Boolean(_) => "boolean",
            Nilai::Daftar(_) => "daftar",
            Nilai::Kamus(_) => "kamus",
            Nilai::Fungsi { .. } => "fungsi",
            Nilai::Kosong => "kosong",
        }
    }

    /// Konversi nilai ke representasi string Indonesia
    pub fn ke_teks(&self) -> String {
        match self {
            Nilai::Teks(s) => s.clone(),
            Nilai::Angka(n) => {
                // Tampilkan tanpa desimal jika angka bulat
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Nilai::Boolean(b) => if *b { "benar".to_string() } else { "salah".to_string() },
            Nilai::Daftar(items) => {
                let elems: Vec<String> = items.iter().map(|v| v.ke_teks()).collect();
                format!("[{}]", elems.join(", "))
            }
            Nilai::Kamus(map) => {
                let mut pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.ke_teks()))
                    .collect();
                pairs.sort(); // urutan konsisten
                format!("{{{}}}", pairs.join(", "))
            }
            Nilai::Fungsi { params, .. } => {
                format!("<fungsi({})>", params.join(", "))
            }
            Nilai::Kosong => "kosong".to_string(),
        }
    }

    /// Konversi nilai ke angka, error jika tidak bisa
    pub fn ke_angka(&self, line: usize) -> Result<f64, IdppError> {
        match self {
            Nilai::Angka(n) => Ok(*n),
            Nilai::Teks(s) => s.trim().parse::<f64>().map_err(|_| IdppError::TipeTidakCocok {
                line,
                diharapkan: "angka".to_string(),
                dapat: format!("teks \"{}\"", s),
            }),
            Nilai::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            other => Err(IdppError::TipeTidakCocok {
                line,
                diharapkan: "angka".to_string(),
                dapat: other.tipe_string().to_string(),
            }),
        }
    }

    /// Konversi nilai ke boolean (truthiness)
    pub fn ke_boolean(&self) -> bool {
        match self {
            Nilai::Boolean(b) => *b,
            Nilai::Angka(n) => *n != 0.0,
            Nilai::Teks(s) => !s.is_empty(),
            Nilai::Daftar(v) => !v.is_empty(),
            Nilai::Kamus(m) => !m.is_empty(),
            Nilai::Kosong => false,
            Nilai::Fungsi { .. } => true,
        }
    }

    /// Apakah nilai dianggap benar (truthy)?
    pub fn is_truthy(&self) -> bool {
        self.ke_boolean()
    }
}

impl PartialEq for Nilai {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Nilai::Angka(a), Nilai::Angka(b)) => a == b,
            (Nilai::Teks(a), Nilai::Teks(b)) => a == b,
            (Nilai::Boolean(a), Nilai::Boolean(b)) => a == b,
            (Nilai::Kosong, Nilai::Kosong) => true,
            (Nilai::Daftar(a), Nilai::Daftar(b)) => a == b,
            _ => false,
        }
    }
}

/// Satu frame environment (scope) yang bisa punya parent scope
#[derive(Debug, Clone)]
pub struct Environment {
    /// Penyimpanan nilai: nama → (nilai, is_konstanta)
    nilai: HashMap<String, (Nilai, bool)>,
    /// Scope induk (untuk closure/nested scope)
    parent: Option<Box<Environment>>,
}

impl Environment {
    /// Buat environment baru (scope global)
    pub fn new() -> Self {
        Environment {
            nilai: HashMap::new(),
            parent: None,
        }
    }

    /// Buat environment anak dari parent (untuk fungsi dan blok)
    pub fn new_child(parent: Environment) -> Self {
        Environment {
            nilai: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Simpan variabel baru atau update yang sudah ada di scope lokal
    pub fn set(&mut self, nama: &str, nilai: Nilai, konstanta: bool) -> Result<(), IdppError> {
        // Cek apakah sudah ada sebagai konstanta
        if let Some((_, is_const)) = self.nilai.get(nama) {
            if *is_const {
                return Err(IdppError::KonstantaTidakBisaDiubah {
                    line: 0, // line diteruskan dari pemanggil
                    nama: nama.to_string(),
                });
            }
        }
        self.nilai.insert(nama.to_string(), (nilai, konstanta));
        Ok(())
    }

    /// Simpan variabel dengan pengecekan konstanta dan informasi baris
    pub fn set_dengan_baris(&mut self, nama: &str, nilai: Nilai, konstanta: bool, line: usize) -> Result<(), IdppError> {
        if let Some((_, is_const)) = self.nilai.get(nama) {
            if *is_const {
                return Err(IdppError::KonstantaTidakBisaDiubah {
                    line,
                    nama: nama.to_string(),
                });
            }
        }
        self.nilai.insert(nama.to_string(), (nilai, konstanta));
        Ok(())
    }

    /// Ambil nilai variabel, telusuri ke scope parent jika tidak ada
    pub fn get(&self, nama: &str) -> Option<&Nilai> {
        if let Some((val, _)) = self.nilai.get(nama) {
            Some(val)
        } else if let Some(parent) = &self.parent {
            parent.get(nama)
        } else {
            None
        }
    }

    /// Update variabel yang sudah ada (tidak membuat baru)
    pub fn update(&mut self, nama: &str, nilai: Nilai, line: usize) -> Result<(), IdppError> {
        if let Some((old_val, is_const)) = self.nilai.get_mut(nama) {
            if *is_const {
                return Err(IdppError::KonstantaTidakBisaDiubah {
                    line,
                    nama: nama.to_string(),
                });
            }
            *old_val = nilai;
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.update(nama, nilai, line)
        } else {
            Err(IdppError::VariabelTidakAda {
                line,
                nama: nama.to_string(),
            })
        }
    }

    /// Apakah variabel ada di scope ini atau parent?
    pub fn has(&self, nama: &str) -> bool {
        self.nilai.contains_key(nama) || self.parent.as_ref().map_or(false, |p| p.has(nama))
    }

    /// Ambil nilai sebagai Result (dengan pesan error)
    pub fn get_atau_error(&self, nama: &str, line: usize) -> Result<Nilai, IdppError> {
        self.get(nama).cloned().ok_or_else(|| IdppError::VariabelTidakAda {
            line,
            nama: nama.to_string(),
        })
    }

    /// Ambil parent dan kembalikan (untuk exit scope)
    pub fn ambil_parent(self) -> Option<Environment> {
        self.parent.map(|p| *p)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
