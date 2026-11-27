use crate::app::NotelyApp;
use crate::theme::{
    self, ACCENT, ACCENT_BORDER, ACCENT_DEEP, ACCENT_SOFT, BORDER, DANGER, GREEN, GREEN_SOFT, INK,
    INK_2, INK_MUTED, SURFACE, SURFACE_SUBTLE,
};
use crate::types::{format_bytes, SettingsTab, APP_VERSION};
use eframe::egui::{self, Align, Color32, CornerRadius, Frame, Layout, Margin, Stroke, Vec2};

pub fn show(app: &mut NotelyApp, _ctx: &egui::Context, ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    let content_w = avail_w.min(780.0_f32).max(340.0_f32);
    let side_pad = ((avail_w - content_w) * 0.5_f32).max(20.0_f32);

    // Centered ScrollArea with auto_shrink([false, true]) so zero excess bottom scroll!
    egui::ScrollArea::vertical()
        .id_salt("settings_scroll")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.add_space(22.0_f32);
            ui.horizontal(|ui| {
                ui.add_space(side_pad);
                ui.allocate_ui_with_layout(
                    Vec2::new(content_w, 0.0_f32),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        ui.label(theme::bold("Pengaturan", 23.0_f32, INK));
                        ui.label(theme::regular(
                            "Kelola preferensi umum, perangkat rekaman, model transkripsi, dan AI ringkasan.",
                            13.0_f32,
                            INK_2,
                        ));

                        ui.add_space(14.0_f32);

                        Frame::new()
                            .fill(SURFACE)
                            .stroke(Stroke::new(1.0_f32, BORDER))
                            .corner_radius(CornerRadius::same(11))
                            .inner_margin(Margin::same(4))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0_f32;
                                    let tab_w = ((content_w - 20.0_f32) / 4.0_f32).max(90.0_f32);
                                    for (tab, lbl) in [
                                        (SettingsTab::General, "Umum"),
                                        (SettingsTab::Recordings, "Rekaman"),
                                        (SettingsTab::Transcription, "Transkripsi"),
                                        (SettingsTab::Summary, "Ringkasan AI"),
                                    ] {
                                        let active = app.settings_tab == tab;
                                        let btn = egui::Button::new(theme::semi(
                                            lbl,
                                            13.0_f32,
                                            if active { ACCENT_DEEP } else { INK_MUTED },
                                        ))
                                        .fill(if active {
                                            ACCENT_SOFT
                                        } else {
                                            Color32::TRANSPARENT
                                        })
                                        .stroke(if active {
                                            Stroke::new(1.0_f32, ACCENT_BORDER)
                                        } else {
                                            Stroke::NONE
                                        })
                                        .corner_radius(CornerRadius::same(8))
                                        .min_size(Vec2::new(tab_w, 32.0_f32));
                                        let resp = ui.add(btn);
                                        if resp.hovered() {
                                            ui.ctx()
                                                .set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        if resp.clicked() {
                                            app.settings_tab = tab;
                                            if tab == SettingsTab::Summary
                                                && app.ollama_catalog.models.is_empty()
                                            {
                                                app.refresh_ollama_catalog();
                                            }
                                        }
                                    }
                                });
                            });

                        ui.add_space(16.0_f32);

                        match app.settings_tab {
                            SettingsTab::General => show_general_tab(app, ui, content_w),
                            SettingsTab::Recordings => show_recordings_tab(app, ui, content_w),
                            SettingsTab::Transcription => show_transcription_tab(app, ui, content_w),
                            SettingsTab::Summary => show_summary_tab(app, ui, content_w),
                        }

                        ui.add_space(28.0_f32);
                    },
                );
            });
        });
}

fn show_general_tab(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    let card_w = width - 38.0_f32;

    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        ui.label(theme::bold("Preferensi Aplikasi", 15.0_f32, INK));
        ui.add_space(10.0_f32);

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
        ui.add_space(6.0_f32);

        setting_row(
            ui,
            "Bahasa Transkripsi",
            "Bahasa utama untuk pengenalan suara saat merekam.",
            |ui| {
                egui::ComboBox::from_id_salt("pref_lang_cb")
                    .selected_text(&app.prefs.language)
                    .width(160.0_f32)
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
        ui.add_space(6.0_f32);

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

    ui.add_space(14.0_f32);

    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        ui.horizontal(|ui| {
            ui.label(theme::bold("Google Calendar", 15.0_f32, INK));
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

        ui.add_space(8.0_f32);
        if app.gcal.connected {
            ui.label(theme::regular(
                format!("Akun aktif: {}", app.gcal.email),
                13.0_f32,
                INK_2,
            ));
            ui.add_space(10.0_f32);

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

            ui.add_space(10.0_f32);
            ui.horizontal(|ui| {
                if theme::primary_btn(ui, "Sinkronkan Sekarang").clicked() {
                    app.sync_google_calendar();
                }
                if theme::danger_soft_btn(ui, "Putuskan Koneksi").clicked() {
                    app.disconnect_google_calendar();
                }
            });
        } else {
            ui.label(theme::regular(
                "Hubungkan Google Calendar agar jadwal & pengingat tersinkron dua arah dengan perangkat Anda.",
                13.0_f32,
                INK_2,
            ));
            ui.add_space(10.0_f32);
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

    ui.add_space(14.0_f32);

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
    let card_w = width - 38.0_f32;

    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        ui.label(theme::bold("Opsi Perekaman", 15.0_f32, INK));
        ui.add_space(10.0_f32);

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
        ui.add_space(6.0_f32);

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

    ui.add_space(14.0_f32);

    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        ui.horizontal(|ui| {
            ui.label(theme::bold("Perangkat Audio", 15.0_f32, INK));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::secondary_btn(ui, "Muat Ulang").clicked() {
                    app.refresh_devices();
                }
            });
        });
        ui.add_space(12.0_f32);

        ui.label(theme::semi(
            "Speaker (Audio Komputer / Loopback)",
            12.5_f32,
            INK_2,
        ));
        ui.add_space(4.0_f32);
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

        ui.add_space(12.0_f32);

        ui.label(theme::semi("Mikrofon", 12.5_f32, INK_2));
        ui.add_space(4.0_f32);
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

        ui.add_space(14.0_f32);
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
                ui.label(theme::semi(&app.audio_test_msg, 12.5_f32, ACCENT_DEEP));
            }
        });
    });
}

fn show_transcription_tab(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    let card_w = width - 38.0_f32;

    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(theme::bold("Akselerasi Perangkat Keras", 15.0_f32, INK));
                let rep = if app.gpu.report.is_empty() {
                    "Memeriksa akselerasi GPU / CPU..."
                } else {
                    &app.gpu.report
                };
                ui.label(theme::regular(rep, 12.5_f32, INK_2));
                if !app.gpu.nvidia.is_empty() {
                    ui.label(theme::semi(
                        format!("GPU Terdeteksi: {}", app.gpu.nvidia),
                        12.0_f32,
                        ACCENT_DEEP,
                    ));
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

    ui.add_space(14.0_f32);

    if let Some(msg) = &app.model_prepare_msg {
        Frame::new()
            .fill(ACCENT_SOFT)
            .stroke(Stroke::new(1.0_f32, ACCENT_BORDER))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.set_width(card_w);
                ui.label(theme::semi(msg, 12.5_f32, ACCENT_DEEP));
            });
        ui.add_space(12.0_f32);
    }

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
                if selected { 1.5_f32 } else { 1.0_f32 },
                if selected { ACCENT } else { BORDER },
            ))
            .corner_radius(CornerRadius::same(13))
            .inner_margin(Margin::symmetric(16, 13));

        frame.show(ui, |ui| {
            ui.set_width(card_w);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(theme::semi(&name, 14.0_f32, INK));
                    ui.label(theme::regular(model_desc(&name), 12.0_f32, INK_MUTED));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if selected {
                        Frame::new()
                            .fill(ACCENT_SOFT)
                            .stroke(Stroke::new(1.0_f32, ACCENT_BORDER))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::symmetric(12, 6))
                            .show(ui, |ui| {
                                ui.label(theme::semi("Terpilih", 12.0_f32, ACCENT_DEEP));
                            });
                    } else if theme::secondary_btn(ui, "Pilih").clicked() {
                        app.select_transcription_model(&name);
                    }
                });
            });
        });
        ui.add_space(10.0_f32);
    }
}

fn show_summary_tab(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    let card_w = width - 38.0_f32;

    theme::card_frame().show(ui, |ui| {
        ui.set_width(card_w);
        ui.label(theme::bold(
            "Penyedia AI Ringkasan & Ekstraksi",
            15.0_f32,
            INK,
        ));
        ui.add_space(10.0_f32);

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
                let btn = egui::Button::new(theme::semi(
                    lbl,
                    12.5_f32,
                    if active { Color32::WHITE } else { INK_2 },
                ))
                .fill(if active { ACCENT } else { SURFACE_SUBTLE })
                .stroke(Stroke::new(1.0_f32, if active { ACCENT } else { BORDER }))
                .corner_radius(CornerRadius::same(9))
                .min_size(Vec2::new(0.0_f32, 32.0_f32));
                if ui.add(btn).clicked() {
                    app.prefs.provider = prov.to_string();
                    app.save_local_prefs();
                }
            }
        });
    });

    ui.add_space(14.0_f32);

    if app.prefs.provider == "ollama" {
        theme::card_frame().show(ui, |ui| {
            ui.set_width(card_w);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(theme::bold("Katalog Model AI Lokal", 15.0_f32, INK));
                    let msg = if app.ollama_catalog.connected {
                        "Mesin Ollama aktif dan siap digunakan secara offline."
                    } else if !app.ollama_catalog.message.is_empty() {
                        &app.ollama_catalog.message
                    } else {
                        "Pilih dan unduh model lokal sesuai kapasitas perangkat Anda."
                    };
                    ui.label(theme::regular(msg, 12.0_f32, INK_2));
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
                ui.add_space(10.0_f32);
                Frame::new()
                    .fill(ACCENT_SOFT)
                    .stroke(Stroke::new(1.0_f32, ACCENT_BORDER))
                    .corner_radius(CornerRadius::same(9))
                    .inner_margin(Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(theme::semi(pull_msg, 12.0_f32, ACCENT_DEEP));
                    });
            }
        });

        ui.add_space(12.0_f32);

        let catalog = app.ollama_catalog.models.clone();
        for m in catalog {
            let selected = app.prefs.summary_model == m.id;
            Frame::new()
                .fill(SURFACE)
                .stroke(Stroke::new(
                    if selected { 1.5_f32 } else { 1.0_f32 },
                    if selected { ACCENT } else { BORDER },
                ))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(Margin::symmetric(15, 12))
                .show(ui, |ui| {
                    ui.set_width(card_w);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(theme::semi(&m.name, 14.0_f32, INK));
                                if !m.tier.is_empty() {
                                    Frame::new()
                                        .fill(SURFACE_SUBTLE)
                                        .corner_radius(CornerRadius::same(6))
                                        .inner_margin(Margin::symmetric(6, 2))
                                        .show(ui, |ui| {
                                            ui.label(theme::regular(&m.tier, 10.5_f32, INK_2));
                                        });
                                }
                                let sz = m.size_bytes.or(m.estimated_bytes).unwrap_or(0);
                                if sz > 0 {
                                    ui.label(theme::regular(
                                        format_bytes(sz),
                                        11.5_f32,
                                        INK_MUTED,
                                    ));
                                }
                            });
                            if !m.description.is_empty() {
                                ui.label(theme::regular(&m.description, 12.0_f32, INK_MUTED));
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
                                        .stroke(Stroke::new(1.0_f32, ACCENT_BORDER))
                                        .corner_radius(CornerRadius::same(8))
                                        .inner_margin(Margin::symmetric(10, 6))
                                        .show(ui, |ui| {
                                            ui.label(theme::semi(
                                                "Aktif",
                                                12.0_f32,
                                                ACCENT_DEEP,
                                            ));
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
            ui.add_space(8.0_f32);
        }
    } else {
        theme::card_frame().show(ui, |ui| {
            ui.set_width(card_w);
            ui.label(theme::bold(
                format!("Konfigurasi Cloud ({})", app.prefs.provider),
                15.0_f32,
                INK,
            ));
            ui.add_space(10.0_f32);

            ui.label(theme::semi("Nama Model", 12.0_f32, INK_2));
            ui.add_space(4.0_f32);
            ui.add_sized(
                Vec2::new(card_w, 34.0_f32),
                egui::TextEdit::singleline(&mut app.prefs.custom_model)
                    .hint_text("Misal: gpt-4o-mini / gemini-2.0-flash / llama-3.3-70b-versatile")
                    .margin(Margin::symmetric(10, 7)),
            );

            ui.add_space(10.0_f32);

            ui.label(theme::semi(
                "API Key (disimpan terenkripsi Windows DPAPI)",
                12.0_f32,
                INK_2,
            ));
            ui.add_space(4.0_f32);
            ui.add_sized(
                Vec2::new(card_w, 34.0_f32),
                egui::TextEdit::singleline(&mut app.cloud_api_key_input)
                    .password(true)
                    .hint_text("Masukkan API key Anda...")
                    .margin(Margin::symmetric(10, 7)),
            );

            if app.prefs.provider == "custom" {
                ui.add_space(10.0_f32);
                ui.label(theme::semi(
                    "Endpoint URL (OpenAI-compatible)",
                    12.0_f32,
                    INK_2,
                ));
                ui.add_space(4.0_f32);
                ui.add_sized(
                    Vec2::new(card_w, 34.0_f32),
                    egui::TextEdit::singleline(&mut app.prefs.endpoint)
                        .hint_text("https://api.contoh.com/v1")
                        .margin(Margin::symmetric(10, 7)),
                );
            }

            ui.add_space(14.0_f32);
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
            ui.label(theme::semi(title, 13.5_f32, INK));
            ui.label(theme::regular(desc, 12.0_f32, INK_MUTED));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), control);
    });
    ui.add_space(6.0_f32);
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
