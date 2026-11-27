use crate::app::NotelyApp;
use crate::theme::{
    self, ACCENT, ACCENT_DEEP, ACCENT_SOFT, BORDER, DANGER, DANGER_SOFT, INK, INK_2, INK_MUTED,
    SURFACE, SURFACE_SUBTLE,
};
use crate::types::APP_VERSION;
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, RichText, Sense, Stroke, Vec2,
};

/// Compact, refined native startup splash (460x290 borderless window).
/// Shows crisp Notely mark, smooth progress bar, live initialization step,
/// and only shows Retry/Skip controls if startup fails or times out.
pub fn show(app: &mut NotelyApp, ctx: &egui::Context) {
    let time = ctx.input(|i| i.time);
    ctx.request_repaint(); // Smooth 60fps progress bar interpolation & shimmer

    // Smoothly interpolate visual progress toward target progress
    let diff = app.splash_target_progress - app.splash_visual_progress;
    app.splash_visual_progress += diff * 0.14;
    if diff.abs() < 0.002 {
        app.splash_visual_progress = app.splash_target_progress;
    }

    egui::CentralPanel::default()
        .frame(
            Frame::new()
                .fill(SURFACE)
                .stroke(Stroke::new(1.0, BORDER))
                .corner_radius(CornerRadius::same(14))
                .inner_margin(Margin::symmetric(30, 26)),
        )
        .show(ctx, |ui| {
            // Allow dragging the borderless splash window from anywhere on empty space
            let drag_rect = ui.max_rect();
            let drag_resp = ui.interact(drag_rect, egui::Id::new("splash_drag"), Sense::drag());
            if drag_resp.dragged() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space(6.0);

                // Brand Row: Logo + Title + Version pill
                ui.horizontal(|ui| {
                    let total_w = 196.0;
                    let pad = ((ui.available_width() - total_w) * 0.5).max(0.0);
                    ui.add_space(pad);

                    if let Some(tex) = &app.logo_texture {
                        let sized = egui::load::SizedTexture::new(tex.id(), tex.size_vec2());
                        ui.add(
                            egui::Image::from_texture(sized)
                                .fit_to_exact_size(Vec2::splat(40.0))
                                .corner_radius(CornerRadius::same(10)),
                        );
                    } else {
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
                        ui.painter()
                            .rect_filled(r, CornerRadius::same(10), ACCENT);
                        ui.painter().text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            "N",
                            egui::FontId::proportional(20.0),
                            Color32::WHITE,
                        );
                    }

                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Notely").size(22.0).color(INK).strong());
                            ui.add_space(4.0);
                            Frame::new()
                                .fill(ACCENT_SOFT)
                                .corner_radius(CornerRadius::same(10))
                                .inner_margin(Margin::symmetric(7, 2))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(format!("v{}", APP_VERSION))
                                            .size(11.0)
                                            .color(ACCENT_DEEP)
                                            .strong(),
                                    );
                                });
                        });
                        ui.label(
                            RichText::new("Ruang catatan & transkripsi AI lokal")
                                .size(11.5)
                                .color(INK_MUTED),
                        );
                    });
                });

                ui.add_space(24.0);

                // Smooth Progress Bar with subtle moving sheen
                let bar_w = ui.available_width().min(380.0);
                let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(bar_w, 7.0), Sense::hover());
                let painter = ui.painter();
                painter.rect_filled(bar_rect, CornerRadius::same(4), ACCENT_SOFT);

                let pct = app.splash_visual_progress.clamp(0.03, 1.0);
                let fill_rect = egui::Rect::from_min_size(
                    bar_rect.min,
                    Vec2::new(bar_rect.width() * pct, bar_rect.height()),
                );
                painter.rect_filled(fill_rect, CornerRadius::same(4), ACCENT);

                // Animated highlight sweep inside filled bar
                if app.splash_error.is_none() && pct < 0.99 {
                    let sheen_t = ((time * 0.9) % 1.0) as f32;
                    let sheen_w = 56.0_f32.min(fill_rect.width());
                    let sheen_x = fill_rect.left() + (fill_rect.width() - sheen_w) * sheen_t;
                    let sheen_rect = egui::Rect::from_min_size(
                        egui::pos2(sheen_x, fill_rect.top()),
                        Vec2::new(sheen_w, fill_rect.height()),
                    );
                    painter.rect_filled(
                        sheen_rect,
                        CornerRadius::same(4),
                        Color32::from_rgba_unmultiplied(255, 255, 255, 70),
                    );
                }

                ui.add_space(12.0);

                // Current task title & percentage
                ui.allocate_ui_with_layout(
                    Vec2::new(bar_w, 20.0),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        let title_col = if app.splash_error.is_some() {
                            DANGER
                        } else {
                            INK
                        };
                        ui.label(
                            RichText::new(&app.splash_step_title)
                                .size(13.0)
                                .color(title_col)
                                .strong(),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("{}%", (pct * 100.0).round() as u32))
                                    .size(12.0)
                                    .color(ACCENT)
                                    .strong(),
                            );
                        });
                    },
                );

                // Sub-detail line
                ui.allocate_ui_with_layout(
                    Vec2::new(bar_w, 18.0),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.label(
                            RichText::new(&app.splash_step_detail)
                                .size(11.5)
                                .color(INK_MUTED),
                        );
                    },
                );

                ui.add_space(16.0);

                // Hardware / status chip OR error recovery buttons (never shown on normal loading!)
                if let Some(err) = app.splash_error.clone() {
                    Frame::new()
                        .fill(DANGER_SOFT)
                        .stroke(Stroke::new(1.0, theme::DANGER_BORDER))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.label(RichText::new(err).size(11.5).color(DANGER));
                        });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if theme::secondary_btn(ui, "Lanjutkan ke Beranda").clicked() {
                            app.finish_splash_transition(ctx);
                        }
                        if theme::primary_btn(ui, "Coba Lagi").clicked() {
                            app.restart_boot_sequence();
                        }
                    });
                } else {
                    Frame::new()
                        .fill(SURFACE_SUBTLE)
                        .stroke(Stroke::new(1.0, BORDER))
                        .corner_radius(CornerRadius::same(14))
                        .inner_margin(Margin::symmetric(12, 5))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 7.0;
                                let pulse = (0.55 + 0.45 * (time * 4.5).sin() as f32)
                                    .clamp(0.3, 1.0);
                                let dot_col = Color32::from_rgba_unmultiplied(
                                    ACCENT.r(),
                                    ACCENT.g(),
                                    ACCENT.b(),
                                    (pulse * 255.0) as u8,
                                );
                                let (dr, _) =
                                    ui.allocate_exact_size(Vec2::splat(7.0), Sense::hover());
                                ui.painter().circle_filled(dr.center(), 3.4, dot_col);
                                ui.label(
                                    RichText::new(&app.splash_hw_chip)
                                        .size(11.5)
                                        .color(INK_2),
                                );
                            });
                        });
                }
            });
        });
}
