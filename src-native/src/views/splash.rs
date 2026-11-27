use crate::app::NotelyApp;
use crate::theme::{
    self, ACCENT, ACCENT_DEEP, ACCENT_SOFT, BORDER, CANVAS, DANGER, DANGER_BORDER, DANGER_SOFT,
    INK, INK_MUTED, SURFACE,
};
use crate::types::APP_VERSION;
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, Sense, Shadow, Stroke, Vec2,
};

/// Integrated in-app loading screen rendered centered inside the full desktop window
/// on Notely's Soft Sage canvas. No tiny floating borderless box, zero blur, crisp studio framing.
pub fn show(app: &mut NotelyApp, ctx: &egui::Context) {
    let time = ctx.input(|i| i.time);
    ctx.request_repaint();

    let diff = app.splash_target_progress - app.splash_visual_progress;
    app.splash_visual_progress += diff * 0.14_f32;
    if diff.abs() < 0.002_f32 {
        app.splash_visual_progress = app.splash_target_progress;
    }

    egui::CentralPanel::default()
        .frame(Frame::new().fill(CANVAS).inner_margin(Margin::same(24)))
        .show(ctx, |ui| {
            let avail = ui.available_size();
            let card_w = 420.0_f32.min((avail.x - 32.0_f32).max(280.0_f32));
            let card_h = if app.splash_error.is_some() {
                320.0_f32
            } else {
                238.0_f32
            };
            let top_pad = ((avail.y - card_h) * 0.44_f32).max(16.0_f32);

            ui.vertical_centered(|ui| {
                ui.add_space(top_pad);

                Frame::new()
                    .fill(SURFACE)
                    .stroke(Stroke::new(1.0_f32, BORDER))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::symmetric(30, 28))
                    .shadow(Shadow {
                        offset: [0, 12],
                        blur: 32,
                        spread: 0,
                        color: Color32::from_rgba_unmultiplied(16, 42, 32, 18),
                    })
                    .show(ui, |ui| {
                        ui.set_width(card_w);

                        // ── Centered Brand Header ──
                        ui.horizontal(|ui| {
                            let header_w = 248.0_f32;
                            let pad = ((card_w - header_w) * 0.5_f32).max(0.0_f32);
                            ui.add_space(pad);

                            if let Some(tex) = &app.logo_texture {
                                let sized =
                                    egui::load::SizedTexture::new(tex.id(), tex.size_vec2());
                                ui.add(
                                    egui::Image::from_texture(sized)
                                        .fit_to_exact_size(Vec2::splat(44.0_f32))
                                        .corner_radius(CornerRadius::same(11)),
                                );
                            } else {
                                let (r, _) =
                                    ui.allocate_exact_size(Vec2::splat(44.0_f32), Sense::hover());
                                ui.painter().rect_filled(r, CornerRadius::same(11), ACCENT);
                            }

                            ui.add_space(10.0_f32);
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 2.0_f32;
                                ui.horizontal(|ui| {
                                    ui.label(theme::bold("Notely", 23.0_f32, INK));
                                    ui.add_space(4.0_f32);
                                    Frame::new()
                                        .fill(ACCENT_SOFT)
                                        .corner_radius(CornerRadius::same(9))
                                        .inner_margin(Margin::symmetric(8, 2))
                                        .show(ui, |ui| {
                                            ui.label(theme::semi(
                                                format!("v{}", APP_VERSION),
                                                11.0_f32,
                                                ACCENT_DEEP,
                                            ));
                                        });
                                });
                                ui.label(theme::regular(
                                    "Ruang catatan & transkripsi AI lokal",
                                    12.0_f32,
                                    INK_MUTED,
                                ));
                            });
                        });

                        ui.add_space(26.0_f32);

                        // ── Crisp Smooth Progress Bar ──
                        let (bar_rect, _) = ui.allocate_exact_size(
                            Vec2::new(card_w, 6.0_f32),
                            Sense::hover(),
                        );
                        let painter = ui.painter();
                        painter.rect_filled(bar_rect, CornerRadius::same(3), ACCENT_SOFT);

                        let pct = app.splash_visual_progress.clamp(0.04_f32, 1.0_f32);
                        let fill_rect = egui::Rect::from_min_size(
                            bar_rect.min,
                            Vec2::new(bar_rect.width() * pct, bar_rect.height()),
                        );
                        painter.rect_filled(fill_rect, CornerRadius::same(3), ACCENT);

                        if app.splash_error.is_none() && pct < 0.99_f32 {
                            let sheen_t = ((time * 0.85) % 1.0) as f32;
                            let sheen_w = 48.0_f32.min(fill_rect.width());
                            let sheen_x =
                                fill_rect.left() + (fill_rect.width() - sheen_w) * sheen_t;
                            let sheen_rect = egui::Rect::from_min_size(
                                egui::pos2(sheen_x, fill_rect.top()),
                                Vec2::new(sheen_w, fill_rect.height()),
                            );
                            painter.rect_filled(
                                sheen_rect,
                                CornerRadius::same(3),
                                Color32::from_rgba_unmultiplied(255, 255, 255, 85),
                            );
                        }

                        ui.add_space(12.0_f32);

                        // ── Single Clean Status Row + Detail ──
                        ui.allocate_ui_with_layout(
                            Vec2::new(card_w, 20.0_f32),
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                let title_col = if app.splash_error.is_some() {
                                    DANGER
                                } else {
                                    INK
                                };
                                ui.label(theme::semi(&app.splash_step_title, 13.0_f32, title_col));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(theme::semi(
                                        format!("{}%", (pct * 100.0_f32).round() as u32),
                                        12.5_f32,
                                        ACCENT,
                                    ));
                                });
                            },
                        );

                        ui.add_space(2.0_f32);
                        ui.allocate_ui_with_layout(
                            Vec2::new(card_w, 18.0_f32),
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                ui.label(theme::regular(
                                    &app.splash_step_detail,
                                    11.5_f32,
                                    INK_MUTED,
                                ));
                            },
                        );

                        if !app.splash_hw_chip.is_empty() && app.splash_error.is_none() {
                            ui.add_space(14.0_f32);
                            ui.separator();
                            ui.add_space(8.0_f32);
                            ui.horizontal(|ui| {
                                let (dr, _) = ui.allocate_exact_size(
                                    Vec2::splat(7.0_f32),
                                    Sense::hover(),
                                );
                                ui.painter()
                                    .circle_filled(dr.center(), 3.2_f32, ACCENT);
                                ui.label(theme::regular(&app.splash_hw_chip, 11.5_f32, INK_MUTED));
                            });
                        }

                        // Error-only recovery actions
                        if let Some(err) = app.splash_error.clone() {
                            ui.add_space(14.0_f32);
                            Frame::new()
                                .fill(DANGER_SOFT)
                                .stroke(Stroke::new(1.0_f32, DANGER_BORDER))
                                .corner_radius(CornerRadius::same(10))
                                .inner_margin(Margin::symmetric(12, 9))
                                .show(ui, |ui| {
                                    ui.set_width(card_w - 24.0_f32);
                                    ui.label(theme::regular(err, 12.0_f32, DANGER));
                                });
                            ui.add_space(12.0_f32);
                            ui.horizontal(|ui| {
                                if theme::secondary_btn(ui, "Lanjutkan ke Beranda").clicked() {
                                    app.finish_splash_transition(ctx);
                                }
                                if theme::primary_btn(ui, "Coba Lagi").clicked() {
                                    app.restart_boot_sequence();
                                }
                            });
                        }
                    });
            });
        });
}
