// Fungsi bawaan bahasa ID++
use crate::environment::Nilai;
use crate::error::IdppError;
use rand::Rng;

fn cek_args(nama: &str, args: &[Nilai], n: usize, line: usize) -> Result<(), IdppError> {
    if args.len() != n {
        Err(IdppError::JumlahArgumenSalah { line, nama: nama.to_string(), diharapkan: n, dapat: args.len() })
    } else { Ok(()) }
}

pub fn panjang(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    cek_args("panjang", args, 1, line)?;
    match &args[0] {
        Nilai::Teks(s) => Ok(Nilai::Angka(s.chars().count() as f64)),
        Nilai::Daftar(v) => Ok(Nilai::Angka(v.len() as f64)),
        other => Err(IdppError::TipeTidakCocok { line, diharapkan: "teks atau daftar".into(), dapat: other.tipe_string().into() }),
    }
}

pub fn huruf_besar(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    cek_args("huruf besar", args, 1, line)?;
    match &args[0] {
        Nilai::Teks(s) => Ok(Nilai::Teks(s.to_uppercase())),
        other => Err(IdppError::TipeTidakCocok { line, diharapkan: "teks".into(), dapat: other.tipe_string().into() }),
    }
}

pub fn huruf_kecil(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    cek_args("huruf kecil", args, 1, line)?;
    match &args[0] {
        Nilai::Teks(s) => Ok(Nilai::Teks(s.to_lowercase())),
        other => Err(IdppError::TipeTidakCocok { line, diharapkan: "teks".into(), dapat: other.tipe_string().into() }),
    }
}

pub fn potong(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    if args.len() != 3 { return Err(IdppError::JumlahArgumenSalah { line, nama: "potong".into(), diharapkan: 3, dapat: args.len() }); }
    let teks = match &args[0] { Nilai::Teks(s) => s.clone(), other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "teks".into(), dapat: other.tipe_string().into() }) };
    let awal = args[1].ke_angka(line)? as usize;
    let akhir = args[2].ke_angka(line)? as usize;
    let chars: Vec<char> = teks.chars().collect();
    let awal = awal.min(chars.len());
    let akhir = akhir.min(chars.len());
    Ok(Nilai::Teks(chars[awal..akhir].iter().collect()))
}

pub fn ganti(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    if args.len() != 3 { return Err(IdppError::JumlahArgumenSalah { line, nama: "ganti".into(), diharapkan: 3, dapat: args.len() }); }
    let teks = match &args[0] { Nilai::Teks(s) => s.clone(), other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "teks".into(), dapat: other.tipe_string().into() }) };
    let dari = match &args[1] { Nilai::Teks(s) => s.clone(), other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "teks".into(), dapat: other.tipe_string().into() }) };
    let ke   = match &args[2] { Nilai::Teks(s) => s.clone(), other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "teks".into(), dapat: other.tipe_string().into() }) };
    Ok(Nilai::Teks(teks.replace(dari.as_str(), ke.as_str())))
}

pub fn mengandung(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    if args.len() != 2 { return Err(IdppError::JumlahArgumenSalah { line, nama: "mengandung".into(), diharapkan: 2, dapat: args.len() }); }
    let teks = match &args[0] { Nilai::Teks(s) => s.clone(), other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "teks".into(), dapat: other.tipe_string().into() }) };
    let cari = match &args[1] { Nilai::Teks(s) => s.clone(), other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "teks".into(), dapat: other.tipe_string().into() }) };
    Ok(Nilai::Boolean(teks.contains(cari.as_str())))
}

pub fn bulatkan(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    cek_args("bulatkan", args, 1, line)?;
    Ok(Nilai::Angka(args[0].ke_angka(line)?.round()))
}

pub fn lantai(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    cek_args("lantai", args, 1, line)?;
    Ok(Nilai::Angka(args[0].ke_angka(line)?.floor()))
}

pub fn langit(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    cek_args("langit", args, 1, line)?;
    Ok(Nilai::Angka(args[0].ke_angka(line)?.ceil()))
}

pub fn mutlak(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    cek_args("mutlak", args, 1, line)?;
    Ok(Nilai::Angka(args[0].ke_angka(line)?.abs()))
}

pub fn acak(_args: &[Nilai], _line: usize) -> Result<Nilai, IdppError> {
    Ok(Nilai::Angka(rand::thread_rng().gen::<f64>()))
}

fn flatten_angka(args: &[Nilai], line: usize) -> Result<Vec<f64>, IdppError> {
    let mut out = Vec::new();
    for a in args {
        match a {
            Nilai::Angka(n) => out.push(*n),
            Nilai::Daftar(v) => { for i in v { out.push(i.ke_angka(line)?); } }
            other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "angka".into(), dapat: other.tipe_string().into() }),
        }
    }
    Ok(out)
}

pub fn maks(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    let v = flatten_angka(args, line)?;
    if v.is_empty() { return Err(IdppError::Runtime { line, pesan: "maks butuh setidaknya 1 argumen".into() }); }
    Ok(Nilai::Angka(v.into_iter().fold(f64::NEG_INFINITY, f64::max)))
}

pub fn min(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    let v = flatten_angka(args, line)?;
    if v.is_empty() { return Err(IdppError::Runtime { line, pesan: "min butuh setidaknya 1 argumen".into() }); }
    Ok(Nilai::Angka(v.into_iter().fold(f64::INFINITY, f64::min)))
}

pub fn akar(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    cek_args("akar", args, 1, line)?;
    let n = args[0].ke_angka(line)?;
    if n < 0.0 { return Err(IdppError::Runtime { line, pesan: "Tidak bisa menghitung akar dari angka negatif".into() }); }
    Ok(Nilai::Angka(n.sqrt()))
}

pub fn angka_dari(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    cek_args("angka dari", args, 1, line)?;
    match &args[0] {
        Nilai::Teks(s) => s.trim().parse::<f64>()
            .map(|n| Nilai::Angka(n.floor()))
            .map_err(|_| IdppError::Runtime { line, pesan: format!("Tidak bisa mengubah \"{}\" menjadi angka", s) }),
        Nilai::Angka(n) => Ok(Nilai::Angka(n.floor())),
        other => Err(IdppError::TipeTidakCocok { line, diharapkan: "teks".into(), dapat: other.tipe_string().into() }),
    }
}

pub fn teks_dari(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    cek_args("teks dari", args, 1, line)?;
    Ok(Nilai::Teks(args[0].ke_teks()))
}

pub fn desimal_dari(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    cek_args("desimal dari", args, 1, line)?;
    match &args[0] {
        Nilai::Teks(s) => s.trim().parse::<f64>().map(Nilai::Angka)
            .map_err(|_| IdppError::Runtime { line, pesan: format!("Tidak bisa mengubah \"{}\" menjadi desimal", s) }),
        Nilai::Angka(n) => Ok(Nilai::Angka(*n)),
        other => Err(IdppError::TipeTidakCocok { line, diharapkan: "teks".into(), dapat: other.tipe_string().into() }),
    }
}

pub fn tipe_dari(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    cek_args("tipe dari", args, 1, line)?;
    Ok(Nilai::Teks(args[0].tipe_string().to_string()))
}
