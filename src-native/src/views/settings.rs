use crate::app::NotelyApp;
use crate::theme::{
    self, ACCENT, ACCENT_BORDER, ACCENT_DEEP, ACCENT_SOFT, BORDER, DANGER, GREEN, GREEN_SOFT, INK,
    INK_2, INK_MUTED, SURFACE, SURFACE_SUBTLE,
};
use crate::types::{format_bytes, SettingsTab, APP_VERSION};
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, RichText, Stroke, Vec2,
};

pub fn show(app: &mut NotelyApp, _ctx: &egui::Context, ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    let content_w = avail_w.min(780.0).max(320.0);
    let side_pad = ((avail_w - content_w) * 0.5).max(16.0);

    // Tight, centered ScrollArea with zero excess bottom scroll dead-space!
    egui::ScrollArea::vertical()
        .id_salt("settings_scroll")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.add_space(side_pad);
                ui.allocate_ui_with_layout(
                    Vec2::new(content_w, 0.0),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        // ── Centered Header & 4-Tab Bar ──
                        ui.label(RichText::new("Pengaturan").size(22.0).color(INK).strong());
                        ui.label(
                            RichText::new(
                                "Kelola preferensi umum, perangkat rekaman, model transkripsi, dan AI ringkasan.",
                            )
                            .size(13.0)
                            .color(INK_2),
                        );

                        ui.add_space(14.0);

                        Frame::new()
                            .fill(SURFACE_SUBTLE)
                            .stroke(Stroke::new(1.0, BORDER))
                            .corner_radius(CornerRadius::same(11))
                            .inner_margin(Margin::same(4))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    let tab_w = ((content_w - 20.0) / 4.0).max(90.0);
                                    for (tab, lbl) in [
                                        (SettingsTab::General, "Umum"),
                                        (SettingsTab::Recordings, "Rekaman"),
                                        (SettingsTab::Transcription, "Transkripsi"),
                                        (SettingsTab::Summary, "Ringkasan AI"),
                                    ] {
                                        let active = app.settings_tab == tab;
                                        let btn = egui::Button::new(
                                            RichText::new(lbl)
                                                .size(13.0)
                                                .color(if active { INK } else { INK_MUTED })
                                                .strong(),
                                        )
                                        .fill(if active { SURFACE } else { Color32::TRANSPARENT })
                                        .stroke(if active {
                                            Stroke::new(1.0, BORDER)
                                        } else {
                                            Stroke::NONE
                                        })
                                        .corner_radius(CornerRadius::same(8))
                                        .min_size(Vec2::new(tab_w, 32.0));
                                        let resp = ui.add(btn);
                                        if resp.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        if resp.clicked() {
                                            app.settings_tab = tab;
                                            if tab == SettingsTab::Summary && app.ollama_catalog.models.is_empty() {
                                                app.refresh_ollama_catalog();
                                            }
                                        }
                                    }
                                });
                            });

                        ui.add_space(16.0);

                        match app.settings_tab {
                            SettingsTab::General => show_general_tab(app, ui, content_w),
                            SettingsTab::Recordings => show_recordings_tab(app, ui, content_w),
                            SettingsTab::Transcription => show_transcription_tab(app, ui, content_w),
                            SettingsTab::Summary => show_summary_tab(app, ui, content_w),
                        }

                        // Compact bottom margin — no excess blank scroll!
                        ui.add_space(28.0);
                    },
                );
            });
        });
}

fn show_general_tab(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    let card_w = width - 34.0;

    // ── Preferensi Umum ──
    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        ui.label(RichText::new("Preferensi Aplikasi").size(15.0).color(INK).strong());
        ui.add_space(10.0);

        setting_row(
            ui,
            "Notifikasi Selesai",
            "Tampilkan pemberitahuan saat transkripsi atau ringkasan selesai.",
            |ui| {
                if theme::toggle_switch(ui, &mut app.prefs.notify).changed() {
                    app.save_local_prefs();
                }
            },
        );

        ui.separator();
        ui.add_space(6.0);

        setting_row(
            ui,
            "Bahasa Transkripsi",
            "Bahasa utama untuk pengenalan suara saat merekam.",
            |ui| {
                egui::ComboBox::from_id_salt("pref_lang_cb")
                    .selected_text(&app.prefs.language)
                    .width(160.0)
                    .show_ui(ui, |ui| {
                        for lang in ["Indonesia", "Inggris", "Otomatis"] {
                            if ui
                                .selectable_value(&mut app.prefs.language, lang.to_string(), lang)
                                .clicked()
                            {
                                app.save_local_prefs();
                            }
                        }
                    });
            },
        );

        ui.separator();
        ui.add_space(6.0);

        setting_row(
            ui,
            "Folder Penyimpanan Catatan",
            &app.data_root_display,
            |ui| {
                if theme::secondary_btn(ui, "Buka Folder").clicked() {
                    app.open_data_folder();
                }
            },
        );
    });

    ui.add_space(14.0);

    // ── Google Calendar Integration ──
    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Google Calendar").size(15.0).color(INK).strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if app.gcal.connected {
                    theme::status_pill(ui, GREEN, "Terhubung", GREEN_SOFT, ACCENT_DEEP);
                } else {
                    theme::status_pill(
                        ui,
                        DANGER,
                        "Belum Terhubung",
                        theme::DANGER_SOFT,
                        DANGER,
                    );
                }
            });
        });

        ui.add_space(8.0);
        if app.gcal.connected {
            ui.label(
                RichText::new(format!("Akun aktif: {}", app.gcal.email))
                    .size(13.0)
                    .color(INK_2),
            );
            ui.add_space(10.0);

            let mut auto = app.gcal.auto_sync;
            setting_row(
                ui,
                "Sinkronisasi Otomatis",
                "Kirim jadwal baru secara otomatis ke Google Calendar.",
                |ui| {
                    if theme::toggle_switch(ui, &mut auto).changed() {
                        app.set_gcal_auto_sync(auto);
                    }
                },
            );

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if theme::primary_btn(ui, "Sinkronkan Sekarang").clicked() {
                    app.sync_google_calendar();
                }
                if theme::danger_soft_btn(ui, "Putuskan Koneksi").clicked() {
                    app.disconnect_google_calendar();
                }
            });
        } else {
            ui.label(
                RichText::new(
                    "Hubungkan Google Calendar agar jadwal & pengingat tersinkron dua arah dengan perangkat Anda.",
                )
                .size(13.0)
                .color(INK_2),
            );
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if theme::primary_btn(ui, "Sambungkan Google Calendar").clicked() {
                    app.connect_google_calendar();
                }
                if theme::secondary_btn(ui, "Impor credentials.json").clicked() {
                    app.import_gcal_json();
                }
            });
        }
    });

    ui.add_space(14.0);

    // ── Tentang & Pembaruan ──
    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        setting_row(
            ui,
            &format!("Notely v{} (Native Rust)", APP_VERSION),
            "Periksa pembaruan otomatis dari GitHub Releases.",
            |ui| {
                let lbl = if app.checking_update {
                    "Memeriksa..."
                } else {
                    "Periksa Pembaruan"
                };
                ui.add_enabled_ui(!app.checking_update, |ui| {
                    if theme::secondary_btn(ui, lbl).clicked() {
                        app.check_updates_manual();
                    }
                });
            },
        );
    });
}

fn show_recordings_tab(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    let card_w = width - 34.0;

    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        ui.label(RichText::new("Opsi Perekaman").size(15.0).color(INK).strong());
        ui.add_space(10.0);

        setting_row(
            ui,
            "Simpan File Audio",
            "Simpan rekaman audio asli agar dapat diproses ulang kapan saja.",
            |ui| {
                if theme::toggle_switch(ui, &mut app.prefs.save_audio).changed() {
                    app.save_local_prefs();
                }
            },
        );

        ui.separator();
        ui.add_space(6.0);

        setting_row(
            ui,
            "Pratinjau Transkrip Langsung",
            "Tampilkan teks secara real-time saat rekaman berlangsung.",
            |ui| {
                if theme::toggle_switch(ui, &mut app.prefs.live_preview).changed() {
                    app.save_local_prefs();
                }
            },
        );
    });

    ui.add_space(14.0);

    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Perangkat Audio").size(15.0).color(INK).strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::secondary_btn(ui, "Muat Ulang").clicked() {
                    app.refresh_devices();
                }
            });
        });
        ui.add_space(12.0);

        ui.label(
            RichText::new("Speaker (Audio Komputer / Loopback)")
                .size(12.5)
                .color(INK_2)
                .strong(),
        );
        ui.add_space(4.0);
        let spk_name = app
            .devices
            .speakers
            .iter()
            .find(|d| d.index == app.selected_speaker)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "Pilih speaker...".to_string());
        egui::ComboBox::from_id_salt("speaker_select_cb")
            .selected_text(spk_name)
            .width(card_w)
            .show_ui(ui, |ui| {
                for dev in &app.devices.speakers {
                    ui.selectable_value(&mut app.selected_speaker, dev.index, &dev.name);
                }
            });

        ui.add_space(12.0);

        ui.label(RichText::new("Mikrofon").size(12.5).color(INK_2).strong());
        ui.add_space(4.0);
        let mic_name = if app.selected_mic < 0 {
            "Tanpa Mikrofon (Hanya Audio Komputer)".to_string()
        } else {
            app.devices
                .microphones
                .iter()
                .find(|d| d.index == app.selected_mic)
                .map(|d| d.name.clone())
                .unwrap_or_else(|| "Tanpa Mikrofon".to_string())
        };
        egui::ComboBox::from_id_salt("mic_select_cb")
            .selected_text(mic_name)
            .width(card_w)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut app.selected_mic,
                    -1,
                    "Tanpa Mikrofon (Hanya Audio Komputer)",
                );
                for dev in &app.devices.microphones {
                    ui.selectable_value(&mut app.selected_mic, dev.index, &dev.name);
                }
            });

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            let testing = app.audio_test_running;
            ui.add_enabled_ui(!testing && !app.is_recording(), |ui| {
                if theme::secondary_btn(
                    ui,
                    if testing {
                        "Menguji Audio..."
                    } else {
                        "Tes Audio 5 Detik"
                    },
                )
                .clicked()
                {
                    app.run_audio_test();
                }
            });
            if !app.audio_test_msg.is_empty() {
                ui.label(RichText::new(&app.audio_test_msg).size(12.5).color(ACCENT_DEEP));
            }
        });
    });
}

fn show_transcription_tab(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    let card_w = width - 34.0;

    // ── GPU / CUDA Card ──
    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Akselerasi Perangkat Keras").size(15.0).color(INK).strong());
                let rep = if app.gpu.report.is_empty() {
                    "Memeriksa akselerasi GPU / CPU..."
                } else {
                    &app.gpu.report
                };
                ui.label(RichText::new(rep).size(12.5).color(INK_2));
                if !app.gpu.nvidia.is_empty() {
                    ui.label(
                        RichText::new(format!("GPU Terdeteksi: {}", app.gpu.nvidia))
                            .size(12.0)
                            .color(ACCENT_DEEP),
                    );
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if !app.gpu.nvidia.is_empty() && !app.gpu.cuda_installed {
                    if theme::primary_btn(ui, "Pasang CUDA").clicked() {
                        app.install_cuda();
                    }
                }
            });
        });
    });

    ui.add_space(14.0);

    // Active download/prepare banner if running
    if let Some(msg) = &app.model_prepare_msg {
        Frame::new()
            .fill(ACCENT_SOFT)
            .stroke(Stroke::new(1.0, ACCENT_BORDER))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.set_width(card_w);
                ui.label(RichText::new(msg).size(12.5).color(ACCENT_DEEP).strong());
            });
        ui.add_space(12.0);
    }

    // ── Transcription Models List ──
    let models = if app.transcribe_models.is_empty() {
        vec![
            "Indonesia Cepat (terlatih, ringan)".to_string(),
            "Indonesia Turbo (terlatih, GPU)".to_string(),
            "Whisper Small".to_string(),
            "Whisper Medium".to_string(),
            "Whisper Large-v3 Turbo".to_string(),
            "Whisper Large-v3".to_string(),
        ]
    } else {
        app.transcribe_models.clone()
    };

    for name in models {
        let selected = app.prefs.model == name;
        let frame = Frame::new()
            .fill(SURFACE)
            .stroke(Stroke::new(
                if selected { 1.5 } else { 1.0 },
                if selected { ACCENT } else { BORDER },
            ))
            .corner_radius(CornerRadius::same(13))
            .inner_margin(Margin::symmetric(16, 13));

        frame.show(ui, |ui| {
            ui.set_width(card_w);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(&name).size(14.0).color(INK).strong());
                    ui.label(
                        RichText::new(model_desc(&name))
                            .size(12.0)
                            .color(INK_MUTED),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if selected {
                        Frame::new()
                            .fill(ACCENT_SOFT)
                            .stroke(Stroke::new(1.0, ACCENT_BORDER))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::symmetric(12, 6))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("Terpilih")
                                        .size(12.0)
                                        .color(ACCENT_DEEP)
                                        .strong(),
                                );
                            });
                    } else if theme::secondary_btn(ui, "Pilih").clicked() {
                        app.select_transcription_model(&name);
                    }
                });
            });
        });
        ui.add_space(10.0);
    }
}

fn show_summary_tab(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    let card_w = width - 34.0;

    // ── Provider Selector ──
    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        ui.label(RichText::new("Penyedia AI Ringkasan & Ekstraksi").size(15.0).color(INK).strong());
        ui.add_space(10.0);

        ui.horizontal_wrapped(|ui| {
            for (prov, lbl) in [
                ("ollama", "AI Lokal (Ollama)"),
                ("openai", "OpenAI"),
                ("gemini", "Google Gemini"),
                ("groq", "Groq"),
                ("openrouter", "OpenRouter"),
                ("custom", "Custom Endpoint"),
            ] {
                let active = app.prefs.provider == prov;
                let btn = egui::Button::new(
                    RichText::new(lbl)
                        .size(12.5)
                        .color(if active { Color32::WHITE } else { INK_2 })
                        .strong(),
                )
                .fill(if active { ACCENT } else { SURFACE_SUBTLE })
                .stroke(Stroke::new(1.0, if active { ACCENT } else { BORDER }))
                .corner_radius(CornerRadius::same(9))
                .min_size(Vec2::new(0.0, 32.0));
                if ui.add(btn).clicked() {
                    app.prefs.provider = prov.to_string();
                    app.save_local_prefs();
                }
            }
        });
    });

    ui.add_space(14.0);

    if app.prefs.provider == "ollama" {
        // Storage & Refresh Header
        theme::card_frame().show(ui, |ui| {
            ui.set_width(card_w);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Katalog Model AI Lokal").size(15.0).color(INK).strong());
                    let msg = if app.ollama_catalog.connected {
                        "Mesin Ollama aktif dan siap digunakan secara offline."
                    } else if !app.ollama_catalog.message.is_empty() {
                        &app.ollama_catalog.message
                    } else {
                        "Pilih dan unduh model lokal sesuai kapasitas perangkat Anda."
                    };
                    ui.label(RichText::new(msg).size(12.0).color(INK_2));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::secondary_btn(ui, "Muat Ulang").clicked() {
                        app.refresh_ollama_catalog();
                    }
                    if theme::secondary_btn(ui, "Folder Model").clicked() {
                        app.pick_model_storage_folder();
                    }
                });
            });

            if let Some(pull_msg) = &app.ollama_pull_msg {
                ui.add_space(10.0);
                Frame::new()
                    .fill(ACCENT_SOFT)
                    .stroke(Stroke::new(1.0, ACCENT_BORDER))
                    .corner_radius(CornerRadius::same(9))
                    .inner_margin(Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(pull_msg).size(12.0).color(ACCENT_DEEP).strong());
                    });
            }
        });

        ui.add_space(12.0);

        let catalog = app.ollama_catalog.models.clone();
        for m in catalog {
            let selected = app.prefs.summary_model == m.id;
            Frame::new()
                .fill(SURFACE)
                .stroke(Stroke::new(
                    if selected { 1.5 } else { 1.0 },
                    if selected { ACCENT } else { BORDER },
                ))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(Margin::symmetric(15, 12))
                .show(ui, |ui| {
                    ui.set_width(card_w);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&m.name).size(14.0).color(INK).strong());
                                if !m.tier.is_empty() {
                                    Frame::new()
                                        .fill(SURFACE_SUBTLE)
                                        .corner_radius(CornerRadius::same(6))
                                        .inner_margin(Margin::symmetric(6, 2))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(&m.tier).size(10.5).color(INK_2),
                                            );
                                        });
                                }
                                let sz = m.size_bytes.or(m.estimated_bytes).unwrap_or(0);
                                if sz > 0 {
                                    ui.label(
                                        RichText::new(format_bytes(sz))
                                            .size(11.5)
                                            .color(INK_MUTED),
                                    );
                                }
                            });
                            if !m.description.is_empty() {
                                ui.label(
                                    RichText::new(&m.description)
                                        .size(12.0)
                                        .color(INK_MUTED),
                                );
                            }
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if m.installed {
                                if theme::danger_soft_btn(ui, "Hapus").clicked() {
                                    app.delete_ollama_model(&m.id);
                                }
                                if selected {
                                    Frame::new()
                                        .fill(ACCENT_SOFT)
                                        .stroke(Stroke::new(1.0, ACCENT_BORDER))
                                        .corner_radius(CornerRadius::same(8))
                                        .inner_margin(Margin::symmetric(10, 6))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new("Aktif")
                                                    .size(12.0)
                                                    .color(ACCENT_DEEP)
                                                    .strong(),
                                            );
                                        });
                                } else if theme::primary_btn(ui, "Gunakan").clicked() {
                                    app.prefs.summary_model = m.id.clone();
                                    app.save_local_prefs();
                                    app.toast("Model ringkasan lokal diperbarui.", false);
                                }
                            } else if theme::secondary_btn(ui, "Unduh").clicked() {
                                app.pull_ollama_model(&m.id);
                            }
                        });
                    });
                });
            ui.add_space(8.0);
        }
    } else {
        // Cloud Provider Configuration
        theme::card_frame().show(ui, |ui| {
            ui.set_width(card_w);
            ui.label(
                RichText::new(format!("Konfigurasi Cloud ({})", app.prefs.provider))
                    .size(15.0)
                    .color(INK)
                    .strong(),
            );
            ui.add_space(10.0);

            ui.label(RichText::new("Nama Model").size(12.0).color(INK_2).strong());
            ui.add_space(4.0);
            ui.add_sized(
                Vec2::new(card_w, 34.0),
                egui::TextEdit::singleline(&mut app.prefs.custom_model)
                    .hint_text("Misal: gpt-4o-mini / gemini-2.0-flash / llama-3.3-70b-versatile")
                    .margin(Margin::symmetric(10, 7)),
            );

            ui.add_space(10.0);

            ui.label(RichText::new("API Key (disimpan terenkripsi Windows DPAPI)").size(12.0).color(INK_2).strong());
            ui.add_space(4.0);
            ui.add_sized(
                Vec2::new(card_w, 34.0),
                egui::TextEdit::singleline(&mut app.cloud_api_key_input)
                    .password(true)
                    .hint_text("Masukkan API key Anda...")
                    .margin(Margin::symmetric(10, 7)),
            );

            if app.prefs.provider == "custom" {
                ui.add_space(10.0);
                ui.label(RichText::new("Endpoint URL (OpenAI-compatible)").size(12.0).color(INK_2).strong());
                ui.add_space(4.0);
                ui.add_sized(
                    Vec2::new(card_w, 34.0),
                    egui::TextEdit::singleline(&mut app.prefs.endpoint)
                        .hint_text("https://api.contoh.com/v1")
                        .margin(Margin::symmetric(10, 7)),
                );
            }

            ui.add_space(14.0);
            ui.horizontal(|ui| {
                if theme::primary_btn(ui, "Simpan Konfigurasi").clicked() {
                    app.save_cloud_config();
                }
                if theme::secondary_btn(ui, "Tes Koneksi").clicked() {
                    app.test_cloud_connection();
                }
            });
        });
    }
}

fn setting_row(ui: &mut egui::Ui, title: &str, desc: &str, control: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(13.5).color(INK).strong());
            ui.label(RichText::new(desc).size(12.0).color(INK_MUTED));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), control);
    });
    ui.add_space(6.0);
}

fn model_desc(name: &str) -> &'static str {
    if name.contains("Indonesia Cepat") {
        "Dioptimalkan khusus bahasa Indonesia · ringan untuk CPU & laptop standar."
    } else if name.contains("Indonesia Turbo") {
        "Akurasi sangat tinggi untuk bahasa Indonesia · direkomendasikan dengan GPU NVIDIA."
    } else if name.contains("Large-v3 Turbo") {
        "Model multibahasa dengan optimasi kecepatan inferensi tinggi."
    } else if name.contains("Large-v3") {
        "Kualitas transkrip multibahasa tertinggi untuk perangkat bertenaga besar."
    } else if name.contains("Medium") {
        "Keseimbangan akurasi dan kecepatan untuk percakapan multibahasa."
    } else {
        "Pilihan ringan dan cepat untuk percakapan multibahasa sehari-hari."
    }
}
