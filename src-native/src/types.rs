use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Notes,
    Recordings,
    Deadlines,
    Settings,
}

impl Page {
    pub fn label(self) -> &'static str {
        match self {
            Page::Home => "Beranda",
            Page::Notes => "Catatan Rekaman",
            Page::Recordings => "Semua Rekaman",
            Page::Deadlines => "Tenggat Tugas",
            Page::Settings => "Pengaturan",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteTab {
    Transcript,
    Summary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadlineView {
    Calendar,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadlineFilter {
    All,
    Upcoming,
    Urgent,
    Overdue,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Recordings,
    Transcription,
    Summary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiStep {
    Input,
    Loading,
    Results,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppPrefs {
    pub model: String,
    pub language: String,
    pub provider: String,
    pub summary_model: String,
    pub summary_language: String,
    pub template: String,
    pub custom_model: String,
    pub endpoint: String,
    pub live_preview: bool,
    pub save_audio: bool,
    pub notify: bool,
    pub sidebar_collapsed: bool,
}

impl AppPrefs {
    pub fn default_prefs() -> Self {
        Self {
            model: "Indonesia Turbo (terlatih, GPU)".to_string(),
            language: "Indonesia".to_string(),
            provider: "ollama".to_string(),
            summary_model: "gemma4:latest".to_string(),
            summary_language: "id".to_string(),
            template: "materi".to_string(),
            custom_model: String::new(),
            endpoint: String::new(),
            live_preview: true,
            save_audio: true,
            notify: true,
            sidebar_collapsed: false,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AudioDevice {
    #[serde(default)]
    pub index: i32,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DevicesResponse {
    #[serde(default)]
    pub speakers: Vec<AudioDevice>,
    #[serde(default)]
    pub microphones: Vec<AudioDevice>,
    #[serde(default)]
    pub default: Option<i32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FolderItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SessionSummaryItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "date")]
    pub when: String,
    #[serde(default)]
    pub folder: String,
    #[serde(default, rename = "audioBytes")]
    pub audio_bytes: u64,
    #[serde(default, rename = "sizeBytes")]
    pub size_bytes: u64,
    #[serde(default, rename = "hasSummary")]
    pub has_summary: bool,
    #[serde(default)]
    pub segments: usize,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TranscriptRow {
    #[serde(default)]
    pub start: f64,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SessionDetail {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "date")]
    pub when: String,
    #[serde(default)]
    pub folder: String,
    #[serde(default)]
    pub meta: Value,
    #[serde(default)]
    pub transcript: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub edited: bool,
    #[serde(default)]
    pub rows: Vec<TranscriptRow>,
    #[serde(default)]
    pub segments: Vec<TranscriptRow>,
    #[serde(default, rename = "audioBytes")]
    pub audio_bytes: u64,
    #[serde(default, rename = "sizeBytes")]
    pub size_bytes: u64,
}

impl SessionDetail {
    pub fn hydrate_from_meta(&mut self) {
        if self.title.is_empty() {
            if let Some(t) = self.meta.get("judul").and_then(|v| v.as_str()) {
                self.title = t.to_string();
            } else {
                self.title = "Rekaman".to_string();
            }
        }
        if self.when.is_empty() {
            if let Some(d) = self.meta.get("tanggal").and_then(|v| v.as_str()) {
                self.when = d.to_string();
            }
        }
        if self.folder.is_empty() {
            if let Some(f) = self.meta.get("folder").and_then(|v| v.as_str()) {
                self.folder = f.to_string();
            }
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeadlineItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub course: String,
    #[serde(default)]
    pub deadline: String,
    #[serde(default)]
    pub time: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub priority: String,
    #[serde(default = "default_reminder")]
    pub reminder: i32,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub gcal_event_id: Option<String>,
    #[serde(default)]
    pub gcal_link: Option<String>,
}

fn default_reminder() -> i32 {
    30
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GcalStatus {
    #[serde(default)]
    pub connected: bool,
    #[serde(default)]
    pub email: String,
    #[serde(default = "default_true", rename = "autoSync")]
    pub auto_sync: bool,
    #[serde(default, rename = "preferredMode")]
    pub preferred_mode: String,
    #[serde(default, rename = "hasCreds")]
    pub has_creds: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LocalModelInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub installed: bool,
    #[serde(default, rename = "sizeBytes")]
    pub size_bytes: Option<u64>,
    #[serde(default, rename = "estimatedBytes")]
    pub estimated_bytes: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct OllamaCatalogResponse {
    #[serde(default)]
    pub connected: bool,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub models: Vec<LocalModelInfo>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GpuResponse {
    #[serde(default)]
    pub report: String,
    #[serde(default)]
    pub nvidia: String,
    #[serde(default, rename = "cudaInstalled")]
    pub cuda_installed: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SpectrumSample {
    #[serde(default)]
    pub bands: Vec<f32>,
    #[serde(default, rename = "sampledAt")]
    pub sampled_at: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LivePreviewItem {
    #[serde(default)]
    pub start: f64,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct JobStatus {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub percent: Option<f32>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub progress: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StatusEvent {
    #[serde(default)]
    pub event: String,
    #[serde(default)]
    pub value: Value,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EngineStatus {
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub session: Option<String>,
    #[serde(default)]
    pub elapsed: Option<f64>,
    #[serde(default)]
    pub latest: u64,
    #[serde(default)]
    pub events: Vec<StatusEvent>,
    #[serde(default)]
    pub preview: Vec<LivePreviewItem>,
    #[serde(default)]
    pub spectra: Vec<SpectrumSample>,
    #[serde(default)]
    pub jobs: HashMap<String, JobStatus>,
}

#[derive(Debug, Clone)]
pub struct AttachedImage {
    pub name: String,
    pub data_url: String,
    pub rgba: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
    pub download_url: String,
    pub asset_name: String,
}

pub fn split_reminder(total_mins: i32) -> (i32, &'static str) {
    if total_mins <= 0 {
        return (0, "none");
    }
    if total_mins % 10080 == 0 {
        return (total_mins / 10080, "weeks");
    }
    if total_mins % 1440 == 0 {
        return (total_mins / 1440, "days");
    }
    if total_mins % 60 == 0 {
        return (total_mins / 60, "hours");
    }
    (total_mins, "minutes")
}

pub fn combine_reminder(amount: i32, unit: &str) -> i32 {
    if unit == "none" {
        return 0;
    }
    let n = amount.max(1);
    match unit {
        "hours" => n * 60,
        "days" => n * 1440,
        "weeks" => n * 10080,
        _ => n,
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 MB".to_string();
    }
    let mb = bytes as f64 / 1_000_000.0;
    if mb >= 1000.0 {
        format!("{:.2} GB", mb / 1000.0)
    } else {
        format!("{:.2} MB", mb)
    }
}

pub fn format_ts(sec: f64) -> String {
    let total = sec.max(0.0) as u64;
    format!("{:02}:{:02}", total / 60, total % 60)
}

pub fn format_when_pretty(raw: &str) -> String {
    let months = [
        "Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des",
    ];
    let s = raw.trim();
    if s.len() >= 16 && s.as_bytes()[4] == b'-' && s.as_bytes()[7] == b'-' {
        let year = &s[0..4];
        let mo: usize = s[5..7].parse().unwrap_or(1);
        let day: u32 = s[8..10].parse().unwrap_or(1);
        let hm = &s[11..16];
        let mon_lbl = months.get(mo.saturating_sub(1)).copied().unwrap_or("");
        return format!("{} {} {} · {}", day, mon_lbl, year, hm);
    }
    s.to_string()
}
