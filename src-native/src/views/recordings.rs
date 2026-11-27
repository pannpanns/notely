use crate::app::NotelyApp;
use crate::theme::{
    self, ACCENT, ACCENT_BORDER, ACCENT_DEEP, ACCENT_SOFT, BORDER, DANGER, INK, INK_2, INK_MUTED,
    SURFACE, SURFACE_SUBTLE,
};
use crate::types::{format_bytes, format_when_pretty};
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, Sense, Stroke, StrokeKind, Vec2,
};

pub fn show(app: &mut NotelyApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    let content_w = avail_w.min(960.0_f32).max(340.0_f32);
    let side_pad = ((avail_w - content_w) * 0.5_f32).max(20.0_f32);

    egui::ScrollArea::vertical()
        .id_salt("recordings_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(22.0_f32);
            ui.horizontal(|ui| {
                ui.add_space(side_pad);
                ui.allocate_ui_with_layout(
                    Vec2::new(content_w, 0.0_f32),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        // ── Header & Actions ──
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                if let Some(fid) = &app.active_folder_id.clone() {
                                    let fname = app
                                        .folders
                                        .iter()
                                        .find(|f| &f.id == fid)
                                        .map(|f| f.name.clone())
                                        .unwrap_or_else(|| "Folder".to_string());
                                    ui.horizontal(|ui| {
                                        if theme::ghost_btn(ui, "Semua Rekaman").clicked() {
                                            app.active_folder_id = None;
                                        }
                                        ui.label(theme::regular("/", 15.0_f32, INK_MUTED));
                                        ui.label(theme::bold(fname, 20.0_f32, INK));
                                    });
                                } else {
                                    ui.label(theme::bold("Semua Rekaman", 23.0_f32, INK));
                                }
                                ui.label(theme::regular(
                                    "Kelola folder, buka transkrip, atau impor rekaman audio/video.",
                                    13.0_f32,
                                    INK_2,
                                ));
                            });

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let target_folder =
                                    app.active_folder_id.clone().unwrap_or_default();
                                if theme::primary_btn(ui, "Impor Audio").clicked() {
                                    app.pick_and_import_audio(&target_folder);
                                }
                                if theme::secondary_btn(ui, "+ Folder Baru").clicked() {
                                    app.folder_modal_id = None;
                                    app.folder_modal_name.clear();
                                    app.show_folder_modal = true;
                                }
                            });
                        });

                        ui.add_space(14.0_f32);

                        // ── Search Bar ──
                        Frame::new()
                            .fill(SURFACE)
                            .stroke(Stroke::new(1.0_f32, BORDER))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::symmetric(10, 4))
                            .show(ui, |ui| {
                                ui.set_width(content_w.min(380.0_f32));
                                ui.horizontal(|ui| {
                                    theme::icon_widget(
                                        ui,
                                        theme::NavIcon::Search,
                                        15.0_f32,
                                        INK_MUTED,
                                    );
                                    ui.add(
                                        egui::TextEdit::singleline(&mut app.recordings_search)
                                            .frame(false)
                                            .desired_width(f32::INFINITY)
                                            .hint_text("Cari judul rekaman..."),
                                    );
                                });
                            });

                        ui.add_space(16.0_f32);

                        // ── Folders Section (root) ──
                        if app.active_folder_id.is_none() && !app.folders.is_empty() {
                            ui.label(theme::semi("Folder", 13.5_f32, INK_2));
                            ui.add_space(8.0_f32);
                            let folders = app.folders.clone();
                            for folder in folders {
                                let count = app
                                    .sessions
                                    .iter()
                                    .filter(|s| s.folder == folder.id)
                                    .count();
                                Frame::new()
                                    .fill(SURFACE)
                                    .stroke(Stroke::new(1.0_f32, BORDER))
                                    .corner_radius(CornerRadius::same(12))
                                    .inner_margin(Margin::symmetric(14, 11))
                                    .show(ui, |ui| {
                                        ui.set_width(content_w - 30.0_f32);
                                        ui.horizontal(|ui| {
                                            let (ic, _) = ui.allocate_exact_size(
                                                Vec2::splat(36.0_f32),
                                                Sense::hover(),
                                            );
                                            ui.painter().rect_filled(
                                                ic,
                                                CornerRadius::same(9),
                                                ACCENT_SOFT,
                                            );
                                            theme::paint_icon(
                                                ui,
                                                ic.shrink(8.0_f32),
                                                theme::NavIcon::Folder,
                                                ACCENT,
                                            );
                                            ui.add_space(6.0_f32);
                                            ui.vertical(|ui| {
                                                ui.label(theme::semi(
                                                    &folder.name,
                                                    14.0_f32,
                                                    INK,
                                                ));
                                                ui.label(theme::regular(
                                                    format!("{} rekaman", count),
                                                    11.5_f32,
                                                    INK_MUTED,
                                                ));
                                            });

                                            ui.with_layout(
                                                Layout::right_to_left(Align::Center),
                                                |ui| {
                                                    if theme::danger_soft_btn(ui, "Hapus").clicked()
                                                    {
                                                        app.delete_folder(&folder.id);
                                                    }
                                                    if theme::secondary_btn(ui, "Ubah Nama")
                                                        .clicked()
                                                    {
                                                        app.folder_modal_id =
                                                            Some(folder.id.clone());
                                                        app.folder_modal_name =
                                                            folder.name.clone();
                                                        app.show_folder_modal = true;
                                                    }
                                                    if theme::primary_btn(ui, "Buka").clicked() {
                                                        app.active_folder_id =
                                                            Some(folder.id.clone());
                                                    }
                                                },
                                            );
                                        });
                                    });
                                ui.add_space(8.0_f32);
                            }
                            ui.add_space(10.0_f32);
                        }

                        // ── Filtered Recordings List ──
                        let q = app.recordings_search.trim().to_lowercase();
                        let filtered: Vec<_> = app
                            .sessions
                            .iter()
                            .filter(|s| {
                                let in_folder = match &app.active_folder_id {
                                    Some(fid) => &s.folder == fid,
                                    None => true,
                                };
                                let matches_q =
                                    q.is_empty() || s.title.to_lowercase().contains(&q);
                                in_folder && matches_q
                            })
                            .cloned()
                            .collect();

                        ui.label(theme::semi(
                            format!("Rekaman ({})", filtered.len()),
                            13.5_f32,
                            INK_2,
                        ));
                        ui.add_space(8.0_f32);

                        if filtered.is_empty() {
                            theme::card_frame().show(ui, |ui| {
                                ui.set_width(content_w - 38.0_f32);
                                ui.vertical_centered(|ui| {
                                    ui.add_space(20.0_f32);
                                    ui.label(theme::regular(
                                        "Tidak ada rekaman yang sesuai.",
                                        13.5_f32,
                                        INK_MUTED,
                                    ));
                                    ui.add_space(20.0_f32);
                                });
                            });
                        } else {
                            for item in filtered {
                                Frame::new()
                                    .fill(SURFACE)
                                    .stroke(Stroke::new(1.0_f32, BORDER))
                                    .corner_radius(CornerRadius::same(12))
                                    .inner_margin(Margin::symmetric(14, 12))
                                    .show(ui, |ui| {
                                        ui.set_width(content_w - 30.0_f32);
                                        ui.horizontal(|ui| {
                                            let (ic, _) = ui.allocate_exact_size(
                                                Vec2::splat(36.0_f32),
                                                Sense::hover(),
                                            );
                                            ui.painter().rect_filled(
                                                ic,
                                                CornerRadius::same(9),
                                                SURFACE_SUBTLE,
                                            );
                                            ui.painter().rect_stroke(
                                                ic,
                                                CornerRadius::same(9),
                                                Stroke::new(1.0_f32, BORDER),
                                                StrokeKind::Inside,
                                            );
                                            theme::paint_icon(
                                                ui,
                                                ic.shrink(8.0_f32),
                                                theme::NavIcon::Notes,
                                                ACCENT,
                                            );
                                            ui.add_space(8.0_f32);
                                            ui.vertical(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(theme::semi(
                                                        &item.title,
                                                        14.0_f32,
                                                        INK,
                                                    ));
                                                    if item.has_summary {
                                                        Frame::new()
                                                            .fill(ACCENT_SOFT)
                                                            .stroke(Stroke::new(
                                                                1.0_f32,
                                                                ACCENT_BORDER,
                                                            ))
                                                            .corner_radius(CornerRadius::same(8))
                                                            .inner_margin(Margin::symmetric(7, 2))
                                                            .show(ui, |ui| {
                                                                ui.label(theme::semi(
                                                                    "Ringkasan",
                                                                    10.5_f32,
                                                                    ACCENT_DEEP,
                                                                ));
                                                            });
                                                    }
                                                });
                                                let meta = format!(
                                                    "{} · Audio {}",
                                                    format_when_pretty(&item.when),
                                                    format_bytes(item.audio_bytes)
                                                );
                                                ui.label(theme::regular(
                                                    meta,
                                                    11.5_f32,
                                                    INK_MUTED,
                                                ));
                                            });

                                            ui.with_layout(
                                                Layout::right_to_left(Align::Center),
                                                |ui| {
                                                    if theme::danger_soft_btn(ui, "Hapus").clicked()
                                                    {
                                                        app.delete_session_target =
                                                            Some(item.clone());
                                                    }
                                                    if theme::secondary_btn(ui, "Ubah").clicked() {
                                                        app.rename_session_id =
                                                            Some(item.id.clone());
                                                        app.rename_session_title =
                                                            item.title.clone();
                                                        app.rename_session_folder =
                                                            item.folder.clone();
                                                    }
                                                    if theme::primary_btn(ui, "Buka").clicked() {
                                                        app.open_session(&item.id);
                                                    }
                                                },
                                            );
                                        });
                                    });
                                ui.add_space(8.0_f32);
                            }
                        }

                        ui.add_space(92.0_f32);
                    },
                );
            });
        });

    if app.show_folder_modal {
        show_folder_modal(app, ctx);
    }
    if app.rename_session_id.is_some() {
        show_rename_session_modal(app, ctx);
    }
    if app.delete_session_target.is_some() {
        show_delete_session_modal(app, ctx);
    }
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

fn show_folder_modal(app: &mut NotelyApp, ctx: &egui::Context) {
    if draw_scrim(ctx, "folder_modal_scrim") {
        app.show_folder_modal = false;
    }
    let is_edit = app.folder_modal_id.is_some();
    egui::Window::new("Folder")
        .id(egui::Id::new("folder_modal_win"))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(400.0_f32, 0.0_f32))
        .frame(theme::modal_frame())
        .show(ctx, |ui| {
            ui.label(theme::bold(
                if is_edit {
                    "Ubah Nama Folder"
                } else {
                    "Buat Folder Baru"
                },
                16.5_f32,
                INK,
            ));
            ui.add_space(12.0_f32);
            ui.add_sized(
                Vec2::new(ui.available_width(), 36.0_f32),
                egui::TextEdit::singleline(&mut app.folder_modal_name)
                    .hint_text("Nama folder, misal: Kuliah Semester 5")
                    .margin(Margin::symmetric(10, 8)),
            );
            ui.add_space(16.0_f32);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::primary_btn(ui, "Simpan").clicked() {
                        app.submit_folder_modal();
                    }
                    if theme::secondary_btn(ui, "Batal").clicked() {
                        app.show_folder_modal = false;
                    }
                });
            });
        });
}

fn show_rename_session_modal(app: &mut NotelyApp, ctx: &egui::Context) {
    if draw_scrim(ctx, "rename_session_scrim") {
        app.rename_session_id = None;
    }
    egui::Window::new("Ubah Rekaman")
        .id(egui::Id::new("rename_session_win"))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(430.0_f32, 0.0_f32))
        .frame(theme::modal_frame())
        .show(ctx, |ui| {
            ui.label(theme::bold("Ubah Detail Rekaman", 16.5_f32, INK));
            ui.add_space(12.0_f32);
            ui.label(theme::semi("Judul Rekaman", 12.0_f32, INK_2));
            ui.add_space(4.0_f32);
            ui.add_sized(
                Vec2::new(ui.available_width(), 36.0_f32),
                egui::TextEdit::singleline(&mut app.rename_session_title)
                    .margin(Margin::symmetric(10, 8)),
            );
            ui.add_space(12.0_f32);
            ui.label(theme::semi("Folder", 12.0_f32, INK_2));
            ui.add_space(4.0_f32);

            let current_fname = if app.rename_session_folder.is_empty() {
                "Tanpa Folder".to_string()
            } else {
                app.folders
                    .iter()
                    .find(|f| f.id == app.rename_session_folder)
                    .map(|f| f.name.clone())
                    .unwrap_or_else(|| "Tanpa Folder".to_string())
            };
            egui::ComboBox::from_id_salt("rename_session_folder_cb")
                .selected_text(current_fname)
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app.rename_session_folder,
                        String::new(),
                        "Tanpa Folder",
                    );
                    for f in &app.folders {
                        ui.selectable_value(
                            &mut app.rename_session_folder,
                            f.id.clone(),
                            &f.name,
                        );
                    }
                });

            ui.add_space(16.0_f32);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::primary_btn(ui, "Simpan").clicked() {
                        app.submit_rename_session();
                    }
                    if theme::secondary_btn(ui, "Batal").clicked() {
                        app.rename_session_id = None;
                    }
                });
            });
        });
}

fn show_delete_session_modal(app: &mut NotelyApp, ctx: &egui::Context) {
    let Some(target) = app.delete_session_target.clone() else {
        return;
    };
    if draw_scrim(ctx, "delete_session_scrim") {
        app.delete_session_target = None;
    }
    egui::Window::new("Hapus Rekaman")
        .id(egui::Id::new("delete_session_win"))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(440.0_f32, 0.0_f32))
        .frame(theme::modal_frame())
        .show(ctx, |ui| {
            ui.label(theme::bold("Hapus Rekaman", 17.0_f32, INK));
            ui.add_space(8.0_f32);
            ui.label(theme::regular(
                format!("Apakah Anda yakin ingin menghapus \"{}\"?", target.title),
                13.5_f32,
                INK_2,
            ));
            ui.add_space(4.0_f32);
            ui.label(theme::regular(
                "Rekaman yang dihapus tidak dapat dipulihkan kembali.",
                12.0_f32,
                DANGER,
            ));
            ui.add_space(18.0_f32);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::danger_btn(ui, "Hapus Semua").clicked() {
                        app.confirm_delete_session(&target.id, false);
                    }
                    if target.audio_bytes > 0
                        && theme::secondary_btn(ui, "Hapus Audio Saja").clicked()
                    {
                        app.confirm_delete_session(&target.id, true);
                    }
                    if theme::ghost_btn(ui, "Batal").clicked() {
                        app.delete_session_target = None;
                    }
                });
            });
        });
}
