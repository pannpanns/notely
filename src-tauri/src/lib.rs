mod updates;
use rand::{distributions::Alphanumeric, Rng};
use std::{net::TcpListener, sync::{Mutex, atomic::{AtomicBool, Ordering}}};
use tauri::Emitter;
use tauri::{Manager, State};
use tauri_plugin_shell::{process::CommandChild, ShellExt};

struct Bridge {
    port: u16,
    token: String,
}
struct EngineChild(Mutex<Option<CommandChild>>);
struct BusyGuard(AtomicBool);

fn stop_engine(app: &tauri::AppHandle) {
    if let Some(process) = app.try_state::<EngineChild>() {
        if let Ok(mut guard) = process.0.lock() {
            if let Some(child) = guard.take() { let _ = child.kill(); }
        }
    }
}

#[tauri::command]
fn open_provider_help(provider: String) -> Result<(), String> {
    let url = match provider.as_str() {
        "gemini" => "https://aistudio.google.com/apikey",
        "openai" => "https://platform.openai.com/api-keys",
        "claude" => "https://platform.claude.com/settings/keys",
        "groq" => "https://console.groq.com/keys",
        "openrouter" => "https://openrouter.ai/settings/keys",
        _ => return Err("Penyedia tidak dikenal.".to_string()),
    };
    std::process::Command::new("explorer.exe").arg(url).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn choose_model_folder() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        rfd::FileDialog::new().set_title("Pilih folder model Notely (boleh di drive D:)")
            .pick_folder().map(|p| p.to_string_lossy().to_string())
    }).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn open_storage(app: tauri::AppHandle) -> Result<(), String> {
    let data = app.path().document_dir().map_err(|e| e.to_string())?.join("Notely");
    std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    std::process::Command::new("explorer.exe").arg(&data).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_busy_guard(value: bool, guard: State<'_, BusyGuard>) { guard.0.store(value, Ordering::Relaxed); }

#[tauri::command]
fn bridge_connection(bridge: State<'_, Bridge>) -> serde_json::Value {
    serde_json::json!({"url": format!("http://127.0.0.1:{}", bridge.port), "token": bridge.token})
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(updates::PendingUpdate::default())
        .setup(|app| {
            let listener = TcpListener::bind("127.0.0.1:0")?;
            let port = listener.local_addr()?.port();
            drop(listener);
            let token: String = rand::thread_rng().sample_iter(&Alphanumeric).take(64).map(char::from).collect();
            let data_root = app.path().document_dir()?.join("Notely");
            std::fs::create_dir_all(&data_root)?;
            let install_dir = std::env::current_exe()?.parent().ok_or("Folder instalasi tidak ditemukan")?.to_path_buf();
            let args = [
                "--install-dir".to_string(), install_dir.to_string_lossy().to_string(),
                "--parent-pid".to_string(), std::process::id().to_string(),
                "--port".to_string(), port.to_string(),
                "--token".to_string(), token.clone(),
                "--data-root".to_string(), data_root.to_string_lossy().to_string(),
            ];
            let (mut rx, child) = app.shell().sidecar("notely-engine")?.args(args).spawn()?;
            app.manage(Bridge { port, token });
            app.manage(BusyGuard(AtomicBool::new(false)));
            app.manage(EngineChild(Mutex::new(Some(child))));
            tauri::async_runtime::spawn(async move { while rx.recv().await.is_some() {} });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![bridge_connection, set_busy_guard, open_storage, choose_model_folder, open_provider_help, updates::check_update, updates::install_update])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if let Some(busy) = window.try_state::<BusyGuard>() {
                    if busy.0.load(Ordering::Relaxed) {
                        api.prevent_close();
                        let _ = window.emit("recording-close-blocked", "Hentikan & simpan rekaman sebelum menutup aplikasi.");
                    }
                }
            }
            if let tauri::WindowEvent::Destroyed = event {
                stop_engine(window.app_handle());
            }
        })
        .build(tauri::generate_context!())
        .expect("Notely gagal memulai antarmuka desktop")
        .run(|app, event| {
            match event {
                tauri::RunEvent::ExitRequested { api, .. } => {
                    if app.state::<BusyGuard>().0.load(Ordering::Relaxed) {
                        api.prevent_exit();
                        let _ = app.emit("recording-close-blocked", "Hentikan & simpan rekaman sebelum menutup aplikasi.");
                    } else {
                        stop_engine(app);
                    }
                }
                tauri::RunEvent::Exit => stop_engine(app),
                _ => {}
            }
        });
}
