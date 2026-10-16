//! Colour palettes and egui style setup.

use egui::{Color32, Context, FontId, Stroke, TextStyle, ThemePreference, Visuals};

use crate::fonts::FontSet;

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub dark: bool,
    pub bg: Color32,
    pub panel_bg: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub heading: Color32,
    pub link: Color32,
    pub inline_code_bg: Color32,
    pub inline_code_text: Color32,
    pub code_bg: Color32,
    pub code_border: Color32,
    pub code_text: Color32,
    pub quote_bar: Color32,
    pub quote_text: Color32,
    pub rule: Color32,
    pub table_border: Color32,
    pub table_stripe: Color32,
    pub table_head_bg: Color32,
    pub selection: Color32,
    pub search_bg: Color32,
    pub search_current_bg: Color32,
    pub accent: Color32,
    pub math: Color32,
    pub note: Color32,
    pub tip: Color32,
    pub important: Color32,
    pub warning: Color32,
    pub caution: Color32,
    pub sidebar_current: Color32,
}

const fn c(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            dark: true,
            bg: c(0x15, 0x17, 0x1b),
            panel_bg: c(0x1b, 0x1e, 0x24),
            text: c(0xd4, 0xd8, 0xdf),
            text_dim: c(0x8a, 0x91, 0x9e),
            heading: c(0xee, 0xf1, 0xf5),
            link: c(0x78, 0xb1, 0xff),
            inline_code_bg: c(0x27, 0x2b, 0x33),
            inline_code_text: c(0xe3, 0xe7, 0xee),
            code_bg: c(0x1c, 0x1f, 0x26),
            code_border: c(0x2a, 0x2f, 0x38),
            code_text: c(0xd4, 0xd8, 0xdf),
            quote_bar: c(0x3d, 0x44, 0x52),
            quote_text: c(0xa8, 0xaf, 0xbb),
            rule: c(0x2a, 0x2f, 0x38),
            table_border: c(0x2f, 0x35, 0x3f),
            table_stripe: c(0x1a, 0x1d, 0x23),
            table_head_bg: c(0x20, 0x24, 0x2c),
            selection: Color32::from_rgba_unmultiplied(80, 130, 220, 110),
            search_bg: c(0x6b, 0x54, 0x10),
            search_current_bg: c(0xe0, 0xaa, 0x2a),
            accent: c(0x78, 0xb1, 0xff),
            math: c(0xc7, 0xa4, 0xff),
            note: c(0x44, 0x93, 0xf8),
            tip: c(0x3f, 0xb9, 0x50),
            important: c(0xab, 0x7d, 0xf8),
            warning: c(0xd2, 0x99, 0x22),
            caution: c(0xf8, 0x51, 0x49),
            sidebar_current: Color32::from_rgba_unmultiplied(120, 177, 255, 40),
        }
    }

    pub fn light() -> Self {
        Self {
            dark: false,
            bg: c(0xff, 0xff, 0xff),
            panel_bg: c(0xf6, 0xf8, 0xfa),
            text: c(0x1f, 0x23, 0x28),
            text_dim: c(0x65, 0x6d, 0x76),
            heading: c(0x11, 0x14, 0x18),
            link: c(0x09, 0x69, 0xda),
            inline_code_bg: c(0xef, 0xf1, 0xf4),
            inline_code_text: c(0x1f, 0x23, 0x28),
            code_bg: c(0xf6, 0xf8, 0xfa),
            code_border: c(0xd8, 0xde, 0xe4),
            code_text: c(0x1f, 0x23, 0x28),
            quote_bar: c(0xd0, 0xd7, 0xde),
            quote_text: c(0x59, 0x63, 0x6e),
            rule: c(0xd8, 0xde, 0xe4),
            table_border: c(0xd0, 0xd7, 0xde),
            table_stripe: c(0xf6, 0xf8, 0xfa),
            table_head_bg: c(0xee, 0xf1, 0xf4),
            selection: Color32::from_rgba_unmultiplied(10, 105, 218, 70),
            search_bg: c(0xff, 0xf1, 0xa8),
            search_current_bg: c(0xff, 0xd4, 0x3b),
            accent: c(0x09, 0x69, 0xda),
            math: c(0x6f, 0x42, 0xc1),
            note: c(0x09, 0x69, 0xda),
            tip: c(0x1a, 0x7f, 0x37),
            important: c(0x82, 0x50, 0xdf),
            warning: c(0x9a, 0x67, 0x00),
            caution: c(0xcf, 0x22, 0x2e),
            sidebar_current: Color32::from_rgba_unmultiplied(9, 105, 218, 30),
        }
    }
}

/// Push the palette, fonts and base text size into egui's style.
pub fn apply(ctx: &Context, pal: &Palette, fonts: &FontSet, base: f32) {
    let theme = if pal.dark { egui::Theme::Dark } else { egui::Theme::Light };
    ctx.set_theme(if pal.dark { ThemePreference::Dark } else { ThemePreference::Light });

    let mut v = if pal.dark { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = pal.panel_bg;
    v.window_fill = pal.panel_bg;
    v.extreme_bg_color = pal.bg;
    v.faint_bg_color = pal.table_stripe;
    v.code_bg_color = pal.inline_code_bg;
    v.hyperlink_color = pal.link;
    v.selection.bg_fill = pal.selection;
    v.selection.stroke = Stroke::new(1.0, pal.accent);
    v.window_stroke = Stroke::new(1.0, pal.rule);
    v.widgets.noninteractive.fg_stroke.color = pal.text;
    v.widgets.noninteractive.bg_stroke.color = pal.rule;
    v.widgets.inactive.fg_stroke.color = pal.text;
    v.widgets.hovered.fg_stroke.color = pal.heading;
    v.widgets.active.fg_stroke.color = pal.heading;
    v.widgets.open.fg_stroke.color = pal.heading;
    v.override_text_color = None;
    ctx.set_visuals_of(theme, v);

    ctx.all_styles_mut(|s| {
        s.text_styles.insert(TextStyle::Body, FontId::new(base, fonts.sans.clone()));
        s.text_styles.insert(TextStyle::Button, FontId::new(base * 0.95, fonts.sans.clone()));
        s.text_styles.insert(TextStyle::Small, FontId::new(base * 0.82, fonts.sans.clone()));
        s.text_styles.insert(TextStyle::Heading, FontId::new(base * 1.3, fonts.sans.clone()));
        s.text_styles.insert(TextStyle::Monospace, FontId::new(base * 0.9, fonts.mono.clone()));
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.scroll.bar_width = 10.0;
        s.interaction.selectable_labels = true;
        s.interaction.multi_widget_text_select = true;
        s.url_in_tooltip = false;
    });
}
