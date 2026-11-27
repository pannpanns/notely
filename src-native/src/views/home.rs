use crate::app::NotelyApp;
use crate::theme::{
    self, ACCENT, ACCENT_DEEP, ACCENT_HOVER, ACCENT_SOFT, AMBER, BORDER, DANGER, INK, INK_2,
    INK_MUTED, SURFACE, SURFACE_HOVER, SURFACE_SUBTLE,
};
use crate::types::{format_when_pretty, Page};
use chrono::{Datelike, Local};
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, RichText, Sense, Stroke, Vec2,
};

fn indonesian_today() -> String {
    let now = Local::now();
    let day_name = match now.weekday() {
        chrono::Weekday::Mon => "Senin",
        chrono::Weekday::Tue => "Selasa",
        chrono::Weekday::Wed => "Rabu",
        chrono::Weekday::Thu => "Kamis",
        chrono::Weekday::Fri => "Jumat",
        chrono::Weekday::Sat => "Sabtu",
        chrono::Weekday::Sun => "Minggu",
    };
    let months = [
        "Januari",
        "Februari",
        "Maret",
        "April",
        "Mei",
        "Juni",
        "Juli",
        "Agustus",
        "September",
        "Oktober",
        "November",
        "Desember",
    ];
    let mon = months
        .get(now.month0() as usize)
        .copied()
        .unwrap_or("Januari");
    format!("{}, {} {} {}", day_name, now.day(), mon, now.year())
}

fn short_month_upper(m: u32) -> &'static str {
    const M: [&str; 12] = [
        "JAN", "FEB", "MAR", "APR", "MEI", "JUN", "JUL", "AGU", "SEP", "OKT", "NOV", "DES",
    ];
    M.get(m.saturating_sub(1) as usize).copied().unwrap_or("")
}

pub fn show(app: &mut NotelyApp, _ctx: &egui::Context, ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    let content_w = avail_w.min(980.0).max(320.0);
    let side_pad = ((avail_w - content_w) * 0.5).max(16.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.add_space(side_pad);
                ui.allocate_ui_with_layout(
                    Vec2::new(content_w, 0.0),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        // ── Date Pill ──
                        Frame::new()
                            .fill(SURFACE)
                            .stroke(Stroke::new(1.0, BORDER))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::symmetric(10, 5))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let (r, _) =
                                        ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                                    theme::draw_nav_icon(
                                        ui.painter(),
                                        r,
                                        theme::NavIcon::Calendar,
                                        ACCENT,
                                    );
                                    ui.label(
                                        RichText::new(indonesian_today())
                                            .size(12.0)
                                            .color(INK_2)
                                            .strong(),
                                    );
                                });
                            });

                        ui.add_space(12.0);

                        // ── Header + Stat Chips ──
                        let active_deadlines: Vec<_> = app
                            .deadlines
                            .iter()
                            .filter(|d| d.status != "completed")
                            .cloned()
                            .collect();
                        let urgent_count = active_deadlines
                            .iter()
                            .filter(|d| d.priority == "high")
                            .count();

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("Ruang catatan dan jadwal Anda.")
                                        .size(25.0)
                                        .color(INK)
                                        .strong(),
                                );
                                ui.add_space(2.0);
                                ui.label(
                                    RichText::new(
                                        "Mulai rekaman baru, kelola transkrip, atau susun tenggat tugas.",
                                    )
                                    .size(13.5)
                                    .color(INK_2),
                                );
                            });

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                // Urgent / Active tasks chip
                                Frame::new()
                                    .fill(SURFACE)
                                    .stroke(Stroke::new(1.0, BORDER))
                                    .corner_radius(CornerRadius::same(12))
                                    .inner_margin(Margin::symmetric(16, 10))
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            let num_col = if urgent_count > 0 {
                                                DANGER
                                            } else {
                                                ACCENT
                                            };
                                            ui.label(
                                                RichText::new(active_deadlines.len().to_string())
                                                    .size(20.0)
                                                    .color(num_col)
                                                    .strong(),
                                            );
                                            let sub = if urgent_count > 0 {
                                                format!("{} mendesak", urgent_count)
                                            } else {
                                                "Tugas aktif".to_string()
                                            };
                                            ui.label(
                                                RichText::new(sub).size(11.5).color(INK_MUTED),
                                            );
                                        });
                                    });

                                ui.add_space(8.0);

                                // Total recordings chip
                                Frame::new()
                                    .fill(SURFACE)
                                    .stroke(Stroke::new(1.0, BORDER))
                                    .corner_radius(CornerRadius::same(12))
                                    .inner_margin(Margin::symmetric(16, 10))
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new(app.sessions.len().to_string())
                                                    .size(20.0)
                                                    .color(INK)
                                                    .strong(),
                                            );
                                            ui.label(
                                                RichText::new("Total rekaman")
                                                    .size(11.5)
                                                    .color(INK_MUTED),
                                            );
                                        });
                                    });
                            });
                        });

                        ui.add_space(20.0);

                        // ── Asymmetric Action Bento ──
                        let narrow = content_w < 680.0;
                        if narrow {
                            show_hero_record_card(app, ui, content_w);
                            ui.add_space(12.0);
                            show_import_action_card(app, ui, content_w);
                            ui.add_space(10.0);
                            show_deadline_action_card(app, ui, content_w, active_deadlines.len());
                        } else {
                            let gap = 14.0;
                            let left_w = (content_w - gap) * 0.54;
                            let right_w = content_w - gap - left_w;
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                show_hero_record_card(app, ui, left_w);
                                ui.allocate_ui_with_layout(
                                    Vec2::new(right_w, 178.0),
                                    Layout::top_down(Align::LEFT),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.y = 12.0;
                                        show_import_action_card(app, ui, right_w);
                                        show_deadline_action_card(
                                            app,
                                            ui,
                                            right_w,
                                            active_deadlines.len(),
                                        );
                                    },
                                );
                            });
                        }

                        ui.add_space(16.0);

                        // ── Bottom 2-Column Bento: Rekaman terakhir + Jadwal terdekat ──
                        let gap = 14.0;
                        let col_w = if narrow {
                            content_w
                        } else {
                            (content_w - gap) * 0.5
                        };

                        if narrow {
                            show_recent_recordings_card(app, ui, col_w);
                            ui.add_space(14.0);
                            show_upcoming_deadlines_card(app, ui, col_w, &active_deadlines);
                        } else {
                            ui.horizontal_top(|ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                show_recent_recordings_card(app, ui, col_w);
                                show_upcoming_deadlines_card(app, ui, col_w, &active_deadlines);
                            });
                        }

                        // Breathing room for floating dock
                        ui.add_space(92.0);
                    },
                );
            });
        });
}

fn show_hero_record_card(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    let is_rec = app.is_recording();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 178.0), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg = if is_rec {
        if resp.hovered() {
            theme::DANGER_HOVER
        } else {
            DANGER
        }
    } else if resp.hovered() {
        ACCENT_HOVER
    } else {
        ACCENT
    };

    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(16), bg);

    // Subtle organic highlight circle in top-right corner
    painter.circle_filled(
        egui::pos2(rect.right() - 28.0, rect.top() + 28.0),
        74.0,
        Color32::from_rgba_unmultiplied(255, 255, 255, 14),
    );

    // Inner layout
    let inner = rect.shrink(22.0);
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                // Soft translucent icon badge
                let (icon_box, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
                ui.painter().rect_filled(
                    icon_box,
                    CornerRadius::same(11),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 34),
                );
                theme::draw_nav_icon(
                    ui.painter(),
                    icon_box.shrink(9.0),
                    theme::NavIcon::Mic,
                    Color32::WHITE,
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(if is_rec { "Sedang merekam" } else { "Mulai" })
                            .size(12.0)
                            .color(Color32::from_rgba_unmultiplied(255, 255, 255, 210))
                            .strong(),
                    );
                });
            });

            ui.add_space(28.0);
            ui.label(
                RichText::new(if is_rec {
                    "Hentikan rekaman"
                } else {
                    "Rekam percakapan"
                })
                .size(20.0)
                .color(Color32::WHITE)
                .strong(),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(if is_rec {
                    "Klik untuk menghentikan rekaman & melihat hasil transkrip lengkap"
                } else {
                    "Tangkap audio komputer & mikrofon dengan transkripsi lokal cepat"
                })
                .size(12.5)
                .color(Color32::from_rgba_unmultiplied(234, 245, 240, 225)),
            );
        });
    });

    if resp.clicked() {
        if is_rec {
            app.stop_recording();
        } else {
            app.start_recording();
        }
    }
}

fn show_import_action_card(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 83.0), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let fill = if resp.hovered() { SURFACE_HOVER } else { SURFACE };
    let border = if resp.hovered() { ACCENT } else { BORDER };
    ui.painter().rect_filled(rect, CornerRadius::same(14), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );

    let inner = rect.shrink(16.0);
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
        ui.horizontal_centered(|ui| {
            let (icon_box, _) = ui.allocate_exact_size(Vec2::splat(38.0), Sense::hover());
            ui.painter()
                .rect_filled(icon_box, CornerRadius::same(10), ACCENT_SOFT);
            theme::draw_nav_icon(ui.painter(), icon_box.shrink(8.0), theme::NavIcon::Upload, ACCENT);
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("Impor rekaman").size(14.5).color(INK).strong());
                ui.label(
                    RichText::new("Ubah file audio atau video menjadi transkrip & ringkasan")
                        .size(12.0)
                        .color(INK_MUTED),
                );
            });
        });
    });

    if resp.clicked() {
        app.pick_and_import_audio("");
    }
}

fn show_deadline_action_card(
    app: &mut NotelyApp,
    ui: &mut egui::Ui,
    width: f32,
    active_count: usize,
) {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 83.0), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let fill = if resp.hovered() { SURFACE_HOVER } else { SURFACE };
    let border = if resp.hovered() { ACCENT } else { BORDER };
    ui.painter().rect_filled(rect, CornerRadius::same(14), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );

    let inner = rect.shrink(16.0);
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
        ui.horizontal_centered(|ui| {
            let (icon_box, _) = ui.allocate_exact_size(Vec2::splat(38.0), Sense::hover());
            ui.painter()
                .rect_filled(icon_box, CornerRadius::same(10), ACCENT_SOFT);
            theme::draw_nav_icon(
                ui.painter(),
                icon_box.shrink(8.0),
                theme::NavIcon::Calendar,
                ACCENT,
            );
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Tenggat & jadwal")
                        .size(14.5)
                        .color(INK)
                        .strong(),
                );
                let sub = if app.gcal.connected {
                    format!("{} jadwal aktif tersinkron Google Calendar", active_count)
                } else {
                    format!("{} jadwal aktif · Ekstrak otomatis dari teks & gambar", active_count)
                };
                ui.label(RichText::new(sub).size(12.0).color(INK_MUTED));
            });
        });
    });

    if resp.clicked() {
        app.page = Page::Deadlines;
    }
}

fn show_recent_recordings_card(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    ui.allocate_ui_with_layout(
        Vec2::new(width, 0.0),
        Layout::top_down(Align::LEFT),
        |ui| {
            theme::card_frame().show(ui, |ui| {
                ui.set_width(width - 34.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Rekaman terakhir")
                            .size(15.0)
                            .color(INK)
                            .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if theme::ghost_btn(ui, "Lihat semua").clicked() {
                            app.page = Page::Recordings;
                        }
                    });
                });

                ui.add_space(8.0);

                if app.sessions.is_empty() {
                    Frame::new()
                        .fill(SURFACE_SUBTLE)
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::symmetric(14, 22))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new("Belum ada rekaman tersimpan.")
                                        .size(13.0)
                                        .color(INK_MUTED),
                                );
                            });
                        });
                } else {
                    let preview: Vec<_> = app.sessions.iter().take(5).cloned().collect();
                    for item in preview {
                        let (row_rect, row_resp) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 54.0),
                            Sense::click(),
                        );
                        if row_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        let row_bg = if row_resp.hovered() {
                            SURFACE_HOVER
                        } else {
                            SURFACE_SUBTLE
                        };
                        ui.painter()
                            .rect_filled(row_rect, CornerRadius::same(10), row_bg);

                        let inner = row_rect.shrink2(Vec2::new(12.0, 8.0));
                        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
                            ui.horizontal_centered(|ui| {
                                let (ic, _) =
                                    ui.allocate_exact_size(Vec2::splat(30.0), Sense::hover());
                                ui.painter().rect_filled(ic, CornerRadius::same(8), SURFACE);
                                theme::draw_nav_icon(
                                    ui.painter(),
                                    ic.shrink(6.0),
                                    theme::NavIcon::Notes,
                                    ACCENT,
                                );
                                ui.add_space(6.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(&item.title)
                                            .size(13.0)
                                            .color(INK)
                                            .strong(),
                                    );
                                    ui.label(
                                        RichText::new(format_when_pretty(&item.when))
                                            .size(11.5)
                                            .color(INK_MUTED),
                                    );
                                });
                            });
                        });

                        if row_resp.clicked() {
                            app.open_session(&item.id);
                        }
                        ui.add_space(6.0);
                    }
                }
            });
        },
    );
}

fn show_upcoming_deadlines_card(
    app: &mut NotelyApp,
    ui: &mut egui::Ui,
    width: f32,
    active_deadlines: &[crate::types::DeadlineItem],
) {
    ui.allocate_ui_with_layout(
        Vec2::new(width, 0.0),
        Layout::top_down(Align::LEFT),
        |ui| {
            theme::card_frame().show(ui, |ui| {
                ui.set_width(width - 34.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Jadwal terdekat")
                            .size(15.0)
                            .color(INK)
                            .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if theme::ghost_btn(ui, "Buka kalender").clicked() {
                            app.page = Page::Deadlines;
                        }
                    });
                });

                ui.add_space(8.0);

                if active_deadlines.is_empty() {
                    Frame::new()
                        .fill(SURFACE_SUBTLE)
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::symmetric(14, 22))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new("Tidak ada tenggat mendatang.")
                                        .size(13.0)
                                        .color(INK_MUTED),
                                );
                            });
                        });
                } else {
                    let mut sorted = active_deadlines.to_vec();
                    sorted.sort_by(|a, b| {
                        let ka = format!("{} {}", a.deadline, a.time);
                        let kb = format!("{} {}", b.deadline, b.time);
                        ka.cmp(&kb)
                    });
                    for item in sorted.into_iter().take(5) {
                        let (row_rect, row_resp) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 56.0),
                            Sense::click(),
                        );
                        if row_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        let row_bg = if row_resp.hovered() {
                            SURFACE_HOVER
                        } else {
                            SURFACE_SUBTLE
                        };
                        ui.painter()
                            .rect_filled(row_rect, CornerRadius::same(10), row_bg);

                        let (day_str, mon_str) = parse_day_month(&item.deadline);
                        let dot_col = match item.priority.as_str() {
                            "high" => DANGER,
                            "low" => ACCENT,
                            _ => AMBER,
                        };

                        let inner = row_rect.shrink2(Vec2::new(12.0, 8.0));
                        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
                            ui.horizontal_centered(|ui| {
                                // Date pill badge
                                Frame::new()
                                    .fill(SURFACE)
                                    .stroke(Stroke::new(1.0, BORDER))
                                    .corner_radius(CornerRadius::same(8))
                                    .inner_margin(Margin::symmetric(8, 3))
                                    .show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.label(
                                                RichText::new(day_str)
                                                    .size(13.0)
                                                    .color(ACCENT_DEEP)
                                                    .strong(),
                                            );
                                            ui.label(
                                                RichText::new(mon_str)
                                                    .size(9.5)
                                                    .color(INK_MUTED)
                                                    .strong(),
                                            );
                                        });
                                    });
                                ui.add_space(8.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(&item.title)
                                            .size(13.0)
                                            .color(INK)
                                            .strong(),
                                    );
                                    let meta = format!(
                                        "{} · {} WIB",
                                        if item.course.is_empty() {
                                            "Umum"
                                        } else {
                                            &item.course
                                        },
                                        if item.time.is_empty() {
                                            "23:59"
                                        } else {
                                            &item.time
                                        }
                                    );
                                    ui.label(RichText::new(meta).size(11.5).color(INK_MUTED));
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let (dr, _) =
                                        ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                                    ui.painter().circle_filled(dr.center(), 4.2, dot_col);
                                });
                            });
                        });

                        if row_resp.clicked() {
                            app.page = Page::Deadlines;
                        }
                        ui.add_space(6.0);
                    }
                }
            });
        },
    );
}

fn parse_day_month(date_str: &str) -> (String, &'static str) {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() == 3 {
        let m: u32 = parts[1].parse().unwrap_or(1);
        let d = parts[2].trim_start_matches('0');
        return (
            if d.is_empty() { "1" } else { d }.to_string(),
            short_month_upper(m),
        );
    }
    ("--".to_string(), "")
}
