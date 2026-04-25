// Cache manager untuk bytecode .idppc

use std::fs;
use std::path::Path;
use crate::bytecode::{ProgramBytecode, VERSI_BYTECODE};

/// Buat path cache dari path source: "file.idpp" → "file.idppc"
pub fn cache_path(source: &str) -> String {
    format!("{}c", source)
}

/// Cek apakah cache valid (lebih baru dari source DAN versi cocok)
pub fn cache_valid(src: &str, cache: &str) -> bool {
    let src_path = Path::new(src);
    let cache_path = Path::new(cache);

    if !cache_path.exists() { return false; }

    let src_time = match fs::metadata(src_path).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let cache_time = match fs::metadata(cache_path).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return false,
    };

    if cache_time <= src_time { return false; }

    // Cek versi bytecode
    match muat_cache(cache) {
        Ok(prog) => prog.versi == VERSI_BYTECODE,
        Err(_) => false,
    }
}

/// Simpan bytecode ke file cache
pub fn simpan_cache(path: &str, program: &ProgramBytecode) -> Result<(), String> {
    let data = bincode::serialize(program).map_err(|e| format!("Gagal serialize: {}", e))?;
    fs::write(path, data).map_err(|e| format!("Gagal tulis cache: {}", e))
}

/// Muat bytecode dari file cache
pub fn muat_cache(path: &str) -> Result<ProgramBytecode, String> {
    let data = fs::read(path).map_err(|e| format!("Gagal baca cache: {}", e))?;
    bincode::deserialize(&data).map_err(|e| format!("Gagal deserialize: {}", e))
}
