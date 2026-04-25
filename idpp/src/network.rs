// Library HTTP bawaan ID++
// Setara dengan library "requests" di Python
//
// Sintaks ID++:
//   simpan resp = http ambil "https://api.example.com".
//   simpan resp = http perbarui "https://..." dengan data kamus { ... } selesai.
//   tulis resp ambil status.
//   tulis resp ambil json.
//   tulis resp ambil teks.

use std::collections::HashMap;
use crate::environment::Nilai;
use crate::error::IdppError;

// Konversi serde_json::Value → Nilai

fn json_ke_nilai(val: serde_json::Value) -> Nilai {
    match val {
        serde_json::Value::Null => Nilai::Kosong,
        serde_json::Value::Bool(b) => Nilai::Boolean(b),
        serde_json::Value::Number(n) => {
            Nilai::Angka(n.as_f64().unwrap_or(0.0))
        }
        serde_json::Value::String(s) => Nilai::Teks(s),
        serde_json::Value::Array(arr) => {
            Nilai::Daftar(arr.into_iter().map(json_ke_nilai).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, json_ke_nilai(v));
            }
            Nilai::Kamus(map)
        }
    }
}

// Konversi Nilai → serde_json::Value (untuk kirim body JSON)

fn nilai_ke_json(val: &Nilai) -> serde_json::Value {
    match val {
        Nilai::Teks(s) => serde_json::Value::String(s.clone()),
        Nilai::Angka(n) => {
            if n.fract() == 0.0 {
                serde_json::Value::Number((*n as i64).into())
            } else {
                serde_json::json!(*n)
            }
        }
        Nilai::Boolean(b) => serde_json::Value::Bool(*b),
        Nilai::Kosong => serde_json::Value::Null,
        Nilai::Daftar(items) => {
            serde_json::Value::Array(items.iter().map(nilai_ke_json).collect())
        }
        Nilai::Kamus(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), nilai_ke_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        Nilai::Fungsi { .. } => serde_json::Value::Null,
    }
}

// Buat objek Respons sebagai Nilai::Kamus 
//
// Setelah request, user bisa akses:
//   resp ambil status   → angka (200, 404, dst)
//   resp ambil ok       → boolean (true jika 2xx)
//   resp ambil teks     → body sebagai teks
//   resp ambil json     → body diparsing sebagai kamus/daftar
//   resp ambil header   → kamus dari response headers
//   resp ambil url      → URL akhir (setelah redirect)

fn buat_respons(
    status: u16,
    body: String,
    headers: HashMap<String, String>,
    url: String,
) -> Nilai {
    let ok = status >= 200 && status < 300;

    // Coba parse JSON, kalau gagal kembalikan Kosong
    let json_val = serde_json::from_str::<serde_json::Value>(&body)
        .map(json_ke_nilai)
        .unwrap_or(Nilai::Kosong);

    let header_map: HashMap<String, Nilai> = headers
        .into_iter()
        .map(|(k, v)| (k, Nilai::Teks(v)))
        .collect();

    let mut resp = HashMap::new();
    resp.insert("status".to_string(), Nilai::Angka(status as f64));
    resp.insert("ok".to_string(), Nilai::Boolean(ok));
    resp.insert("teks".to_string(), Nilai::Teks(body));
    resp.insert("json".to_string(), json_val);
    resp.insert("header".to_string(), Nilai::Kamus(header_map));
    resp.insert("url".to_string(), Nilai::Teks(url));

    Nilai::Kamus(resp)
}

// Helper: Nilai::Kamus → HashMap<String, String> 

fn kamus_ke_str_map(val: &Nilai, nama: &str, line: usize) -> Result<HashMap<String, String>, IdppError> {
    match val {
        Nilai::Kamus(m) => {
            let mut out = HashMap::new();
            for (k, v) in m {
                out.insert(k.clone(), v.ke_teks());
            }
            Ok(out)
        }
        other => Err(IdppError::TipeTidakCocok {
            line,
            diharapkan: format!("kamus untuk {}", nama),
            dapat: other.tipe_string().to_string(),
        }),
    }
}

// Opsi Request (header, param, auth, timeout) 

#[derive(Default)]
pub struct OpsiRequest {
    pub header: HashMap<String, String>,
    pub param: HashMap<String, String>,  // query string
    pub username: Option<String>,        // Basic Auth
    pub password: Option<String>,
    pub timeout_detik: u64,             // default 30
}

impl OpsiRequest {
    pub fn new() -> Self {
        OpsiRequest {
            timeout_detik: 30,
            ..Default::default()
        }
    }

    /// Buat dari Nilai::Kamus opsi
    /// Kamus boleh berisi: header, param, auth (daftar [user, pass]), timeout
    pub fn dari_kamus(kamus: &Nilai, line: usize) -> Result<Self, IdppError> {
        let mut opsi = OpsiRequest::new();
        if let Nilai::Kamus(map) = kamus {
            if let Some(h) = map.get("header") {
                opsi.header = kamus_ke_str_map(h, "header", line)?;
            }
            if let Some(p) = map.get("param") {
                opsi.param = kamus_ke_str_map(p, "param", line)?;
            }
            if let Some(Nilai::Daftar(auth)) = map.get("auth") {
                if auth.len() >= 2 {
                    opsi.username = Some(auth[0].ke_teks());
                    opsi.password = Some(auth[1].ke_teks());
                }
            }
            if let Some(t) = map.get("timeout") {
                opsi.timeout_detik = t.ke_angka(line)? as u64;
            }
        }
        Ok(opsi)
    }
}

// Buat reqwest::blocking::Client 

fn buat_client(timeout_detik: u64) -> Result<reqwest::blocking::Client, IdppError> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_detik))
        .user_agent(concat!("idpp/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| IdppError::Runtime {
            line: 0,
            pesan: format!("Gagal membuat HTTP client: {}", e),
        })
}

fn kumpul_respons(resp: reqwest::blocking::Response, line: usize) -> Result<Nilai, IdppError> {
    let status = resp.status().as_u16();
    let url = resp.url().to_string();
    let headers: HashMap<String, String> = resp
        .headers()
        .iter()
        .filter_map(|(k, v)| {
            v.to_str().ok().map(|val| (k.to_string(), val.to_string()))
        })
        .collect();
    let body = resp.text().map_err(|e| IdppError::Runtime {
        line,
        pesan: format!("Gagal membaca respons: {}", e),
    })?;
    Ok(buat_respons(status, body, headers, url))
}

// Fungsi Publik

/// http_ambil("url") atau http_ambil("url", kamus_opsi)
/// GET request
pub fn http_ambil(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    if args.is_empty() {
        return Err(IdppError::Runtime { line, pesan: "http ambil butuh minimal 1 argumen: URL".into() });
    }
    let url = match &args[0] {
        Nilai::Teks(s) => s.clone(),
        other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "teks (URL)".into(), dapat: other.tipe_string().into() }),
    };
    let opsi = if args.len() >= 2 {
        OpsiRequest::dari_kamus(&args[1], line)?
    } else {
        OpsiRequest::new()
    };

    let client = buat_client(opsi.timeout_detik)?;
    let mut req = client.get(&url);
    for (k, v) in &opsi.header {
        req = req.header(k, v);
    }
    if !opsi.param.is_empty() {
        req = req.query(&opsi.param.iter().collect::<Vec<_>>());
    }
    if let (Some(u), Some(p)) = (&opsi.username, &opsi.password) {
        req = req.basic_auth(u, Some(p));
    }
    let resp = req.send().map_err(|e| IdppError::Runtime { line, pesan: format!("HTTP GET gagal: {}", e) })?;
    kumpul_respons(resp, line)
}

/// http_kirim("url", body_nilai) atau http_kirim("url", body, kamus_opsi)
/// POST request - body dikirim sebagai JSON
pub fn http_kirim(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    if args.len() < 2 {
        return Err(IdppError::Runtime { line, pesan: "http kirim butuh minimal 2 argumen: URL dan data".into() });
    }
    let url = match &args[0] {
        Nilai::Teks(s) => s.clone(),
        other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "teks (URL)".into(), dapat: other.tipe_string().into() }),
    };
    let body = nilai_ke_json(&args[1]);
    let opsi = if args.len() >= 3 {
        OpsiRequest::dari_kamus(&args[2], line)?
    } else {
        OpsiRequest::new()
    };

    let client = buat_client(opsi.timeout_detik)?;
    let mut req = client.post(&url).json(&body);
    for (k, v) in &opsi.header {
        req = req.header(k, v);
    }
    if !opsi.param.is_empty() {
        req = req.query(&opsi.param.iter().collect::<Vec<_>>());
    }
    if let (Some(u), Some(p)) = (&opsi.username, &opsi.password) {
        req = req.basic_auth(u, Some(p));
    }
    let resp = req.send().map_err(|e| IdppError::Runtime { line, pesan: format!("HTTP POST gagal: {}", e) })?;
    kumpul_respons(resp, line)
}

/// http_ubah("url", body) - PUT request
pub fn http_ubah(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    if args.len() < 2 {
        return Err(IdppError::Runtime { line, pesan: "http ubah butuh minimal 2 argumen: URL dan data".into() });
    }
    let url = match &args[0] {
        Nilai::Teks(s) => s.clone(),
        other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "teks (URL)".into(), dapat: other.tipe_string().into() }),
    };
    let body = nilai_ke_json(&args[1]);
    let opsi = if args.len() >= 3 { OpsiRequest::dari_kamus(&args[2], line)? } else { OpsiRequest::new() };

    let client = buat_client(opsi.timeout_detik)?;
    let mut req = client.put(&url).json(&body);
    for (k, v) in &opsi.header { req = req.header(k, v); }
    if !opsi.param.is_empty() { req = req.query(&opsi.param.iter().collect::<Vec<_>>()); }
    if let (Some(u), Some(p)) = (&opsi.username, &opsi.password) { req = req.basic_auth(u, Some(p)); }
    let resp = req.send().map_err(|e| IdppError::Runtime { line, pesan: format!("HTTP PUT gagal: {}", e) })?;
    kumpul_respons(resp, line)
}

/// http_hapus("url") - DELETE request
pub fn http_hapus(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    if args.is_empty() {
        return Err(IdppError::Runtime { line, pesan: "http hapus butuh minimal 1 argumen: URL".into() });
    }
    let url = match &args[0] {
        Nilai::Teks(s) => s.clone(),
        other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "teks (URL)".into(), dapat: other.tipe_string().into() }),
    };
    let opsi = if args.len() >= 2 { OpsiRequest::dari_kamus(&args[1], line)? } else { OpsiRequest::new() };

    let client = buat_client(opsi.timeout_detik)?;
    let mut req = client.delete(&url);
    for (k, v) in &opsi.header { req = req.header(k, v); }
    if let (Some(u), Some(p)) = (&opsi.username, &opsi.password) { req = req.basic_auth(u, Some(p)); }
    let resp = req.send().map_err(|e| IdppError::Runtime { line, pesan: format!("HTTP DELETE gagal: {}", e) })?;
    kumpul_respons(resp, line)
}

/// http_perbarui("url", body) - PATCH request
pub fn http_perbarui(args: &[Nilai], line: usize) -> Result<Nilai, IdppError> {
    if args.len() < 2 {
        return Err(IdppError::Runtime { line, pesan: "http perbarui butuh minimal 2 argumen: URL dan data".into() });
    }
    let url = match &args[0] {
        Nilai::Teks(s) => s.clone(),
        other => return Err(IdppError::TipeTidakCocok { line, diharapkan: "teks (URL)".into(), dapat: other.tipe_string().into() }),
    };
    let body = nilai_ke_json(&args[1]);
    let opsi = if args.len() >= 3 { OpsiRequest::dari_kamus(&args[2], line)? } else { OpsiRequest::new() };

    let client = buat_client(opsi.timeout_detik)?;
    let mut req = client.patch(&url).json(&body);
    for (k, v) in &opsi.header { req = req.header(k, v); }
    if !opsi.param.is_empty() { req = req.query(&opsi.param.iter().collect::<Vec<_>>()); }
    if let (Some(u), Some(p)) = (&opsi.username, &opsi.password) { req = req.basic_auth(u, Some(p)); }
    let resp = req.send().map_err(|e| IdppError::Runtime { line, pesan: format!("HTTP PATCH gagal: {}", e) })?;
    kumpul_respons(resp, line)
}
