// Embed ikon dan metadata ke .exe Windows

fn main() {
    // Beritahu Cargo untuk rebuild jika aset berubah
    println!("cargo:rerun-if-changed=assets/idpp.ico");
    println!("cargo:rerun-if-changed=build.rs");

    // Hanya proses di Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    let ico_path = "assets/idpp.ico";

    // Embed ikon dan metadata ke .exe
    if std::path::Path::new(ico_path).exists() {
        let mut res = winres::WindowsResource::new();
        res.set_icon(ico_path);
        res.set("ProductName", "ID++");
        res.set("FileDescription", "Bahasa Pemrograman Indonesia ID++");
        res.set("CompanyName", "KyuzenStudio");
        res.set("LegalCopyright", "Copyright \u{00a9} 2026 KyuzenStudio");
        res.set("FileVersion", "0.1.0.0");
        res.set("ProductVersion", "0.1.0.0");
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0000000100000000);

        if let Err(e) = res.compile() {
            eprintln!("Peringatan: Gagal embed ikon - {}", e);
        }
    }
}
