// Entry point CLI untuk bahasa ID++

mod ast;
mod builtin;
mod bytecode;
mod cache;
mod compiler;
mod environment;
mod error;
mod interpreter;
mod lexer;
mod network;
mod parser;
mod repl;
mod token;
mod vm;

use colored::*;
use std::env;
use std::fs;
use std::process;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Byte ICO yang di-embed langsung ke dalam binary saat kompilasi
/// File ini dibuat otomatis oleh build.rs dari assets/idpp.png
#[cfg(windows)]
const ICO_BYTES: &[u8] = include_bytes!("../assets/idpp.ico");

fn main() {
    // Auto-registrasi ikon file .idpp di Windows (berjalan diam-diam di background)
    #[cfg(windows)]
    auto_register_windows();

    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => repl::mulai(),
        2 => match args[1].as_str() {
            "--versi" | "-v" | "--version" => tampilkan_versi(),
            "--bantuan" | "-b" => tampilkan_bantuan(),
            "--no-cache" => {
                eprintln!("{}", "Error: --no-cache membutuhkan file.".red());
                process::exit(1);
            }
            path => jalankan_file(path, true),
        },
        3 => {
            if args[1] == "--no-cache" {
                jalankan_file(&args[2], false);
            } else {
                eprintln!("{}", "Error: Argumen tidak dikenal.".red());
                tampilkan_bantuan();
                process::exit(1);
            }
        }
        _ => {
            eprintln!("{}", "Error: Terlalu banyak argumen.".red());
            tampilkan_bantuan();
            process::exit(1);
        }
    }
}

/// Registrasi otomatis ikon .idpp di Windows Explorer.
/// - Menyimpan ICO ke %LOCALAPPDATA%\idpp\idpp.ico
/// - Menulis registry HKCU\Software\Classes\.idpp
/// - Hanya berjalan sekali jika belum terdaftar (cek registry flag)
/// - Gagal diam-diam agar tidak mengganggu pengguna
#[cfg(windows)]
fn auto_register_windows() {
    use std::path::PathBuf;
    use winreg::enums::*;
    use winreg::RegKey;

    // Cek apakah sudah pernah didaftarkan dengan versi yang sama
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let flag_path = r"Software\idpp\install";
    if let Ok(flag_key) = hkcu.open_subkey(flag_path) {
        if let Ok(ver) = flag_key.get_value::<String, _>("version") {
            if ver == VERSION {
                return; // sudah terdaftar, skip
            }
        }
    }

    // Tentukan folder tujuan ICO: %LOCALAPPDATA%\idpp\
    let local_app_data = match env::var("LOCALAPPDATA") {
        Ok(p) => PathBuf::from(p),
        Err(_) => return,
    };
    let idpp_dir = local_app_data.join("idpp");
    let ico_path = idpp_dir.join("idpp.ico");

    // Buat folder dan tulis ICO dari bytes yang di-embed
    if fs::create_dir_all(&idpp_dir).is_err() {
        return;
    }
    if fs::write(&ico_path, ICO_BYTES).is_err() {
        return;
    }

    // Dapatkan path exe yang sedang berjalan
    let exe_path = match env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };
    let exe_str = exe_path.to_string_lossy();
    let ico_str = ico_path.to_string_lossy();

    // Tulis registry keys di HKCU (tidak butuh admin)
    let classes = match hkcu.open_subkey_with_flags(r"Software\Classes", KEY_ALL_ACCESS) {
        Ok(k) => k,
        Err(_) => return,
    };

    // .idpp → idpp_file
    let (ext_key, _) = match classes.create_subkey(r".idpp") {
        Ok(k) => k,
        Err(_) => return,
    };
    let _ = ext_key.set_value("", &"idpp_file");

    // idpp_file → deskripsi
    let (type_key, _) = match classes.create_subkey(r"idpp_file") {
        Ok(k) => k,
        Err(_) => return,
    };
    let _ = type_key.set_value("", &"ID++ Source File");

    // DefaultIcon
    let (icon_key, _) = match type_key.create_subkey(r"DefaultIcon") {
        Ok(k) => k,
        Err(_) => return,
    };
    let ico_val = format!("\"{}\",0", ico_str);
    let _ = icon_key.set_value("", &ico_val);

    // Shell → open → command (double-click buka dengan idpp)
    let (cmd_key, _) = match type_key.create_subkey(r"shell\open\command") {
        Ok(k) => k,
        Err(_) => return,
    };
    let cmd_val = format!("\"{}\" \"%1\"", exe_str);
    let _ = cmd_key.set_value("", &cmd_val);

    // Simpan flag versi agar tidak diulang
    if let Ok((flag_key, _)) = hkcu.create_subkey(flag_path) {
        let _ = flag_key.set_value("version", &VERSION);
    }

    // Beritahu Windows Explorer untuk refresh icon cache
    refresh_explorer_icons();
}

/// Paksa Windows Explorer refresh ikon tanpa restart
#[cfg(windows)]
fn refresh_explorer_icons() {
    use std::process::Command;
    // ie4uinit mereset icon cache
    let _ = Command::new("ie4uinit.exe").arg("-show").output();
}

fn tampilkan_versi() {
    println!("ID++ versi {}", VERSION.green().bold());
    println!("Bahasa pemrograman berbahasa Indonesia");
    println!("https://github.com/rillToMe/idpp");
}

fn tampilkan_bantuan() {
    println!("\n{}", "Penggunaan: idpp <file.idpp>".bold());
    println!("\nPerintah:");
    println!("  idpp file.idpp          Jalankan file ID++");
    println!("  idpp --no-cache f.idpp  Jalankan tanpa cache");
    println!("  idpp --versi            Tampilkan versi");
    println!("  idpp --bantuan          Tampilkan bantuan ini");
    println!("  idpp                    Masuk mode interaktif (REPL)");
    println!("\nContoh:");
    println!("  idpp halo.idpp");
    println!("  idpp --no-cache halo.idpp\n");
}

fn jalankan_file(path: &str, use_cache: bool) {
    if !path.ends_with(".idpp") {
        eprintln!("{}", "Peringatan: Ekstensi file biasanya .idpp".yellow());
    }

    let cache_file = cache::cache_path(path);

    // Coba load dari cache
    if use_cache && cache::cache_valid(path, &cache_file) {
        if let Ok(program) = cache::muat_cache(&cache_file) {
            let mut mesin = vm::VM::new();
            if let Err(e) = mesin.run(&program) {
                eprintln!("{}", e.to_string().red());
                process::exit(1);
            }
            return;
        }
    }

    // Parse dari source
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", format!("Gagal membaca file '{}': {}", path, e).red());
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    match lexer.tokenize() {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(stmts) => {
                    // Compile ke bytecode
                    let comp = compiler::Compiler::new();
                    let program = comp.compile(&stmts);

                    // Simpan cache (silent fail)
                    if use_cache {
                        let _ = cache::simpan_cache(&cache_file, &program);
                    }

                    // Eksekusi via VM
                    let mut mesin = vm::VM::new();
                    if let Err(e) = mesin.run(&program) {
                        eprintln!("{}", e.to_string().red());
                        process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("{}", e.to_string().red());
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e.to_string().red());
            process::exit(1);
        }
    }
}
