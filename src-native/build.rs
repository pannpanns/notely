fn main() {
    #[cfg(target_os = "windows")]
    {
        let icon_path = "../src-tauri/icons/notely-icon.ico";
        if std::path::Path::new(icon_path).exists() {
            let mut res = winresource::WindowsResource::new();
            res.set_icon(icon_path);
            res.set("ProductName", "Notely");
            res.set("FileDescription", "Notely Desktop — Transkripsi & Catatan AI Lokal");
            let _ = res.compile();
        }
    }
}
