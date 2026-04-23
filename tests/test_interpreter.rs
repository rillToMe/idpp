// Unit test untuk interpreter ID++

use idpp::interpreter::Interpreter;
use idpp::lexer::Lexer;
use idpp::parser::Parser;
use std::fs;

// Helper function to run code and capture output (for testing)
// Karena println! menulis ke stdout, untuk unit test kita uji apakah kode berjalan tanpa error
// Untuk output, kita bisa test nilai variabel di environment (butuh penyesuaian interpreter)
// Untuk saat ini, kita pastikan program valid tidak panik/error.

fn jalankan_kode(kode: &str) -> Result<(), idpp::error::IdppError> {
    let mut lexer = Lexer::new(kode);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse()?;
    let mut interpreter = Interpreter::new();
    interpreter.run(stmts)?;
    Ok(())
}

#[test]
fn test_halo_dunia() {
    let kode = r#"tulis "Halo, Dunia!"."#;
    assert!(jalankan_kode(kode).is_ok());
}

#[test]
fn test_variabel_angka() {
    let kode = r#"
        simpan a sebagai 10.
        simpan b sebagai 5.
        simpan c sebagai a tambah b.
    "#;
    assert!(jalankan_kode(kode).is_ok());
}

#[test]
fn test_variabel_teks() {
    let kode = r#"
        simpan nama sebagai "Budi".
        tulis "Halo, ", nama, "!".
    "#;
    assert!(jalankan_kode(kode).is_ok());
}

#[test]
fn test_kondisi_jika() {
    let kode = r#"
        simpan nilai sebagai 85.
        jika nilai lebih dari sama 80 maka
            tulis "Lulus".
        lainnya
            tulis "Tidak Lulus".
        selesai.
    "#;
    assert!(jalankan_kode(kode).is_ok());
}

#[test]
fn test_loop_selama() {
    let kode = r#"
        simpan i sebagai 1.
        selama i kurang dari sama 5 lakukan
            tambah i dengan 1.
        selesai.
    "#;
    assert!(jalankan_kode(kode).is_ok());
}

#[test]
fn test_loop_untuk() {
    let kode = r#"
        simpan buah sebagai daftar "apel", "jeruk", "mangga".
        untuk setiap b dalam buah lakukan
            tulis b.
        selesai.
    "#;
    assert!(jalankan_kode(kode).is_ok());
}

#[test]
fn test_fungsi_sederhana() {
    let kode = r#"
        buat fungsi sapa dengan nama
            tulis "Halo, ", nama.
        selesai.
        jalankan sapa dengan "Andi".
    "#;
    assert!(jalankan_kode(kode).is_ok());
}

#[test]
fn test_fungsi_rekursif() {
    let kode = r#"
        buat fungsi faktorial dengan n
            jika n kurang dari sama 1 maka
                kembalikan 1.
            selesai.
            simpan s sebagai jalankan faktorial dengan n kurang 1.
            kembalikan n kali s.
        selesai.
        simpan hasil sebagai jalankan faktorial dengan 5.
    "#;
    assert!(jalankan_kode(kode).is_ok());
}

#[test]
fn test_daftar_operasi() {
    let kode = r#"
        simpan angka sebagai daftar 1, 2, 3.
        tulis angka di 0.
        ubah angka di 1 menjadi 99.
        tambahkan 4 ke angka.
        hapus item terakhir dari angka.
    "#;
    assert!(jalankan_kode(kode).is_ok());
}

#[test]
fn test_kamus_operasi() {
    let kode = r#"
        simpan orang sebagai kamus
            nama: "Budi",
            umur: 20.
        selesai.
        tulis orang ambil nama.
        ubah orang umur menjadi 21.
        tambah orang dengan kota bernilai "Jakarta".
    "#;
    assert!(jalankan_kode(kode).is_ok());
}

#[test]
fn test_error_handling() {
    let kode = r#"
        coba
            lempar "Error buatan!".
        tangkap galat
            tulis "Kena error: ", galat.
        akhirnya
            tulis "Selesai".
        selesai.
    "#;
    assert!(jalankan_kode(kode).is_ok());
}

#[test]
fn test_bagi_nol() {
    let kode = r#"
        simpan a sebagai 10 bagi 0.
    "#;
    let res = jalankan_kode(kode);
    assert!(res.is_err());
    if let Err(e) = res {
        let msg = e.to_string();
        assert!(msg.contains("Tidak bisa membagi dengan nol"));
    }
}

#[test]
fn test_konstanta_tidak_bisa_diubah() {
    let kode = r#"
        tetap PI sebagai 3.14.
        simpan PI sebagai 3.14159.
    "#;
    let res = jalankan_kode(kode);
    assert!(res.is_err());
    if let Err(e) = res {
        let msg = e.to_string();
        assert!(msg.contains("Konstanta 'PI' tidak bisa diubah"));
    }
}
