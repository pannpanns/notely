use crate::app::NotelyApp;
use crate::theme::{
    self, ACCENT, ACCENT_BORDER, ACCENT_DEEP, ACCENT_SOFT, BORDER, DANGER, INK, INK_2, INK_MUTED,
    SURFACE, SURFACE_SUBTLE,
};
use crate::types::{format_ts, format_when_pretty, NoteTab, Page};
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, RichText, Sense, Stroke, Vec2,
};

pub fn show(app: &mut NotelyApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    let time = ctx.input(|i| i.time);
    let is_rec = app.is_recording();

    // ── Live Recording Banner ──
    if is_rec {
        Frame::new()
            .fill(ACCENT_SOFT)
            .stroke(Stroke::new(1.0, ACCENT_BORDER))
            .inner_margin(Margin::symmetric(20, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let pulse = (0.55 + 0.45 * (time * 5.0).sin() as f32).clamp(0.35, 1.0);
                    let dot = Color32::from_rgba_unmultiplied(
                        DANGER.r(),
                        DANGER.g(),
                        DANGER.b(),
                        (pulse * 255.0) as u8,
                    );
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                    ui.painter().circle_filled(r.center(), 4.2, dot);
                    ui.label(
                        RichText::new("Mendengarkan secara langsung")
                            .size(12.5)
                            .color(ACCENT_DEEP)
                            .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format_ts(app.status.elapsed.unwrap_or(0.0)))
                                .monospace()
                                .size(12.5)
                                .color(ACCENT_DEEP)
                                .strong(),
                        );
                    });
                });
            });
    }

    // ── Top Segmented Switcher & Session Actions ──
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(Margin::symmetric(20, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Clean Neutral Segmented Switcher (Transkrip | Ringkasan)
                Frame::new()
                    .fill(SURFACE_SUBTLE)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::same(3))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            for (tab, label) in [
                                (NoteTab::Transcript, "Transkrip"),
                                (NoteTab::Summary, "Ringkasan"),
                            ] {
                                let active = app.note_tab == tab;
                                let btn = egui::Button::new(
                                    RichText::new(label)
                                        .size(12.5)
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
                                .min_size(Vec2::new(88.0, 28.0));
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

                ui.add_space(10.0);

                if let Some(sess) = &app.active_session {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&sess.title).size(13.5).color(INK).strong());
                        ui.label(
                            RichText::new(format_when_pretty(&sess.when))
                                .size(11.0)
                                .color(INK_MUTED),
                        );
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

    // ── Main Content Area ──
    let avail_w = ui.available_width();
    let content_w = avail_w.min(780.0).max(320.0);
    let side_pad = ((avail_w - content_w) * 0.5).max(16.0);

    match app.note_tab {
        NoteTab::Transcript => {
            show_transcript_tab(app, ctx, ui, content_w, side_pad, time);
        }
        NoteTab::Summary => {
            show_summary_tab(app, ctx, ui, content_w, side_pad, time);
        }
    }

    // ── Edit Transcript Modal ──
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
            ui.add_space(18.0);
            ui.horizontal(|ui| {
                ui.add_space(side_pad);
                ui.allocate_ui_with_layout(
                    Vec2::new(content_w, 0.0),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        // Case 1: Active live recording or live preview available
                        if is_rec {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Transkrip Langsung")
                                        .size(16.0)
                                        .color(INK)
                                        .strong(),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let count = app.live_segments.len();
                                    Frame::new()
                                        .fill(SURFACE)
                                        .stroke(Stroke::new(1.0, BORDER))
                                        .corner_radius(CornerRadius::same(8))
                                        .inner_margin(Margin::symmetric(9, 3))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format!("{} bagian", count))
                                                    .size(11.5)
                                                    .color(INK_2),
                                            );
                                        });
                                });
                            });
                            ui.add_space(10.0);

                            for seg in &app.live_segments {
                                draw_transcript_card(ui, content_w, seg.start, &seg.text, false, time);
                                ui.add_space(8.0);
                            }

                            // Streaming draft card
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
                                    ui.add_space(8.0);
                                }
                            } else if app.live_segments.is_empty() {
                                theme::card_frame().show(ui, |ui| {
                                    ui.set_width(content_w - 34.0);
                                    ui.horizontal(|ui| {
                                        theme::draw_wave_bars(ui, time, ACCENT);
                                        ui.add_space(6.0);
                                        ui.label(
                                            RichText::new(
                                                "Mendengarkan percakapan... Teks akan muncul otomatis saat suara terdeteksi.",
                                            )
                                            .size(13.0)
                                            .color(INK_2),
                                        );
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
                                ui.label(
                                    RichText::new("Transkrip").size(16.0).color(INK).strong(),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let count = rows.len();
                                    Frame::new()
                                        .fill(SURFACE)
                                        .stroke(Stroke::new(1.0, BORDER))
                                        .corner_radius(CornerRadius::same(8))
                                        .inner_margin(Margin::symmetric(9, 3))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format!("{} bagian", count))
                                                    .size(11.5)
                                                    .color(INK_2),
                                            );
                                        });
                                });
                            });
                            ui.add_space(10.0);

                            if !rows.is_empty() && !sess.edited {
                                for r in rows {
                                    if !r.text.trim().is_empty() {
                                        draw_transcript_card(ui, content_w, r.start, &r.text, false, time);
                                        ui.add_space(8.0);
                                    }
                                }
                            } else if !sess.transcript.trim().is_empty() {
                                theme::card_frame().show(ui, |ui| {
                                    ui.set_width(content_w - 34.0);
                                    ui.label(
                                        RichText::new(&sess.transcript).size(13.5).color(INK),
                                    );
                                });
                            } else {
                                theme::card_frame().show(ui, |ui| {
                                    ui.set_width(content_w - 34.0);
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(12.0);
                                        ui.label(
                                            RichText::new("Transkrip belum tersedia untuk rekaman ini.")
                                                .size(13.5)
                                                .color(INK_MUTED),
                                        );
                                        ui.add_space(12.0);
                                    });
                                });
                            }
                        } else {
                            // Empty state
                            ui.add_space(36.0);
                            theme::card_frame().show(ui, |ui| {
                                ui.set_width(content_w - 34.0);
                                ui.vertical_centered(|ui| {
                                    ui.add_space(16.0);
                                    ui.label(
                                        RichText::new("Belum ada rekaman yang dibuka")
                                            .size(17.0)
                                            .color(INK)
                                            .strong(),
                                    );
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new(
                                            "Mulai rekaman baru atau buka salah satu rekaman tersimpan untuk membaca transkrip & membuat ringkasan AI.",
                                        )
                                        .size(13.0)
                                        .color(INK_2),
                                    );
                                    ui.add_space(16.0);
                                    ui.horizontal(|ui| {
                                        let btn_w = 240.0;
                                        ui.add_space(((ui.available_width() - btn_w) * 0.5).max(0.0));
                                        if theme::primary_btn(ui, "Mulai Rekam").clicked() {
                                            app.start_recording();
                                        }
                                        if theme::secondary_btn(ui, "Lihat Semua Rekaman").clicked() {
                                            app.page = Page::Recordings;
                                        }
                                    });
                                    ui.add_space(16.0);
                                });
                            });
                        }

                        ui.add_space(96.0);
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
            .stroke(Stroke::new(1.2, ACCENT_BORDER))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::symmetric(16, 12))
    } else {
        Frame::new()
            .fill(SURFACE)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::symmetric(16, 12))
    };

    frame.show(ui, |ui| {
        ui.set_width(width - 34.0);
        ui.horizontal(|ui| {
            Frame::new()
                .fill(if is_live_draft { SURFACE } else { SURFACE_SUBTLE })
                .stroke(Stroke::new(1.0, BORDER))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(7, 2))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("[{}]", format_ts(start)))
                            .monospace()
                            .size(11.0)
                            .color(ACCENT_DEEP)
                            .strong(),
                    );
                });

            if is_live_draft {
                ui.add_space(6.0);
                Frame::new()
                    .fill(SURFACE)
                    .stroke(Stroke::new(1.0, ACCENT_BORDER))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(8, 2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            theme::draw_wave_bars(ui, time, ACCENT);
                            ui.label(
                                RichText::new("Mengetik langsung")
                                    .size(11.0)
                                    .color(ACCENT_DEEP)
                                    .strong(),
                            );
                        });
                    });
            }
        });

        ui.add_space(6.0);

        if is_live_draft {
            let caret = if ((time * 2.2) as i64) % 2 == 0 {
                "|"
            } else {
                ""
            };
            ui.label(
                RichText::new(format!("{}{}", text.trim(), caret))
                    .size(13.5)
                    .color(INK),
            );
        } else {
            ui.label(RichText::new(text.trim()).size(13.5).color(INK));
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
    // Reserve clean fixed height at bottom for the Prompt AI composer (NEVER floating at top!)
    let composer_h = 128.0;
    let scroll_h = (ui.available_height() - composer_h).max(160.0);

    // ── Scrollable Summary Body ──
    egui::ScrollArea::vertical()
        .id_salt("notes_summary_scroll")
        .max_height(scroll_h)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(18.0);
            ui.horizontal(|ui| {
                ui.add_space(side_pad);
                ui.allocate_ui_with_layout(
                    Vec2::new(content_w, 0.0),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        if app.summary_running {
                            theme::card_frame().show(ui, |ui| {
                                ui.set_width(content_w - 34.0);
                                ui.horizontal(|ui| {
                                    theme::draw_orbit_loader(ui, 12.0, time);
                                    ui.add_space(8.0);
                                    ui.vertical(|ui| {
                                        ui.label(
                                            RichText::new("Menyusun ringkasan AI...")
                                                .size(14.0)
                                                .color(INK)
                                                .strong(),
                                        );
                                        if !app.summary_status_msg.is_empty() {
                                            ui.label(
                                                RichText::new(&app.summary_status_msg)
                                                    .size(12.0)
                                                    .color(INK_2),
                                            );
                                        }
                                    });
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if theme::danger_soft_btn(ui, "Batalkan").clicked() {
                                            app.cancel_summary();
                                        }
                                    });
                                });

                                if !app.summary_stream_text.trim().is_empty() {
                                    ui.add_space(12.0);
                                    ui.separator();
                                    ui.add_space(10.0);
                                    theme::render_markdown_clean(ui, &app.summary_stream_text);
                                }
                            });
                        } else if let Some(sess) = app.active_session.clone() {
                            if !sess.summary.trim().is_empty() {
                                theme::card_frame().show(ui, |ui| {
                                    ui.set_width(content_w - 34.0);
                                    theme::render_markdown_clean(ui, &sess.summary);
                                });
                            } else {
                                theme::card_frame().show(ui, |ui| {
                                    ui.set_width(content_w - 34.0);
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(18.0);
                                        ui.label(
                                            RichText::new("Ringkasan belum dibuat")
                                                .size(16.0)
                                                .color(INK)
                                                .strong(),
                                        );
                                        ui.add_space(6.0);
                                        ui.label(
                                            RichText::new(
                                                "Pilih gaya ringkasan atau tulis instruksi khusus pada Prompt AI di bawah.",
                                            )
                                            .size(13.0)
                                            .color(INK_2),
                                        );
                                        ui.add_space(18.0);
                                    });
                                });
                            }
                        } else {
                            theme::card_frame().show(ui, |ui| {
                                ui.set_width(content_w - 34.0);
                                ui.vertical_centered(|ui| {
                                    ui.add_space(18.0);
                                    ui.label(
                                        RichText::new("Buka rekaman terlebih dahulu untuk membuat ringkasan AI.")
                                            .size(13.5)
                                            .color(INK_MUTED),
                                    );
                                    ui.add_space(18.0);
                                });
                            });
                        }

                        ui.add_space(20.0);
                    },
                );
            });
        });

    // ── Bottom-Anchored Prompt AI Composer (Flush at bottom!) ──
    ui.add_space(4.0);
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space((side_pad - 16.0).max(0.0));
                ui.allocate_ui_with_layout(
                    Vec2::new(content_w, 0.0),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        // Top Row: Template selector + Model info
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Prompt AI").size(12.5).color(ACCENT_DEEP).strong());
                            ui.add_space(8.0);
                            for (key, lbl) in [
                                ("materi", "Materi"),
                                ("rapat", "Rapat"),
                                ("poin", "Poin Utama"),
                                ("tanya", "Tanya Jawab"),
                            ] {
                                let selected = app.prefs.template == key;
                                let btn = egui::Button::new(
                                    RichText::new(lbl)
                                        .size(11.5)
                                        .color(if selected { ACCENT_DEEP } else { INK_2 }),
                                )
                                .fill(if selected { ACCENT_SOFT } else { SURFACE_SUBTLE })
                                .stroke(Stroke::new(
                                    1.0,
                                    if selected { ACCENT_BORDER } else { BORDER },
                                ))
                                .corner_radius(CornerRadius::same(12))
                                .min_size(Vec2::new(0.0, 24.0));
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
                                ui.label(RichText::new(model_lbl).size(11.0).color(INK_MUTED));
                            });
                        });

                        ui.add_space(8.0);

                        // Prompt Input + Generate Button
                        ui.horizontal(|ui| {
                            let input_w = (content_w - 146.0).max(160.0);
                            let resp = ui.add_sized(
                                Vec2::new(input_w, 38.0),
                                egui::TextEdit::singleline(&mut app.summary_prompt)
                                    .hint_text("Tulis instruksi tambahan (opsional), misal: 'Buat poin persiapan ujian'...")
                                    .margin(Margin::symmetric(10, 9)),
                            );
                            let enter_pressed = resp.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter));

                            let can_run =
                                app.active_session.is_some() && !app.summary_running && !app.is_recording();
                            ui.add_enabled_ui(can_run, |ui| {
                                let btn_lbl = if app.summary_running {
                                    "Memproses..."
                                } else {
                                    "Buat Ringkasan"
                                };
                                if theme::primary_btn_sized(ui, btn_lbl, Vec2::new(134.0, 38.0))
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
    // Scrim backdrop
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
        .fixed_size(Vec2::new(660.0, 480.0))
        .frame(theme::modal_frame())
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Edit Transkrip").size(17.0).color(INK).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::ghost_btn(ui, "Tutup").clicked() {
                        app.show_edit_transcript_modal = false;
                    }
                });
            });
            ui.add_space(10.0);

            egui::ScrollArea::vertical()
                .max_height(350.0)
                .show(ui, |ui| {
                    ui.add_sized(
                        Vec2::new(ui.available_width(), 340.0),
                        egui::TextEdit::multiline(&mut app.edit_transcript_buf)
                            .desired_width(f32::INFINITY)
                            .margin(Margin::same(10)),
                    );
                });

            ui.add_space(14.0);
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
