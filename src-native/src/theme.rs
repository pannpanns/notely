use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame, Margin,
    Pos2, Rect, Response, RichText, Sense, Shadow, Shape, Stroke, TextStyle, Ui, Vec2,
};
use std::sync::Arc;

// ── Notely Brand Color Palette (Preserved & Refined) ──
pub const ACCENT: Color32 = Color32::from_rgb(0x18, 0x78, 0x6C); // Forest Teal
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x13, 0x63, 0x58);
pub const ACCENT_DEEP: Color32 = Color32::from_rgb(0x0F, 0x4F, 0x46);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0xEA, 0xF5, 0xF0);
pub const ACCENT_BORDER: Color32 = Color32::from_rgb(0xC5, 0xE0, 0xD4);

pub const CANVAS: Color32 = Color32::from_rgb(0xF3, 0xF6, 0xF4); // Soft Sage outer shell
pub const SIDEBAR_BG: Color32 = Color32::from_rgb(0xF3, 0xF6, 0xF4);
pub const SURFACE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const SURFACE_SUBTLE: Color32 = Color32::from_rgb(0xF8, 0xFA, 0xF9);
pub const SURFACE_HOVER: Color32 = Color32::from_rgb(0xEE, 0xF4, 0xF1);

pub const BORDER: Color32 = Color32::from_rgb(0xE2, 0xEA, 0xE5);
pub const BORDER_INPUT: Color32 = Color32::from_rgb(0xD5, 0xE0, 0xDA);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(0xCD, 0xDA, 0xD3);

pub const INK: Color32 = Color32::from_rgb(0x17, 0x2A, 0x23); // Deep Pine Ink
pub const INK_2: Color32 = Color32::from_rgb(0x4B, 0x61, 0x57);
pub const INK_MUTED: Color32 = Color32::from_rgb(0x7D, 0x91, 0x87);

pub const DANGER: Color32 = Color32::from_rgb(0xD9, 0x38, 0x45);
pub const DANGER_HOVER: Color32 = Color32::from_rgb(0xBD, 0x28, 0x35);
pub const DANGER_SOFT: Color32 = Color32::from_rgb(0xFE, 0xF2, 0xF2);
pub const DANGER_BORDER: Color32 = Color32::from_rgb(0xFE, 0xCA, 0xCA);

pub const GREEN: Color32 = Color32::from_rgb(0x16, 0xA3, 0x4A);
pub const GREEN_SOFT: Color32 = Color32::from_rgb(0xEC, 0xFD, 0xF5);
pub const AMBER: Color32 = Color32::from_rgb(0xD9, 0x77, 0x06);
pub const AMBER_SOFT: Color32 = Color32::from_rgb(0xFF, 0xF7, 0xEB);
#[allow(dead_code)]
pub const BLUE_SOFT: Color32 = Color32::from_rgb(0xEF, 0xF6, 0xFF);
#[allow(dead_code)]
pub const BLUE_INK: Color32 = Color32::from_rgb(0x1D, 0x4E, 0xD8);

pub fn bold_family() -> FontFamily {
    FontFamily::Name("bold".into())
}

pub fn semi_family() -> FontFamily {
    FontFamily::Name("semibold".into())
}

pub fn bold(text: impl Into<String>, size: f32, color: Color32) -> RichText {
    RichText::new(text)
        .font(FontId::new(size, bold_family()))
        .color(color)
}

pub fn semi(text: impl Into<String>, size: f32, color: Color32) -> RichText {
    RichText::new(text)
        .font(FontId::new(size, semi_family()))
        .color(color)
}

pub fn regular(text: impl Into<String>, size: f32, color: Color32) -> RichText {
    RichText::new(text)
        .font(FontId::new(size, FontFamily::Proportional))
        .color(color)
}

pub fn mono(text: impl Into<String>, size: f32, color: Color32) -> RichText {
    RichText::new(text)
        .font(FontId::new(size, FontFamily::Monospace))
        .color(color)
}

pub fn apply_notely_theme(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    let regular_loaded = load_first_font(
        &mut fonts,
        "segoe_ui_regular",
        &[r"C:\Windows\Fonts\segoeui.ttf", r"C:\Windows\Fonts\arial.ttf"],
    );
    if regular_loaded {
        if let Some(fam) = fonts.families.get_mut(&FontFamily::Proportional) {
            fam.insert(0, "segoe_ui_regular".to_owned());
        }
    }

    let semi_loaded = load_first_font(
        &mut fonts,
        "segoe_ui_semibold",
        &[r"C:\Windows\Fonts\seguisb.ttf", r"C:\Windows\Fonts\segoeuib.ttf"],
    );
    let mut semi_list = Vec::new();
    if semi_loaded {
        semi_list.push("segoe_ui_semibold".to_owned());
    }
    if regular_loaded {
        semi_list.push("segoe_ui_regular".to_owned());
    }
    if let Some(prop) = fonts.families.get(&FontFamily::Proportional) {
        semi_list.extend(prop.iter().cloned());
    }
    fonts.families.insert(semi_family(), semi_list);

    let bold_loaded = load_first_font(
        &mut fonts,
        "segoe_ui_bold",
        &[r"C:\Windows\Fonts\segoeuib.ttf", r"C:\Windows\Fonts\seguisb.ttf"],
    );
    let mut bold_list = Vec::new();
    if bold_loaded {
        bold_list.push("segoe_ui_bold".to_owned());
    }
    if semi_loaded {
        bold_list.push("segoe_ui_semibold".to_owned());
    }
    if let Some(prop) = fonts.families.get(&FontFamily::Proportional) {
        bold_list.extend(prop.iter().cloned());
    }
    fonts.families.insert(bold_family(), bold_list);

    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\consola.ttf") {
        fonts
            .font_data
            .insert("consolas".to_owned(), Arc::new(FontData::from_owned(bytes)));
        if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
            family.insert(0, "consolas".to_owned());
        }
    }
    ctx.set_fonts(fonts);

    // Crisp tessellation & subpixel text snapping on High-DPI monitors
    ctx.tessellation_options_mut(|o| {
        o.feathering = true;
        o.feathering_size_in_pixels = 0.65_f32;
        o.round_text_to_pixels = true;
    });

    let mut style = (*ctx.style()).clone();
    style.animation_time = 0.16_f32;

    style.text_styles = [
        (TextStyle::Heading, FontId::new(22.0_f32, bold_family())),
        (TextStyle::Body, FontId::new(13.5_f32, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(13.0_f32, semi_family())),
        (TextStyle::Small, FontId::new(11.5_f32, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(12.0_f32, FontFamily::Monospace)),
    ]
    .into();

    style.spacing.item_spacing = Vec2::new(8.0_f32, 8.0_f32);
    style.spacing.button_padding = Vec2::new(13.0_f32, 7.0_f32);
    style.spacing.window_margin = Margin::same(16);
    style.spacing.scroll = egui::style::ScrollStyle::thin();

    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = CANVAS;
    visuals.window_fill = SURFACE;
    visuals.extreme_bg_color = SURFACE_SUBTLE;
    visuals.faint_bg_color = SURFACE_SUBTLE;
    visuals.override_text_color = Some(INK);
    visuals.window_corner_radius = CornerRadius::same(16);
    visuals.window_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.window_shadow = Shadow {
        offset: [0, 10],
        blur: 28,
        spread: 0,
        color: Color32::from_rgba_unmultiplied(16, 42, 32, 22),
    };
    visuals.popup_shadow = Shadow {
        offset: [0, 6],
        blur: 18,
        spread: 0,
        color: Color32::from_rgba_unmultiplied(16, 42, 32, 18),
    };

    visuals.selection.bg_fill = ACCENT_SOFT;
    visuals.selection.stroke = Stroke::new(1.2_f32, ACCENT);

    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.weak_bg_fill = SURFACE_SUBTLE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, INK);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(9);

    visuals.widgets.inactive.bg_fill = SURFACE;
    visuals.widgets.inactive.weak_bg_fill = SURFACE;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER_INPUT);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, INK);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(9);

    visuals.widgets.hovered.bg_fill = SURFACE_HOVER;
    visuals.widgets.hovered.weak_bg_fill = SURFACE_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, INK);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(9);

    visuals.widgets.active.bg_fill = ACCENT_SOFT;
    visuals.widgets.active.weak_bg_fill = ACCENT_SOFT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.2_f32, ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, ACCENT_DEEP);
    visuals.widgets.active.corner_radius = CornerRadius::same(9);

    visuals.widgets.open.bg_fill = SURFACE;
    visuals.widgets.open.weak_bg_fill = SURFACE;
    visuals.widgets.open.bg_stroke = Stroke::new(1.2_f32, ACCENT);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, INK);
    visuals.widgets.open.corner_radius = CornerRadius::same(9);

    style.visuals = visuals;
    ctx.set_style(style);
}

fn load_first_font(fonts: &mut FontDefinitions, key: &str, candidates: &[&str]) -> bool {
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert(key.to_owned(), Arc::new(FontData::from_owned(bytes)));
            return true;
        }
    }
    false
}

pub fn card_frame() -> Frame {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(18))
        .shadow(Shadow {
            offset: [0, 2],
            blur: 10,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(18, 44, 34, 10),
        })
}

#[allow(dead_code)]
pub fn subtle_card_frame() -> Frame {
    Frame::new()
        .fill(SURFACE_SUBTLE)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(14))
}

pub fn modal_frame() -> Frame {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(Margin::same(22))
        .shadow(Shadow {
            offset: [0, 20],
            blur: 48,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(14, 36, 27, 42),
        })
}

pub fn primary_btn(ui: &mut Ui, label: &str) -> Response {
    let btn = egui::Button::new(semi(label, 13.0_f32, Color32::WHITE))
        .fill(ACCENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(9))
        .min_size(Vec2::new(0.0_f32, 35.0_f32));
    let resp = ui.add(btn);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn primary_btn_sized(ui: &mut Ui, label: &str, min_size: Vec2) -> Response {
    let btn = egui::Button::new(semi(label, 13.0_f32, Color32::WHITE))
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
    let btn = egui::Button::new(semi(label, 13.0_f32, INK))
        .fill(SURFACE)
        .stroke(Stroke::new(1.0_f32, BORDER_STRONG))
        .corner_radius(CornerRadius::same(9))
        .min_size(Vec2::new(0.0_f32, 35.0_f32));
    let resp = ui.add(btn);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn danger_btn(ui: &mut Ui, label: &str) -> Response {
    let btn = egui::Button::new(semi(label, 13.0_f32, Color32::WHITE))
        .fill(DANGER)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(9))
        .min_size(Vec2::new(0.0_f32, 35.0_f32));
    let resp = ui.add(btn);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn danger_soft_btn(ui: &mut Ui, label: &str) -> Response {
    let btn = egui::Button::new(semi(label, 12.5_f32, DANGER))
        .fill(DANGER_SOFT)
        .stroke(Stroke::new(1.0_f32, DANGER_BORDER))
        .corner_radius(CornerRadius::same(8))
        .min_size(Vec2::new(0.0_f32, 31.0_f32));
    let resp = ui.add(btn);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn ghost_btn(ui: &mut Ui, label: &str) -> Response {
    let btn = egui::Button::new(semi(label, 12.5_f32, INK_2))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(8))
        .min_size(Vec2::new(0.0_f32, 28.0_f32));
    let resp = ui.add(btn);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn toggle_switch(ui: &mut Ui, on: &mut bool) -> Response {
    let desired = Vec2::new(40.0_f32, 22.0_f32);
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
        let radius = rect.height() * 0.5_f32;
        ui.painter()
            .rect_filled(rect, CornerRadius::same(radius as u8), bg);
        let knob_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = Pos2::new(knob_x, rect.center().y);
        ui.painter()
            .circle_filled(center, radius - 3.0_f32, Color32::WHITE);
    }
    resp
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    ((a as f32) + ((b as f32) - (a as f32)) * t.clamp(0.0_f32, 1.0_f32)).round() as u8
}

pub fn status_pill(ui: &mut Ui, dot: Color32, text: &str, bg: Color32, fg: Color32) {
    Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(9, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0_f32;
                let (dot_rect, _) =
                    ui.allocate_exact_size(Vec2::splat(7.0_f32), Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 3.4_f32, dot);
                ui.label(semi(text, 11.5_f32, fg));
            });
        });
}

/// Non-clickable indicator badge (Google Calendar dot + label underneath CUDA pill)
pub fn non_clickable_indicator(ui: &mut Ui, dot: Color32, text: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0_f32;
        let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(7.0_f32), Sense::hover());
        ui.painter().circle_filled(dot_rect.center(), 3.2_f32, dot);
        ui.label(regular(text, 11.0_f32, INK_2));
    });
}

/// Smooth dual-ring orbital loader in Notely Forest Teal
pub fn draw_orbit_loader(ui: &mut Ui, radius: f32, time: f64) {
    let size = Vec2::splat(radius * 2.0_f32 + 12.0_f32);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let center = rect.center();
    let painter = ui.painter();

    painter.circle_stroke(center, radius, Stroke::new(3.0_f32, ACCENT_SOFT));

    let start_angle = (time * 3.2) as f32;
    let sweep = 2.15_f32;
    let segments = 24;
    let mut pts = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let a = start_angle + sweep * (i as f32 / segments as f32);
        pts.push(Pos2::new(
            center.x + radius * a.cos(),
            center.y + radius * a.sin(),
        ));
    }
    painter.add(Shape::line(pts, Stroke::new(3.2_f32, ACCENT)));

    let inner_r = radius * 0.62_f32;
    let inner_start = -(time * 4.4) as f32;
    let inner_sweep = 1.6_f32;
    let mut inner_pts = Vec::with_capacity(18);
    for i in 0..=16 {
        let a = inner_start + inner_sweep * (i as f32 / 16.0_f32);
        inner_pts.push(Pos2::new(
            center.x + inner_r * a.cos(),
            center.y + inner_r * a.sin(),
        ));
    }
    painter.add(Shape::line(inner_pts, Stroke::new(2.4_f32, ACCENT_HOVER)));

    let pulse = 0.5_f32 + 0.5_f32 * (time * 5.0).sin() as f32;
    painter.circle_filled(center, 3.0_f32 + pulse * 1.5_f32, ACCENT);
}

/// 3-bar animated typing / listening wave indicator
pub fn draw_wave_bars(ui: &mut Ui, time: f64, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(16.0_f32, 14.0_f32), Sense::hover());
    let painter = ui.painter();
    for idx in 0..3 {
        let phase = (time * 6.5 + (idx as f64) * 1.3).sin() as f32;
        let h = 4.0_f32 + (phase * 0.5_f32 + 0.5_f32) * 9.0_f32;
        let x = rect.left() + 2.0_f32 + (idx as f32) * 5.0_f32;
        let bar = Rect::from_center_size(
            Pos2::new(x, rect.center().y),
            Vec2::new(2.8_f32, h),
        );
        painter.rect_filled(bar, CornerRadius::same(2), color);
    }
}

/// 18-bar animated spectrum equalizer for Floating Recording Dock
pub fn draw_spectrum_bars(ui: &mut Ui, bands: &[f32], active: bool, time: f64) {
    let count = 18usize;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(112.0_f32, 26.0_f32), Sense::hover());
    let painter = ui.painter();
    let step = rect.width() / count as f32;
    for i in 0..count {
        let level = if active {
            let raw = bands.get(i).copied().unwrap_or(0.0_f32);
            let idle_wave =
                ((time * 5.0 + i as f64 * 0.45).sin() as f32 * 0.5_f32 + 0.5_f32) * 0.14_f32;
            raw.max(idle_wave).clamp(0.10_f32, 1.0_f32)
        } else {
            0.12_f32
        };
        let h = (level * rect.height()).max(3.5_f32);
        let cx = rect.left() + (i as f32 + 0.5_f32) * step;
        let bar_rect =
            Rect::from_center_size(Pos2::new(cx, rect.center().y), Vec2::new(3.0_f32, h));
        let col = if active { DANGER } else { BORDER_STRONG };
        painter.rect_filled(bar_rect, CornerRadius::same(2), col);
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum NavIcon {
    Home,
    Notes,
    Recordings,
    Deadlines,
    Settings,
    Mic,
    Square,
    Upload,
    SidebarToggle,
    Calendar,
    Folder,
    Spark,
    Search,
    Plus,
    Trash,
    Edit,
    Copy,
    Check,
    ArrowUpRight,
    ChevronRight,
    Refresh,
    X,
}

const SVG_HOME: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg>"##;
const SVG_NOTES: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="M10 9H8"/><path d="M16 13H8"/><path d="M16 17H8"/></svg>"##;
const SVG_FOLDER: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>"##;
const SVG_CALENDAR: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 2v4"/><path d="M16 2v4"/><rect width="18" height="18" x="3" y="4" rx="2"/><path d="M3 10h18"/></svg>"##;
const SVG_SETTINGS: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>"##;
const SVG_MIC: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><line x1="12" x2="12" y1="19" y2="22"/></svg>"##;
const SVG_SQUARE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="#ffffff" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="5" y="5" rx="2"/></svg>"##;
const SVG_UPLOAD: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" x2="12" y1="3" y2="15"/></svg>"##;
const SVG_PANEL_LEFT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/></svg>"##;
const SVG_SPARKLES: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9.937 15.5A2 2 0 0 0 8.5 14.063l-6.135-1.582a.5.5 0 0 1 0-.962L8.5 9.936A2 2 0 0 0 9.937 8.5l1.582-6.135a.5.5 0 0 1 .963 0L14.063 8.5A2 2 0 0 0 15.5 9.937l6.135 1.581a.5.5 0 0 1 0 .964L15.5 14.063a2 2 0 0 0-1.437 1.437l-1.582 6.135a.5.5 0 0 1-.963 0z"/><path d="M20 3v4"/><path d="M22 5h-4"/></svg>"##;
const SVG_SEARCH: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>"##;
const SVG_PLUS: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5v14"/></svg>"##;
const SVG_TRASH: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>"##;
const SVG_EDIT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/></svg>"##;
const SVG_COPY: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>"##;
const SVG_CHECK: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>"##;
const SVG_ARROW_UP_RIGHT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M7 7h10v10"/><path d="M7 17 17 7"/></svg>"##;
const SVG_CHEVRON_RIGHT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 18 6-6-6-6"/></svg>"##;
const SVG_REFRESH: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/></svg>"##;
const SVG_X: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>"##;

fn icon_uri_and_bytes(icon: NavIcon) -> (&'static str, &'static [u8]) {
    match icon {
        NavIcon::Home => ("bytes://lucide/home.svg", SVG_HOME),
        NavIcon::Notes => ("bytes://lucide/notes.svg", SVG_NOTES),
        NavIcon::Recordings | NavIcon::Folder => ("bytes://lucide/folder.svg", SVG_FOLDER),
        NavIcon::Deadlines | NavIcon::Calendar => ("bytes://lucide/calendar.svg", SVG_CALENDAR),
        NavIcon::Settings => ("bytes://lucide/settings.svg", SVG_SETTINGS),
        NavIcon::Mic => ("bytes://lucide/mic.svg", SVG_MIC),
        NavIcon::Square => ("bytes://lucide/square.svg", SVG_SQUARE),
        NavIcon::Upload => ("bytes://lucide/upload.svg", SVG_UPLOAD),
        NavIcon::SidebarToggle => ("bytes://lucide/panel-left.svg", SVG_PANEL_LEFT),
        NavIcon::Spark => ("bytes://lucide/sparkles.svg", SVG_SPARKLES),
        NavIcon::Search => ("bytes://lucide/search.svg", SVG_SEARCH),
        NavIcon::Plus => ("bytes://lucide/plus.svg", SVG_PLUS),
        NavIcon::Trash => ("bytes://lucide/trash.svg", SVG_TRASH),
        NavIcon::Edit => ("bytes://lucide/edit.svg", SVG_EDIT),
        NavIcon::Copy => ("bytes://lucide/copy.svg", SVG_COPY),
        NavIcon::Check => ("bytes://lucide/check.svg", SVG_CHECK),
        NavIcon::ArrowUpRight => ("bytes://lucide/arrow-up-right.svg", SVG_ARROW_UP_RIGHT),
        NavIcon::ChevronRight => ("bytes://lucide/chevron-right.svg", SVG_CHEVRON_RIGHT),
        NavIcon::Refresh => ("bytes://lucide/refresh.svg", SVG_REFRESH),
        NavIcon::X => ("bytes://lucide/x.svg", SVG_X),
    }
}

/// Anti-aliased Lucide SVG icon painted cleanly inside `rect`
pub fn paint_icon(ui: &egui::Ui, rect: Rect, icon: NavIcon, color: Color32) {
    let (uri, bytes) = icon_uri_and_bytes(icon);
    egui::Image::from_bytes(uri, bytes)
        .fit_to_exact_size(rect.size())
        .tint(color)
        .paint_at(ui, rect);
}

#[allow(dead_code)]
pub fn icon_widget(ui: &mut Ui, icon: NavIcon, size: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    paint_icon(ui, rect, icon, color);
}

/// Clean markdown renderer for AI summaries
pub fn render_markdown_clean(ui: &mut Ui, content: &str) {
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            ui.add_space(6.0_f32);
            continue;
        }
        if let Some(rest) = line.strip_prefix("### ") {
            ui.add_space(6.0_f32);
            ui.label(semi(strip_md_stars(rest), 14.5_f32, ACCENT_DEEP));
            ui.add_space(2.0_f32);
        } else if let Some(rest) = line.strip_prefix("## ") {
            ui.add_space(10.0_f32);
            ui.label(bold(strip_md_stars(rest), 16.5_f32, INK));
            ui.add_space(3.0_f32);
        } else if let Some(rest) = line.strip_prefix("# ") {
            ui.add_space(10.0_f32);
            ui.label(bold(strip_md_stars(rest), 18.5_f32, INK));
            ui.add_space(4.0_f32);
        } else if let Some(rest) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            ui.horizontal_wrapped(|ui| {
                ui.add_space(6.0_f32);
                ui.label(semi("•", 13.5_f32, ACCENT));
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
            if is_bold {
                ui.label(semi(chunk, 13.5_f32, INK));
            } else {
                ui.label(regular(chunk, 13.5_f32, INK));
            }
        }
        remaining = &remaining[idx + 2..];
        is_bold = !is_bold;
    }
    if !remaining.is_empty() {
        if is_bold {
            ui.label(semi(remaining, 13.5_f32, INK));
        } else {
            ui.label(regular(remaining, 13.5_f32, INK));
        }
    }
}

/// Crisp top-right foreground toast (NEVER blurred, always highest layer)
pub fn draw_toast(ctx: &egui::Context, msg: &str, is_error: bool) {
    egui::Area::new(egui::Id::new("notely_toast_area"))
        .order(egui::Order::Tooltip)
        .anchor(egui::Align2::RIGHT_TOP, Vec2::new(-20.0_f32, 20.0_f32))
        .interactable(false)
        .show(ctx, |ui| {
            let (bg, border, dot, fg) = if is_error {
                (DANGER_SOFT, DANGER_BORDER, DANGER, DANGER)
            } else {
                (SURFACE, ACCENT_BORDER, ACCENT, INK)
            };
            Frame::new()
                .fill(bg)
                .stroke(Stroke::new(1.2_f32, border))
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
                        let (r, _) =
                            ui.allocate_exact_size(Vec2::splat(8.0_f32), Sense::hover());
                        ui.painter().circle_filled(r.center(), 4.0_f32, dot);
                        ui.label(semi(msg, 13.0_f32, fg));
                    });
                });
        });
}
