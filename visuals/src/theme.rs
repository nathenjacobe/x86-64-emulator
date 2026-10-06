//! shared colours and the dark theme installed when the window opens

use eframe::egui::{self, Color32};

/// accent used for state that changed during the last step
pub const ACCENT: Color32 = Color32::from_rgb(0xff, 0xc4, 0x66);

/// background tint for the instruction pointer row and byte
pub const POINTER_BG: Color32 = Color32::from_rgb(0x3d, 0x33, 0x0a);

/// dim colour for offsets, padding and other secondary text
pub const DIM: Color32 = Color32::from_rgb(0x8a, 0x8f, 0x98);

/// installs the dark monospace theme used by every panel
pub fn install(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.all_styles_mut(|style| {
        style.visuals = egui::Visuals::dark();
        style.visuals.selection.bg_fill = ACCENT.gamma_multiply(0.4);
        style.visuals.hyperlink_color = ACCENT;
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, egui::FontId::monospace(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(17.0));
    });
}
