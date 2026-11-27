use crate::api::{open_external_url, open_folder_in_explorer, ApiClient, EngineBridge};
use crate::theme::{
    self, ACCENT, ACCENT_DEEP, ACCENT_SOFT, BORDER, CANVAS, DANGER, DANGER_HOVER, GREEN,
    GREEN_SOFT, INK, INK_2, INK_MUTED, SIDEBAR_BG, SURFACE, SURFACE_HOVER, SURFACE_SUBTLE,
};
use crate::types::{
    format_ts, split_reminder, AiStep, AppPrefs, AttachedImage, DeadlineFilter, DeadlineItem,
    DeadlineView, DevicesResponse, EngineStatus, FolderItem, GcalStatus, GpuResponse, NoteTab,
    OllamaCatalogResponse, Page, SessionDetail, SessionSummaryItem, SettingsTab, TranscriptRow,
    UpdateInfo, APP_VERSION,
};
use crate::updater::{self, UpdateDownloadEvent};
use crate::views;
use base64::Engine as _;
use chrono::{Datelike, Local};
use eframe::egui::{
    self, Align, Color32, ColorImage, CornerRadius, Frame, Layout, Margin, RichText, Sense, Shadow,
    Stroke, TextureHandle, TextureOptions, Vec2,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

const NOTELY_LOGO_PNG: &[u8] = include_bytes!("../../src-tauri/icons/notely-icon.png");

pub enum BgMsg {
    BootProgress {
        progress: f32,
        title: String,
        detail: String,
        hw_chip: Option<String>,
    },
    BootReady {
        bridge: EngineBridge,
        devices: DevicesResponse,
        gpu: GpuResponse,
        sessions: Vec<SessionSummaryItem>,
        folders: Vec<FolderItem>,
        deadlines: Vec<DeadlineItem>,
        gcal: GcalStatus,
        transcribe_models: Vec<String>,
        ollama_catalog: OllamaCatalogResponse,
    },
    BootFailed(String),
    StatusUpdate(EngineStatus),
    SessionsLoaded(Vec<SessionSummaryItem>),
    FoldersLoaded(Vec<FolderItem>),
    SessionOpened(SessionDetail),
    DevicesLoaded(DevicesResponse),
    GpuLoaded(GpuResponse),
    DeadlinesLoaded(Vec<DeadlineItem>),
    GcalStatusLoaded(GcalStatus),
    OllamaCatalogLoaded(OllamaCatalogResponse),
    UpdateAvailable {
        info: Option<UpdateInfo>,
        manual: bool,
    },
    UpdateDownload(UpdateDownloadEvent),
    Toast {
        msg: String,
        is_error: bool,
    },
    JobStarted {
        kind: &'static str,
        job_id: String,
    },
    GcalSyncDone(Result<String, String>),
    OpenUrl(String),
    ReloadAllData,
    RefreshActiveSession(String),
}

pub struct NotelyApp {
    // ── Bridge & Worker Channel ──
    bridge: Option<EngineBridge>,
    api: Option<ApiClient>,
    bg_tx: Sender<BgMsg>,
    bg_rx: Receiver<BgMsg>,

    // ── Splash & Window State ──
    pub(crate) in_splash: bool,
    pub(crate) splash_target_progress: f32,
    pub(crate) splash_visual_progress: f32,
    pub(crate) splash_step_title: String,
    pub(crate) splash_step_detail: String,
    pub(crate) splash_hw_chip: String,
    pub(crate) splash_error: Option<String>,
    splash_ready_at: Option<Instant>,
    pub(crate) logo_texture: Option<TextureHandle>,

    // ── Core Navigation & Preferences ──
    pub(crate) page: Page,
    pub(crate) note_tab: NoteTab,
    pub(crate) deadline_view: DeadlineView,
    pub(crate) deadline_filter: DeadlineFilter,
    pub(crate) settings_tab: SettingsTab,
    pub(crate) prefs: AppPrefs,
    pub(crate) data_root_display: String,

    // ── Live Engine State ──
    pub(crate) status: EngineStatus,
    last_event_id: u64,
    last_poll_at: Instant,
    poll_in_flight: bool,
    last_gcal_pull_at: Instant,
    pub(crate) live_segments: Vec<TranscriptRow>,
    pub(crate) spectrum_bands: Vec<f32>,

    // ── Backend Data ──
    pub(crate) devices: DevicesResponse,
    pub(crate) selected_speaker: i32,
    pub(crate) selected_mic: i32,
    pub(crate) gpu: GpuResponse,
    pub(crate) sessions: Vec<SessionSummaryItem>,
    pub(crate) folders: Vec<FolderItem>,
    pub(crate) active_session: Option<SessionDetail>,
    pub(crate) deadlines: Vec<DeadlineItem>,
    pub(crate) gcal: GcalStatus,
    pub(crate) transcribe_models: Vec<String>,
    pub(crate) ollama_catalog: OllamaCatalogResponse,

    // ── Notes & Summary State ──
    pub(crate) summary_prompt: String,
    pub(crate) summary_running: bool,
    pub(crate) summary_status_msg: String,
    pub(crate) summary_stream_text: String,
    active_summary_job: Option<String>,
    active_ai_extract_job: Option<String>,
    active_reprocess_job: Option<String>,
    pub(crate) show_edit_transcript_modal: bool,
    pub(crate) edit_transcript_buf: String,

    // ── Recordings Page State ──
    pub(crate) active_folder_id: Option<String>,
    pub(crate) recordings_search: String,
    pub(crate) show_folder_modal: bool,
    pub(crate) folder_modal_id: Option<String>,
    pub(crate) folder_modal_name: String,
    pub(crate) rename_session_id: Option<String>,
    pub(crate) rename_session_title: String,
    pub(crate) rename_session_folder: String,
    pub(crate) delete_session_target: Option<SessionSummaryItem>,

    // ── Deadlines & AI Extractor State ──
    pub(crate) cal_year: i32,
    pub(crate) cal_month: u32,
    pub(crate) gcal_syncing: bool,
    pub(crate) show_manual_deadline_modal: bool,
    pub(crate) manual_deadline: DeadlineItem,
    pub(crate) manual_rem_amount: String,
    pub(crate) manual_rem_unit: String,
    pub(crate) calendar_day_detail: Option<String>,
    pub(crate) delete_deadline_target: Option<DeadlineItem>,
    pub(crate) complete_deadline_target: Option<DeadlineItem>,

    pub(crate) show_ai_deadline_modal: bool,
    pub(crate) ai_step: AiStep,
    pub(crate) ai_prompt: String,
    pub(crate) ai_images: Vec<AttachedImage>,
    pub(crate) ai_image_textures: Vec<TextureHandle>,
    pub(crate) lightbox_image_idx: Option<usize>,
    pub(crate) ai_status_msg: String,
    pub(crate) ai_extracted: Vec<DeadlineItem>,
    pub(crate) ai_active_idx: usize,
    pub(crate) ai_rem_amount: String,
    pub(crate) ai_rem_unit: String,
    ai_extract_job_seen: bool,

    // ── Settings & Job Indicators ──
    pub(crate) audio_test_running: bool,
    pub(crate) audio_test_msg: String,
    pub(crate) model_prepare_msg: Option<String>,
    pub(crate) ollama_pull_msg: Option<String>,
    pub(crate) cloud_api_key_input: String,

    // ── Updater & Toast & Close Guard ──
    pub(crate) checking_update: bool,
    pub(crate) available_update: Option<UpdateInfo>,
    pub(crate) update_downloading: bool,
    pub(crate) update_progress: f32,
    pub(crate) update_status_text: String,
    toast_msg: Option<(String, bool, Instant)>,
    show_close_guard: bool,
    allow_close_now: bool,
}

impl NotelyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_notely_theme(&cc.egui_ctx);

        let logo_texture = load_texture_from_png(&cc.egui_ctx, "notely_brand_logo", NOTELY_LOGO_PNG);
        let prefs = load_local_prefs();
        let data_root = dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Notely");

        let today = Local::now().date_naive();
        let (bg_tx, bg_rx) = mpsc::channel();

        let mut app = Self {
            bridge: None,
            api: None,
            bg_tx,
            bg_rx,
            in_splash: true,
            splash_target_progress: 0.12,
            splash_visual_progress: 0.05,
            splash_step_title: "Menghubungkan mesin lokal...".to_string(),
            splash_step_detail: "Menyalakan layanan transkripsi & AI Notely".to_string(),
            splash_hw_chip: "Memeriksa perangkat audio & GPU...".to_string(),
            splash_error: None,
            splash_ready_at: None,
            logo_texture,
            page: Page::Home,
            note_tab: NoteTab::Transcript,
            deadline_view: DeadlineView::Calendar,
            deadline_filter: DeadlineFilter::All,
            settings_tab: SettingsTab::General,
            prefs,
            data_root_display: data_root.to_string_lossy().to_string(),
            status: EngineStatus::default(),
            last_event_id: 0,
            last_poll_at: Instant::now(),
            poll_in_flight: false,
            last_gcal_pull_at: Instant::now(),
            live_segments: Vec::new(),
            spectrum_bands: vec![0.0; 18],
            devices: DevicesResponse::default(),
            selected_speaker: -1,
            selected_mic: -1,
            gpu: GpuResponse::default(),
            sessions: Vec::new(),
            folders: Vec::new(),
            active_session: None,
            deadlines: Vec::new(),
            gcal: GcalStatus::default(),
            transcribe_models: Vec::new(),
            ollama_catalog: OllamaCatalogResponse::default(),
            summary_prompt: String::new(),
            summary_running: false,
            summary_status_msg: String::new(),
            summary_stream_text: String::new(),
            active_summary_job: None,
            active_ai_extract_job: None,
            active_reprocess_job: None,
            show_edit_transcript_modal: false,
            edit_transcript_buf: String::new(),
            active_folder_id: None,
            recordings_search: String::new(),
            show_folder_modal: false,
            folder_modal_id: None,
            folder_modal_name: String::new(),
            rename_session_id: None,
            rename_session_title: String::new(),
            rename_session_folder: String::new(),
            delete_session_target: None,
            cal_year: today.year(),
            cal_month: today.month(),
            gcal_syncing: false,
            show_manual_deadline_modal: false,
            manual_deadline: DeadlineItem::default(),
            manual_rem_amount: "30".to_string(),
            manual_rem_unit: "minutes".to_string(),
            calendar_day_detail: None,
            delete_deadline_target: None,
            complete_deadline_target: None,
            show_ai_deadline_modal: false,
            ai_step: AiStep::Input,
            ai_prompt: String::new(),
            ai_images: Vec::new(),
            ai_image_textures: Vec::new(),
            lightbox_image_idx: None,
            ai_status_msg: String::new(),
            ai_extracted: Vec::new(),
            ai_active_idx: 0,
            ai_rem_amount: "30".to_string(),
            ai_rem_unit: "minutes".to_string(),
            ai_extract_job_seen: false,
            audio_test_running: false,
            audio_test_msg: String::new(),
            model_prepare_msg: None,
            ollama_pull_msg: None,
            cloud_api_key_input: String::new(),
            checking_update: false,
            available_update: None,
            update_downloading: false,
            update_progress: 0.0,
            update_status_text: String::new(),
            toast_msg: None,
            show_close_guard: false,
            allow_close_now: false,
        };

        app.restart_boot_sequence();
        app
    }

    pub(crate) fn restart_boot_sequence(&mut self) {
        self.splash_error = None;
        self.splash_ready_at = None;
        self.splash_target_progress = 0.14;
        self.splash_step_title = "Menghubungkan mesin lokal...".to_string();
        self.splash_step_detail = "Menyalakan layanan transkripsi & AI Notely".to_string();
        self.splash_hw_chip = "Memeriksa perangkat audio & GPU...".to_string();

        let tx = self.bg_tx.clone();
        let saved_model = self.prefs.model.clone();

        std::thread::spawn(move || {
            let bridge = match EngineBridge::spawn() {
                Ok(b) => b,
                Err(e) => {
                    let _ = tx.send(BgMsg::BootFailed(e));
                    return;
                }
            };
            let api = bridge.client_handle();

            let _ = tx.send(BgMsg::BootProgress {
                progress: 0.30,
                title: "Menunggu server lokal siap...".to_string(),
                detail: "Memverifikasi koneksi aman lokal".to_string(),
                hw_chip: None,
            });

            // Wait up to 25 seconds for notely-engine.exe HTTP server
            let mut ready = false;
            for _ in 0..100 {
                if api.get_json::<EngineStatus>("/status?since=0").is_ok() {
                    ready = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            if !ready {
                let _ = tx.send(BgMsg::BootFailed(
                    "Mesin lokal membutuhkan waktu terlalu lama untuk merespons.".to_string(),
                ));
                return;
            }

            let _ = tx.send(BgMsg::BootProgress {
                progress: 0.52,
                title: "Memeriksa audio & akselerasi GPU...".to_string(),
                detail: "Mendeteksi speaker, mikrofon, dan dukungan CUDA".to_string(),
                hw_chip: None,
            });

            let devices: DevicesResponse = api.get_json("/devices").unwrap_or_default();
            let gpu: GpuResponse = api.get_json("/gpu").unwrap_or_default();
            let chip = if !gpu.nvidia.is_empty() {
                if gpu.cuda_installed {
                    format!("GPU siap: {} (CUDA)", gpu.nvidia)
                } else {
                    format!("GPU terdeteksi: {}", gpu.nvidia)
                }
            } else {
                "Mode CPU aktif · Siap digunakan".to_string()
            };

            let _ = tx.send(BgMsg::BootProgress {
                progress: 0.72,
                title: "Memuat catatan & jadwal Anda...".to_string(),
                detail: "Membaca rekaman tersimpan, folder, dan Google Calendar".to_string(),
                hw_chip: Some(chip),
            });

            let sessions: Vec<SessionSummaryItem> = api.get_json("/sessions").unwrap_or_default();
            let folders: Vec<FolderItem> = api.get_json("/folders").unwrap_or_default();
            let dl_val: Value = api.get_json("/deadlines").unwrap_or_else(|_| json!({}));
            let deadlines: Vec<DeadlineItem> = dl_val
                .get("deadlines")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let gcal: GcalStatus = api.get_json("/gcalendar/status").unwrap_or_default();
            let models_val: Value = api.get_json("/models").unwrap_or_else(|_| json!({}));
            let transcribe_models: Vec<String> = models_val
                .get("transcribe")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            let _ = tx.send(BgMsg::BootProgress {
                progress: 0.88,
                title: "Menyiapkan model AI & memeriksa pembaruan...".to_string(),
                detail: "Menyinkronkan katalog model lokal".to_string(),
                hw_chip: None,
            });

            // Trigger non-blocking prepare of default transcription model
            if !saved_model.is_empty() {
                let _ = api.post_json::<Value>(
                    "/model/prepare",
                    json!({ "model": saved_model, "gpu": "Otomatis" }),
                );
            }

            let ollama_catalog: OllamaCatalogResponse =
                api.get_json("/ollama/catalog").unwrap_or_default();

            // Background update check (does not block splash transition)
            let upd_tx = tx.clone();
            std::thread::spawn(move || {
                if let Ok(info) = updater::check_latest_release() {
                    if info.is_some() {
                        let _ = upd_tx.send(BgMsg::UpdateAvailable {
                            info,
                            manual: false,
                        });
                    }
                }
            });

            let _ = tx.send(BgMsg::BootProgress {
                progress: 1.0,
                title: "Siap digunakan".to_string(),
                detail: "Membuka ruang kerja Notely...".to_string(),
                hw_chip: None,
            });

            let _ = tx.send(BgMsg::BootReady {
                bridge,
                devices,
                gpu,
                sessions,
                folders,
                deadlines,
                gcal,
                transcribe_models,
                ollama_catalog,
            });
        });
    }

    pub(crate) fn finish_splash_transition(&mut self, ctx: &egui::Context) {
        if !self.in_splash {
            return;
        }
        self.in_splash = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(820.0, 560.0)));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(1240.0, 780.0)));
        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
    }

    pub(crate) fn is_recording(&self) -> bool {
        self.status.state == "recording" || self.status.state == "stopping"
    }

    pub(crate) fn toast(&mut self, msg: impl Into<String>, is_error: bool) {
        self.toast_msg = Some((msg.into(), is_error, Instant::now()));
    }

    pub(crate) fn save_local_prefs(&self) {
        let path = prefs_file_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(s) = serde_json::to_string_pretty(&self.prefs) {
            let _ = std::fs::write(path, s);
        }
    }

    pub(crate) fn open_data_folder(&self) {
        open_folder_in_explorer(Path::new(&self.data_root_display));
    }

    // ── Recording & Session Operations ──
    pub(crate) fn start_recording(&mut self) {
        let Some(api) = self.api.clone() else {
            self.toast("Mesin lokal belum siap.", true);
            return;
        };
        let tx = self.bg_tx.clone();
        let speaker = self.selected_speaker;
        let mic = if self.selected_mic >= 0 {
            Some(self.selected_mic)
        } else {
            None
        };
        let model = self.prefs.model.clone();
        let lang = self.prefs.language.clone();
        let save_audio = self.prefs.save_audio;
        let live_preview = self.prefs.live_preview;

        self.live_segments.clear();
        self.page = Page::Notes;
        self.note_tab = NoteTab::Transcript;

        std::thread::spawn(move || {
            let body = json!({
                "speaker": speaker,
                "microphone": mic,
                "model": model,
                "language": lang,
                "gpu": "Otomatis",
                "saveAudio": save_audio,
                "livePreview": live_preview,
            });
            match api.post_json::<Value>("/start", body) {
                Ok(_) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: "Rekaman dimulai.".to_string(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    pub(crate) fn stop_recording(&mut self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || match api.post_json::<Value>("/stop", json!({})) {
            Ok(_) => {
                let _ = tx.send(BgMsg::Toast {
                    msg: "Menghentikan rekaman & menyusun transkrip akhir...".to_string(),
                    is_error: false,
                });
            }
            Err(e) => {
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    pub(crate) fn open_session(&mut self, id: &str) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        let sid = id.to_string();
        self.page = Page::Notes;
        std::thread::spawn(move || {
            let path = format!("/session?id={}", url_encode(&sid));
            match api.get_json::<SessionDetail>(&path) {
                Ok(mut detail) => {
                    detail.hydrate_from_meta();
                    let _ = tx.send(BgMsg::SessionOpened(detail));
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    pub(crate) fn pick_and_import_audio(&mut self, folder_id: &str) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Audio / Video",
                &["wav", "mp3", "m4a", "flac", "ogg", "mp4", "mkv", "webm", "aac"],
            )
            .pick_file()
        else {
            return;
        };
        self.import_audio_path(path, folder_id);
    }

    fn import_audio_path(&mut self, path: PathBuf, folder_id: &str) {
        let Some(api) = self.api.clone() else {
            self.toast("Mesin lokal belum siap.", true);
            return;
        };
        let tx = self.bg_tx.clone();
        let fid = folder_id.to_string();
        let model = self.prefs.model.clone();
        self.toast("Mengimpor & mentranskripsi file audio...", false);
        std::thread::spawn(move || match api.post_raw_file(&path, &fid) {
            Ok(val) => {
                let _ = tx.send(BgMsg::ReloadAllData);
                if let Some(sid) = val
                    .get("session")
                    .or_else(|| val.get("id"))
                    .and_then(|v| v.as_str())
                {
                    let _ = tx.send(BgMsg::RefreshActiveSession(sid.to_string()));
                    // Trigger automatic transcription for the imported audio
                    if let Ok(jval) = api.post_json::<Value>(
                        "/reprocess",
                        json!({ "session": sid, "model": model, "gpu": "Otomatis" }),
                    ) {
                        if let Some(jid) = jval.get("job").and_then(|v| v.as_str()) {
                            let _ = tx.send(BgMsg::JobStarted {
                                kind: "transcribe",
                                job_id: jid.to_string(),
                            });
                        }
                    }
                }
                let _ = tx.send(BgMsg::Toast {
                    msg: "Audio berhasil diimpor. Transkripsi sedang diproses.".to_string(),
                    is_error: false,
                });
            }
            Err(e) => {
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    pub(crate) fn copy_active_note(&mut self) {
        let Some(sess) = &self.active_session else {
            return;
        };
        let text = match self.note_tab {
            NoteTab::Transcript => {
                if !sess.transcript.trim().is_empty() {
                    sess.transcript.clone()
                } else {
                    let rows = if !sess.rows.is_empty() {
                        &sess.rows
                    } else {
                        &sess.segments
                    };
                    rows.iter()
                        .map(|r| format!("[{}] {}", format_ts(r.start), r.text.trim()))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            NoteTab::Summary => sess.summary.clone(),
        };
        if text.trim().is_empty() {
            self.toast("Tidak ada teks untuk disalin.", true);
            return;
        }
        if let Ok(mut cb) = arboard::Clipboard::new() {
            if cb.set_text(text).is_ok() {
                self.toast("Teks disalin ke papan klip.", false);
                return;
            }
        }
        self.toast("Gagal menyalin ke papan klip.", true);
    }

    pub(crate) fn open_edit_transcript_modal(&mut self) {
        if let Some(sess) = &self.active_session {
            if !sess.transcript.trim().is_empty() {
                self.edit_transcript_buf = sess.transcript.clone();
            } else {
                let rows = if !sess.rows.is_empty() {
                    &sess.rows
                } else {
                    &sess.segments
                };
                self.edit_transcript_buf = rows
                    .iter()
                    .map(|r| r.text.trim())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
            }
            self.show_edit_transcript_modal = true;
        }
    }

    pub(crate) fn save_edited_transcript(&mut self) {
        let (Some(api), Some(sess)) = (self.api.clone(), self.active_session.clone()) else {
            return;
        };
        let tx = self.bg_tx.clone();
        let text = self.edit_transcript_buf.clone();
        let sid = sess.id.clone();
        self.show_edit_transcript_modal = false;

        std::thread::spawn(move || {
            match api.post_json::<Value>("/session/edit", json!({ "id": sid, "text": text })) {
                Ok(_) => {
                    let _ = tx.send(BgMsg::RefreshActiveSession(sid));
                    let _ = tx.send(BgMsg::Toast {
                        msg: "Perubahan transkrip disimpan.".to_string(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    pub(crate) fn reprocess_active_session(&mut self) {
        let (Some(api), Some(sess)) = (self.api.clone(), self.active_session.clone()) else {
            return;
        };
        let tx = self.bg_tx.clone();
        let body = json!({
            "session": sess.id,
            "model": self.prefs.model,
            "gpu": "Otomatis",
        });
        self.toast("Memproses ulang transkrip audio...", false);
        std::thread::spawn(move || match api.post_json::<Value>("/reprocess", body) {
            Ok(v) => {
                if let Some(jid) = v.get("job").and_then(|x| x.as_str()) {
                    let _ = tx.send(BgMsg::JobStarted {
                        kind: "transcribe",
                        job_id: jid.to_string(),
                    });
                }
            }
            Err(e) => {
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    pub(crate) fn run_summary(&mut self) {
        let (Some(api), Some(sess)) = (self.api.clone(), self.active_session.clone()) else {
            return;
        };
        let tx = self.bg_tx.clone();
        let model = if self.prefs.provider == "ollama" {
            self.prefs.summary_model.clone()
        } else {
            self.prefs.custom_model.clone()
        };
        let template_hint = match self.prefs.template.as_str() {
            "rapat" => "Format ringkasan rapat: keputusan utama, tindak lanjut, dan penanggung jawab.",
            "poin" => "Format poin-poin inti yang ringkas dan mudah dipindai.",
            "tanya" => "Format tanya jawab untuk persiapan evaluasi dan pemahaman konsep.",
            _ => "Format catatan materi terstruktur dengan konsep kunci dan contoh penting.",
        };
        let instructions = if self.summary_prompt.trim().is_empty() {
            template_hint.to_string()
        } else {
            format!("{} {}", template_hint, self.summary_prompt.trim())
        };
        let body = json!({
            "session": sess.id,
            "provider": self.prefs.provider,
            "model": model,
            "instructions": instructions,
            "context": "",
            "endpoint": self.prefs.endpoint,
        });
        self.summary_running = true;
        self.active_summary_job = None;
        self.summary_status_msg = "Memulai penyusunan ringkasan AI...".to_string();
        self.summary_stream_text.clear();

        std::thread::spawn(move || match api.post_json::<Value>("/summary", body) {
            Ok(v) => {
                if let Some(jid) = v.get("job").and_then(|x| x.as_str()) {
                    let _ = tx.send(BgMsg::JobStarted {
                        kind: "summary",
                        job_id: jid.to_string(),
                    });
                }
            }
            Err(e) => {
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    pub(crate) fn cancel_summary(&mut self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        self.summary_running = false;
        std::thread::spawn(move || {
            let _ = api.post_json::<Value>("/summary/cancel", json!({}));
        });
        self.toast("Pembuatan ringkasan dibatalkan.", false);
    }

    // ── Folder & Session CRUD ──
    pub(crate) fn submit_folder_modal(&mut self) {
        let name = self.folder_modal_name.trim().to_string();
        if name.is_empty() {
            self.toast("Nama folder tidak boleh kosong.", true);
            return;
        }
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        let edit_id = self.folder_modal_id.clone();
        self.show_folder_modal = false;

        std::thread::spawn(move || {
            let res = if let Some(id) = edit_id {
                api.post_json::<Value>("/folders/rename", json!({ "id": id, "name": name }))
            } else {
                api.post_json::<Value>("/folders/create", json!({ "name": name }))
            };
            match res {
                Ok(_) => {
                    let _ = tx.send(BgMsg::ReloadAllData);
                    let _ = tx.send(BgMsg::Toast {
                        msg: "Folder disimpan.".to_string(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    pub(crate) fn delete_folder(&mut self, id: &str) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        let fid = id.to_string();
        if self.active_folder_id.as_deref() == Some(id) {
            self.active_folder_id = None;
        }
        std::thread::spawn(move || {
            match api.post_json::<Value>("/folders/delete", json!({ "id": fid })) {
                Ok(_) => {
                    let _ = tx.send(BgMsg::ReloadAllData);
                    let _ = tx.send(BgMsg::Toast {
                        msg: "Folder dihapus.".to_string(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    pub(crate) fn submit_rename_session(&mut self) {
        let (Some(api), Some(id)) = (self.api.clone(), self.rename_session_id.take()) else {
            return;
        };
        let title = self.rename_session_title.trim().to_string();
        let folder = self.rename_session_folder.clone();
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || {
            match api.post_json::<Value>(
                "/session/update",
                json!({ "id": id, "title": title, "folder": folder }),
            ) {
                Ok(_) => {
                    let _ = tx.send(BgMsg::ReloadAllData);
                    let _ = tx.send(BgMsg::Toast {
                        msg: "Detail rekaman diperbarui.".to_string(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    pub(crate) fn confirm_delete_session(&mut self, id: &str, audio_only: bool) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        let sid = id.to_string();
        self.delete_session_target = None;
        if !audio_only && self.active_session.as_ref().map(|s| s.id.as_str()) == Some(id) {
            self.active_session = None;
        }
        std::thread::spawn(move || {
            match api.post_json::<Value>(
                "/session/delete",
                json!({ "id": sid, "audioOnly": audio_only }),
            ) {
                Ok(_) => {
                    let _ = tx.send(BgMsg::ReloadAllData);
                    let _ = tx.send(BgMsg::Toast {
                        msg: if audio_only {
                            "File audio berhasil dihapus.".to_string()
                        } else {
                            "Rekaman berhasil dihapus.".to_string()
                        },
                        is_error: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    // ── Deadlines & AI Extractor ──
    pub(crate) fn open_manual_deadline_modal(
        &mut self,
        existing: Option<DeadlineItem>,
        default_date: Option<String>,
    ) {
        let item = existing.unwrap_or_else(|| DeadlineItem {
            id: String::new(),
            title: String::new(),
            course: "Akademik".to_string(),
            deadline: default_date
                .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string()),
            time: "23:59".to_string(),
            description: String::new(),
            priority: "medium".to_string(),
            reminder: 30,
            status: "pending".to_string(),
            gcal_event_id: None,
            gcal_link: None,
        });
        let (amt, unit) = split_reminder(item.reminder);
        self.manual_rem_amount = if amt == 0 {
            "30".to_string()
        } else {
            amt.to_string()
        };
        self.manual_rem_unit = unit.to_string();
        self.manual_deadline = item;
        self.show_manual_deadline_modal = true;
    }

    pub(crate) fn submit_manual_deadline(&mut self) {
        if self.manual_deadline.title.trim().is_empty() {
            self.toast("Judul tugas wajib diisi.", true);
            return;
        }
        let Some(api) = self.api.clone() else {
            return;
        };
        let item = self.manual_deadline.clone();
        let tx = self.bg_tx.clone();
        self.show_manual_deadline_modal = false;

        std::thread::spawn(move || {
            match api.post_json::<Value>("/deadlines/save", json!({ "item": item })) {
                Ok(_) => {
                    let _ = tx.send(BgMsg::ReloadAllData);
                    let _ = tx.send(BgMsg::Toast {
                        msg: "Tenggat tugas berhasil disimpan.".to_string(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    pub(crate) fn toggle_deadline_status(&mut self, id: &str) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        let did = id.to_string();
        std::thread::spawn(move || {
            match api.post_json::<Value>("/deadlines/toggle", json!({ "id": did })) {
                Ok(_) => {
                    let _ = tx.send(BgMsg::ReloadAllData);
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    pub(crate) fn confirm_delete_deadline(&mut self, id: &str) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        let did = id.to_string();
        self.delete_deadline_target = None;
        std::thread::spawn(move || {
            match api.post_json::<Value>("/deadlines/delete", json!({ "id": did })) {
                Ok(_) => {
                    let _ = tx.send(BgMsg::ReloadAllData);
                    let _ = tx.send(BgMsg::Toast {
                        msg: "Tugas dihapus.".to_string(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    pub(crate) fn open_ai_deadline_modal(&mut self) {
        self.ai_step = AiStep::Input;
        self.ai_prompt.clear();
        self.clear_ai_images();
        self.ai_extracted.clear();
        self.ai_active_idx = 0;
        self.ai_status_msg.clear();
        self.ai_extract_job_seen = false;
        self.show_ai_deadline_modal = true;
    }

    pub(crate) fn clear_ai_images(&mut self) {
        self.ai_images.clear();
        self.ai_image_textures.clear();
        self.lightbox_image_idx = None;
    }

    pub(crate) fn remove_ai_image(&mut self, idx: usize) {
        if idx < self.ai_images.len() {
            self.ai_images.remove(idx);
        }
        if idx < self.ai_image_textures.len() {
            self.ai_image_textures.remove(idx);
        }
        if self.lightbox_image_idx == Some(idx) {
            self.lightbox_image_idx = None;
        }
    }

    pub(crate) fn pick_ai_images(&mut self, ctx: &egui::Context) {
        let Some(paths) = rfd::FileDialog::new()
            .add_filter("Gambar", &["png", "jpg", "jpeg", "webp"])
            .pick_files()
        else {
            return;
        };
        let mut added = 0usize;
        for p in paths {
            if let Ok(bytes) = std::fs::read(&p) {
                let name = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("gambar.png")
                    .to_string();
                if self.attach_image_bytes(ctx, name, &bytes) {
                    added += 1;
                }
            }
        }
        if added > 0 {
            self.toast(format!("{} gambar berhasil dilampirkan.", added), false);
        }
    }

    fn attach_image_bytes(&mut self, ctx: &egui::Context, name: String, bytes: &[u8]) -> bool {
        let Ok(dyn_img) = image::load_from_memory(bytes) else {
            return false;
        };
        let rgba_img = dyn_img.to_rgba8();
        let (w, h) = rgba_img.dimensions();
        let rgba = rgba_img.into_raw();

        // Re-encode as PNG for reliable backend OCR & multimodal vision data URL
        let mut png_bytes = Vec::new();
        {
            let mut cursor = std::io::Cursor::new(&mut png_bytes);
            if dyn_img.write_to(&mut cursor, image::ImageFormat::Png).is_err() {
                png_bytes = bytes.to_vec();
            }
        }
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
        let data_url = format!("data:image/png;base64,{}", b64);

        let color_img = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
        let tex_id = format!("ai_img_{}_{}", self.ai_images.len(), name);
        let tex = ctx.load_texture(tex_id, color_img, TextureOptions::LINEAR);

        self.ai_images.push(AttachedImage {
            name,
            data_url,
            rgba,
            width: w as usize,
            height: h as usize,
        });
        self.ai_image_textures.push(tex);
        true
    }

    fn try_paste_clipboard_image(&mut self, ctx: &egui::Context) {
        let Ok(mut cb) = arboard::Clipboard::new() else {
            return;
        };
        let Ok(img_data) = cb.get_image() else {
            return;
        };
        let w = img_data.width as u32;
        let h = img_data.height as u32;
        let raw = img_data.bytes.into_owned();
        if let Some(rgba_buf) = image::RgbaImage::from_raw(w, h, raw) {
            let dyn_img = image::DynamicImage::ImageRgba8(rgba_buf);
            let mut png_bytes = Vec::new();
            let mut cursor = std::io::Cursor::new(&mut png_bytes);
            if dyn_img.write_to(&mut cursor, image::ImageFormat::Png).is_ok() {
                let name = format!("clipboard_{}.png", self.ai_images.len() + 1);
                if self.attach_image_bytes(ctx, name, &png_bytes) {
                    self.toast("Gambar dari papan klip berhasil ditempel.", false);
                }
            }
        }
    }

    pub(crate) fn start_ai_deadline_extract(&mut self) {
        let Some(api) = self.api.clone() else {
            self.toast("Mesin lokal belum siap.", true);
            return;
        };
        let tx = self.bg_tx.clone();
        let prompt = self.ai_prompt.clone();
        let images: Vec<String> = self.ai_images.iter().map(|i| i.data_url.clone()).collect();
        let first_img = images.first().cloned().unwrap_or_default();
        let model = if self.prefs.provider == "ollama" {
            self.prefs.summary_model.clone()
        } else {
            self.prefs.custom_model.clone()
        };
        let body = json!({
            "prompt": prompt,
            "image": first_img,
            "images": images,
            "provider": self.prefs.provider,
            "model": model,
            "endpoint": self.prefs.endpoint,
        });

        self.ai_step = AiStep::Loading;
        self.ai_status_msg = "Menganalisis teks & gambar tugas...".to_string();
        self.ai_extract_job_seen = false;
        self.active_ai_extract_job = None;

        std::thread::spawn(move || match api.post_json::<Value>("/deadlines/extract", body) {
            Ok(v) => {
                if let Some(jid) = v.get("job").and_then(|x| x.as_str()) {
                    let _ = tx.send(BgMsg::JobStarted {
                        kind: "deadline_extract",
                        job_id: jid.to_string(),
                    });
                }
            }
            Err(e) => {
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    pub(crate) fn select_ai_result_tab(&mut self, idx: usize) {
        if let Some(item) = self.ai_extracted.get(idx) {
            self.ai_active_idx = idx;
            let (amt, unit) = split_reminder(item.reminder);
            self.ai_rem_amount = if amt == 0 {
                "30".to_string()
            } else {
                amt.to_string()
            };
            self.ai_rem_unit = unit.to_string();
        }
    }

    pub(crate) fn save_ai_extracted_deadlines(&mut self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let items = self.ai_extracted.clone();
        if items.is_empty() {
            return;
        }
        let tx = self.bg_tx.clone();
        self.show_ai_deadline_modal = false;
        self.toast("Menyimpan jadwal hasil ekstraksi...", false);

        std::thread::spawn(move || {
            match api.post_json::<Value>("/deadlines/batch", json!({ "items": items })) {
                Ok(_) => {
                    let _ = tx.send(BgMsg::ReloadAllData);
                    let _ = tx.send(BgMsg::Toast {
                        msg: "Semua jadwal berhasil disimpan.".to_string(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    // ── Google Calendar Actions ──
    pub(crate) fn connect_google_calendar(&mut self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || match api.post_json::<Value>("/gcalendar/auth-url", json!({})) {
            Ok(v) => {
                if let Some(url) = v.get("authUrl").and_then(|u| u.as_str()) {
                    let _ = tx.send(BgMsg::OpenUrl(url.to_string()));
                    let _ = tx.send(BgMsg::Toast {
                        msg: "Selesaikan izin Google Calendar di peramban Anda.".to_string(),
                        is_error: false,
                    });
                }
            }
            Err(e) => {
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    pub(crate) fn import_gcal_json(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        else {
            return;
        };
        let Ok(raw) = std::fs::read_to_string(&path) else {
            self.toast("Gagal membaca file JSON.", true);
            return;
        };
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || {
            match api.post_json::<Value>("/gcalendar/import-json", json!({ "json": raw })) {
                Ok(_) => {
                    let _ = tx.send(BgMsg::ReloadAllData);
                    let _ = tx.send(BgMsg::Toast {
                        msg: "Kredensial Google Calendar berhasil diimpor.".to_string(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    pub(crate) fn set_gcal_auto_sync(&mut self, enabled: bool) {
        self.gcal.auto_sync = enabled;
        let Some(api) = self.api.clone() else {
            return;
        };
        std::thread::spawn(move || {
            let _ = api.post_json::<Value>("/gcalendar/config", json!({ "autoSync": enabled }));
        });
    }

    pub(crate) fn disconnect_google_calendar(&mut self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || {
            let _ = api.post_json::<Value>("/gcalendar/disconnect", json!({}));
            let _ = tx.send(BgMsg::ReloadAllData);
            let _ = tx.send(BgMsg::Toast {
                msg: "Koneksi Google Calendar diputuskan.".to_string(),
                is_error: false,
            });
        });
    }

    pub(crate) fn sync_google_calendar(&mut self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        self.gcal_syncing = true;
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || {
            let _ = api.post_json::<Value>("/gcalendar/pull", json!({}));
            match api.post_json::<Value>("/gcalendar/sync-all", json!({})) {
                Ok(v) => {
                    let synced = v.get("synced").and_then(|x| x.as_u64()).unwrap_or(0);
                    let _ = tx.send(BgMsg::GcalSyncDone(Ok(format!(
                        "Sinkronisasi selesai ({} jadwal diperbarui).",
                        synced
                    ))));
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::GcalSyncDone(Err(e)));
                }
            }
        });
    }

    // ── Settings: Audio, Models, Ollama, Cloud, Updater ──
    pub(crate) fn refresh_devices(&mut self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || {
            if let Ok(d) = api.get_json::<DevicesResponse>("/devices") {
                let _ = tx.send(BgMsg::DevicesLoaded(d));
            }
        });
    }

    pub(crate) fn run_audio_test(&mut self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        self.audio_test_running = true;
        self.audio_test_msg = "Memeriksa suara selama 5 detik...".to_string();
        let spk = self.selected_speaker;
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || {
            if let Err(e) = api.post_json::<Value>("/test/audio", json!({ "speaker": spk })) {
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    pub(crate) fn install_cuda(&mut self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        self.toast("Memulai pemasangan akselerasi CUDA...", false);
        std::thread::spawn(move || {
            if let Err(e) = api.post_json::<Value>("/cuda/prepare", json!({})) {
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    pub(crate) fn select_transcription_model(&mut self, name: &str) {
        self.prefs.model = name.to_string();
        self.save_local_prefs();
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        let model = name.to_string();
        self.model_prepare_msg = Some(format!("Menyiapkan model {}...", model));
        std::thread::spawn(move || {
            if let Err(e) = api.post_json::<Value>(
                "/model/prepare",
                json!({ "model": model, "gpu": "Otomatis" }),
            ) {
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    pub(crate) fn refresh_ollama_catalog(&mut self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || {
            if let Ok(cat) = api.get_json::<OllamaCatalogResponse>("/ollama/catalog") {
                let _ = tx.send(BgMsg::OllamaCatalogLoaded(cat));
            }
        });
    }

    pub(crate) fn pull_ollama_model(&mut self, id: &str) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        let mid = id.to_string();
        self.ollama_pull_msg = Some(format!("Mengunduh model {}...", mid));
        std::thread::spawn(move || {
            if let Err(e) = api.post_json::<Value>("/ollama/pull", json!({ "model": mid })) {
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    pub(crate) fn delete_ollama_model(&mut self, id: &str) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        let mid = id.to_string();
        std::thread::spawn(move || {
            match api.post_json::<Value>("/ollama/delete", json!({ "model": mid })) {
                Ok(_) => {
                    if let Ok(cat) = api.get_json::<OllamaCatalogResponse>("/ollama/catalog") {
                        let _ = tx.send(BgMsg::OllamaCatalogLoaded(cat));
                    }
                    let _ = tx.send(BgMsg::Toast {
                        msg: "Model lokal dihapus.".to_string(),
                        is_error: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::Toast {
                        msg: e,
                        is_error: true,
                    });
                }
            }
        });
    }

    pub(crate) fn pick_model_storage_folder(&mut self) {
        let Some(folder) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        let fstr = folder.to_string_lossy().to_string();
        self.toast("Memindahkan penyimpanan model AI...", false);
        std::thread::spawn(move || {
            if let Err(e) = api.post_json::<Value>("/storage/select", json!({ "folder": fstr })) {
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    pub(crate) fn save_cloud_config(&mut self) {
        self.save_local_prefs();
        let Some(api) = self.api.clone() else {
            return;
        };
        let prov = self.prefs.provider.clone();
        let key = self.cloud_api_key_input.trim().to_string();
        let tx = self.bg_tx.clone();
        if !key.is_empty() {
            std::thread::spawn(move || {
                match api.post_json::<Value>(
                    "/credentials/save",
                    json!({ "provider": prov, "apiKey": key }),
                ) {
                    Ok(_) => {
                        let _ = tx.send(BgMsg::Toast {
                            msg: "Konfigurasi & API Key tersimpan aman (DPAPI).".to_string(),
                            is_error: false,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(BgMsg::Toast {
                            msg: e,
                            is_error: true,
                        });
                    }
                }
            });
        } else {
            self.toast("Konfigurasi penyedia AI disimpan.", false);
        }
    }

    pub(crate) fn test_cloud_connection(&mut self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        let body = json!({
            "provider": self.prefs.provider,
            "model": self.prefs.custom_model,
            "apiKey": self.cloud_api_key_input,
            "endpoint": self.prefs.endpoint,
        });
        self.toast("Menguji koneksi penyedia AI...", false);
        std::thread::spawn(move || match api.post_json::<Value>("/provider/test", body) {
            Ok(_) => {
                let _ = tx.send(BgMsg::Toast {
                    msg: "Koneksi penyedia AI berhasil.".to_string(),
                    is_error: false,
                });
            }
            Err(e) => {
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    pub(crate) fn check_updates_manual(&mut self) {
        self.checking_update = true;
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || match updater::check_latest_release() {
            Ok(info) => {
                let _ = tx.send(BgMsg::UpdateAvailable { info, manual: true });
            }
            Err(e) => {
                let _ = tx.send(BgMsg::UpdateAvailable {
                    info: None,
                    manual: false,
                });
                let _ = tx.send(BgMsg::Toast {
                    msg: e,
                    is_error: true,
                });
            }
        });
    }

    fn start_update_download(&mut self) {
        let Some(info) = self.available_update.clone() else {
            return;
        };
        self.update_downloading = true;
        self.update_progress = 0.0;
        self.update_status_text = format!("Mengunduh {}...", info.asset_name);

        let (utx, urx) = mpsc::channel();
        updater::download_installer(info, utx);
        let bg_tx = self.bg_tx.clone();
        std::thread::spawn(move || {
            while let Ok(ev) = urx.recv() {
                let done = matches!(
                    ev,
                    UpdateDownloadEvent::ReadyToInstall(_) | UpdateDownloadEvent::Error(_)
                );
                let _ = bg_tx.send(BgMsg::UpdateDownload(ev));
                if done {
                    break;
                }
            }
        });
    }

    fn reload_all_data(&self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || {
            if let Ok(s) = api.get_json::<Vec<SessionSummaryItem>>("/sessions") {
                let _ = tx.send(BgMsg::SessionsLoaded(s));
            }
            if let Ok(f) = api.get_json::<Vec<FolderItem>>("/folders") {
                let _ = tx.send(BgMsg::FoldersLoaded(f));
            }
            if let Ok(v) = api.get_json::<Value>("/deadlines") {
                if let Some(d) = v
                    .get("deadlines")
                    .and_then(|x| serde_json::from_value::<Vec<DeadlineItem>>(x.clone()).ok())
                {
                    let _ = tx.send(BgMsg::DeadlinesLoaded(d));
                }
            }
            if let Ok(g) = api.get_json::<GcalStatus>("/gcalendar/status") {
                let _ = tx.send(BgMsg::GcalStatusLoaded(g));
            }
            if let Ok(gpu) = api.get_json::<GpuResponse>("/gpu") {
                let _ = tx.send(BgMsg::GpuLoaded(gpu));
            }
        });
    }

    // ── Background Message Pump & Polling ──
    fn process_bg_messages(&mut self, ctx: &egui::Context) {
        let mut trigger_reload = false;
        let mut refresh_session_id: Option<String> = None;

        while let Ok(msg) = self.bg_rx.try_recv() {
            match msg {
                BgMsg::BootProgress {
                    progress,
                    title,
                    detail,
                    hw_chip,
                } => {
                    self.splash_target_progress = progress;
                    self.splash_step_title = title;
                    self.splash_step_detail = detail;
                    if let Some(c) = hw_chip {
                        self.splash_hw_chip = c;
                    }
                }
                BgMsg::BootReady {
                    bridge,
                    devices,
                    gpu,
                    sessions,
                    folders,
                    deadlines,
                    gcal,
                    transcribe_models,
                    ollama_catalog,
                } => {
                    self.api = Some(bridge.client_handle());
                    self.data_root_display = bridge.data_root.to_string_lossy().to_string();
                    self.bridge = Some(bridge);
                    self.selected_speaker = devices
                        .default
                        .or_else(|| devices.speakers.first().map(|s| s.index))
                        .unwrap_or(-1);
                    self.devices = devices;
                    self.gpu = gpu;
                    self.sessions = sessions;
                    self.folders = folders;
                    self.deadlines = deadlines;
                    self.gcal = gcal;
                    self.transcribe_models = transcribe_models;
                    self.ollama_catalog = ollama_catalog;
                    self.splash_target_progress = 1.0;
                    self.splash_ready_at = Some(Instant::now());
                }
                BgMsg::BootFailed(err) => {
                    self.splash_error = Some(err);
                    self.splash_step_title = "Gagal memulai mesin lokal".to_string();
                    self.splash_step_detail =
                        "Anda dapat mencoba lagi atau melanjutkan ke Beranda.".to_string();
                }
                BgMsg::StatusUpdate(st) => {
                    self.poll_in_flight = false;
                    self.handle_engine_status(st);
                }
                BgMsg::SessionsLoaded(s) => self.sessions = s,
                BgMsg::FoldersLoaded(f) => self.folders = f,
                BgMsg::SessionOpened(detail) => self.active_session = Some(detail),
                BgMsg::DevicesLoaded(d) => {
                    if self.selected_speaker < 0 {
                        self.selected_speaker = d
                            .default
                            .or_else(|| d.speakers.first().map(|s| s.index))
                            .unwrap_or(-1);
                    }
                    self.devices = d;
                }
                BgMsg::GpuLoaded(g) => self.gpu = g,
                BgMsg::DeadlinesLoaded(d) => self.deadlines = d,
                BgMsg::GcalStatusLoaded(g) => self.gcal = g,
                BgMsg::OllamaCatalogLoaded(cat) => self.ollama_catalog = cat,
                BgMsg::UpdateAvailable { info, manual } => {
                    self.checking_update = false;
                    if let Some(upd) = info {
                        self.available_update = Some(upd);
                    } else if manual {
                        self.toast(
                            format!("Notely v{} sudah versi terbaru.", APP_VERSION),
                            false,
                        );
                    }
                }
                BgMsg::UpdateDownload(ev) => match ev {
                    UpdateDownloadEvent::Progress { downloaded, total } => {
                        if total > 0 {
                            self.update_progress = (downloaded as f32 / total as f32).clamp(0.0, 1.0);
                            self.update_status_text = format!(
                                "Mengunduh... {:.1} MB / {:.1} MB",
                                downloaded as f64 / 1_000_000.0,
                                total as f64 / 1_000_000.0
                            );
                        }
                    }
                    UpdateDownloadEvent::ReadyToInstall(path) => {
                        self.update_downloading = false;
                        self.update_progress = 1.0;
                        if let Err(e) = updater::launch_installer(&path) {
                            self.toast(e, true);
                        } else {
                            if let Some(b) = &mut self.bridge {
                                b.stop();
                            }
                            self.allow_close_now = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                    UpdateDownloadEvent::Error(e) => {
                        self.update_downloading = false;
                        self.toast(e, true);
                    }
                },
                BgMsg::Toast { msg, is_error } => self.toast(msg, is_error),
                BgMsg::JobStarted { kind, job_id } => match kind {
                    "summary" => self.active_summary_job = Some(job_id),
                    "deadline_extract" => self.active_ai_extract_job = Some(job_id),
                    "transcribe" => self.active_reprocess_job = Some(job_id),
                    _ => {}
                },
                BgMsg::GcalSyncDone(res) => {
                    self.gcal_syncing = false;
                    match res {
                        Ok(msg) => {
                            self.toast(msg, false);
                            trigger_reload = true;
                        }
                        Err(e) => self.toast(e, true),
                    }
                }
                BgMsg::OpenUrl(url) => open_external_url(&url),
                BgMsg::ReloadAllData => trigger_reload = true,
                BgMsg::RefreshActiveSession(sid) => refresh_session_id = Some(sid),
            }
        }

        if trigger_reload {
            self.reload_all_data();
        }
        if let Some(sid) = refresh_session_id {
            self.open_session(&sid);
        }

        // Smooth transition from Splash -> Main App once visual bar reaches ~98%
        if self.in_splash {
            if let Some(t) = self.splash_ready_at {
                if t.elapsed() >= Duration::from_millis(420) && self.splash_visual_progress >= 0.96 {
                    self.finish_splash_transition(ctx);
                }
            }
        }
    }

    fn handle_engine_status(&mut self, st: EngineStatus) {
        self.last_event_id = st.latest.max(self.last_event_id);

        // Update animated spectrum bars
        if let Some(sample) = st.spectra.last() {
            if !sample.bands.is_empty() {
                self.spectrum_bands = sample.bands.clone();
            }
        } else if st.state == "idle" {
            for b in &mut self.spectrum_bands {
                *b *= 0.7;
            }
        }

        // Process discrete engine events
        for ev in &st.events {
            match ev.event.as_str() {
                "transcript" => {
                    if let Some(arr) = ev.value.as_array() {
                        for item in arr {
                            let start = item
                                .get("start")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);
                            let text = item
                                .get("text")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            if !text.trim().is_empty() {
                                self.live_segments.push(TranscriptRow { start, text });
                            }
                        }
                    }
                }
                "done" => {
                    self.reload_all_data();
                    if let Some(sid) = st
                        .session
                        .as_deref()
                        .or_else(|| ev.value.get("session").and_then(|v| v.as_str()))
                        .or_else(|| ev.value.get("id").and_then(|v| v.as_str()))
                    {
                        self.open_session(sid);
                    }
                    self.toast("Rekaman & transkrip selesai disimpan.", false);
                }
                "error" => {
                    if let Some(m) = ev.value.as_str() {
                        self.toast(m, true);
                    }
                }
                _ => {}
            }
        }

        // Process tracked background jobs safely by job_id
        if let Some(jid) = self.active_summary_job.clone() {
            if let Some(job) = st.jobs.get(&jid) {
                if job.status == "running" {
                    self.summary_running = true;
                    if !job.message.is_empty() {
                        self.summary_status_msg = job.message.clone();
                    }
                    if let Some(p) = &job.progress {
                        if let Some(chunk) = p.get("text").and_then(|v| v.as_str()) {
                            self.summary_stream_text = chunk.to_string();
                        }
                    }
                } else if job.status == "done" {
                    self.summary_running = false;
                    self.active_summary_job = None;
                    self.summary_stream_text.clear();
                    if let Some(sess) = &self.active_session {
                        let sid = sess.id.clone();
                        self.open_session(&sid);
                    }
                    self.reload_all_data();
                    self.toast("Ringkasan AI selesai dibuat.", false);
                } else if job.status == "error" || job.status == "cancelled" {
                    self.summary_running = false;
                    self.active_summary_job = None;
                    if job.status == "error" {
                        self.toast(job.message.clone(), true);
                    }
                }
            }
        }

        if let Some(jid) = self.active_ai_extract_job.clone() {
            if let Some(job) = st.jobs.get(&jid) {
                if self.show_ai_deadline_modal && self.ai_step == AiStep::Loading {
                    if job.status == "running" {
                        self.ai_extract_job_seen = true;
                        if !job.message.is_empty() {
                            self.ai_status_msg = job.message.clone();
                        }
                    } else if job.status == "done" {
                        self.active_ai_extract_job = None;
                        let parsed = parse_extracted_deadlines(job.result.as_ref());
                        self.ai_extracted = parsed;
                        self.ai_step = AiStep::Results;
                        self.select_ai_result_tab(0);
                    } else if job.status == "error" {
                        self.active_ai_extract_job = None;
                        self.ai_step = AiStep::Input;
                        self.toast(
                            if job.message.is_empty() {
                                "Ekstraksi AI gagal.".to_string()
                            } else {
                                job.message.clone()
                            },
                            true,
                        );
                    }
                }
            }
        }

        if let Some(jid) = self.active_reprocess_job.clone() {
            if let Some(job) = st.jobs.get(&jid) {
                if job.status == "done" {
                    self.active_reprocess_job = None;
                    if let Some(sess) = &self.active_session {
                        let sid = sess.id.clone();
                        self.open_session(&sid);
                    }
                    self.reload_all_data();
                    self.toast("Transkripsi audio selesai.", false);
                } else if job.status == "error" {
                    self.active_reprocess_job = None;
                    self.toast(job.message.clone(), true);
                }
            }
        }

        // Non-tracked status jobs (audio_test, model, ollama)
        for job in st.jobs.values() {
            match job.kind.as_str() {
                "audio_test" => {
                    if job.status == "running" {
                        self.audio_test_running = true;
                        self.audio_test_msg = job.message.clone();
                    } else if self.audio_test_running && job.status == "done" {
                        self.audio_test_running = false;
                        if let Some(m) = job
                            .result
                            .as_ref()
                            .and_then(|r| r.get("message"))
                            .and_then(|v| v.as_str())
                        {
                            self.audio_test_msg = m.to_string();
                        }
                    } else if self.audio_test_running && job.status == "error" {
                        self.audio_test_running = false;
                        self.audio_test_msg = job.message.clone();
                    }
                }
                "model" => {
                    if job.status == "running" {
                        let pct = job
                            .percent
                            .map(|p| format!(" ({:.0}%)", p))
                            .unwrap_or_default();
                        self.model_prepare_msg = Some(format!("{}{}", job.message, pct));
                    } else if self.model_prepare_msg.is_some() && job.status == "done" {
                        self.model_prepare_msg = None;
                        self.toast("Model transkripsi siap digunakan.", false);
                    } else if self.model_prepare_msg.is_some() && job.status == "error" {
                        self.model_prepare_msg = None;
                        self.toast(job.message.clone(), true);
                    }
                }
                "ollama" => {
                    if job.status == "running" {
                        let pct = job
                            .percent
                            .map(|p| format!(" ({:.0}%)", p))
                            .unwrap_or_default();
                        self.ollama_pull_msg = Some(format!("{}{}", job.message, pct));
                    } else if self.ollama_pull_msg.is_some() && job.status == "done" {
                        self.ollama_pull_msg = None;
                        self.refresh_ollama_catalog();
                        self.toast("Model AI lokal selesai diunduh.", false);
                    } else if self.ollama_pull_msg.is_some() && job.status == "error" {
                        self.ollama_pull_msg = None;
                        self.toast(job.message.clone(), true);
                    }
                }
                _ => {}
            }
        }

        self.status = st;
    }

    fn schedule_polls(&mut self) {
        let Some(api) = self.api.clone() else {
            return;
        };
        let interval = if self.is_recording() || self.summary_running || self.ai_step == AiStep::Loading
        {
            Duration::from_millis(180)
        } else {
            Duration::from_millis(550)
        };

        if !self.poll_in_flight && self.last_poll_at.elapsed() >= interval {
            self.poll_in_flight = true;
            self.last_poll_at = Instant::now();
            let since = self.last_event_id;
            let api_poll = api.clone();
            let tx = self.bg_tx.clone();
            std::thread::spawn(move || {
                let path = format!("/status?since={}", since);
                if let Ok(st) = api_poll.get_json::<EngineStatus>(&path) {
                    let _ = tx.send(BgMsg::StatusUpdate(st));
                } else {
                    let _ = tx.send(BgMsg::StatusUpdate(EngineStatus::default()));
                }
            });
        }

        // Background 2-way Google Calendar pull every 25s when connected
        if self.gcal.connected && self.last_gcal_pull_at.elapsed() >= Duration::from_secs(25) {
            self.last_gcal_pull_at = Instant::now();
            let api2 = api.clone();
            let tx = self.bg_tx.clone();
            std::thread::spawn(move || {
                if api2.post_json::<Value>("/gcalendar/pull", json!({})).is_ok() {
                    if let Ok(v) = api2.get_json::<Value>("/deadlines") {
                        if let Some(d) = v.get("deadlines").and_then(|x| {
                            serde_json::from_value::<Vec<DeadlineItem>>(x.clone()).ok()
                        }) {
                            let _ = tx.send(BgMsg::DeadlinesLoaded(d));
                        }
                    }
                }
            });
        }
    }

    // ── Sidebar Chrome (NO redundant "Riwayat Rekaman", smooth rail animation) ──
    fn draw_sidebar(&mut self, ctx: &egui::Context) {
        let expanded_t = ctx.animate_bool(
            egui::Id::new("notely_sidebar_anim"),
            !self.prefs.sidebar_collapsed,
        );
        let width = egui::lerp(66.0..=222.0, expanded_t);
        let show_labels = expanded_t > 0.45;

        egui::SidePanel::left("notely_sidebar")
            .resizable(false)
            .exact_width(width)
            .frame(
                Frame::new()
                    .fill(SIDEBAR_BG)
                    .stroke(Stroke::new(1.0, BORDER))
                    .inner_margin(Margin::symmetric(10, 14)),
            )
            .show(ctx, |ui| {
                // Brand mark row
                ui.horizontal(|ui| {
                    if !show_labels {
                        ui.add_space(4.0);
                    }
                    if let Some(tex) = &self.logo_texture {
                        let sized = egui::load::SizedTexture::new(tex.id(), tex.size_vec2());
                        ui.add(
                            egui::Image::from_texture(sized)
                                .fit_to_exact_size(Vec2::splat(34.0))
                                .corner_radius(CornerRadius::same(9)),
                        );
                    } else {
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::hover());
                        ui.painter().rect_filled(r, CornerRadius::same(9), ACCENT);
                    }
                    if show_labels {
                        ui.add_space(4.0);
                        ui.label(RichText::new("Notely").size(17.5).color(INK).strong());
                    }
                });

                ui.add_space(20.0);

                // Main Nav Items (Beranda, Catatan Rekaman, Semua Rekaman, Tenggat Tugas)
                let nav_items = [
                    (Page::Home, theme::NavIcon::Home, "Beranda"),
                    (Page::Notes, theme::NavIcon::Notes, "Catatan Rekaman"),
                    (Page::Recordings, theme::NavIcon::Recordings, "Semua Rekaman"),
                    (Page::Deadlines, theme::NavIcon::Deadlines, "Tenggat Tugas"),
                ];

                for (target, icon, label) in nav_items {
                    let active = self.page == target;
                    let row_w = ui.available_width();
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(row_w, 38.0), Sense::click());
                    if resp.hovered() {
                        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    let bg = if active {
                        ACCENT_SOFT
                    } else if resp.hovered() {
                        SURFACE_HOVER
                    } else {
                        Color32::TRANSPARENT
                    };
                    ui.painter().rect_filled(rect, CornerRadius::same(9), bg);
                    if active {
                        let bar = egui::Rect::from_min_size(
                            egui::pos2(rect.left() + 2.0, rect.top() + 8.0),
                            Vec2::new(3.0, rect.height() - 16.0),
                        );
                        ui.painter().rect_filled(bar, CornerRadius::same(2), ACCENT);
                    }

                    let icon_col = if active { ACCENT_DEEP } else { INK_2 };
                    let icon_rect = if show_labels {
                        egui::Rect::from_center_size(
                            egui::pos2(rect.left() + 20.0, rect.center().y),
                            Vec2::splat(17.0),
                        )
                    } else {
                        egui::Rect::from_center_size(rect.center(), Vec2::splat(18.0))
                    };
                    theme::draw_nav_icon(ui.painter(), icon_rect, icon, icon_col);

                    if show_labels {
                        ui.painter().text(
                            egui::pos2(rect.left() + 38.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            label,
                            egui::FontId::proportional(13.0),
                            if active { ACCENT_DEEP } else { INK },
                        );
                    } else {
                        resp.clone().on_hover_text(label);
                    }

                    if resp.clicked() {
                        self.page = target;
                    }
                    ui.add_space(4.0);
                }

                // Bottom Pinned Controls (Start/Stop Record, Import Audio, Settings)
                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    // Settings Button
                    let settings_active = self.page == Page::Settings;
                    let (s_rect, s_resp) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 38.0),
                        Sense::click(),
                    );
                    if s_resp.hovered() {
                        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    let s_bg = if settings_active {
                        ACCENT_SOFT
                    } else if s_resp.hovered() {
                        SURFACE_HOVER
                    } else {
                        Color32::TRANSPARENT
                    };
                    ui.painter().rect_filled(s_rect, CornerRadius::same(9), s_bg);
                    let s_icon = if show_labels {
                        egui::Rect::from_center_size(
                            egui::pos2(s_rect.left() + 20.0, s_rect.center().y),
                            Vec2::splat(17.0),
                        )
                    } else {
                        egui::Rect::from_center_size(s_rect.center(), Vec2::splat(18.0))
                    };
                    theme::draw_nav_icon(
                        ui.painter(),
                        s_icon,
                        theme::NavIcon::Settings,
                        if settings_active { ACCENT_DEEP } else { INK_2 },
                    );
                    if show_labels {
                        ui.painter().text(
                            egui::pos2(s_rect.left() + 38.0, s_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Pengaturan",
                            egui::FontId::proportional(13.0),
                            if settings_active { ACCENT_DEEP } else { INK },
                        );
                    } else {
                        s_resp.clone().on_hover_text("Pengaturan");
                    }
                    if s_resp.clicked() {
                        self.page = Page::Settings;
                    }

                    ui.add_space(6.0);

                    // Import Audio Button
                    let (i_rect, i_resp) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 38.0),
                        Sense::click(),
                    );
                    if i_resp.hovered() {
                        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    ui.painter().rect_filled(
                        i_rect,
                        CornerRadius::same(9),
                        if i_resp.hovered() { SURFACE_HOVER } else { SURFACE },
                    );
                    ui.painter().rect_stroke(
                        i_rect,
                        CornerRadius::same(9),
                        Stroke::new(1.0, BORDER),
                        egui::StrokeKind::Inside,
                    );
                    let i_icon = if show_labels {
                        egui::Rect::from_center_size(
                            egui::pos2(i_rect.left() + 20.0, i_rect.center().y),
                            Vec2::splat(16.0),
                        )
                    } else {
                        egui::Rect::from_center_size(i_rect.center(), Vec2::splat(17.0))
                    };
                    theme::draw_nav_icon(ui.painter(), i_icon, theme::NavIcon::Upload, ACCENT);
                    if show_labels {
                        ui.painter().text(
                            egui::pos2(i_rect.left() + 38.0, i_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Impor audio",
                            egui::FontId::proportional(13.0),
                            INK,
                        );
                    } else {
                        i_resp.clone().on_hover_text("Impor audio");
                    }
                    if i_resp.clicked() {
                        self.pick_and_import_audio("");
                    }

                    ui.add_space(6.0);

                    // Primary Record / Stop Button
                    let is_rec = self.is_recording();
                    let (r_rect, r_resp) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 40.0),
                        Sense::click(),
                    );
                    if r_resp.hovered() {
                        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    let r_bg = if is_rec {
                        if r_resp.hovered() { DANGER_HOVER } else { DANGER }
                    } else if r_resp.hovered() {
                        theme::ACCENT_HOVER
                    } else {
                        ACCENT
                    };
                    ui.painter().rect_filled(r_rect, CornerRadius::same(10), r_bg);
                    let r_icon = if show_labels {
                        egui::Rect::from_center_size(
                            egui::pos2(r_rect.left() + 20.0, r_rect.center().y),
                            Vec2::splat(16.0),
                        )
                    } else {
                        egui::Rect::from_center_size(r_rect.center(), Vec2::splat(17.0))
                    };
                    if is_rec {
                        let sq = egui::Rect::from_center_size(r_icon.center(), Vec2::splat(10.0));
                        ui.painter().rect_filled(sq, CornerRadius::same(2), Color32::WHITE);
                    } else {
                        theme::draw_nav_icon(
                            ui.painter(),
                            r_icon,
                            theme::NavIcon::Mic,
                            Color32::WHITE,
                        );
                    }
                    if show_labels {
                        ui.painter().text(
                            egui::pos2(r_rect.left() + 38.0, r_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            if is_rec { "Hentikan" } else { "Mulai rekam" },
                            egui::FontId::proportional(13.0),
                            Color32::WHITE,
                        );
                    } else {
                        r_resp.clone().on_hover_text(if is_rec {
                            "Hentikan rekaman"
                        } else {
                            "Mulai rekam"
                        });
                    }
                    if r_resp.clicked() {
                        if is_rec {
                            self.stop_recording();
                        } else {
                            self.start_recording();
                        }
                    }
                });
            });
    }

    // ── Top App Bar (Breadcrumb + CUDA pill & non-clickable Google Calendar indicator right beneath!) ──
    fn draw_topbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("notely_topbar")
            .exact_height(58.0)
            .frame(
                Frame::new()
                    .fill(SURFACE)
                    .stroke(Stroke::new(1.0, BORDER))
                    .inner_margin(Margin::symmetric(16, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // Sidebar collapse button
                    let (t_rect, t_resp) =
                        ui.allocate_exact_size(Vec2::splat(30.0), Sense::click());
                    if t_resp.hovered() {
                        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                        ui.painter()
                            .rect_filled(t_rect, CornerRadius::same(7), SURFACE_HOVER);
                    }
                    theme::draw_nav_icon(
                        ui.painter(),
                        t_rect.shrink(6.0),
                        theme::NavIcon::SidebarToggle,
                        INK_2,
                    );
                    if t_resp.clicked() {
                        self.prefs.sidebar_collapsed = !self.prefs.sidebar_collapsed;
                        self.save_local_prefs();
                    }

                    ui.add_space(6.0);
                    ui.label(RichText::new("Notely").size(13.5).color(INK).strong());
                    ui.label(RichText::new("/").size(13.5).color(INK_MUTED));
                    ui.label(RichText::new(self.page.label()).size(13.5).color(INK_2));

                    // Right-aligned stacked status block: CUDA badge on top, non-clickable Google Calendar right below!
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.vertical(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let backend = match self.status.backend.as_deref() {
                                    Some(b) if !b.is_empty() => b.to_string(),
                                    _ if self.gpu.cuda_installed => "CUDA".to_string(),
                                    _ => "CPU".to_string(),
                                };
                                let is_cuda = backend.to_uppercase().contains("CUDA");
                                let (dot, bg, fg) = if is_cuda {
                                    (GREEN, GREEN_SOFT, ACCENT_DEEP)
                                } else {
                                    (INK_MUTED, SURFACE_SUBTLE, INK_2)
                                };
                                theme::status_pill(ui, dot, &backend, bg, fg);
                            });

                            ui.add_space(2.0);

                            // Pure non-clickable Google Calendar indicator (Green = Connected, Red = Disconnected)
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let (dot, lbl) = if self.gcal.connected {
                                    (GREEN, "Google Calendar")
                                } else {
                                    (DANGER, "Google Calendar")
                                };
                                theme::non_clickable_indicator(ui, dot, lbl);
                            });
                        });
                    });
                });
            });
    }

    // ── Floating Recording Dock (bottom-center pill with 18-bar animated spectrum) ──
    fn draw_floating_dock(&mut self, ctx: &egui::Context) {
        if self.page == Page::Settings || self.page == Page::Notes && self.note_tab == NoteTab::Summary {
            return;
        }
        let time = ctx.input(|i| i.time);
        let is_rec = self.is_recording();

        egui::Area::new(egui::Id::new("notely_floating_dock"))
            .order(egui::Order::Middle)
            .anchor(egui::Align2::CENTER_BOTTOM, Vec2::new(34.0, -18.0))
            .show(ctx, |ui| {
                Frame::new()
                    .fill(SURFACE)
                    .stroke(Stroke::new(1.0, if is_rec { theme::DANGER_BORDER } else { BORDER }))
                    .corner_radius(CornerRadius::same(28))
                    .inner_margin(Margin::symmetric(12, 8))
                    .shadow(Shadow {
                        offset: [0, 8],
                        blur: 24,
                        spread: 0,
                        color: Color32::from_rgba_unmultiplied(15, 38, 30, 32),
                    })
                    .show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            ui.spacing_mut().item_spacing.x = 12.0;

                            // Record / Stop circular button with pulsing halo
                            let (btn_rect, btn_resp) =
                                ui.allocate_exact_size(Vec2::splat(38.0), Sense::click());
                            if btn_resp.hovered() {
                                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if is_rec {
                                let halo = (0.5 + 0.5 * (time * 5.0).sin() as f32) * 5.0;
                                ui.painter().circle_filled(
                                    btn_rect.center(),
                                    19.0 + halo,
                                    Color32::from_rgba_unmultiplied(217, 56, 69, 45),
                                );
                            }
                            let circle_col = if is_rec {
                                DANGER
                            } else if btn_resp.hovered() {
                                DANGER_HOVER
                            } else {
                                DANGER
                            };
                            ui.painter()
                                .circle_filled(btn_rect.center(), 18.0, circle_col);
                            if is_rec {
                                let sq = egui::Rect::from_center_size(
                                    btn_rect.center(),
                                    Vec2::splat(11.0),
                                );
                                ui.painter().rect_filled(sq, CornerRadius::same(3), Color32::WHITE);
                            } else {
                                theme::draw_nav_icon(
                                    ui.painter(),
                                    btn_rect.shrink(9.0),
                                    theme::NavIcon::Mic,
                                    Color32::WHITE,
                                );
                            }
                            if btn_resp.clicked() {
                                if is_rec {
                                    self.stop_recording();
                                } else {
                                    self.start_recording();
                                }
                            }

                            // 18-bar spectrum equalizer
                            theme::draw_spectrum_bars(ui, &self.spectrum_bands, is_rec, time);

                            if is_rec {
                                ui.label(
                                    RichText::new(format_ts(self.status.elapsed.unwrap_or(0.0)))
                                        .monospace()
                                        .size(12.5)
                                        .color(DANGER)
                                        .strong(),
                                );
                            }
                        });
                    });
            });
    }

    fn draw_update_modal(&mut self, ctx: &egui::Context) {
        let Some(upd) = self.available_update.clone() else {
            return;
        };
        egui::Area::new(egui::Id::new("update_modal_scrim"))
            .order(egui::Order::Middle)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                let screen = ctx.screen_rect();
                ui.allocate_rect(screen, Sense::click());
                ui.painter()
                    .rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(110));
            });

        egui::Window::new("Pembaruan Tersedia")
            .id(egui::Id::new("update_modal_win"))
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(470.0, 0.0))
            .frame(theme::modal_frame())
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("Pembaruan Notely v{} Tersedia", upd.version))
                        .size(17.5)
                        .color(INK)
                        .strong(),
                );
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        theme::render_markdown_clean(ui, &upd.notes);
                    });

                if self.update_downloading {
                    ui.add_space(12.0);
                    let (bar, _) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 7.0),
                        Sense::hover(),
                    );
                    ui.painter().rect_filled(bar, CornerRadius::same(4), ACCENT_SOFT);
                    let fill = egui::Rect::from_min_size(
                        bar.min,
                        Vec2::new(bar.width() * self.update_progress, bar.height()),
                    );
                    ui.painter().rect_filled(fill, CornerRadius::same(4), ACCENT);
                    ui.add_space(4.0);
                    ui.label(RichText::new(&self.update_status_text).size(12.0).color(INK_2));
                }

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_enabled_ui(!self.update_downloading, |ui| {
                            if theme::primary_btn(ui, "Unduh & Pasang").clicked() {
                                self.start_update_download();
                            }
                            if theme::secondary_btn(ui, "Nanti").clicked() {
                                self.available_update = None;
                            }
                        });
                    });
                });
            });
    }

    fn draw_close_guard_modal(&mut self, ctx: &egui::Context) {
        egui::Area::new(egui::Id::new("close_guard_scrim"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                let screen = ctx.screen_rect();
                ui.allocate_rect(screen, Sense::click());
                ui.painter()
                    .rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(120));
            });

        egui::Window::new("Rekaman Berjalan")
            .id(egui::Id::new("close_guard_win"))
            .order(egui::Order::Foreground)
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(420.0, 0.0))
            .frame(theme::modal_frame())
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("Rekaman Sedang Berlangsung")
                        .size(17.0)
                        .color(INK)
                        .strong(),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(
                        "Jika Anda menutup aplikasi sekarang, rekaman aktif akan dihentikan. Apakah Anda yakin?",
                    )
                    .size(13.0)
                    .color(INK_2),
                );
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if theme::danger_btn(ui, "Hentikan & Keluar").clicked() {
                            if let Some(b) = &mut self.bridge {
                                b.stop();
                            }
                            self.allow_close_now = true;
                            self.show_close_guard = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if theme::secondary_btn(ui, "Lanjutkan Merekam").clicked() {
                            self.show_close_guard = false;
                        }
                    });
                });
            });
    }
}

impl eframe::App for NotelyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Intercept OS window close when actively recording
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_close_now {
            if self.is_recording() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_close_guard = true;
            } else if let Some(b) = &mut self.bridge {
                b.stop();
            }
        }

        self.process_bg_messages(ctx);

        if self.in_splash {
            views::splash::show(self, ctx);
            return;
        }

        self.schedule_polls();

        // Keep smooth 60fps repainting during live audio / animations
        if self.is_recording()
            || self.summary_running
            || self.ai_step == AiStep::Loading
            || self.update_downloading
        {
            ctx.request_repaint_after(Duration::from_millis(33));
        } else {
            ctx.request_repaint_after(Duration::from_millis(250));
        }

        // Global Ctrl+V clipboard image paste when AI Deadline Extractor modal is open on Input step
        if self.show_ai_deadline_modal && self.ai_step == AiStep::Input {
            let paste_pressed = ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::V));
            if paste_pressed {
                self.try_paste_clipboard_image(ctx);
            }
        }

        // Handle Drag-and-Drop files (images into AI modal, or audio import)
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        for path in dropped {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if self.show_ai_deadline_modal
                && self.ai_step == AiStep::Input
                && matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp")
            {
                if let Ok(bytes) = std::fs::read(&path) {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("gambar.png")
                        .to_string();
                    if self.attach_image_bytes(ctx, name, &bytes) {
                        self.toast("Gambar berhasil dilampirkan.", false);
                    }
                }
            } else if matches!(
                ext.as_str(),
                "wav" | "mp3" | "m4a" | "flac" | "ogg" | "mp4" | "mkv" | "webm" | "aac"
            ) {
                let fid = self.active_folder_id.clone().unwrap_or_default();
                self.import_audio_path(path, &fid);
            }
        }

        // Render Main Native Chrome
        self.draw_sidebar(ctx);
        self.draw_topbar(ctx);

        egui::CentralPanel::default()
            .frame(Frame::new().fill(CANVAS).inner_margin(Margin::ZERO))
            .show(ctx, |ui| match self.page {
                Page::Home => views::home::show(self, ctx, ui),
                Page::Notes => views::notes::show(self, ctx, ui),
                Page::Recordings => views::recordings::show(self, ctx, ui),
                Page::Deadlines => views::deadlines::show(self, ctx, ui),
                Page::Settings => views::settings::show(self, ctx, ui),
            });

        self.draw_floating_dock(ctx);

        if self.available_update.is_some() {
            self.draw_update_modal(ctx);
        }
        if self.show_close_guard {
            self.draw_close_guard_modal(ctx);
        }

        // Crisp foreground Toast (never blurred, auto-dismisses after 4 seconds)
        if let Some((msg, is_err, created)) = &self.toast_msg {
            if created.elapsed() < Duration::from_secs(4) {
                theme::draw_toast(ctx, msg, *is_err);
            } else {
                self.toast_msg = None;
            }
        }
    }
}

fn load_texture_from_png(
    ctx: &egui::Context,
    name: &str,
    bytes: &[u8],
) -> Option<TextureHandle> {
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let ci = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba.into_raw());
    Some(ctx.load_texture(name, ci, TextureOptions::LINEAR))
}

fn prefs_file_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Notely")
        .join("native_prefs.json")
}

fn load_local_prefs() -> AppPrefs {
    let path = prefs_file_path();
    if let Ok(raw) = std::fs::read_to_string(path) {
        if let Ok(p) = serde_json::from_str::<AppPrefs>(&raw) {
            return p;
        }
    }
    AppPrefs::default_prefs()
}

fn parse_extracted_deadlines(result: Option<&Value>) -> Vec<DeadlineItem> {
    let Some(val) = result else {
        return Vec::new();
    };
    let arr = val
        .get("items")
        .or_else(|| val.get("deadlines"))
        .or(Some(val));
    let Some(list) = arr.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in list {
        if let Ok(mut item) = serde_json::from_value::<DeadlineItem>(entry.clone()) {
            if item.priority.is_empty() {
                item.priority = "medium".to_string();
            }
            if item.course.is_empty() {
                item.course = "Akademik".to_string();
            }
            if item.time.is_empty() {
                item.time = "23:59".to_string();
            }
            if item.status.is_empty() {
                item.status = "pending".to_string();
            }
            out.push(item);
        }
    }
    out
}

fn url_encode(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' {
                (b as char).to_string()
            } else {
                format!("%{:02X}", b)
            }
        })
        .collect()
}
