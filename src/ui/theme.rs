//! Mawthooq visual language, taken from the 5A1F software zone.
//!
//! Dark ink surfaces, neon-olive structure, coral signals, one-pixel rules,
//! sharp corners and a faint technical grid. No external assets are requested;
//! an installed system font is used only when it is already on the device.

use eframe::egui;

/// Deep page surface (`--brand-void`).
pub const VOID: egui::Color32 = egui::Color32::from_rgb(0x0c, 0x10, 0x0e);
/// Raised panels (`--brand-void-raised`).
pub const VOID_RAISED: egui::Color32 = egui::Color32::from_rgb(0x12, 0x19, 0x15);
/// Card surface (`--brand-void-card`).
pub const VOID_CARD: egui::Color32 = egui::Color32::from_rgb(0x16, 0x20, 0x1a);
/// Card border (`--brand-void-line`).
pub const VOID_LINE: egui::Color32 = egui::Color32::from_rgb(0x23, 0x30, 0x2a);
/// Primary type (`--color-fg-inverse`).
pub const VOID_INK: egui::Color32 = egui::Color32::from_rgb(0xf2, 0xf5, 0xee);
/// Secondary type (`--color-fg-inverse-muted`).
pub const VOID_MUTED: egui::Color32 = egui::Color32::from_rgb(0x9f, 0xb3, 0xa4);
/// Neon olive (`--brand-neon-primary`).
pub const NEON: egui::Color32 = egui::Color32::from_rgb(0x6f, 0xe3, 0xa1);
/// Neon coral (`--brand-neon-secondary`).
pub const NEON_CORAL: egui::Color32 = egui::Color32::from_rgb(0xff, 0x7a, 0x5c);

const OVERLINE_STYLE: &str = "mawthooq-overline";

fn overline_style() -> egui::TextStyle {
    egui::TextStyle::Name(OVERLINE_STYLE.into())
}

/// Install the software-zone palette, type scale and local font fallback.
pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style
        .text_styles
        .insert(egui::TextStyle::Small, egui::FontId::proportional(10.0));
    style
        .text_styles
        .insert(overline_style(), egui::FontId::proportional(10.0));
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(11.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(42.0));
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(13.0));

    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);

    let visuals = &mut style.visuals;
    visuals.panel_fill = VOID;
    visuals.window_fill = VOID;
    visuals.extreme_bg_color = VOID_CARD;
    visuals.faint_bg_color = VOID_LINE;
    visuals.override_text_color = Some(VOID_INK);
    visuals.hyperlink_color = NEON;
    visuals.error_fg_color = NEON_CORAL;
    visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(0x6f, 0xe3, 0xa1, 70);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, NEON);

    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_fill = VOID_CARD;
    widgets.noninteractive.weak_bg_fill = VOID_CARD;
    widgets.noninteractive.fg_stroke.color = VOID_MUTED;
    widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, hologram_rule());
    widgets.noninteractive.corner_radius = egui::CornerRadius::same(0);

    widgets.inactive.bg_fill = VOID_CARD;
    widgets.inactive.weak_bg_fill = VOID_CARD;
    widgets.inactive.fg_stroke.color = VOID_INK;
    widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, VOID_LINE);
    widgets.inactive.corner_radius = egui::CornerRadius::same(0);

    widgets.hovered.bg_fill = VOID_RAISED;
    widgets.hovered.weak_bg_fill = VOID_RAISED;
    widgets.hovered.fg_stroke.color = NEON;
    widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, NEON);
    widgets.hovered.corner_radius = egui::CornerRadius::same(0);

    widgets.active.bg_fill = NEON;
    widgets.active.weak_bg_fill = VOID_RAISED;
    widgets.active.fg_stroke.color = VOID;
    widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, NEON);
    widgets.active.corner_radius = egui::CornerRadius::same(0);

    ctx.set_style(style);
    install_system_font(ctx);
}

/// rgba(242, 245, 238, 0.12) — the site's inverse hairline.
fn hologram_rule() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(0xf2, 0xf5, 0xee, 31)
}

/// Faint 72px technical grid used behind the software zone.
pub fn grid_background(ui: &mut egui::Ui) {
    let painter = ui.painter();
    let rect = ui.max_rect();
    let stroke = egui::Stroke::new(
        1.0_f32,
        egui::Color32::from_rgba_unmultiplied(0xf2, 0xf5, 0xee, 12),
    );
    let step = 72.0_f32;

    let mut x = rect.left();
    while x <= rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            stroke,
        );
        x += step;
    }
    let mut y = rect.top();
    while y <= rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            stroke,
        );
        y += step;
    }
}

/// Small tracked label, neon-olive like `.overline` in the site.
pub fn overline(ui: &mut egui::Ui, text: impl Into<String>) {
    let text: String = text.into();
    ui.label(
        egui::RichText::new(text)
            .text_style(overline_style())
            .color(NEON),
    );
}

/// Editorial page header with overline, oversized title and quiet lead.
pub fn page_header(ui: &mut egui::Ui, label: &str, heading: &str, lead: &str) {
    overline(ui, label);
    ui.add_space(10.0);
    ui.label(egui::RichText::new(heading).color(VOID_INK).size(42.0));
    ui.add_space(10.0);
    ui.label(egui::RichText::new(lead).color(VOID_MUTED).size(15.0));
    ui.add_space(18.0);

    let min = ui.min_rect();
    let y = ui.cursor().min.y;
    ui.painter().line_segment(
        [egui::pos2(min.left(), y), egui::pos2(min.right(), y)],
        egui::Stroke::new(1.0_f32, VOID_LINE),
    );
    ui.add_space(8.0);
}

/// Bordered software-zone card with a numbered art rail.
pub fn card(ui: &mut egui::Ui, index: &str, title: &str, body_fn: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(VOID_CARD)
        .stroke(egui::Stroke::new(1.0_f32, VOID_LINE))
        .corner_radius(egui::CornerRadius::same(0))
        .inner_margin(egui::Margin::symmetric(22, 20))
        .show(ui, |ui| {
            let full_width = ui.available_width();
            let art_width = (full_width * 0.26).clamp(96.0, 168.0);
            let right_width = (full_width - art_width - 22.0).max(220.0);

            ui.horizontal(|ui| {
                let (art_rect, _) =
                    ui.allocate_exact_size(egui::vec2(art_width, 132.0), egui::Sense::hover());
                draw_card_art(ui.painter(), art_rect, index);

                ui.add_space(22.0);
                ui.set_width(right_width);
                ui.vertical(|ui| {
                    overline(ui, title);
                    ui.add_space(12.0);
                    body_fn(ui);
                });
            });
        });
}

fn draw_card_art(painter: &egui::Painter, rect: egui::Rect, index: &str) {
    painter.rect_filled(rect, egui::CornerRadius::same(0), VOID_RAISED);
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(0),
        egui::Stroke::new(1.0_f32, VOID_LINE),
        egui::StrokeKind::Inside,
    );

    let grid_stroke = egui::Stroke::new(
        1.0_f32,
        egui::Color32::from_rgba_unmultiplied(0xf2, 0xf5, 0xee, 20),
    );
    let mut x = rect.left() + 18.0;
    while x < rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            grid_stroke,
        );
        x += 24.0;
    }
    let mut y = rect.top() + 18.0;
    while y < rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            grid_stroke,
        );
        y += 24.0;
    }

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        index,
        egui::FontId::monospace(36.0),
        NEON_CORAL,
    );
    painter.text(
        rect.left_top() + egui::vec2(10.0, 8.0),
        egui::Align2::LEFT_TOP,
        "5A1F",
        egui::FontId::proportional(9.0),
        VOID_MUTED,
    );
}

/// Brand lockup for the top navigation.
pub fn brand_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::hover());
        draw_mark(ui.painter(), rect);

        ui.add_space(8.0);
        ui.vertical(|ui| {
            overline(ui, "5A1F STUDIO");
            ui.label(
                egui::RichText::new("Mawthooq  ·  موثوق")
                    .color(VOID_INK)
                    .size(18.0),
            );
        });
    });
}

fn draw_mark(painter: &egui::Painter, rect: egui::Rect) {
    let r = rect.shrink(3.0);
    let stroke = egui::Stroke::new(1.5_f32, NEON);
    painter.line_segment([r.left_top(), r.right_top()], stroke);
    painter.line_segment([r.left_top(), r.left_bottom()], stroke);
    painter.line_segment([r.right_top(), r.right_bottom()], stroke);
    painter.line_segment([r.left_bottom(), r.right_bottom()], stroke);

    let c = r.center();
    painter.circle_filled(egui::pos2(c.x, c.y - 3.0), 3.5, NEON_CORAL);
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(c.x - 1.5, c.y - 1.0),
            egui::pos2(c.x + 1.5, c.y + 7.0),
        ),
        egui::CornerRadius::same(0),
        NEON_CORAL,
    );
}

/// Solid neon call-to-action with sharp corners.
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> bool {
    button(ui, text, true)
}

/// Outlined neon secondary action.
pub fn secondary_button(ui: &mut egui::Ui, text: &str) -> bool {
    button(ui, text, false)
}

fn button(ui: &mut egui::Ui, text: &str, primary: bool) -> bool {
    let width = (text.chars().count() as f32 * 7.0 + 38.0).clamp(160.0, 300.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 36.0), egui::Sense::click());

    let fill = if primary {
        if response.hovered() {
            NEON_CORAL
        } else {
            NEON
        }
    } else if response.hovered() {
        VOID_RAISED
    } else {
        egui::Color32::TRANSPARENT
    };
    let stroke = egui::Stroke::new(1.0_f32, if response.hovered() { NEON_CORAL } else { NEON });
    let text_color = if primary { VOID } else { NEON };

    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(0), fill);
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(0),
        stroke,
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(11.0),
        text_color,
    );

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}

/// Quiet status line: neon for confirmations, coral for failures.
pub fn status(ui: &mut egui::Ui, text: &str, is_error: bool) {
    let color = if is_error { NEON_CORAL } else { NEON };
    ui.label(egui::RichText::new(text).color(color).size(13.0));
}

/// Prefer an already-installed system font so Arabic glyphs render correctly.
/// This never fetches anything from the network.
fn install_system_font(ctx: &egui::Context) {
    let candidates: &[&str] = &[
        #[cfg(target_os = "windows")]
        r"C:\Windows\Fonts\arial.ttf",
        #[cfg(target_os = "linux")]
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        #[cfg(target_os = "linux")]
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        #[cfg(target_os = "android")]
        "/system/fonts/NotoSansArabic-Regular.ttf",
        #[cfg(target_os = "android")]
        "/system/fonts/Roboto-Regular.ttf",
    ];

    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "mawthooq-ui".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "mawthooq-ui".to_owned());
            ctx.set_fonts(fonts);
            return;
        }
    }
}
