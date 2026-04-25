// Definisi OpCode bytecode flat untuk VM ID++
// Semua instruksi flat, control flow via jump offset, variabel via u32 ID

use serde::{Serialize, Deserialize};

pub const VERSI_BYTECODE: u32 = 1;

/// OpCode - instruksi tunggal untuk VM stack-based
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OpCode {
    // Konstanta
    PushAngka(f64),
    PushTeks(u32),           // index ke teks_pool
    PushBool(bool),
    PushKosong,

    // Variabel (u32 ID)
    LoadVar(u32),            // push nilai variabel ke stack
    StoreVar(u32),           // pop → simpan ke slot
    StoreConst(u32),         // pop → simpan sebagai konstanta

    // Aritmatika (pop 2, push 1)
    Tambah,
    Kurang,
    Kali,
    Bagi,
    Sisa,
    Pangkat,

    // Unary (pop 1, push 1)
    Negatif,
    Bukan,

    // Perbandingan (pop 2, push bool)
    CmpGt,    // >
    CmpLt,    // <
    CmpGe,    // >=
    CmpLe,    // <=
    CmpEq,    // ==
    CmpNe,    // !=

    // Logika (pop 2, push bool)
    Dan,
    Atau,

    // Control Flow
    Jump(i32),               // lompat relatif (bisa negatif)
    JumpIfFalse(i32),        // pop, lompat jika falsy

    // Stack
    Pop,

    // I/O
    Cetak(u16),              // pop N item, concat, println
    Tanya(u32, u32),         // (teks_pool_id pertanyaan, var_id target)

    // Fungsi
    DefFunc(u32),            // index ke fungsi_tabel → daftarkan di runtime
    Call(u32, u16),          // (var_id nama fungsi, jumlah arg)
    Return,                  // pop → return value

    // Koleksi
    BuatDaftar(u16),         // pop N → push Daftar
    BuatKamus(u16),          // pop N pasangan (value, key_teks_id) → push Kamus
    PushKamusKey(u32),       // push key string (teks_pool index)
    AksesDaftar,             // pop [index, list] → push value
    AksesKamus,              // pop [key, dict] → push value
    PunyaKunci,              // pop [key, dict] → push bool

    // Mutasi Koleksi
    AppendDaftar(u32),       // pop value → append ke var_id
    HapusPertama(u32),       // remove pertama dari var_id
    HapusTerakhir(u32),      // remove terakhir dari var_id
    SetDaftarIdx(u32),       // pop [value, index] → var[index] = value
    SetKamusKey(u32, u32),   // pop value → var_id[teks_id] = value
    InsertKamusKey(u32, u32),// pop value → insert var_id[teks_id]

    // Modifikasi Variabel
    AddVar(u32),             // pop value → var += value
    SubVar(u32),             // pop value → var -= value
    MulVar(u32),             // pop value → var *= value
    DivVar(u32),             // pop value → var /= value

    // Iterasi (untuk for-each)
    SetupIter(u32),          // pop iterable → simpan ke temp slot
    IterNext(u32, i32),      // (var_id loop_var, jump_offset jika habis)

    // Loop Control
    Hentikan,                // break - VM cari handler loop terdekat
    Lanjut,                  // continue - VM cari handler loop terdekat
    EnterLoop(i32, i32),     // (continue_offset, break_offset) relatif
    ExitLoop,                // pop loop handler

    // Error Handling
    SetupTry(i32, i32),      // (catch_jump, finally_jump) relatif dari IP saat ini
    EndTry,                  // cleanup handler
    SetCatchVar(u32),        // simpan error message ke var_id
    Lempar,                  // pop → throw error

    // Builtin
    CallBuiltin(u8, u16),    // (builtin_id, jumlah arg)

    // HTTP
    HttpReq(u8, u16),        // (metode 0-4, jumlah arg)

    // Rentang
    BuatRentang,             // pop [end, start] → push Daftar

    // Scope
    PushScope,
    PopScope,

    // Debug
    Line(u32),               // set nomor baris untuk error reporting

    // Program
    Nop,
    Halt,
}

/// Info fungsi yang didefinisikan user
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FungsiInfo {
    pub nama_id: u32,               // var_id untuk nama fungsi
    pub params: Vec<u32>,           // var_id untuk tiap parameter
    pub instruksi: Vec<OpCode>,     // bytecode body fungsi
}

/// Program bytecode lengkap yang bisa di-cache
#[derive(Serialize, Deserialize, Debug)]
pub struct ProgramBytecode {
    pub versi: u32,
    pub teks_pool: Vec<String>,       // constant string pool
    pub simbol: Vec<String>,          // index → nama variabel (untuk error msg)
    pub fungsi_tabel: Vec<FungsiInfo>,
    pub instruksi: Vec<OpCode>,       // instruksi utama (main)
}
