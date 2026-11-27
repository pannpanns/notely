use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame, Margin,
    Painter, Pos2, Rect, Response, RichText, Sense, Shadow, Shape, Stroke, StrokeKind, TextStyle,
    Ui, Vec2,
};
use std::sync::Arc;

// ── Notely Brand Color Palette (100% Preserved) ──
pub const ACCENT: Color32 = Color32::from_rgb(0x18, 0x78, 0x6C); // Forest Teal
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x13, 0x63, 0x58);
pub const ACCENT_DEEP: Color32 = Color32::from_rgb(0x0F, 0x4F, 0x46);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0xEA, 0xF5, 0xF0);
pub const ACCENT_BORDER: Color32 = Color32::from_rgb(0xC2, 0xE2, 0xD7);

pub const CANVAS: Color32 = Color32::from_rgb(0xF3, 0xF6, 0xF4); // Soft Sage
pub const SIDEBAR_BG: Color32 = Color32::from_rgb(0xF8, 0xFA, 0xF9);
pub const SURFACE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const SURFACE_SUBTLE: Color32 = Color32::from_rgb(0xF7, 0xFA, 0xF8);
pub const SURFACE_HOVER: Color32 = Color32::from_rgb(0xEE, 0xF4, 0xF1);

pub const BORDER: Color32 = Color32::from_rgb(0xE1, 0xEA, 0xE5);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(0xCB, 0xDA, 0xD2);

pub const INK: Color32 = Color32::from_rgb(0x17, 0x2A, 0x23); // Deep Pine Ink
pub const INK_2: Color32 = Color32::from_rgb(0x4B, 0x61, 0x57);
pub const INK_MUTED: Color32 = Color32::from_rgb(0x7D, 0x91, 0x87);

pub const DANGER: Color32 = Color32::from_rgb(0xD9, 0x38, 0x45);
pub const DANGER_HOVER: Color32 = Color32::from_rgb(0xBE, 0x2B, 0x37);
pub const DANGER_SOFT: Color32 = Color32::from_rgb(0xFE, 0xF2, 0xF2);
pub const DANGER_BORDER: Color32 = Color32::from_rgb(0xFA, 0xCA, 0xCF);

pub const GREEN: Color32 = Color32::from_rgb(0x16, 0xA3, 0x4A);
pub const GREEN_SOFT: Color32 = Color32::from_rgb(0xEC, 0xFD, 0xF5);
pub const AMBER: Color32 = Color32::from_rgb(0xD9, 0x77, 0x06);
pub const AMBER_SOFT: Color32 = Color32::from_rgb(0xFF, 0xFB, 0xEB);
pub const BLUE_SOFT: Color32 = Color32::from_rgb(0xEF, 0xF6, 0xFF);
pub const BLUE_INK: Color32 = Color32::from_rgb(0x1D, 0x4E, 0xD8);

pub fn apply_notely_theme(ctx: &egui::Context) {
    // Load crisp Windows Segoe UI font if available, keeping egui default as fallback
    let mut fonts = FontDefinitions::default();
    let font_candidates = [
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\arial.ttf",
    ];
    for path in font_candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert("segoe_ui".to_owned(), Arc::new(FontData::from_owned(bytes)));
            if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
                family.insert(0, "segoe_ui".to_owned());
            }
            break;
        }
    }
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\consola.ttf") {
        fonts
            .font_data
            .insert("consolas".to_owned(), Arc::new(FontData::from_owned(bytes)));
        if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
            family.insert(0, "consolas".to_owned());
        }
    }
    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    style.animation_time = 0.16;

    // Typography hierarchy (4px spacing discipline)
    style.text_styles = [
        (TextStyle::Heading, FontId::new(22.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(13.5, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(11.5, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
    ]
    .into();

    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(12.0, 7.0);
    style.spacing.window_margin = Margin::same(16);
    style.spacing.scroll = egui::style::ScrollStyle::thin();

    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = CANVAS;
    visuals.window_fill = SURFACE;
    visuals.extreme_bg_color = SURFACE;
    visuals.faint_bg_color = SURFACE_SUBTLE;
    visuals.override_text_color = Some(INK);
    visuals.window_corner_radius = CornerRadius::same(16);
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_shadow = Shadow {
        offset: [0, 10],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(26),
    };
    visuals.popup_shadow = Shadow {
        offset: [0, 6],
        blur: 18,
        spread: 0,
        color: Color32::from_black_alpha(22),
    };

    visuals.selection.bg_fill = ACCENT_SOFT;
    visuals.selection.stroke = Stroke::new(1.2, ACCENT);

    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.weak_bg_fill = SURFACE_SUBTLE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, INK);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);

    visuals.widgets.inactive.bg_fill = SURFACE;
    visuals.widgets.inactive.weak_bg_fill = SURFACE;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, INK);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);

    visuals.widgets.hovered.bg_fill = SURFACE_HOVER;
    visuals.widgets.hovered.weak_bg_fill = SURFACE_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, INK);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);

    visuals.widgets.active.bg_fill = ACCENT_SOFT;
    visuals.widgets.active.weak_bg_fill = ACCENT_SOFT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.2, ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, ACCENT_DEEP);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);

    visuals.widgets.open.bg_fill = SURFACE;
    visuals.widgets.open.weak_bg_fill = SURFACE;
    visuals.widgets.open.bg_stroke = Stroke::new(1.2, ACCENT);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, INK);
    visuals.widgets.open.corner_radius = CornerRadius::same(8);

    style.visuals = visuals;
    ctx.set_style(style);
}

pub fn card_frame() -> Frame {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(16))
        .shadow(Shadow {
            offset: [0, 2],
            blur: 10,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(15, 38, 30, 10),
        })
}

pub fn subtle_card_frame() -> Frame {
    Frame::new()
        .fill(SURFACE_SUBTLE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(14))
}

pub fn modal_frame() -> Frame {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(Margin::same(22))
        .shadow(Shadow {
            offset: [0, 16],
            blur: 42,
            spread: 0,
            color: Color32::from_black_alpha(46),
        })
}

pub fn primary_btn(ui: &mut Ui, label: &str) -> Response {
    let btn = egui::Button::new(RichText::new(label).size(13.0).color(Color32::WHITE).strong())
        .fill(ACCENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(9))
        .min_size(Vec2::new(0.0, 34.0));
    let resp = ui.add(btn);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn primary_btn_sized(ui: &mut Ui, label: &str, min_size: Vec2) -> Response {
    let btn = egui::Button::new(RichText::new(label).size(13.0).color(Color32::WHITE).strong())
        .fill(ACCENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(9))
        .min_size(min_size);
    let resp = ui.add(btn);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn secondary_btn(ui: &mut Ui, label: &str) -> Response {
    let btn = egui::Button::new(RichText::new(label).size(13.0).color(INK).strong())
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER_STRONG))
        .corner_radius(CornerRadius::same(9))
        .min_size(Vec2::new(0.0, 34.0));
    let resp = ui.add(btn);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn danger_btn(ui: &mut Ui, label: &str) -> Response {
    let btn = egui::Button::new(RichText::new(label).size(13.0).color(Color32::WHITE).strong())
        .fill(DANGER)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(9))
        .min_size(Vec2::new(0.0, 34.0));
    let resp = ui.add(btn);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn danger_soft_btn(ui: &mut Ui, label: &str) -> Response {
    let btn = egui::Button::new(RichText::new(label).size(12.5).color(DANGER).strong())
        .fill(DANGER_SOFT)
        .stroke(Stroke::new(1.0, DANGER_BORDER))
        .corner_radius(CornerRadius::same(8))
        .min_size(Vec2::new(0.0, 30.0));
    let resp = ui.add(btn);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn ghost_btn(ui: &mut Ui, label: &str) -> Response {
    let btn = egui::Button::new(RichText::new(label).size(12.5).color(INK_2))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(8))
        .min_size(Vec2::new(0.0, 28.0));
    let resp = ui.add(btn);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn toggle_switch(ui: &mut Ui, on: &mut bool) -> Response {
    let desired = Vec2::new(40.0, 22.0);
    let (rect, mut resp) = ui.allocate_exact_size(desired, Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(resp.id, *on);
        let bg = Color32::from_rgb(
            lerp_u8(0xCB, 0x18, how_on),
            lerp_u8(0xD5, 0x78, how_on),
            lerp_u8(0xD1, 0x6C, how_on),
        );
        let radius = rect.height() * 0.5;
        ui.painter().rect_filled(rect, CornerRadius::same(radius as u8), bg);
        let knob_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = Pos2::new(knob_x, rect.center().y);
        ui.painter()
            .circle_filled(center, radius - 3.0, Color32::WHITE);
    }
    resp
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    ((a as f32) + ((b as f32) - (a as f32)) * t.clamp(0.0, 1.0)).round() as u8
}

pub fn status_pill(ui: &mut Ui, dot: Color32, text: &str, bg: Color32, fg: Color32) {
    Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(9, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(7.0), Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 3.5, dot);
                ui.label(RichText::new(text).size(11.5).color(fg).strong());
            });
        });
}

/// Non-clickable indicator badge (specifically requested for Google Calendar below CUDA badge)
pub fn non_clickable_indicator(ui: &mut Ui, dot: Color32, text: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(7.0), Sense::hover());
        ui.painter().circle_filled(dot_rect.center(), 3.2, dot);
        ui.label(RichText::new(text).size(11.0).color(INK_2));
    });
}

/// Smooth dual-ring orbital loader in Notely Forest Teal
pub fn draw_orbit_loader(ui: &mut Ui, radius: f32, time: f64) {
    let size = Vec2::splat(radius * 2.0 + 12.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let center = rect.center();
    let painter = ui.painter();

    // Subtle static base ring
    painter.circle_stroke(center, radius, Stroke::new(3.0, ACCENT_SOFT));

    // Outer spinning arc
    let start_angle = (time * 3.2) as f32;
    let sweep = 2.15_f32;
    let segments = 24;
    let mut pts = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let a = start_angle + sweep * (i as f32 / segments as f32);
        pts.push(Pos2::new(center.x + radius * a.cos(), center.y + radius * a.sin()));
    }
    painter.add(Shape::line(pts, Stroke::new(3.2, ACCENT)));

    // Inner counter-rotating arc
    let inner_r = radius * 0.62;
    let inner_start = -(time * 4.4) as f32;
    let inner_sweep = 1.6_f32;
    let mut inner_pts = Vec::with_capacity(18);
    for i in 0..=16 {
        let a = inner_start + inner_sweep * (i as f32 / 16.0);
        inner_pts.push(Pos2::new(
            center.x + inner_r * a.cos(),
            center.y + inner_r * a.sin(),
        ));
    }
    painter.add(Shape::line(inner_pts, Stroke::new(2.4, ACCENT_HOVER)));

    // Center pulsing dot
    let pulse = 0.5 + 0.5 * (time * 5.0).sin() as f32;
    painter.circle_filled(center, 3.0 + pulse * 1.5, ACCENT);
}

/// 3-bar animated typing / listening wave indicator
pub fn draw_wave_bars(ui: &mut Ui, time: f64, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(16.0, 14.0), Sense::hover());
    let painter = ui.painter();
    for idx in 0..3 {
        let phase = (time * 6.5 + (idx as f64) * 1.3).sin() as f32;
        let h = 4.0 + (phase * 0.5 + 0.5) * 9.0;
        let x = rect.left() + 2.0 + (idx as f32) * 5.0;
        let bar = Rect::from_center_size(Pos2::new(x, rect.center().y), Vec2::new(2.8, h));
        painter.rect_filled(bar, CornerRadius::same(2), color);
    }
}

/// 18-bar animated spectrum equalizer for Floating Recording Dock
pub fn draw_spectrum_bars(ui: &mut Ui, bands: &[f32], active: bool, time: f64) {
    let count = 18usize;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(112.0, 26.0), Sense::hover());
    let painter = ui.painter();
    let step = rect.width() / count as f32;
    for i in 0..count {
        let level = if active {
            let raw = bands.get(i).copied().unwrap_or(0.0);
            let idle_wave = ((time * 5.0 + i as f64 * 0.45).sin() as f32 * 0.5 + 0.5) * 0.14;
            (raw.max(idle_wave)).clamp(0.10, 1.0)
        } else {
            0.12
        };
        let h = (level * rect.height()).max(3.5);
        let cx = rect.left() + (i as f32 + 0.5) * step;
        let bar_rect = Rect::from_center_size(Pos2::new(cx, rect.center().y), Vec2::new(3.0, h));
        let col = if active { DANGER } else { BORDER_STRONG };
        painter.rect_filled(bar_rect, CornerRadius::same(2), col);
    }
}

#[derive(Clone, Copy)]
pub enum NavIcon {
    Home,
    Notes,
    Recordings,
    Deadlines,
    Settings,
    Mic,
    Upload,
    SidebarToggle,
    Calendar,
    Folder,
    Spark,
}

/// Crisp vector geometric icons (zero Unicode emojis)
pub fn draw_nav_icon(painter: &Painter, rect: Rect, icon: NavIcon, color: Color32) {
    let c = rect.center();
    let r = rect.width().min(rect.height()) * 0.42;
    let stroke = Stroke::new(1.6, color);

    match icon {
        NavIcon::Home => {
            // Simple crisp house silhouette
            let roof = vec![
                Pos2::new(c.x - r, c.y - r * 0.05),
                Pos2::new(c.x, c.y - r * 0.92),
                Pos2::new(c.x + r, c.y - r * 0.05),
            ];
            painter.add(Shape::line(roof, stroke));
            let body = Rect::from_min_max(
                Pos2::new(c.x - r * 0.68, c.y - r * 0.12),
                Pos2::new(c.x + r * 0.68, c.y + r * 0.85),
            );
            painter.rect_stroke(body, CornerRadius::same(2), stroke, StrokeKind::Inside);
        }
        NavIcon::Notes => {
            let doc = Rect::from_center_size(c, Vec2::new(r * 1.45, r * 1.8));
            painter.rect_stroke(doc, CornerRadius::same(2), stroke, StrokeKind::Inside);
            for row in [-0.35_f32, 0.0, 0.35] {
                let y = c.y + r * row;
                let w = if row > 0.2 { r * 0.35 } else { r * 0.46 };
                painter.line_segment([Pos2::new(c.x - r * 0.44, y), Pos2::new(c.x + w, y)], stroke);
            }
        }
        NavIcon::Recordings | NavIcon::Folder => {
            let base = Rect::from_min_max(
                Pos2::new(c.x - r * 0.92, c.y - r * 0.45),
                Pos2::new(c.x + r * 0.92, c.y + r * 0.78),
            );
            painter.rect_stroke(base, CornerRadius::same(3), stroke, StrokeKind::Inside);
            painter.line_segment(
                [
                    Pos2::new(c.x - r * 0.85, c.y - r * 0.45),
                    Pos2::new(c.x - r * 0.2, c.y - r * 0.78),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x - r * 0.2, c.y - r * 0.78),
                    Pos2::new(c.x + r * 0.15, c.y - r * 0.45),
                ],
                stroke,
            );
        }
        NavIcon::Deadlines | NavIcon::Calendar => {
            let cal = Rect::from_center_size(c + Vec2::new(0.0, r * 0.1), Vec2::new(r * 1.75, r * 1.55));
            painter.rect_stroke(cal, CornerRadius::same(3), stroke, StrokeKind::Inside);
            painter.line_segment(
                [
                    Pos2::new(cal.left(), cal.top() + r * 0.48),
                    Pos2::new(cal.right(), cal.top() + r * 0.48),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x - r * 0.42, cal.top() - r * 0.28),
                    Pos2::new(c.x - r * 0.42, cal.top() + r * 0.14),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + r * 0.42, cal.top() - r * 0.28),
                    Pos2::new(c.x + r * 0.42, cal.top() + r * 0.14),
                ],
                stroke,
            );
        }
        NavIcon::Settings => {
            painter.circle_stroke(c, r * 0.72, stroke);
            painter.circle_filled(c, r * 0.25, color);
            for i in 0..6 {
                let a = (i as f32) * std::f32::consts::TAU / 6.0;
                let p1 = Pos2::new(c.x + r * 0.72 * a.cos(), c.y + r * 0.72 * a.sin());
                let p2 = Pos2::new(c.x + r * 1.02 * a.cos(), c.y + r * 1.02 * a.sin());
                painter.line_segment([p1, p2], stroke);
            }
        }
        NavIcon::Mic => {
            let pill = Rect::from_center_size(
                Pos2::new(c.x, c.y - r * 0.22),
                Vec2::new(r * 0.78, r * 1.15),
            );
            painter.rect_stroke(pill, CornerRadius::same(6), stroke, StrokeKind::Inside);
            let arc_y = c.y + r * 0.32;
            painter.line_segment(
                [Pos2::new(c.x - r * 0.65, arc_y), Pos2::new(c.x + r * 0.65, arc_y)],
                stroke,
            );
            painter.line_segment(
                [Pos2::new(c.x, arc_y), Pos2::new(c.x, c.y + r * 0.85)],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x - r * 0.45, c.y + r * 0.85),
                    Pos2::new(c.x + r * 0.45, c.y + r * 0.85),
                ],
                stroke,
            );
        }
        NavIcon::Upload => {
            painter.line_segment(
                [Pos2::new(c.x, c.y + r * 0.45), Pos2::new(c.x, c.y - r * 0.72)],
                stroke,
            );
            painter.add(Shape::line(
                vec![
                    Pos2::new(c.x - r * 0.48, c.y - r * 0.24),
                    Pos2::new(c.x, c.y - r * 0.75),
                    Pos2::new(c.x + r * 0.48, c.y - r * 0.24),
                ],
                stroke,
            ));
            painter.line_segment(
                [
                    Pos2::new(c.x - r * 0.78, c.y + r * 0.75),
                    Pos2::new(c.x + r * 0.78, c.y + r * 0.75),
                ],
                stroke,
            );
        }
        NavIcon::SidebarToggle => {
            let box_rect = Rect::from_center_size(c, Vec2::new(r * 1.85, r * 1.55));
            painter.rect_stroke(box_rect, CornerRadius::same(3), stroke, StrokeKind::Inside);
            painter.line_segment(
                [
                    Pos2::new(box_rect.left() + r * 0.62, box_rect.top()),
                    Pos2::new(box_rect.left() + r * 0.62, box_rect.bottom()),
                ],
                stroke,
            );
        }
        NavIcon::Spark => {
            // Clean 4-point star polygon
            let pts = vec![
                Pos2::new(c.x, c.y - r * 0.95),
                Pos2::new(c.x + r * 0.26, c.y - r * 0.26),
                Pos2::new(c.x + r * 0.95, c.y),
                Pos2::new(c.x + r * 0.26, c.y + r * 0.26),
                Pos2::new(c.x, c.y + r * 0.95),
                Pos2::new(c.x - r * 0.26, c.y + r * 0.26),
                Pos2::new(c.x - r * 0.95, c.y),
                Pos2::new(c.x - r * 0.26, c.y - r * 0.26),
                Pos2::new(c.x, c.y - r * 0.95),
            ];
            painter.add(Shape::line(pts, stroke));
        }
    }
}

/// Clean markdown renderer for AI summaries (headings, bullets, numbered items, bold inline)
pub fn render_markdown_clean(ui: &mut Ui, content: &str) {
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            ui.add_space(6.0);
            continue;
        }
        if let Some(rest) = line.strip_prefix("### ") {
            ui.add_space(6.0);
            ui.label(RichText::new(strip_md_stars(rest)).size(14.5).color(ACCENT_DEEP).strong());
            ui.add_space(2.0);
        } else if let Some(rest) = line.strip_prefix("## ") {
            ui.add_space(10.0);
            ui.label(RichText::new(strip_md_stars(rest)).size(16.5).color(INK).strong());
            ui.add_space(3.0);
        } else if let Some(rest) = line.strip_prefix("# ") {
            ui.add_space(10.0);
            ui.label(RichText::new(strip_md_stars(rest)).size(18.5).color(INK).strong());
            ui.add_space(4.0);
        } else if let Some(rest) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            ui.horizontal_wrapped(|ui| {
                ui.add_space(6.0);
                ui.label(RichText::new("•").size(13.5).color(ACCENT).strong());
                render_inline_bold(ui, rest);
            });
        } else {
            ui.horizontal_wrapped(|ui| {
                render_inline_bold(ui, line);
            });
        }
    }
}

fn strip_md_stars(s: &str) -> String {
    s.replace("**", "").replace("__", "")
}

fn render_inline_bold(ui: &mut Ui, text: &str) {
    let mut remaining = text;
    let mut is_bold = false;
    while let Some(idx) = remaining.find("**") {
        let chunk = &remaining[..idx];
        if !chunk.is_empty() {
            let mut rt = RichText::new(chunk).size(13.5).color(INK);
            if is_bold {
                rt = rt.strong();
            }
            ui.label(rt);
        }
        remaining = &remaining[idx + 2..];
        is_bold = !is_bold;
    }
    if !remaining.is_empty() {
        let mut rt = RichText::new(remaining).size(13.5).color(INK);
        if is_bold {
            rt = rt.strong();
        }
        ui.label(rt);
    }
}

/// Crisp top-right foreground toast (NEVER blurred, always highest layer)
pub fn draw_toast(ctx: &egui::Context, msg: &str, is_error: bool) {
    egui::Area::new(egui::Id::new("notely_toast_area"))
        .order(egui::Order::Tooltip)
        .anchor(egui::Align2::RIGHT_TOP, Vec2::new(-20.0, 20.0))
        .interactable(false)
        .show(ctx, |ui| {
            let (bg, border, dot, fg) = if is_error {
                (DANGER_SOFT, DANGER_BORDER, DANGER, DANGER)
            } else {
                (SURFACE, ACCENT_BORDER, ACCENT, INK)
            };
            Frame::new()
                .fill(bg)
                .stroke(Stroke::new(1.2, border))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(Margin::symmetric(14, 10))
                .shadow(Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 0,
                    color: Color32::from_black_alpha(38),
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                        ui.painter().circle_filled(r.center(), 4.0, dot);
                        ui.label(RichText::new(msg).size(13.0).color(fg).strong());
                    });
                });
        });
}
