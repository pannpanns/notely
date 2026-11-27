use crate::app::NotelyApp;
use crate::theme::{
    self, ACCENT, ACCENT_BORDER, ACCENT_DEEP, ACCENT_SOFT, BORDER, DANGER, INK, INK_2, INK_MUTED,
    SURFACE, SURFACE_SUBTLE,
};
use crate::types::{format_ts, format_when_pretty, NoteTab, Page};
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, Sense, Stroke, Vec2,
};

pub fn show(app: &mut NotelyApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    let time = ctx.input(|i| i.time);
    let is_rec = app.is_recording();

    // ── Live Recording Banner ──
    if is_rec {
        Frame::new()
            .fill(ACCENT_SOFT)
            .stroke(Stroke::new(1.0_f32, ACCENT_BORDER))
            .inner_margin(Margin::symmetric(20, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let pulse =
                        (0.55_f32 + 0.45_f32 * (time * 5.0).sin() as f32).clamp(0.35_f32, 1.0_f32);
                    let dot = Color32::from_rgba_unmultiplied(
                        DANGER.r(),
                        DANGER.g(),
                        DANGER.b(),
                        (pulse * 255.0_f32) as u8,
                    );
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(10.0_f32), Sense::hover());
                    ui.painter().circle_filled(r.center(), 4.2_f32, dot);
                    ui.label(theme::semi(
                        "Mendengarkan secara langsung",
                        12.5_f32,
                        ACCENT_DEEP,
                    ));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(theme::mono(
                            format_ts(app.status.elapsed.unwrap_or(0.0)),
                            12.5_f32,
                            ACCENT_DEEP,
                        ));
                    });
                });
            });
    }

    // ── Top Segmented Switcher & Session Actions ──
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .inner_margin(Margin::symmetric(20, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                Frame::new()
                    .fill(SURFACE_SUBTLE)
                    .stroke(Stroke::new(1.0_f32, BORDER))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::same(3))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0_f32;
                            for (tab, label) in [
                                (NoteTab::Transcript, "Transkrip"),
                                (NoteTab::Summary, "Ringkasan"),
                            ] {
                                let active = app.note_tab == tab;
                                let btn = egui::Button::new(theme::semi(
                                    label,
                                    12.5_f32,
                                    if active { INK } else { INK_MUTED },
                                ))
                                .fill(if active { SURFACE } else { Color32::TRANSPARENT })
                                .stroke(if active {
                                    Stroke::new(1.0_f32, BORDER)
                                } else {
                                    Stroke::NONE
                                })
                                .corner_radius(CornerRadius::same(8))
                                .min_size(Vec2::new(92.0_f32, 29.0_f32));
                                let resp = ui.add(btn);
                                if resp.hovered() {
                                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if resp.clicked() {
                                    app.note_tab = tab;
                                }
                            }
                        });
                    });

                ui.add_space(10.0_f32);

                if let Some(sess) = &app.active_session {
                    ui.vertical(|ui| {
                        ui.label(theme::semi(&sess.title, 13.5_f32, INK));
                        ui.label(theme::regular(
                            format_when_pretty(&sess.when),
                            11.0_f32,
                            INK_MUTED,
                        ));
                    });
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if app.active_session.is_some() {
                        if theme::secondary_btn(ui, "Salin").clicked() {
                            app.copy_active_note();
                        }
                        if app.note_tab == NoteTab::Transcript {
                            if theme::secondary_btn(ui, "Edit Transkrip").clicked() {
                                app.open_edit_transcript_modal();
                            }
                            if !is_rec && theme::secondary_btn(ui, "Proses Ulang").clicked() {
                                app.reprocess_active_session();
                            }
                        }
                    }
                });
            });
        });

    let avail_w = ui.available_width();
    let content_w = avail_w.min(780.0_f32).max(320.0_f32);
    let side_pad = ((avail_w - content_w) * 0.5_f32).max(16.0_f32);

    match app.note_tab {
        NoteTab::Transcript => {
            show_transcript_tab(app, ctx, ui, content_w, side_pad, time);
        }
        NoteTab::Summary => {
            show_summary_tab(app, ctx, ui, content_w, side_pad, time);
        }
    }

    if app.show_edit_transcript_modal {
        show_edit_modal(app, ctx);
    }
}

fn show_transcript_tab(
    app: &mut NotelyApp,
    _ctx: &egui::Context,
    ui: &mut egui::Ui,
    content_w: f32,
    side_pad: f32,
    time: f64,
) {
    let is_rec = app.is_recording();

    egui::ScrollArea::vertical()
        .id_salt("notes_transcript_scroll")
        .auto_shrink([false, false])
        .stick_to_bottom(is_rec)
        .show(ui, |ui| {
            ui.add_space(18.0_f32);
            ui.horizontal(|ui| {
                ui.add_space(side_pad);
                ui.allocate_ui_with_layout(
                    Vec2::new(content_w, 0.0_f32),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        if is_rec {
                            ui.horizontal(|ui| {
                                ui.label(theme::bold("Transkrip Langsung", 16.5_f32, INK));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let count = app.live_segments.len();
                                    Frame::new()
                                        .fill(SURFACE)
                                        .stroke(Stroke::new(1.0_f32, BORDER))
                                        .corner_radius(CornerRadius::same(8))
                                        .inner_margin(Margin::symmetric(9, 3))
                                        .show(ui, |ui| {
                                            ui.label(theme::semi(
                                                format!("{} bagian", count),
                                                11.5_f32,
                                                INK_2,
                                            ));
                                        });
                                });
                            });
                            ui.add_space(10.0_f32);

                            for seg in &app.live_segments {
                                draw_transcript_card(ui, content_w, seg.start, &seg.text, false, time);
                                ui.add_space(8.0_f32);
                            }

                            if let Some(draft) = app.status.preview.last() {
                                if !draft.text.trim().is_empty() {
                                    draw_transcript_card(
                                        ui,
                                        content_w,
                                        draft.start,
                                        &draft.text,
                                        true,
                                        time,
                                    );
                                    ui.add_space(8.0_f32);
                                }
                            } else if app.live_segments.is_empty() {
                                theme::card_frame().show(ui, |ui| {
                                    ui.set_width(content_w - 38.0_f32);
                                    ui.horizontal(|ui| {
                                        theme::draw_wave_bars(ui, time, ACCENT);
                                        ui.add_space(6.0_f32);
                                        ui.label(theme::regular(
                                            "Mendengarkan percakapan... Teks akan muncul otomatis saat suara terdeteksi.",
                                            13.0_f32,
                                            INK_2,
                                        ));
                                    });
                                });
                            }
                        } else if let Some(sess) = app.active_session.clone() {
                            let rows = if !sess.rows.is_empty() {
                                &sess.rows
                            } else {
                                &sess.segments
                            };

                            ui.horizontal(|ui| {
                                ui.label(theme::bold("Transkrip", 16.5_f32, INK));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let count = rows.len();
                                    Frame::new()
                                        .fill(SURFACE)
                                        .stroke(Stroke::new(1.0_f32, BORDER))
                                        .corner_radius(CornerRadius::same(8))
                                        .inner_margin(Margin::symmetric(9, 3))
                                        .show(ui, |ui| {
                                            ui.label(theme::semi(
                                                format!("{} bagian", count),
                                                11.5_f32,
                                                INK_2,
                                            ));
                                        });
                                });
                            });
                            ui.add_space(10.0_f32);

                            if !rows.is_empty() && !sess.edited {
                                for r in rows {
                                    if !r.text.trim().is_empty() {
                                        draw_transcript_card(ui, content_w, r.start, &r.text, false, time);
                                        ui.add_space(8.0_f32);
                                    }
                                }
                            } else if !sess.transcript.trim().is_empty() {
                                theme::card_frame().show(ui, |ui| {
                                    ui.set_width(content_w - 38.0_f32);
                                    ui.label(theme::regular(&sess.transcript, 13.5_f32, INK));
                                });
                            } else {
                                theme::card_frame().show(ui, |ui| {
                                    ui.set_width(content_w - 38.0_f32);
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(14.0_f32);
                                        ui.label(theme::regular(
                                            "Transkrip belum tersedia untuk rekaman ini.",
                                            13.5_f32,
                                            INK_MUTED,
                                        ));
                                        ui.add_space(14.0_f32);
                                    });
                                });
                            }
                        } else {
                            ui.add_space(36.0_f32);
                            theme::card_frame().show(ui, |ui| {
                                ui.set_width(content_w - 38.0_f32);
                                ui.vertical_centered(|ui| {
                                    ui.add_space(18.0_f32);
                                    ui.label(theme::bold(
                                        "Belum ada rekaman yang dibuka",
                                        17.0_f32,
                                        INK,
                                    ));
                                    ui.add_space(6.0_f32);
                                    ui.label(theme::regular(
                                        "Mulai rekaman baru atau buka salah satu rekaman tersimpan untuk membaca transkrip & membuat ringkasan AI.",
                                        13.0_f32,
                                        INK_2,
                                    ));
                                    ui.add_space(16.0_f32);
                                    ui.horizontal(|ui| {
                                        let btn_w = 240.0_f32;
                                        ui.add_space(((ui.available_width() - btn_w) * 0.5_f32).max(0.0_f32));
                                        if theme::primary_btn(ui, "Mulai Rekam").clicked() {
                                            app.start_recording();
                                        }
                                        if theme::secondary_btn(ui, "Lihat Semua Rekaman").clicked() {
                                            app.page = Page::Recordings;
                                        }
                                    });
                                    ui.add_space(18.0_f32);
                                });
                            });
                        }

                        ui.add_space(96.0_f32);
                    },
                );
            });
        });
}

fn draw_transcript_card(
    ui: &mut egui::Ui,
    width: f32,
    start: f64,
    text: &str,
    is_live_draft: bool,
    time: f64,
) {
    let frame = if is_live_draft {
        Frame::new()
            .fill(ACCENT_SOFT)
            .stroke(Stroke::new(1.2_f32, ACCENT_BORDER))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::symmetric(16, 12))
    } else {
        Frame::new()
            .fill(SURFACE)
            .stroke(Stroke::new(1.0_f32, BORDER))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::symmetric(16, 12))
    };

    frame.show(ui, |ui| {
        ui.set_width(width - 34.0_f32);
        ui.horizontal(|ui| {
            Frame::new()
                .fill(if is_live_draft { SURFACE } else { SURFACE_SUBTLE })
                .stroke(Stroke::new(1.0_f32, BORDER))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(7, 2))
                .show(ui, |ui| {
                    ui.label(theme::mono(
                        format!("[{}]", format_ts(start)),
                        11.0_f32,
                        ACCENT_DEEP,
                    ));
                });

            if is_live_draft {
                ui.add_space(6.0_f32);
                Frame::new()
                    .fill(SURFACE)
                    .stroke(Stroke::new(1.0_f32, ACCENT_BORDER))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(8, 2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            theme::draw_wave_bars(ui, time, ACCENT);
                            ui.label(theme::semi("Mengetik langsung", 11.0_f32, ACCENT_DEEP));
                        });
                    });
            }
        });

        ui.add_space(6.0_f32);

        if is_live_draft {
            let caret = if ((time * 2.2) as i64) % 2 == 0 {
                "|"
            } else {
                ""
            };
            ui.label(theme::regular(
                format!("{}{}", text.trim(), caret),
                13.5_f32,
                INK,
            ));
        } else {
            ui.label(theme::regular(text.trim(), 13.5_f32, INK));
        }
    });
}

fn show_summary_tab(
    app: &mut NotelyApp,
    _ctx: &egui::Context,
    ui: &mut egui::Ui,
    content_w: f32,
    side_pad: f32,
    time: f64,
) {
    let composer_h = 128.0_f32;
    let scroll_h = (ui.available_height() - composer_h).max(160.0_f32);

    egui::ScrollArea::vertical()
        .id_salt("notes_summary_scroll")
        .max_height(scroll_h)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(18.0_f32);
            ui.horizontal(|ui| {
                ui.add_space(side_pad);
                ui.allocate_ui_with_layout(
                    Vec2::new(content_w, 0.0_f32),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        if app.summary_running {
                            theme::card_frame().show(ui, |ui| {
                                ui.set_width(content_w - 38.0_f32);
                                ui.horizontal(|ui| {
                                    theme::draw_orbit_loader(ui, 12.0_f32, time);
                                    ui.add_space(8.0_f32);
                                    ui.vertical(|ui| {
                                        ui.label(theme::bold(
                                            "Menyusun ringkasan AI...",
                                            14.0_f32,
                                            INK,
                                        ));
                                        if !app.summary_status_msg.is_empty() {
                                            ui.label(theme::regular(
                                                &app.summary_status_msg,
                                                12.0_f32,
                                                INK_2,
                                            ));
                                        }
                                    });
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if theme::danger_soft_btn(ui, "Batalkan").clicked() {
                                            app.cancel_summary();
                                        }
                                    });
                                });

                                if !app.summary_stream_text.trim().is_empty() {
                                    ui.add_space(12.0_f32);
                                    ui.separator();
                                    ui.add_space(10.0_f32);
                                    theme::render_markdown_clean(ui, &app.summary_stream_text);
                                }
                            });
                        } else if let Some(sess) = app.active_session.clone() {
                            if !sess.summary.trim().is_empty() {
                                theme::card_frame().show(ui, |ui| {
                                    ui.set_width(content_w - 38.0_f32);
                                    theme::render_markdown_clean(ui, &sess.summary);
                                });
                            } else {
                                theme::card_frame().show(ui, |ui| {
                                    ui.set_width(content_w - 38.0_f32);
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(18.0_f32);
                                        ui.label(theme::bold(
                                            "Ringkasan belum dibuat",
                                            16.0_f32,
                                            INK,
                                        ));
                                        ui.add_space(6.0_f32);
                                        ui.label(theme::regular(
                                            "Pilih gaya ringkasan atau tulis instruksi khusus pada Prompt AI di bawah.",
                                            13.0_f32,
                                            INK_2,
                                        ));
                                        ui.add_space(18.0_f32);
                                    });
                                });
                            }
                        } else {
                            theme::card_frame().show(ui, |ui| {
                                ui.set_width(content_w - 38.0_f32);
                                ui.vertical_centered(|ui| {
                                    ui.add_space(18.0_f32);
                                    ui.label(theme::regular(
                                        "Buka rekaman terlebih dahulu untuk membuat ringkasan AI.",
                                        13.5_f32,
                                        INK_MUTED,
                                    ));
                                    ui.add_space(18.0_f32);
                                });
                            });
                        }

                        ui.add_space(20.0_f32);
                    },
                );
            });
        });

    // ── Bottom-Anchored Prompt AI Composer (Flush at bottom) ──
    ui.add_space(4.0_f32);
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(CornerRadius {
            nw: 0,
            ne: 0,
            sw: 16,
            se: 16,
        })
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space((side_pad - 16.0_f32).max(0.0_f32));
                ui.allocate_ui_with_layout(
                    Vec2::new(content_w, 0.0_f32),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.label(theme::semi("Prompt AI", 12.5_f32, ACCENT_DEEP));
                            ui.add_space(8.0_f32);
                            for (key, lbl) in [
                                ("materi", "Materi"),
                                ("rapat", "Rapat"),
                                ("poin", "Poin Utama"),
                                ("tanya", "Tanya Jawab"),
                            ] {
                                let selected = app.prefs.template == key;
                                let btn = egui::Button::new(theme::semi(
                                    lbl,
                                    11.5_f32,
                                    if selected { ACCENT_DEEP } else { INK_2 },
                                ))
                                .fill(if selected { ACCENT_SOFT } else { SURFACE_SUBTLE })
                                .stroke(Stroke::new(
                                    1.0_f32,
                                    if selected { ACCENT_BORDER } else { BORDER },
                                ))
                                .corner_radius(CornerRadius::same(12))
                                .min_size(Vec2::new(0.0_f32, 24.0_f32));
                                if ui.add(btn).clicked() {
                                    app.prefs.template = key.to_string();
                                    app.save_local_prefs();
                                }
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let model_lbl = if app.prefs.provider == "ollama" {
                                    format!("Lokal: {}", app.prefs.summary_model)
                                } else {
                                    format!("Cloud: {}", app.prefs.provider)
                                };
                                ui.label(theme::regular(model_lbl, 11.0_f32, INK_MUTED));
                            });
                        });

                        ui.add_space(8.0_f32);

                        ui.horizontal(|ui| {
                            let input_w = (content_w - 146.0_f32).max(160.0_f32);
                            let resp = ui.add_sized(
                                Vec2::new(input_w, 38.0_f32),
                                egui::TextEdit::singleline(&mut app.summary_prompt)
                                    .hint_text("Tulis instruksi tambahan (opsional), misal: 'Buat poin persiapan ujian'...")
                                    .margin(Margin::symmetric(10, 9)),
                            );
                            let enter_pressed = resp.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter));

                            let can_run = app.active_session.is_some()
                                && !app.summary_running
                                && !app.is_recording();
                            ui.add_enabled_ui(can_run, |ui| {
                                let btn_lbl = if app.summary_running {
                                    "Memproses..."
                                } else {
                                    "Buat Ringkasan"
                                };
                                if theme::primary_btn_sized(
                                    ui,
                                    btn_lbl,
                                    Vec2::new(134.0_f32, 38.0_f32),
                                )
                                .clicked()
                                    || (enter_pressed && can_run)
                                {
                                    app.run_summary();
                                }
                            });
                        });
                    },
                );
            });
        });
}

fn show_edit_modal(app: &mut NotelyApp, ctx: &egui::Context) {
    egui::Area::new(egui::Id::new("edit_transcript_scrim"))
        .order(egui::Order::Middle)
        .fixed_pos(egui::Pos2::ZERO)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let resp = ui.allocate_rect(screen, Sense::click());
            ui.painter()
                .rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(110));
            if resp.clicked() {
                app.show_edit_transcript_modal = false;
            }
        });

    egui::Window::new("Edit Transkrip")
        .id(egui::Id::new("edit_transcript_modal"))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(660.0_f32, 480.0_f32))
        .frame(theme::modal_frame())
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(theme::bold("Edit Transkrip", 17.0_f32, INK));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::ghost_btn(ui, "Tutup").clicked() {
                        app.show_edit_transcript_modal = false;
                    }
                });
            });
            ui.add_space(10.0_f32);

            egui::ScrollArea::vertical()
                .max_height(350.0_f32)
                .show(ui, |ui| {
                    ui.add_sized(
                        Vec2::new(ui.available_width(), 340.0_f32),
                        egui::TextEdit::multiline(&mut app.edit_transcript_buf)
                            .desired_width(f32::INFINITY)
                            .margin(Margin::same(10)),
                    );
                });

            ui.add_space(14.0_f32);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::primary_btn(ui, "Simpan Perubahan").clicked() {
                        app.save_edited_transcript();
                    }
                    if theme::secondary_btn(ui, "Batal").clicked() {
                        app.show_edit_transcript_modal = false;
                    }
                });
            });
        });
}
