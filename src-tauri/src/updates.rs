use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};

#[derive(Default)]
pub struct PendingUpdate(Mutex<Option<Update>>);

#[tauri::command]
pub async fn check_update(app: tauri::AppHandle, pending: State<'_, PendingUpdate>) -> Result<Option<serde_json::Value>, String> {
    let update = app.updater().map_err(|e| e.to_string())?
        .check().await.map_err(|e| e.to_string())?;
    let metadata = update.as_ref().map(|u| serde_json::json!({
        "version": u.version, "currentVersion": u.current_version, "notes": u.body
    }));
    *pending.0.lock().map_err(|e| e.to_string())? = update;
    Ok(metadata)
}

#[tauri::command]
pub async fn install_update(app: tauri::AppHandle, pending: State<'_, PendingUpdate>) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    if app.state::<super::BusyGuard>().0.load(Ordering::Relaxed) {
        return Err("Hentikan dan simpan rekaman sebelum update.".into());
    }
    let update = pending.0.lock().map_err(|e| e.to_string())?.take()
        .ok_or("Periksa pembaruan kembali sebelum mengunduh.")?;
    let mut received = 0usize;
    // The official updater validates the signature before returning these bytes.
    let bytes = update.download(|size, total| {
        received += size;
        let _ = app.emit("notely-update-progress", serde_json::json!({"received":received,"total":total}));
    }, || {}).await.map_err(|e| e.to_string())?;
    if app.state::<super::BusyGuard>().0.load(Ordering::Relaxed) {
        return Err("Rekaman sedang berjalan. Simpan dahulu, kemudian coba update kembali.".into());
    }
    launch_installer(&app, &bytes)?;
    super::stop_engine(&app);
    app.exit(0);
    Ok(())
}

#[cfg(windows)]
fn launch_installer(app: &tauri::AppHandle, bytes: &[u8]) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let cache = app.path().app_cache_dir().map_err(|e| e.to_string())?.join("updates");
    std::fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    let installer = cache.join("notely-update.exe");
    let helper = cache.join("install-update.ps1");
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let directory = executable.parent().ok_or("Folder Notely tidak ditemukan")?;
    std::fs::write(&installer, bytes).map_err(|e| e.to_string())?;
    std::fs::write(&helper, include_str!("../windows/install-update.ps1")).map_err(|e| e.to_string())?;
    let powershell = std::env::var_os("SystemRoot").map(std::path::PathBuf::from)
        .ok_or("SystemRoot tidak ditemukan")?.join("System32/WindowsPowerShell/v1.0/powershell.exe");
    std::process::Command::new(powershell)
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(&helper).arg("-Installer").arg(&installer)
        .arg("-InstallDir").arg(directory).arg("-AppPid").arg(std::process::id().to_string())
        .creation_flags(0x08000000).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(windows))]
fn launch_installer(_app: &tauri::AppHandle, _bytes: &[u8]) -> Result<(), String> {
    Err("Updater ini hanya mendukung Windows NSIS x64.".into())
}
