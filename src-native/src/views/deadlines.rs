use crate::app::NotelyApp;
use crate::theme::{
    self, ACCENT, ACCENT_BORDER, ACCENT_DEEP, ACCENT_SOFT, AMBER, AMBER_SOFT, BORDER, DANGER,
    DANGER_BORDER, DANGER_SOFT, GREEN, GREEN_SOFT, INK, INK_2, INK_MUTED, SURFACE, SURFACE_HOVER,
    SURFACE_SUBTLE,
};
use crate::types::{
    combine_reminder, split_reminder, AiStep, DeadlineFilter, DeadlineItem, DeadlineView,
};
use chrono::{Datelike, Local, NaiveDate};
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, RichText, Sense, Stroke, StrokeKind,
    Vec2,
};

const MONTHS_ID: [&str; 12] = [
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

pub fn show(app: &mut NotelyApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    let content_w = avail_w.min(1040.0).max(320.0);
    let side_pad = ((avail_w - content_w) * 0.5).max(16.0);

    egui::ScrollArea::vertical()
        .id_salt("deadlines_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.add_space(side_pad);
                ui.allocate_ui_with_layout(
                    Vec2::new(content_w, 0.0),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        // ── Header Row ──
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("Tenggat Tugas")
                                        .size(22.0)
                                        .color(INK)
                                        .strong(),
                                );
                                let sub = if app.gcal.connected {
                                    format!("Tersambung dengan Google Calendar ({})", app.gcal.email)
                                } else {
                                    "Atur jadwal kuliah, tugas, dan pengingat otomatis dari teks atau gambar.".to_string()
                                };
                                ui.label(RichText::new(sub).size(13.0).color(INK_2));
                            });

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if theme::primary_btn(ui, "Buat dengan AI").clicked() {
                                    app.open_ai_deadline_modal();
                                }
                                if theme::secondary_btn(ui, "+ Tambah Manual").clicked() {
                                    app.open_manual_deadline_modal(None, None);
                                }
                                if app.gcal.connected {
                                    let sync_lbl = if app.gcal_syncing {
                                        "Menyinkronkan..."
                                    } else {
                                        "Sinkronkan"
                                    };
                                    ui.add_enabled_ui(!app.gcal_syncing, |ui| {
                                        if theme::secondary_btn(ui, sync_lbl).clicked() {
                                            app.sync_google_calendar();
                                        }
                                    });
                                }
                            });
                        });

                        ui.add_space(14.0);

                        // ── Segmented Switcher (Kalender | Daftar Tugas) ──
                        ui.horizontal(|ui| {
                            Frame::new()
                                .fill(SURFACE_SUBTLE)
                                .stroke(Stroke::new(1.0, BORDER))
                                .corner_radius(CornerRadius::same(10))
                                .inner_margin(Margin::same(3))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 2.0;
                                        for (v, lbl) in [
                                            (DeadlineView::Calendar, "Kalender"),
                                            (DeadlineView::List, "Daftar Tugas"),
                                        ] {
                                            let active = app.deadline_view == v;
                                            let btn = egui::Button::new(
                                                RichText::new(lbl)
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
                                            .min_size(Vec2::new(100.0, 28.0));
                                            if ui.add(btn).clicked() {
                                                app.deadline_view = v;
                                            }
                                        }
                                    });
                                });

                            if app.deadline_view == DeadlineView::List {
                                ui.add_space(12.0);
                                for (f, lbl) in [
                                    (DeadlineFilter::All, "Semua"),
                                    (DeadlineFilter::Upcoming, "Mendatang"),
                                    (DeadlineFilter::Urgent, "Mendesak"),
                                    (DeadlineFilter::Overdue, "Terlewat"),
                                    (DeadlineFilter::Completed, "Selesai"),
                                ] {
                                    let active = app.deadline_filter == f;
                                    let btn = egui::Button::new(
                                        RichText::new(lbl)
                                            .size(12.0)
                                            .color(if active { ACCENT_DEEP } else { INK_2 }),
                                    )
                                    .fill(if active { ACCENT_SOFT } else { SURFACE })
                                    .stroke(Stroke::new(
                                        1.0,
                                        if active { ACCENT_BORDER } else { BORDER },
                                    ))
                                    .corner_radius(CornerRadius::same(14))
                                    .min_size(Vec2::new(0.0, 28.0));
                                    if ui.add(btn).clicked() {
                                        app.deadline_filter = f;
                                    }
                                }
                            }
                        });

                        ui.add_space(16.0);

                        match app.deadline_view {
                            DeadlineView::Calendar => {
                                show_calendar_view(app, ui, content_w);
                            }
                            DeadlineView::List => {
                                show_list_view(app, ui, content_w);
                            }
                        }

                        ui.add_space(92.0);
                    },
                );
            });
        });

    // ── Modals ──
    if app.show_manual_deadline_modal {
        show_manual_modal(app, ctx);
    }
    if app.show_ai_deadline_modal {
        show_ai_modal(app, ctx);
    }
    if app.calendar_day_detail.is_some() {
        show_day_detail_modal(app, ctx);
    }
    if app.delete_deadline_target.is_some() {
        show_delete_deadline_modal(app, ctx);
    }
    if app.complete_deadline_target.is_some() {
        show_complete_deadline_modal(app, ctx);
    }
    if let Some(idx) = app.lightbox_image_idx {
        show_image_lightbox(app, ctx, idx);
    }
}

fn show_calendar_view(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    let year = app.cal_year;
    let month = app.cal_month;
    let today = Local::now().date_naive();

    theme::card_frame().show(ui, |ui| {
        ui.set_width(width - 34.0);

        // Month Header
        ui.horizontal(|ui| {
            let mon_name = MONTHS_ID
                .get((month.clamp(1, 12) - 1) as usize)
                .copied()
                .unwrap_or("Januari");
            ui.label(
                RichText::new(format!("{} {}", mon_name, year))
                    .size(17.0)
                    .color(INK)
                    .strong(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::secondary_btn(ui, "Berikutnya >").clicked() {
                    if app.cal_month == 12 {
                        app.cal_month = 1;
                        app.cal_year += 1;
                    } else {
                        app.cal_month += 1;
                    }
                }
                if theme::secondary_btn(ui, "Hari Ini").clicked() {
                    app.cal_year = today.year();
                    app.cal_month = today.month();
                }
                if theme::secondary_btn(ui, "< Sebelumnya").clicked() {
                    if app.cal_month == 1 {
                        app.cal_month = 12;
                        app.cal_year -= 1;
                    } else {
                        app.cal_month -= 1;
                    }
                }
            });
        });

        ui.add_space(12.0);

        // Day headers (SEN..MIN)
        let inner_w = width - 34.0;
        let gap = 6.0;
        let cell_w = ((inner_w - gap * 6.0) / 7.0).max(40.0);
        let day_labels = ["SEN", "SEL", "RAB", "KAM", "JUM", "SAB", "MIN"];

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for lbl in day_labels {
                let (r, _) = ui.allocate_exact_size(Vec2::new(cell_w, 22.0), Sense::hover());
                ui.painter().text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    lbl,
                    egui::FontId::proportional(11.0),
                    INK_MUTED,
                );
            }
        });

        ui.add_space(6.0);

        let first_of_month =
            NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(today);
        let start_offset = first_of_month.weekday().num_days_from_monday() as i32;
        let days_in_month = days_in(year, month);
        let total_cells = ((start_offset + days_in_month as i32 + 6) / 7) * 7;

        let deadlines = app.deadlines.clone();

        for week in 0..(total_cells / 7) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for col in 0..7 {
                    let cell_idx = week * 7 + col;
                    let day_num = cell_idx - start_offset + 1;
                    let cell_h = 84.0;
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(cell_w, cell_h), Sense::click());

                    if day_num < 1 || day_num > days_in_month as i32 {
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(10), Color32::from_rgb(0xFA, 0xFC, 0xFB));
                        continue;
                    }

                    let date_iso = format!("{:04}-{:02}-{:02}", year, month, day_num);
                    let is_today =
                        year == today.year() && month == today.month() && day_num as u32 == today.day();
                    let day_items: Vec<&DeadlineItem> = deadlines
                        .iter()
                        .filter(|d| d.deadline == date_iso)
                        .collect();

                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }

                    let bg = if is_today {
                        ACCENT_SOFT
                    } else if resp.hovered() {
                        SURFACE_HOVER
                    } else {
                        SURFACE_SUBTLE
                    };
                    let stroke = if is_today {
                        Stroke::new(1.5, ACCENT)
                    } else {
                        Stroke::new(1.0, BORDER)
                    };

                    ui.painter().rect_filled(rect, CornerRadius::same(10), bg);
                    ui.painter()
                        .rect_stroke(rect, CornerRadius::same(10), stroke, StrokeKind::Inside);

                    // Day number
                    ui.painter().text(
                        egui::pos2(rect.left() + 9.0, rect.top() + 8.0),
                        egui::Align2::LEFT_TOP,
                        day_num.to_string(),
                        egui::FontId::proportional(12.5),
                        if is_today { ACCENT_DEEP } else { INK },
                    );

                    // Task pills (up to 2)
                    let mut py = rect.top() + 28.0;
                    for item in day_items.iter().take(2) {
                        let (pill_bg, pill_fg) = if item.status == "completed" {
                            (BORDER, INK_MUTED)
                        } else if item.priority == "high" {
                            (DANGER_SOFT, DANGER)
                        } else if item.priority == "low" {
                            (GREEN_SOFT, ACCENT_DEEP)
                        } else {
                            (AMBER_SOFT, AMBER)
                        };
                        let pill_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.left() + 5.0, py),
                            Vec2::new((rect.width() - 10.0).max(12.0), 18.0),
                        );
                        ui.painter()
                            .rect_filled(pill_rect, CornerRadius::same(5), pill_bg);
                        let short = truncate_chars(&item.title, 14);
                        ui.painter().text(
                            egui::pos2(pill_rect.left() + 5.0, pill_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            short,
                            egui::FontId::proportional(10.5),
                            pill_fg,
                        );
                        py += 20.0;
                    }
                    if day_items.len() > 2 {
                        ui.painter().text(
                            egui::pos2(rect.left() + 8.0, py + 2.0),
                            egui::Align2::LEFT_TOP,
                            format!("+{} lagi", day_items.len() - 2),
                            egui::FontId::proportional(10.0),
                            INK_MUTED,
                        );
                    }

                    if resp.clicked() {
                        app.calendar_day_detail = Some(date_iso);
                    }
                }
            });
            ui.add_space(gap);
        }
    });
}

fn show_list_view(app: &mut NotelyApp, ui: &mut egui::Ui, width: f32) {
    let today_iso = Local::now().format("%Y-%m-%d").to_string();
    let mut items: Vec<DeadlineItem> = app
        .deadlines
        .iter()
        .filter(|d| match app.deadline_filter {
            DeadlineFilter::All => true,
            DeadlineFilter::Upcoming => d.status != "completed" && d.deadline >= today_iso,
            DeadlineFilter::Urgent => d.status != "completed" && d.priority == "high",
            DeadlineFilter::Overdue => d.status != "completed" && !d.deadline.is_empty() && d.deadline < today_iso,
            DeadlineFilter::Completed => d.status == "completed",
        })
        .cloned()
        .collect();

    items.sort_by(|a, b| {
        let ca = a.status == "completed";
        let cb = b.status == "completed";
        ca.cmp(&cb)
            .then_with(|| a.deadline.cmp(&b.deadline))
            .then_with(|| a.time.cmp(&b.time))
    });

    if items.is_empty() {
        theme::card_frame().show(ui, |ui| {
            ui.set_width(width - 34.0);
            ui.vertical_centered(|ui| {
                ui.add_space(24.0);
                ui.label(
                    RichText::new("Belum ada tugas pada filter ini.")
                        .size(14.0)
                        .color(INK_MUTED),
                );
                ui.add_space(10.0);
                if theme::primary_btn(ui, "Buat Jadwal dengan AI").clicked() {
                    app.open_ai_deadline_modal();
                }
                ui.add_space(24.0);
            });
        });
        return;
    }

    for item in items {
        let is_done = item.status == "completed";
        Frame::new()
            .fill(SURFACE)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(CornerRadius::same(13))
            .inner_margin(Margin::symmetric(16, 13))
            .show(ui, |ui| {
                ui.set_width(width - 34.0);
                ui.horizontal(|ui| {
                    // Completion checkbox
                    let (cb_rect, cb_resp) =
                        ui.allocate_exact_size(Vec2::splat(20.0), Sense::click());
                    if cb_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    ui.painter().rect_filled(
                        cb_rect,
                        CornerRadius::same(6),
                        if is_done { ACCENT } else { SURFACE_SUBTLE },
                    );
                    ui.painter().rect_stroke(
                        cb_rect,
                        CornerRadius::same(6),
                        Stroke::new(1.2, if is_done { ACCENT } else { theme::BORDER_STRONG }),
                        StrokeKind::Inside,
                    );
                    if is_done {
                        ui.painter().text(
                            cb_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "v",
                            egui::FontId::proportional(12.0),
                            Color32::WHITE,
                        );
                    }
                    if cb_resp.clicked() {
                        if is_done {
                            app.toggle_deadline_status(&item.id);
                        } else {
                            app.complete_deadline_target = Some(item.clone());
                        }
                    }

                    ui.add_space(8.0);

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            let mut title_rt = RichText::new(&item.title)
                                .size(14.0)
                                .color(if is_done { INK_MUTED } else { INK })
                                .strong();
                            if is_done {
                                title_rt = title_rt.strikethrough();
                            }
                            ui.label(title_rt);

                            if !item.course.is_empty() {
                                Frame::new()
                                    .fill(SURFACE_SUBTLE)
                                    .stroke(Stroke::new(1.0, BORDER))
                                    .corner_radius(CornerRadius::same(7))
                                    .inner_margin(Margin::symmetric(7, 2))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(&item.course)
                                                .size(11.0)
                                                .color(INK_2),
                                        );
                                    });
                            }

                            // Priority pill
                            let (p_lbl, p_bg, p_fg) = match item.priority.as_str() {
                                "high" => ("Tinggi", DANGER_SOFT, DANGER),
                                "low" => ("Rendah", GREEN_SOFT, ACCENT_DEEP),
                                _ => ("Sedang", AMBER_SOFT, AMBER),
                            };
                            Frame::new()
                                .fill(p_bg)
                                .corner_radius(CornerRadius::same(7))
                                .inner_margin(Margin::symmetric(7, 2))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(p_lbl).size(10.5).color(p_fg).strong(),
                                    );
                                });

                            if item.gcal_event_id.is_some() {
                                Frame::new()
                                    .fill(ACCENT_SOFT)
                                    .corner_radius(CornerRadius::same(7))
                                    .inner_margin(Margin::symmetric(7, 2))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new("Google Calendar")
                                                .size(10.5)
                                                .color(ACCENT_DEEP)
                                                .strong(),
                                        );
                                    });
                            }
                        });

                        ui.add_space(3.0);

                        let rem_str = format_reminder_label(item.reminder);
                        let meta = format!(
                            "{} · {} WIB · Pengingat: {}",
                            item.deadline,
                            if item.time.is_empty() {
                                "23:59"
                            } else {
                                &item.time
                            },
                            rem_str
                        );
                        ui.label(RichText::new(meta).size(12.0).color(INK_MUTED));

                        if !item.description.trim().is_empty() {
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(truncate_chars(&item.description, 120))
                                    .size(12.0)
                                    .color(INK_2),
                            );
                        }
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if theme::danger_soft_btn(ui, "Hapus").clicked() {
                            app.delete_deadline_target = Some(item.clone());
                        }
                        if theme::secondary_btn(ui, "Edit").clicked() {
                            app.open_manual_deadline_modal(Some(item.clone()), None);
                        }
                    });
                });
            });
        ui.add_space(10.0);
    }
}

fn show_manual_modal(app: &mut NotelyApp, ctx: &egui::Context) {
    if draw_scrim(ctx, "manual_deadline_scrim") {
        app.show_manual_deadline_modal = false;
    }

    let is_edit = !app.manual_deadline.id.is_empty();
    egui::Window::new("Manual Deadline")
        .id(egui::Id::new("manual_deadline_win"))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(500.0, 0.0))
        .frame(theme::modal_frame())
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(if is_edit {
                        "Edit Tenggat Tugas"
                    } else {
                        "Tambah Tenggat Tugas"
                    })
                    .size(17.5)
                    .color(INK)
                    .strong(),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::ghost_btn(ui, "Tutup").clicked() {
                        app.show_manual_deadline_modal = false;
                    }
                });
            });

            ui.add_space(14.0);

            draw_deadline_form_fields(
                ui,
                &mut app.manual_deadline,
                &mut app.manual_rem_amount,
                &mut app.manual_rem_unit,
                "manual_form",
            );

            ui.add_space(18.0);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::primary_btn(ui, "Simpan Tugas").clicked() {
                        app.submit_manual_deadline();
                    }
                    if theme::secondary_btn(ui, "Batal").clicked() {
                        app.show_manual_deadline_modal = false;
                    }
                });
            });
        });
}

fn show_ai_modal(app: &mut NotelyApp, ctx: &egui::Context) {
    let time = ctx.input(|i| i.time);
    if draw_scrim(ctx, "ai_deadline_scrim") && app.ai_step != AiStep::Loading {
        app.show_ai_deadline_modal = false;
    }

    egui::Window::new("AI Deadline Extractor")
        .id(egui::Id::new("ai_deadline_win"))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(620.0, 0.0))
        .frame(theme::modal_frame())
        .show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                let (ic, _) = ui.allocate_exact_size(Vec2::splat(36.0), Sense::hover());
                ui.painter()
                    .rect_filled(ic, CornerRadius::same(10), ACCENT_SOFT);
                theme::draw_nav_icon(ui.painter(), ic.shrink(7.0), theme::NavIcon::Spark, ACCENT);
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Ekstrak Jadwal dengan AI")
                            .size(17.5)
                            .color(INK)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("Susun tugas & tenggat otomatis dari teks atau tangkapan layar.")
                            .size(12.0)
                            .color(INK_MUTED),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if app.ai_step != AiStep::Loading && theme::ghost_btn(ui, "Tutup").clicked() {
                        app.show_ai_deadline_modal = false;
                    }
                });
            });

            ui.add_space(14.0);

            match app.ai_step {
                AiStep::Input => show_ai_step_input(app, ctx, ui),
                AiStep::Loading => show_ai_step_loading(app, ui, time),
                AiStep::Results => show_ai_step_results(app, ui),
            }
        });
}

/// Stage 1: Clean, spacious input screen with multi-image attach + zoom lightbox (NO quick examples, NO inner scroll!)
fn show_ai_step_input(app: &mut NotelyApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    if app.gcal.connected {
        Frame::new()
            .fill(GREEN_SOFT)
            .stroke(Stroke::new(1.0, ACCENT_BORDER))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let (d, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                    ui.painter().circle_filled(d.center(), 3.8, GREEN);
                    ui.label(
                        RichText::new("Google Calendar terhubung — jadwal akan otomatis tersinkron.")
                            .size(12.0)
                            .color(ACCENT_DEEP)
                            .strong(),
                    );
                });
            });
        ui.add_space(12.0);
    }

    ui.label(
        RichText::new("Tulis instruksi atau tempel teks jadwal:")
            .size(12.5)
            .color(INK)
            .strong(),
    );
    ui.add_space(6.0);

    ui.add_sized(
        Vec2::new(ui.available_width(), 96.0),
        egui::TextEdit::multiline(&mut app.ai_prompt)
            .hint_text("Contoh: 'Besok presentasi Basis Data jam 10.00' atau tempel pengumuman kelas...")
            .margin(Margin::same(10)),
    );

    ui.add_space(14.0);

    // Multi-Image Attachment Box (click thumbnail to zoom in Lightbox!)
    Frame::new()
        .fill(SURFACE_SUBTLE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let title = if app.ai_images.is_empty() {
                        "Lampirkan Gambar / Screenshot (Opsional)".to_string()
                    } else {
                        format!("Gambar Terlampir ({})", app.ai_images.len())
                    };
                    ui.label(RichText::new(title).size(13.0).color(INK).strong());
                    let hint = if app.ai_images.is_empty() {
                        "Pilih banyak gambar sekaligus, seret ke jendela, atau tempel dengan Ctrl + V"
                    } else {
                        "Klik gambar untuk memperbesar & memeriksa detail sebelum diekstrak"
                    };
                    ui.label(RichText::new(hint).size(11.5).color(INK_MUTED));
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if !app.ai_images.is_empty()
                        && theme::danger_soft_btn(ui, "Hapus Semua").clicked()
                    {
                        app.clear_ai_images();
                    }
                    if theme::secondary_btn(ui, "+ Pilih Gambar").clicked() {
                        app.pick_ai_images(ctx);
                    }
                });
            });

            if !app.ai_images.is_empty() {
                ui.add_space(10.0);
                let mut remove_idx: Option<usize> = None;
                let mut open_zoom: Option<usize> = None;

                egui::ScrollArea::horizontal()
                    .id_salt("ai_images_strip")
                    .max_height(88.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 10.0;
                            for idx in 0..app.ai_images.len() {
                                let thumb_size = Vec2::new(76.0, 76.0);
                                let (rect, resp) =
                                    ui.allocate_exact_size(thumb_size, Sense::click());
                                if resp.hovered() {
                                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if let Some(tex) = app.ai_image_textures.get(idx) {
                                    let sized =
                                        egui::load::SizedTexture::new(tex.id(), tex.size_vec2());
                                    egui::Image::from_texture(sized)
                                        .corner_radius(CornerRadius::same(8))
                                        .paint_at(ui, rect);
                                } else {
                                    ui.painter().rect_filled(
                                        rect,
                                        CornerRadius::same(8),
                                        SURFACE,
                                    );
                                }
                                ui.painter().rect_stroke(
                                    rect,
                                    CornerRadius::same(8),
                                    Stroke::new(
                                        1.2,
                                        if resp.hovered() { ACCENT } else { BORDER },
                                    ),
                                    StrokeKind::Inside,
                                );

                                // Small red 'x' badge in top-right corner
                                let x_center = egui::pos2(rect.right() - 9.0, rect.top() + 9.0);
                                let x_rect = egui::Rect::from_center_size(
                                    x_center,
                                    Vec2::splat(16.0),
                                );
                                let x_resp = ui.interact(
                                    x_rect,
                                    egui::Id::new(("ai_img_rm", idx)),
                                    Sense::click(),
                                );
                                ui.painter().circle_filled(x_center, 8.0, DANGER);
                                ui.painter().text(
                                    x_center,
                                    egui::Align2::CENTER_CENTER,
                                    "x",
                                    egui::FontId::proportional(10.0),
                                    Color32::WHITE,
                                );

                                if x_resp.clicked() {
                                    remove_idx = Some(idx);
                                } else if resp.clicked() {
                                    open_zoom = Some(idx);
                                }
                            }
                        });
                    });

                if let Some(i) = remove_idx {
                    app.remove_ai_image(i);
                }
                if let Some(i) = open_zoom {
                    app.lightbox_image_idx = Some(i);
                }
            }
        });

    ui.add_space(16.0);

    // Footer
    ui.horizontal(|ui| {
        let model_info = if app.prefs.provider == "ollama" {
            format!("Model: AI Lokal ({})", app.prefs.summary_model)
        } else {
            format!("Model: Cloud ({})", app.prefs.provider)
        };
        ui.label(RichText::new(model_info).size(12.0).color(INK_MUTED));

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let can_extract = !app.ai_prompt.trim().is_empty() || !app.ai_images.is_empty();
            ui.add_enabled_ui(can_extract, |ui| {
                if theme::primary_btn(ui, "Ekstrak Jadwal").clicked() {
                    app.start_ai_deadline_extract();
                }
            });
            if theme::secondary_btn(ui, "Batal").clicked() {
                app.show_ai_deadline_modal = false;
            }
        });
    });
}

/// Stage 2: Dedicated centered analysis screen with multi-ring orbital loader
fn show_ai_step_loading(app: &mut NotelyApp, ui: &mut egui::Ui, time: f64) {
    ui.ctx().request_repaint();
    Frame::new()
        .fill(SURFACE_SUBTLE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::symmetric(24, 42))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                theme::draw_orbit_loader(ui, 24.0, time);
                ui.add_space(16.0);
                ui.label(
                    RichText::new("Sedang menganalisis jadwal...")
                        .size(16.0)
                        .color(INK)
                        .strong(),
                );
                ui.add_space(6.0);
                let msg = if app.ai_status_msg.is_empty() {
                    "Membaca teks dan konteks gambar, mohon tunggu sebentar..."
                } else {
                    &app.ai_status_msg
                };
                ui.label(RichText::new(msg).size(12.5).color(INK_2));
            });
        });

    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if theme::secondary_btn(ui, "Kembali").clicked() {
                app.ai_step = AiStep::Input;
            }
        });
    });
}

/// Stage 3: Clean dedicated review & edit screen for extracted tasks (zero cluttered scrolling!)
fn show_ai_step_results(app: &mut NotelyApp, ui: &mut egui::Ui) {
    if app.ai_extracted.is_empty() {
        Frame::new()
            .fill(SURFACE_SUBTLE)
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(22))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Tidak ada jadwal yang terdeteksi dari input Anda.")
                            .size(13.5)
                            .color(INK_2),
                    );
                });
            });
        ui.add_space(14.0);
        if theme::secondary_btn(ui, "Kembali ke Input").clicked() {
            app.ai_step = AiStep::Input;
        }
        return;
    }

    // Clamp active task index
    if app.ai_active_idx >= app.ai_extracted.len() {
        app.ai_active_idx = app.ai_extracted.len() - 1;
    }

    // Multi-task tab bar when > 1 task extracted
    if app.ai_extracted.len() > 1 {
        ui.horizontal_wrapped(|ui| {
            for idx in 0..app.ai_extracted.len() {
                let active = idx == app.ai_active_idx;
                let lbl = format!("Tugas {}", idx + 1);
                let btn = egui::Button::new(
                    RichText::new(lbl)
                        .size(12.0)
                        .color(if active { Color32::WHITE } else { INK_2 })
                        .strong(),
                )
                .fill(if active { ACCENT } else { SURFACE_SUBTLE })
                .stroke(Stroke::new(1.0, if active { ACCENT } else { BORDER }))
                .corner_radius(CornerRadius::same(8))
                .min_size(Vec2::new(72.0, 28.0));
                if ui.add(btn).clicked() {
                    app.select_ai_result_tab(idx);
                }
            }
        });
        ui.add_space(12.0);
    }

    let idx = app.ai_active_idx;
    if let Some(item) = app.ai_extracted.get_mut(idx) {
        draw_deadline_form_fields(
            ui,
            item,
            &mut app.ai_rem_amount,
            &mut app.ai_rem_unit,
            "ai_result_form",
        );
    }

    ui.add_space(16.0);

    ui.horizontal(|ui| {
        if theme::secondary_btn(ui, "Kembali ke Input").clicked() {
            app.ai_step = AiStep::Input;
        }
        if app.ai_extracted.len() > 1 && theme::danger_soft_btn(ui, "Hapus Tab Ini").clicked() {
            app.ai_extracted.remove(idx);
            let next_i = idx.min(app.ai_extracted.len().saturating_sub(1));
            app.select_ai_result_tab(next_i);
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let count = app.ai_extracted.len();
            let save_lbl = if app.gcal.connected {
                format!("Simpan & Sinkron ({})", count)
            } else {
                format!("Simpan {} Tugas", count)
            };
            if theme::primary_btn(ui, &save_lbl).clicked() {
                app.save_ai_extracted_deadlines();
            }
        });
    });
}

fn draw_deadline_form_fields(
    ui: &mut egui::Ui,
    item: &mut DeadlineItem,
    rem_amount: &mut String,
    rem_unit: &mut String,
    id_prefix: &str,
) {
    // Title
    ui.label(RichText::new("Judul Tugas / Jadwal").size(12.0).color(INK_2).strong());
    ui.add_space(4.0);
    ui.add_sized(
        Vec2::new(ui.available_width(), 34.0),
        egui::TextEdit::singleline(&mut item.title)
            .hint_text("Misal: Pengumpulan Makalah Sistem Operasi")
            .margin(Margin::symmetric(10, 7)),
    );

    ui.add_space(10.0);

    // Row: Category + Priority
    let half_w = ((ui.available_width() - 12.0) * 0.5).max(120.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        ui.allocate_ui_with_layout(Vec2::new(half_w, 0.0), Layout::top_down(Align::LEFT), |ui| {
            ui.label(RichText::new("Kategori / Mata Kuliah").size(12.0).color(INK_2).strong());
            ui.add_space(4.0);
            ui.add_sized(
                Vec2::new(half_w, 34.0),
                egui::TextEdit::singleline(&mut item.course)
                    .hint_text("Akademik / Kerja / Pribadi")
                    .margin(Margin::symmetric(10, 7)),
            );
        });

        ui.allocate_ui_with_layout(Vec2::new(half_w, 0.0), Layout::top_down(Align::LEFT), |ui| {
            ui.label(RichText::new("Prioritas").size(12.0).color(INK_2).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                for (val, lbl, active_bg, active_fg) in [
                    ("high", "Tinggi", DANGER_SOFT, DANGER),
                    ("medium", "Sedang", AMBER_SOFT, AMBER),
                    ("low", "Rendah", GREEN_SOFT, ACCENT_DEEP),
                ] {
                    let sel = item.priority == val;
                    let btn = egui::Button::new(
                        RichText::new(lbl)
                            .size(12.0)
                            .color(if sel { active_fg } else { INK_2 })
                            .strong(),
                    )
                    .fill(if sel { active_bg } else { SURFACE_SUBTLE })
                    .stroke(Stroke::new(1.0, if sel { active_fg } else { BORDER }))
                    .corner_radius(CornerRadius::same(8))
                    .min_size(Vec2::new((half_w - 12.0) / 3.0, 34.0));
                    if ui.add(btn).clicked() {
                        item.priority = val.to_string();
                    }
                }
            });
        });
    });

    ui.add_space(10.0);

    // Row: Date + Time + Flexible Custom Reminder
    let col3_w = ((ui.available_width() - 20.0) / 3.0).max(100.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        ui.allocate_ui_with_layout(Vec2::new(col3_w, 0.0), Layout::top_down(Align::LEFT), |ui| {
            ui.label(RichText::new("Tanggal (YYYY-MM-DD)").size(12.0).color(INK_2).strong());
            ui.add_space(4.0);
            ui.add_sized(
                Vec2::new(col3_w, 34.0),
                egui::TextEdit::singleline(&mut item.deadline)
                    .hint_text("2026-11-30")
                    .margin(Margin::symmetric(10, 7)),
            );
        });

        ui.allocate_ui_with_layout(Vec2::new(col3_w, 0.0), Layout::top_down(Align::LEFT), |ui| {
            ui.label(RichText::new("Waktu (HH:MM)").size(12.0).color(INK_2).strong());
            ui.add_space(4.0);
            ui.add_sized(
                Vec2::new(col3_w, 34.0),
                egui::TextEdit::singleline(&mut item.time)
                    .hint_text("23:59")
                    .margin(Margin::symmetric(10, 7)),
            );
        });

        ui.allocate_ui_with_layout(Vec2::new(col3_w, 0.0), Layout::top_down(Align::LEFT), |ui| {
            ui.label(RichText::new("Pengingat Fleksibel").size(12.0).color(INK_2).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let disabled = rem_unit == "none";
                ui.add_enabled_ui(!disabled, |ui| {
                    ui.add_sized(
                        Vec2::new(48.0, 34.0),
                        egui::TextEdit::singleline(rem_amount)
                            .margin(Margin::symmetric(6, 7)),
                    );
                });
                let unit_lbl = reminder_unit_label(rem_unit);
                egui::ComboBox::from_id_salt(format!("{}_rem_unit", id_prefix))
                    .selected_text(unit_lbl)
                    .width((col3_w - 54.0).max(70.0))
                    .show_ui(ui, |ui| {
                        for (k, lbl) in [
                            ("minutes", "Menit"),
                            ("hours", "Jam"),
                            ("days", "Hari"),
                            ("weeks", "Minggu"),
                            ("none", "Tanpa"),
                        ] {
                            ui.selectable_value(rem_unit, k.to_string(), lbl);
                        }
                    });
            });
        });
    });

    // Sync flexible reminder back to item.reminder
    let amt: i32 = rem_amount.trim().parse().unwrap_or(30);
    item.reminder = combine_reminder(amt, rem_unit);

    ui.add_space(10.0);

    // Description
    ui.label(RichText::new("Catatan / Deskripsi (Opsional)").size(12.0).color(INK_2).strong());
    ui.add_space(4.0);
    ui.add_sized(
        Vec2::new(ui.available_width(), 58.0),
        egui::TextEdit::multiline(&mut item.description)
            .hint_text("Detail pengumpulan, tautan rapat, atau catatan tambahan...")
            .margin(Margin::same(8)),
    );
}

fn show_day_detail_modal(app: &mut NotelyApp, ctx: &egui::Context) {
    let Some(date_iso) = app.calendar_day_detail.clone() else {
        return;
    };
    if draw_scrim(ctx, "day_detail_scrim") {
        app.calendar_day_detail = None;
    }

    let day_tasks: Vec<DeadlineItem> = app
        .deadlines
        .iter()
        .filter(|d| d.deadline == date_iso)
        .cloned()
        .collect();

    egui::Window::new("Detail Tanggal")
        .id(egui::Id::new("day_detail_win"))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(460.0, 0.0))
        .frame(theme::modal_frame())
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Jadwal Tanggal {}", date_iso))
                        .size(16.5)
                        .color(INK)
                        .strong(),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::ghost_btn(ui, "Tutup").clicked() {
                        app.calendar_day_detail = None;
                    }
                });
            });
            ui.add_space(12.0);

            if day_tasks.is_empty() {
                ui.label(
                    RichText::new("Belum ada tugas pada tanggal ini.")
                        .size(13.0)
                        .color(INK_MUTED),
                );
            } else {
                for task in day_tasks {
                    Frame::new()
                        .fill(SURFACE_SUBTLE)
                        .stroke(Stroke::new(1.0, BORDER))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::symmetric(12, 9))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(&task.title)
                                            .size(13.5)
                                            .color(INK)
                                            .strong(),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "{} · {} WIB",
                                            task.course, task.time
                                        ))
                                        .size(11.5)
                                        .color(INK_MUTED),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if theme::secondary_btn(ui, "Edit").clicked() {
                                        app.calendar_day_detail = None;
                                        app.open_manual_deadline_modal(Some(task.clone()), None);
                                    }
                                });
                            });
                        });
                    ui.add_space(6.0);
                }
            }

            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::primary_btn(ui, "+ Tambah di Tanggal Ini").clicked() {
                        app.calendar_day_detail = None;
                        app.open_manual_deadline_modal(None, Some(date_iso));
                    }
                });
            });
        });
}

fn show_delete_deadline_modal(app: &mut NotelyApp, ctx: &egui::Context) {
    let Some(target) = app.delete_deadline_target.clone() else {
        return;
    };
    if draw_scrim(ctx, "delete_deadline_scrim") {
        app.delete_deadline_target = None;
    }

    egui::Window::new("Hapus Tenggat Tugas")
        .id(egui::Id::new("delete_deadline_win"))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(430.0, 0.0))
        .frame(theme::modal_frame())
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Hapus Tenggat Tugas")
                    .size(17.0)
                    .color(INK)
                    .strong(),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!(
                    "Apakah Anda yakin ingin menghapus tugas \"{}\"?",
                    target.title
                ))
                .size(13.5)
                .color(INK_2),
            );
            ui.add_space(6.0);
            Frame::new()
                .fill(DANGER_SOFT)
                .stroke(Stroke::new(1.0, DANGER_BORDER))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::symmetric(10, 7))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("Tugas yang telah dihapus tidak dapat dipulihkan kembali.")
                            .size(12.0)
                            .color(DANGER),
                    );
                });
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::danger_btn(ui, "Hapus Tugas").clicked() {
                        app.confirm_delete_deadline(&target.id);
                    }
                    if theme::secondary_btn(ui, "Batal").clicked() {
                        app.delete_deadline_target = None;
                    }
                });
            });
        });
}

fn show_complete_deadline_modal(app: &mut NotelyApp, ctx: &egui::Context) {
    let Some(target) = app.complete_deadline_target.clone() else {
        return;
    };
    if draw_scrim(ctx, "complete_deadline_scrim") {
        app.complete_deadline_target = None;
    }

    egui::Window::new("Selesaikan Tugas")
        .id(egui::Id::new("complete_deadline_win"))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(420.0, 0.0))
        .frame(theme::modal_frame())
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Tandai Tugas Selesai?")
                    .size(17.0)
                    .color(INK)
                    .strong(),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!(
                    "Tugas \"{}\" akan dipindahkan ke daftar selesai.",
                    target.title
                ))
                .size(13.5)
                .color(INK_2),
            );
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::primary_btn(ui, "Ya, Tandai Selesai").clicked() {
                        app.complete_deadline_target = None;
                        app.toggle_deadline_status(&target.id);
                    }
                    if theme::secondary_btn(ui, "Batal").clicked() {
                        app.complete_deadline_target = None;
                    }
                });
            });
        });
}

/// Full-Screen Crisp Zoom Lightbox so user can verify attached images clearly!
fn show_image_lightbox(app: &mut NotelyApp, ctx: &egui::Context, idx: usize) {
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.lightbox_image_idx = None;
        return;
    }

    let Some(img) = app.ai_images.get(idx).cloned() else {
        app.lightbox_image_idx = None;
        return;
    };

    egui::Area::new(egui::Id::new("image_lightbox_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::Pos2::ZERO)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let bg_resp = ui.allocate_rect(screen, Sense::click());
            ui.painter()
                .rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(205));
            if bg_resp.clicked() {
                app.lightbox_image_idx = None;
            }
        });

    let screen = ctx.screen_rect();
    let max_w = (screen.width() * 0.86).max(360.0);
    let max_h = (screen.height() * 0.82).max(280.0);

    egui::Window::new("Pratinjau Gambar")
        .id(egui::Id::new("image_lightbox_win"))
        .order(egui::Order::Foreground)
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(theme::modal_frame())
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} ({} x {} px)",
                        img.name, img.width, img.height
                    ))
                    .size(14.0)
                    .color(INK)
                    .strong(),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::secondary_btn(ui, "Tutup (Esc)").clicked() {
                        app.lightbox_image_idx = None;
                    }
                });
            });
            ui.add_space(10.0);

            if let Some(tex) = app.ai_image_textures.get(idx) {
                let aspect = img.width as f32 / (img.height.max(1) as f32);
                let mut disp_w = (max_w - 48.0).min(img.width as f32);
                let mut disp_h = disp_w / aspect;
                if disp_h > max_h - 80.0 {
                    disp_h = max_h - 80.0;
                    disp_w = disp_h * aspect;
                }
                ui.vertical_centered(|ui| {
                    let sized = egui::load::SizedTexture::new(tex.id(), tex.size_vec2());
                    ui.add(
                        egui::Image::from_texture(sized)
                            .fit_to_exact_size(Vec2::new(disp_w.max(120.0), disp_h.max(120.0)))
                            .corner_radius(CornerRadius::same(10)),
                    );
                });
            }
        });
}

fn draw_scrim(ctx: &egui::Context, id: &str) -> bool {
    let mut clicked = false;
    egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Middle)
        .fixed_pos(egui::Pos2::ZERO)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let resp = ui.allocate_rect(screen, Sense::click());
            ui.painter()
                .rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(110));
            clicked = resp.clicked();
        });
    clicked
}

fn days_in(year: i32, month: u32) -> u32 {
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    let cur = NaiveDate::from_ymd_opt(year, month, 1);
    match (cur, next) {
        (Some(c), Some(n)) => (n - c).num_days() as u32,
        _ => 30,
    }
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i >= max_chars {
            out.push_str("...");
            break;
        }
        out.push(ch);
    }
    out
}

fn reminder_unit_label(unit: &str) -> &'static str {
    match unit {
        "hours" => "Jam",
        "days" => "Hari",
        "weeks" => "Minggu",
        "none" => "Tanpa",
        _ => "Menit",
    }
}

fn format_reminder_label(mins: i32) -> String {
    let (amt, unit) = split_reminder(mins);
    if unit == "none" {
        return "Tanpa pengingat".to_string();
    }
    let u = match unit {
        "hours" => "jam",
        "days" => "hari",
        "weeks" => "minggu",
        _ => "menit",
    };
    format!("{} {} sebelumnya", amt, u)
}
