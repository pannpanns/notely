use crate::app::NotelyApp;
use crate::theme::{
    self, ACCENT, ACCENT_BORDER, ACCENT_DEEP, ACCENT_HOVER, ACCENT_SOFT, AMBER, BORDER, DANGER,
    INK, INK_2, INK_MUTED, SURFACE, SURFACE_HOVER, SURFACE_SUBTLE,
};
use crate::types::{format_when_pretty, Page};
use chrono::{Datelike, Local};
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, Sense, Stroke, StrokeKind, Vec2,
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
    let content_w = avail_w.min(960.0_f32).max(340.0_f32);
    let side_pad = ((avail_w - content_w) * 0.5_f32).max(20.0_f32);

    egui::ScrollArea::vertical()
        .id_salt("home_studio_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(24.0_f32);
            ui.horizontal(|ui| {
                ui.add_space(side_pad);
                ui.allocate_ui_with_layout(
                    Vec2::new(content_w, 0.0_f32),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        // ── Date Pill ──
                        Frame::new()
                            .fill(SURFACE)
                            .stroke(Stroke::new(1.0_f32, BORDER))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::symmetric(11, 5))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 7.0_f32;
                                    theme::icon_widget(
                                        ui,
                                        theme::NavIcon::Calendar,
                                        14.0_f32,
                                        ACCENT,
                                    );
                                    ui.label(theme::semi(indonesian_today(), 12.0_f32, INK_2));
                                });
                            });

                        ui.add_space(12.0_f32);

                        // ── Header + Aligned KPI Chips ──
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
                                ui.spacing_mut().item_spacing.y = 4.0_f32;
                                ui.label(theme::bold(
                                    "Ruang catatan dan jadwal Anda.",
                                    26.0_f32,
                                    INK,
                                ));
                                ui.label(theme::regular(
                                    "Mulai rekaman baru, kelola transkrip, atau susun tenggat tugas.",
                                    13.5_f32,
                                    INK_2,
                                ));
                            });

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                // Tasks / Urgent KPI card
                                Frame::new()
                                    .fill(SURFACE)
                                    .stroke(Stroke::new(1.0_f32, BORDER))
                                    .corner_radius(CornerRadius::same(12))
                                    .inner_margin(Margin::symmetric(16, 10))
                                    .show(ui, |ui| {
                                        ui.set_min_width(82.0_f32);
                                        ui.vertical(|ui| {
                                            let num_col = if urgent_count > 0 {
                                                DANGER
                                            } else {
                                                ACCENT
                                            };
                                            ui.label(theme::bold(
                                                active_deadlines.len().to_string(),
                                                22.0_f32,
                                                num_col,
                                            ));
                                            let sub = if urgent_count > 0 {
                                                format!("{} mendesak", urgent_count)
                                            } else {
                                                "Tugas aktif".to_string()
                                            };
                                            ui.label(theme::regular(sub, 11.5_f32, INK_MUTED));
                                        });
                                    });

                                ui.add_space(8.0_f32);

                                // Total recordings KPI card
                                Frame::new()
                                    .fill(SURFACE)
                                    .stroke(Stroke::new(1.0_f32, BORDER))
                                    .corner_radius(CornerRadius::same(12))
                                    .inner_margin(Margin::symmetric(16, 10))
                                    .show(ui, |ui| {
                                        ui.set_min_width(82.0_f32);
                                        ui.vertical(|ui| {
                                            ui.label(theme::bold(
                                                app.sessions.len().to_string(),
                                                22.0_f32,
                                                INK,
                                            ));
                                            ui.label(theme::regular(
                                                "Total rekaman",
                                                11.5_f32,
                                                INK_MUTED,
                                            ));
                                        });
                                    });
                            });
                        });

                        ui.add_space(22.0_f32);

                        // ── Asymmetric Action Bento Grid ──
                        let narrow = content_w < 680.0_f32;
                        if narrow {
                            show_hero_record_card(app, ui, content_w);
                            ui.add_space(12.0_f32);
                            show_import_action_card(app, ui, content_w);
                            ui.add_space(10.0_f32);
                            show_deadline_action_card(app, ui, content_w, active_deadlines.len());
                        } else {
                            let gap = 14.0_f32;
                            let left_w = ((content_w - gap) * 0.54_f32).round();
                            let right_w = content_w - gap - left_w;
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                show_hero_record_card(app, ui, left_w);
                                ui.allocate_ui_with_layout(
                                    Vec2::new(right_w, 182.0_f32),
                                    Layout::top_down(Align::LEFT),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.y = 12.0_f32;
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

                        ui.add_space(16.0_f32);

                        // ── Bottom 2-Column Bento: Rekaman terakhir + Jadwal terdekat ──
                        let gap = 14.0_f32;
                        let col_w = if narrow {
                            content_w
                        } else {
                            (content_w - gap) * 0.5_f32
                        };

                        if narrow {
                            show_recent_recordings_card(app, ui, col_w);
                            ui.add_space(14.0_f32);
                            show_upcoming_deadlines_card(app, ui, col_w, &active_deadlines);
                        } else {
                            ui.horizontal_top(|ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                show_recent_recordings_card(app, ui, col_w);
                                show_upcoming_deadlines_card(app, ui, col_w, &active_deadlines);
                            });
                        }

                        ui.add_space(92.0_f32);
                    },
                );
            });
        });
}

fn show_hero_record_card(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    let is_rec = app.is_recording();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 182.0_f32), Sense::click());
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

    // Layered ambient highlight circle top-right
    painter.circle_filled(
        egui::pos2(rect.right() - 26.0_f32, rect.top() + 26.0_f32),
        86.0_f32,
        Color32::from_rgba_unmultiplied(255, 255, 255, 16),
    );
    painter.circle_filled(
        egui::pos2(rect.right() - 18.0_f32, rect.top() + 18.0_f32),
        46.0_f32,
        Color32::from_rgba_unmultiplied(255, 255, 255, 12),
    );

    let inner = rect.shrink(22.0_f32);
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let (icon_box, _) =
                    ui.allocate_exact_size(Vec2::splat(42.0_f32), Sense::hover());
                ui.painter().rect_filled(
                    icon_box,
                    CornerRadius::same(11),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 36),
                );
                theme::paint_icon(
                    ui,
                    icon_box.shrink(10.0_f32),
                    if is_rec {
                        theme::NavIcon::Square
                    } else {
                        theme::NavIcon::Mic
                    },
                    Color32::WHITE,
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    theme::icon_widget(
                        ui,
                        theme::NavIcon::ArrowUpRight,
                        17.0_f32,
                        Color32::from_rgba_unmultiplied(255, 255, 255, 210),
                    );
                });
            });

            ui.add_space(32.0_f32);
            ui.label(theme::bold(
                if is_rec {
                    "Hentikan rekaman"
                } else {
                    "Rekam percakapan"
                },
                20.5_f32,
                Color32::WHITE,
            ));
            ui.add_space(4.0_f32);
            ui.label(theme::regular(
                if is_rec {
                    "Klik untuk menghentikan rekaman & melihat hasil transkrip lengkap"
                } else {
                    "Tangkap audio komputer & mikrofon dengan transkripsi lokal cepat"
                },
                12.5_f32,
                Color32::from_rgba_unmultiplied(234, 245, 240, 230),
            ));
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
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 85.0_f32), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let fill = if resp.hovered() { SURFACE_HOVER } else { SURFACE };
    let border = if resp.hovered() { ACCENT } else { BORDER };
    ui.painter().rect_filled(rect, CornerRadius::same(14), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0_f32, border),
        StrokeKind::Inside,
    );

    let inner = rect.shrink2(Vec2::new(16.0_f32, 14.0_f32));
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
        ui.horizontal_centered(|ui| {
            let (icon_box, _) = ui.allocate_exact_size(Vec2::splat(38.0_f32), Sense::hover());
            ui.painter()
                .rect_filled(icon_box, CornerRadius::same(10), ACCENT_SOFT);
            theme::paint_icon(
                ui,
                icon_box.shrink(9.0_f32),
                theme::NavIcon::Upload,
                ACCENT,
            );
            ui.add_space(10.0_f32);
            ui.vertical(|ui| {
                ui.label(theme::semi("Impor rekaman", 14.5_f32, INK));
                ui.label(theme::regular(
                    "Ubah file audio atau video menjadi transkrip & ringkasan",
                    12.0_f32,
                    INK_MUTED,
                ));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                theme::icon_widget(
                    ui,
                    theme::NavIcon::ArrowUpRight,
                    15.0_f32,
                    if resp.hovered() { ACCENT } else { INK_MUTED },
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
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 85.0_f32), Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let fill = if resp.hovered() { SURFACE_HOVER } else { SURFACE };
    let border = if resp.hovered() { ACCENT } else { BORDER };
    ui.painter().rect_filled(rect, CornerRadius::same(14), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0_f32, border),
        StrokeKind::Inside,
    );

    let inner = rect.shrink2(Vec2::new(16.0_f32, 14.0_f32));
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
        ui.horizontal_centered(|ui| {
            let (icon_box, _) = ui.allocate_exact_size(Vec2::splat(38.0_f32), Sense::hover());
            ui.painter()
                .rect_filled(icon_box, CornerRadius::same(10), ACCENT_SOFT);
            theme::paint_icon(
                ui,
                icon_box.shrink(9.0_f32),
                theme::NavIcon::Calendar,
                ACCENT,
            );
            ui.add_space(10.0_f32);
            ui.vertical(|ui| {
                ui.label(theme::semi("Tenggat & jadwal", 14.5_f32, INK));
                let sub = if app.gcal.connected {
                    format!("{} jadwal aktif tersinkron Google Calendar", active_count)
                } else {
                    format!(
                        "{} jadwal aktif · Ekstrak otomatis dari teks & gambar",
                        active_count
                    )
                };
                ui.label(theme::regular(sub, 12.0_f32, INK_MUTED));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                theme::icon_widget(
                    ui,
                    theme::NavIcon::ArrowUpRight,
                    15.0_f32,
                    if resp.hovered() { ACCENT } else { INK_MUTED },
                );
            });
        });
    });

    if resp.clicked() {
        app.page = Page::Deadlines;
    }
}

fn show_recent_recordings_card(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    ui.allocate_ui_with_layout(
        Vec2::new(width, 0.0_f32),
        Layout::top_down(Align::LEFT),
        |ui| {
            theme::card_frame().show(ui, |ui| {
                ui.set_width(width - 38.0_f32);
                ui.horizontal(|ui| {
                    ui.label(theme::bold("Rekaman terakhir", 15.0_f32, INK));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if theme::ghost_btn(ui, "Lihat semua").clicked() {
                            app.page = Page::Recordings;
                        }
                    });
                });

                ui.add_space(10.0_f32);

                if app.sessions.is_empty() {
                    Frame::new()
                        .fill(SURFACE_SUBTLE)
                        .stroke(Stroke::new(1.0_f32, BORDER))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::symmetric(14, 24))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(theme::regular(
                                    "Belum ada rekaman tersimpan.",
                                    13.0_f32,
                                    INK_MUTED,
                                ));
                            });
                        });
                } else {
                    let preview: Vec<_> = app.sessions.iter().take(5).cloned().collect();
                    for item in preview {
                        let (row_rect, row_resp) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 56.0_f32),
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
                        if row_resp.hovered() {
                            ui.painter().rect_stroke(
                                row_rect,
                                CornerRadius::same(10),
                                Stroke::new(1.0_f32, ACCENT_BORDER),
                                StrokeKind::Inside,
                            );
                        }

                        let inner = row_rect.shrink2(Vec2::new(12.0_f32, 8.0_f32));
                        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
                            ui.horizontal_centered(|ui| {
                                let (ic, _) = ui.allocate_exact_size(
                                    Vec2::splat(32.0_f32),
                                    Sense::hover(),
                                );
                                ui.painter().rect_filled(ic, CornerRadius::same(8), SURFACE);
                                ui.painter().rect_stroke(
                                    ic,
                                    CornerRadius::same(8),
                                    Stroke::new(1.0_f32, BORDER),
                                    StrokeKind::Inside,
                                );
                                theme::paint_icon(
                                    ui,
                                    ic.shrink(7.0_f32),
                                    theme::NavIcon::Notes,
                                    ACCENT,
                                );
                                ui.add_space(6.0_f32);
                                ui.vertical(|ui| {
                                    ui.label(theme::semi(&item.title, 13.0_f32, INK));
                                    ui.label(theme::regular(
                                        format_when_pretty(&item.when),
                                        11.5_f32,
                                        INK_MUTED,
                                    ));
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    theme::icon_widget(
                                        ui,
                                        theme::NavIcon::ChevronRight,
                                        14.0_f32,
                                        if row_resp.hovered() {
                                            ACCENT
                                        } else {
                                            INK_MUTED
                                        },
                                    );
                                });
                            });
                        });

                        if row_resp.clicked() {
                            app.open_session(&item.id);
                        }
                        ui.add_space(6.0_f32);
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
        Vec2::new(width, 0.0_f32),
        Layout::top_down(Align::LEFT),
        |ui| {
            theme::card_frame().show(ui, |ui| {
                ui.set_width(width - 38.0_f32);
                ui.horizontal(|ui| {
                    ui.label(theme::bold("Jadwal terdekat", 15.0_f32, INK));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if theme::ghost_btn(ui, "Buka kalender").clicked() {
                            app.page = Page::Deadlines;
                        }
                    });
                });

                ui.add_space(10.0_f32);

                if active_deadlines.is_empty() {
                    Frame::new()
                        .fill(SURFACE_SUBTLE)
                        .stroke(Stroke::new(1.0_f32, BORDER))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::symmetric(14, 24))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(theme::regular(
                                    "Tidak ada tenggat mendatang.",
                                    13.0_f32,
                                    INK_MUTED,
                                ));
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
                            Vec2::new(ui.available_width(), 56.0_f32),
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
                        if row_resp.hovered() {
                            ui.painter().rect_stroke(
                                row_rect,
                                CornerRadius::same(10),
                                Stroke::new(1.0_f32, ACCENT_BORDER),
                                StrokeKind::Inside,
                            );
                        }

                        let (day_str, mon_str) = parse_day_month(&item.deadline);
                        let dot_col = match item.priority.as_str() {
                            "high" => DANGER,
                            "low" => ACCENT,
                            _ => AMBER,
                        };

                        let inner = row_rect.shrink2(Vec2::new(12.0_f32, 8.0_f32));
                        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
                            ui.horizontal_centered(|ui| {
                                Frame::new()
                                    .fill(SURFACE)
                                    .stroke(Stroke::new(1.0_f32, BORDER))
                                    .corner_radius(CornerRadius::same(8))
                                    .inner_margin(Margin::symmetric(8, 3))
                                    .show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.label(theme::bold(day_str, 13.0_f32, ACCENT_DEEP));
                                            ui.label(theme::semi(mon_str, 9.5_f32, INK_MUTED));
                                        });
                                    });
                                ui.add_space(8.0_f32);
                                ui.vertical(|ui| {
                                    ui.label(theme::semi(&item.title, 13.0_f32, INK));
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
                                    ui.label(theme::regular(meta, 11.5_f32, INK_MUTED));
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let (dr, _) = ui.allocate_exact_size(
                                        Vec2::splat(10.0_f32),
                                        Sense::hover(),
                                    );
                                    ui.painter().circle_filled(dr.center(), 4.2_f32, dot_col);
                                });
                            });
                        });

                        if row_resp.clicked() {
                            app.page = Page::Deadlines;
                        }
                        ui.add_space(6.0_f32);
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
