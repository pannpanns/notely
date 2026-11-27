use crate::types::{UpdateInfo, APP_VERSION};
use serde::Deserialize;
use std::io::{Read, Write};
use std::sync::mpsc::Sender;

const RELEASES_API: &str = "https://api.github.com/repos/pannpanns/notely/releases/latest";

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    body: Option<String>,
    assets: Vec<GhAsset>,
}

fn parse_semver(s: &str) -> (u32, u32, u32) {
    let clean = s.trim().trim_start_matches('v');
    let mut parts = clean.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

pub fn check_latest_release() -> Result<Option<UpdateInfo>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .user_agent(format!("Notely-Desktop/{}", APP_VERSION))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(RELEASES_API)
        .send()
        .map_err(|e| format!("Gagal memeriksa pembaruan: {}", e))?;
    if !resp.status().is_success() {
        return Ok(None);
    }
    let rel: GhRelease = resp.json().map_err(|e| e.to_string())?;
    let latest_ver = rel.tag_name.trim_start_matches('v').to_string();
    if parse_semver(&latest_ver) <= parse_semver(APP_VERSION) {
        return Ok(None);
    }
    let asset = rel
        .assets
        .into_iter()
        .find(|a| a.name.ends_with("-setup.exe") || a.name.ends_with(".exe"));
    let Some(asset) = asset else {
        return Ok(None);
    };
    Ok(Some(UpdateInfo {
        version: latest_ver,
        notes: rel.body.unwrap_or_else(|| "Pembaruan terbaru Notely tersedia.".to_string()),
        download_url: asset.browser_download_url,
        asset_name: asset.name,
    }))
}

#[derive(Debug, Clone)]
pub enum UpdateDownloadEvent {
    Progress { downloaded: u64, total: u64 },
    ReadyToInstall(std::path::PathBuf),
    Error(String),
}

pub fn download_installer(info: UpdateInfo, tx: Sender<UpdateDownloadEvent>) {
    std::thread::spawn(move || {
        let client = match reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(600))
            .user_agent(format!("Notely-Desktop/{}", APP_VERSION))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(UpdateDownloadEvent::Error(e.to_string()));
                return;
            }
        };

        let mut resp = match client.get(&info.download_url).send() {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                let _ = tx.send(UpdateDownloadEvent::Error(format!(
                    "Unduhan gagal (HTTP {})",
                    r.status()
                )));
                return;
            }
            Err(e) => {
                let _ = tx.send(UpdateDownloadEvent::Error(format!("Koneksi gagal: {}", e)));
                return;
            }
        };

        let total = resp.content_length().unwrap_or(0);
        let dest = std::env::temp_dir().join(&info.asset_name);
        let mut file = match std::fs::File::create(&dest) {
            Ok(f) => f,
            Err(e) => {
                let _ = tx.send(UpdateDownloadEvent::Error(e.to_string()));
                return;
            }
        };

        let mut buf = [0u8; 65536];
        let mut downloaded = 0u64;
        loop {
            match resp.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(e) = file.write_all(&buf[..n]) {
                        let _ = tx.send(UpdateDownloadEvent::Error(e.to_string()));
                        return;
                    }
                    downloaded += n as u64;
                    let _ = tx.send(UpdateDownloadEvent::Progress { downloaded, total });
                }
                Err(e) => {
                    let _ = tx.send(UpdateDownloadEvent::Error(e.to_string()));
                    return;
                }
            }
        }
        let _ = file.flush();
        let _ = tx.send(UpdateDownloadEvent::ReadyToInstall(dest));
    });
}

pub fn launch_installer(path: &std::path::Path) -> Result<(), String> {
    std::process::Command::new(path)
        .spawn()
        .map_err(|e| format!("Gagal menjalankan installer: {}", e))?;
    Ok(())
}
