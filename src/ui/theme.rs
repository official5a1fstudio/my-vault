//! Mawthooq visual language.
//!
//! A warm editorial surface (paper + ink) with olive structure and coral
//! emphasis. All colors are taken from the 5A1F Studio design tokens; corners
//! stay sharp and boundaries are expressed with one-pixel hairlines.

use eframe::egui;

/// Warm paper page background.
pub const PAPER: egui::Color32 = egui::Color32::from_rgb(0xf5, 0xf0, 0xe5);
/// Slightly lighter card surface.
pub const PAPER_RAISED: egui::Color32 = egui::Color32::from_rgb(0xff, 0xfb, 0xf0);
/// Deep sunken paper.
pub const PAPER_DEEP: egui::Color32 = egui::Color32::from_rgb(0xe8, 0xdf, 0xcf);
/// Primary type color.
pub const INK: egui::Color32 = egui::Color32::from_rgb(0x22, 0x24, 0x20);
/// Secondary type color.
pub const INK_SOFT: egui::Color32 = egui::Color32::from_rgb(0x45, 0x47, 0x3f);
/// Muted type color.
pub const INK_FAINT: egui::Color32 = egui::Color32::from_rgb(0x67, 0x67, 0x5d);
/// Structural olive.
pub const OLIVE: egui::Color32 = egui::Color32::from_rgb(0x25, 0x4f, 0x43);
/// Pressed / hovered olive.
pub const OLIVE_DEEP: egui::Color32 = egui::Color32::from_rgb(0x18, 0x3c, 0x32);
/// Soft olive wash.
pub const OLIVE_SOFT: egui::Color32 = egui::Color32::from_rgb(0xd8, 0xe0, 0xd0);
/// Highlight / error accent.
pub const CORAL: egui::Color32 = egui::Color32::from_rgb(0xcf, 0x62, 0x48);
/// Stronger coral for error text.
pub const CORAL_DEEP: egui::Color32 = egui::Color32::from_rgb(0xa8, 0x45, 0x2f);
/// rgba(34, 36, 32, 0.15) hairline.
pub fn hairline() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(0x22, 0x24, 0x20, 38)
}

/// rgba(34, 36, 32, 0.28) stronger rule.
pub fn hairline_strong() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(0x22, 0x24, 0x20, 71)
}

const OVERLINE_STYLE: &str = "mawthooq-overline";

fn overline_style() -> egui::TextStyle {
    egui::TextStyle::Name(OVERLINE_STYLE.into())
}

/// Install the light editorial palette and type scale. Safe to call repeatedly.
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
        .insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(28.0));
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(13.0));

    style.spacing.item_spacing = egui::vec2(10.0, 8.0);

    let visuals = &mut style.visuals;
    visuals.panel_fill = PAPER;
    visuals.window_fill = PAPER;
    visuals.extreme_bg_color = PAPER_RAISED;
    visuals.faint_bg_color = OLIVE_SOFT;
    visuals.text_color = INK;
    visuals.hyperlink_color = OLIVE;
    visuals.error_fg_color = CORAL_DEEP;
    visuals.selection.bg_fill = OLIVE_SOFT;
    visuals.selection.stroke = egui::Stroke::new(1.0, OLIVE);

    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_fill = PAPER;
    widgets.noninteractive.weak_bg_fill = PAPER;
    widgets.noninteractive.fg_color = INK_SOFT;
    widgets.noninteractive.stroke = egui::Stroke::new(1.0, hairline());
    widgets.noninteractive.corner_radius = egui::CornerRadius::same(0);

    widgets.inactive.bg_fill = PAPER_RAISED;
    widgets.inactive.weak_bg_fill = PAPER_RAISED;
    widgets.inactive.fg_color = INK;
    widgets.inactive.stroke = egui::Stroke::new(1.0, hairline());
    widgets.inactive.corner_radius = egui::CornerRadius::same(2);

    widgets.hovered.bg_fill = OLIVE_SOFT;
    widgets.hovered.weak_bg_fill = OLIVE_SOFT;
    widgets.hovered.fg_color = INK;
    widgets.hovered.stroke = egui::Stroke::new(1.0, OLIVE);
    widgets.hovered.corner_radius = egui::CornerRadius::same(2);

    widgets.active.bg_fill = OLIVE;
    widgets.active.weak_bg_fill = OLIVE_SOFT;
    widgets.active.fg_color = PAPER_RAISED;
    widgets.active.stroke = egui::Stroke::new(1.0, OLIVE_DEEP);
    widgets.active.corner_radius = egui::CornerRadius::same(2);

    ctx.set_style(style);
}

/// Small tracked label used above headings and card titles.
pub fn overline(ui: &mut egui::Ui, text: impl Into<String>) {
    let text: String = text.into();
    ui.label(
        egui::RichText::new(text)
            .text_style(overline_style())
            .color(OLIVE),
    );
}

/// Editorial page heading: overline, display title and a quiet lead paragraph.
pub fn page_header(ui: &mut egui::Ui, label: &str, heading: &str, lead: &str) {
    overline(ui, label);
    ui.add_space(8.0);
    ui.label(egui::RichText::new(heading).color(INK).size(30.0));
    ui.add_space(8.0);
    ui.label(egui::RichText::new(lead).color(INK_SOFT).size(15.0));
}

/// Bordered editorial card with a hairline frame and overline title.
pub fn card(ui: &mut egui::Ui, title: &str, body_fn: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(PAPER_RAISED)
        .stroke(egui::Stroke::new(1.0, hairline()))
        .corner_radius(egui::CornerRadius::same(2))
        .inner_margin(20.0)
        .show(ui, |ui| {
            overline(ui, title);
            ui.add_space(12.0);
            body_fn(ui);
        });
}

/// Brand lockup for the top navigation: vector vault mark, name and studio.
pub fn brand_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
        draw_mark(ui.painter(), rect);
        ui.add_space(6.0);
        ui.vertical(|ui| {
            overline(ui, "5A1F STUDIO");
            ui.label(egui::RichText::new("Mawthooq  ·  موثوق").color(INK).size(18.0));
        });
    });
}

fn draw_mark(painter: &egui::Painter, rect: egui::Rect) {
    let r = rect.shrink(2.0);
    let stroke = egui::Stroke::new(1.5, OLIVE);
    painter.line_segment([r.left_top(), r.right_top()], stroke);
    painter.line_segment([r.left_top(), r.left_bottom()], stroke);
    painter.line_segment([r.right_top(), r.right_bottom()], stroke);
    painter.line_segment([r.left_bottom(), r.right_bottom()], stroke);

    let c = r.center();
    painter.circle_filled(egui::pos2(c.x, c.y - 3.0), 3.5, CORAL);
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(c.x - 1.5, c.y - 1.0),
            egui::pos2(c.x + 1.5, c.y + 7.0),
        ),
        egui::CornerRadius::same(0),
        CORAL,
    );
}

/// Solid olive call-to-action with sharp corners.
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> bool {
    let width = (text.chars().count() as f32 * 7.2 + 36.0).clamp(150.0, 280.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 34.0), egui::Sense::click());
    let fill = if response.hovered() {
        OLIVE_DEEP
    } else {
        OLIVE
    };
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(2), fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(13.0),
        PAPER_RAISED,
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}

/// Quiet status line; failures use coral, confirmations use olive.
pub fn status(ui: &mut egui::Ui, text: &str, is_error: bool) {
    let color = if is_error { CORAL_DEEP } else { OLIVE };
    ui.label(egui::RichText::new(text).color(color).size(13.0));
}
