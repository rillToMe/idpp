// REPL interaktif untuk bahasa ID++

use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;

/// Cek apakah buffer sudah membentuk pernyataan yang lengkap
/// (diakhiri titik atau kata penutup blok)
fn pernyataan_selesai(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.ends_with('.')
}

/// Cek apakah buffer mengandung blok yang belum ditutup
/// (jika ada lebih banyak pembuka blok daripada 'selesai')
fn ada_blok_terbuka(buffer: &str) -> bool {
    let pembuka = ["jika ", "selama ", "ulangi ", "untuk ", "buat fungsi", "coba", "lainnya", "atau jika "];
    let mut buka = 0i32;
    for line in buffer.lines() {
        let t = line.trim();
        if pembuka.iter().any(|p| t.starts_with(p)) { buka += 1; }
        if t.starts_with("selesai") { buka -= 1; }
        if t.starts_with("lainnya") || t.starts_with("atau jika") || t.starts_with("tangkap") || t.starts_with("akhirnya") {
            // bukan pembuka baru
        }
    }
    buka > 0
}

pub fn mulai() {
    println!("{}", "ID++ REPL (Read-Eval-Print Loop)".cyan().bold());
    println!("Ketik {}, {}, atau {}.",
        ".bantuan".green(),
        ".keluar".red(),
        ".bersih".yellow()
    );
    println!("Akhiri pernyataan dengan titik {}", "( . )".bold());
    println!("Tekan {} untuk membatalkan input multi-baris.\n", "Enter kosong".dimmed());

    let mut rl = DefaultEditor::new().unwrap();
    let mut interpreter = Interpreter::new();
    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() {
            "id++ > ".green().to_string()
        } else {
            "...  > ".yellow().to_string()
        };

        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                // Perintah REPL khusus (hanya di awal, tanpa buffer)
                if buffer.is_empty() && trimmed.starts_with('.') {
                    match trimmed {
                        ".keluar" | ".exit" => {
                            println!("{}", "Sampai jumpa!".cyan());
                            break;
                        }
                        ".bantuan" | ".help" => {
                            println!("\n{}", "Perintah REPL:".bold());
                            println!("  {}  Keluar dari REPL", ".keluar".red());
                            println!("  {}  Bersihkan layar", ".bersih".yellow());
                            println!("  {} Tampilkan bantuan", ".bantuan".green());
                            println!("\n{}", "Contoh:".bold());
                            println!("  {}  → Halo, Dunia!", "tulis \"Halo, Dunia!\".".cyan());
                            println!("  {}  → 30.0", "tulis 10 + 20.".cyan());
                            println!("  {}  → simpan variabel", "simpan nama = \"ID++\".".cyan());
                            println!();
                            continue;
                        }
                        ".bersih" | ".clear" => {
                            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                            continue;
                        }
                        ".versi" | ".version" => {
                            println!("ID++ v{}", env!("CARGO_PKG_VERSION"));
                            continue;
                        }
                        _ => {
                            println!("{} Ketik {} untuk daftar perintah.",
                                "Perintah tidak dikenal.".red(),
                                ".bantuan".green()
                            );
                            continue;
                        }
                    }
                }

                // Enter kosong: batalkan buffer atau evaluasi paksa
                if trimmed.is_empty() {
                    if buffer.is_empty() {
                        continue; // tidak ada yang dilakukan
                    }
                    // Jika masih ada blok terbuka, minta ditutup dulu
                    if ada_blok_terbuka(&buffer) {
                        println!("{}", "(Ada blok yang belum ditutup dengan 'selesai.' - lanjutkan atau Ctrl+C untuk batal)".dimmed());
                        continue;
                    }
                    // Buffer tidak kosong dan tidak ada blok terbuka → evaluasi sekarang
                    // (user menekan Enter kosong sebagai tanda selesai)
                    evaluasi(&buffer, &mut interpreter);
                    buffer.clear();
                    continue;
                }

                // Tambahkan baris ke buffer 
                buffer.push_str(trimmed);
                buffer.push('\n');
                rl.add_history_entry(trimmed).unwrap();

                // Evaluasi jika pernyataan selesai (diakhiri titik)
                // DAN tidak ada blok yang masih terbuka
                if pernyataan_selesai(trimmed) && !ada_blok_terbuka(&buffer) {
                    evaluasi(&buffer, &mut interpreter);
                    buffer.clear();
                }
            }

            // Ctrl+C: batalkan buffer saat ini
            Err(ReadlineError::Interrupted) => {
                if buffer.is_empty() {
                    println!("{}", "Tekan .keluar atau Ctrl+D untuk keluar.".dimmed());
                } else {
                    println!("{}", "(Input dibatalkan)".dimmed());
                    buffer.clear();
                }
            }

            // Ctrl+D / EOF: keluar
            Err(ReadlineError::Eof) => {
                println!("{}", "\nSampai jumpa!".cyan());
                break;
            }

            Err(err) => {
                eprintln!("Error REPL: {:?}", err);
                break;
            }
        }
    }
}

fn evaluasi(kode: &str, interpreter: &mut Interpreter) {
    let mut lexer = Lexer::new(kode);
    match lexer.tokenize() {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(stmts) => {
                    if let Err(e) = interpreter.run(stmts) {
                        eprintln!("{}", e.to_string().red());
                    }
                }
                Err(e) => eprintln!("{}", e.to_string().red()),
            }
        }
        Err(e) => eprintln!("{}", e.to_string().red()),
    }
}
