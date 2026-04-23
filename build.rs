// Embed ikon dan metadata ke .exe Windows

fn main() {
    // Beritahu Cargo untuk rebuild jika aset berubah
    println!("cargo:rerun-if-changed=assets/idpp.png");
    println!("cargo:rerun-if-changed=build.rs");

    // Hanya proses di Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    let png_path = "assets/idpp.png";
    let ico_path = "assets/idpp.ico";

    // Konversi PNG ke ICO jika PNG tersedia
    if std::path::Path::new(png_path).exists() {
        if let Ok(img) = image::open(png_path) {
            let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

            // Buat beberapa ukuran ikon standar Windows
            for &size in &[256u32, 128, 64, 48, 32, 16] {
                let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
                let rgba = resized.to_rgba8();
                let icon_image = ico::IconImage::from_rgba_data(size, size, rgba.into_raw());
                icon_dir.add_entry(ico::IconDirEntry::encode(&icon_image).unwrap());
            }

            if let Ok(ico_file) = std::fs::File::create(ico_path) {
                let _ = icon_dir.write(ico_file);
            }
        }
    }

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
