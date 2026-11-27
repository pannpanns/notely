use rand::{distributions::Alphanumeric, Rng};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::Duration;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub struct EngineBridge {
    pub base_url: String,
    pub token: String,
    pub data_root: PathBuf,
    child: Option<Child>,
    client: reqwest::blocking::Client,
}

impl EngineBridge {
    pub fn spawn() -> Result<Self, String> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        drop(listener);

        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let data_root = dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Notely");
        let _ = std::fs::create_dir_all(&data_root);

        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let candidates = [
            exe_dir.join("notely-engine.exe"),
            exe_dir.join("notely-engine-x86_64-pc-windows-msvc.exe"),
            exe_dir.join("binaries").join("notely-engine.exe"),
            PathBuf::from("dist").join("notely-engine").join("notely-engine.exe"),
        ];

        let engine_exe = candidates
            .into_iter()
            .find(|p| p.exists())
            .ok_or_else(|| {
                format!(
                    "notely-engine.exe tidak ditemukan di sekitar {}",
                    exe_dir.display()
                )
            })?;

        let mut cmd = Command::new(&engine_exe);
        cmd.args([
            "--install-dir",
            &exe_dir.to_string_lossy(),
            "--parent-pid",
            &std::process::id().to_string(),
            "--port",
            &port.to_string(),
            "--token",
            &token,
            "--data-root",
            &data_root.to_string_lossy(),
        ]);
        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let child = cmd
            .spawn()
            .map_err(|e| format!("Gagal menjalankan mesin lokal: {}", e))?;

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self {
            base_url: format!("http://127.0.0.1:{}", port),
            token,
            data_root,
            child: Some(child),
            client,
        })
    }

    pub fn client_handle(&self) -> ApiClient {
        ApiClient {
            base_url: self.base_url.clone(),
            token: self.token.clone(),
            client: self.client.clone(),
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for EngineBridge {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Clone)]
pub struct ApiClient {
    pub base_url: String,
    pub token: String,
    client: reqwest::blocking::Client,
}

impl ApiClient {
    pub fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .map_err(|e| format!("Koneksi lokal terputus: {}", e))?;
        let status = resp.status();
        let val: Value = resp
            .json()
            .map_err(|e| format!("Respons tidak valid: {}", e))?;
        if !status.is_success() {
            let msg = val
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Permintaan gagal.");
            return Err(msg.to_string());
        }
        serde_json::from_value(val).map_err(|e| format!("Gagal membaca data: {}", e))
    }

    pub fn post_json<T: DeserializeOwned>(&self, path: &str, body: Value) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .map_err(|e| format!("Koneksi lokal terputus: {}", e))?;
        let status = resp.status();
        let val: Value = resp
            .json()
            .map_err(|e| format!("Respons tidak valid: {}", e))?;
        if !status.is_success() {
            let msg = val
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Permintaan gagal.");
            return Err(msg.to_string());
        }
        serde_json::from_value(val).map_err(|e| format!("Gagal membaca data: {}", e))
    }

    pub fn post_raw_file(&self, file_path: &Path, folder_id: &str) -> Result<Value, String> {
        let bytes = std::fs::read(file_path).map_err(|e| format!("Gagal membaca file: {}", e))?;
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("rekaman.wav");
        let encoded_name: String = file_name
            .bytes()
            .map(|b| {
                if b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_' {
                    (b as char).to_string()
                } else {
                    format!("%{:02X}", b)
                }
            })
            .collect();
        let url = format!("{}/import", self.base_url);
        let upload_client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .map_err(|e| e.to_string())?;
        let resp = upload_client
            .post(&url)
            .bearer_auth(&self.token)
            .header("X-File-Name", encoded_name)
            .header("X-Folder-Id", folder_id)
            .body(bytes)
            .send()
            .map_err(|e| format!("Gagal mengunggah audio: {}", e))?;
        let status = resp.status();
        let val: Value = resp.json().map_err(|e| e.to_string())?;
        if !status.is_success() {
            let msg = val
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Impor audio gagal.");
            return Err(msg.to_string());
        }
        Ok(val)
    }
}

pub fn open_external_url(url: &str) {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return;
    }
    #[cfg(target_os = "windows")]
    {
        let escaped = url.replace('\'', "''");
        let mut cmd = Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("Start-Process '{}'", escaped),
        ]);
        cmd.creation_flags(0x08000000);
        let _ = cmd.spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
}

pub fn open_folder_in_explorer(path: &Path) {
    let _ = std::fs::create_dir_all(path);
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer.exe").arg(path).spawn();
    }
}
