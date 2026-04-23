// REPL interaktif untuk bahasa ID++

use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;

pub fn mulai() {
    println!("{}", "ID++ REPL (Read-Eval-Print Loop)".cyan().bold());
    println!("Ketik '.bantuan' untuk info, '.keluar' untuk keluar.");
    println!("Akhiri setiap pernyataan dengan titik (.).");

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
                let line = line.trim();
                
                // Perintah REPL khusus
                if buffer.is_empty() && line.starts_with('.') {
                    match line {
                        ".keluar" => break,
                        ".bantuan" => {
                            println!("Perintah REPL:");
                            println!("  .keluar   Keluar dari REPL");
                            println!("  .bersih   Bersihkan layar");
                            println!("  .bantuan  Tampilkan bantuan ini");
                            continue;
                        }
                        ".bersih" => {
                            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                            continue;
                        }
                        _ => {
                            println!("Perintah '{}' tidak dikenal.", line);
                            continue;
                        }
                    }
                }

                if line.is_empty() { continue; }

                buffer.push_str(line);
                buffer.push('\n');
                rl.add_history_entry(line).unwrap();

                // Cek apakah pernyataan sudah selesai (diakhiri titik)
                if !line.ends_with('.') && !line.ends_with("selesai.") {
                    continue; // Lanjut baca baris berikutnya
                }

                let mut lexer = Lexer::new(&buffer);
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

                buffer.clear(); // Bersihkan buffer untuk pernyataan selanjutnya
            }
            Err(ReadlineError::Interrupted) => {
                if buffer.is_empty() {
                    println!("Dibatalkan.");
                    break;
                } else {
                    println!("(Batal input multiline)");
                    buffer.clear();
                }
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
