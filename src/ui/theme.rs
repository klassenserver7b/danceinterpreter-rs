//! Design system for the redesigned Danceinterpreter UI.
//!
//! The look is ported from the approved design prototype (`docs/src/App.tsx`
//! plus `docs/src/index.css`): a calm, sage-green "music studio" theme with
//! `Manrope` for headings and `DM Sans` for body text (bundled as static
//! instances, see `resources/main/fonts/`).

use iced::border::Radius;
use iced::font::{Family, Style as FontStyle, Weight};
use iced::widget::{button, checkbox, container, pick_list, scrollable, text_input, toggler};
use iced::{Background, Border, Color, Font, Theme, color};

/// Body font family (DM Sans statics are bundled).
pub const FONT_BODY: &str = "DM Sans";
/// Heading font family (Manrope statics are bundled).
pub const FONT_HEAD: &str = "Manrope";
/// Outlined icon font (Material Symbols).
pub const FONT_ICON: &str = "Material Symbols Outlined";
/// Filled icon font (Material Icons).
pub const FONT_ICON_FILLED: &str = "Material Icons";

pub fn body_font(weight: Weight) -> Font {
    Font {
        family: Family::Name(FONT_BODY),
        weight,
        style: FontStyle::Normal,
        stretch: iced::font::Stretch::Normal,
    }
}

pub fn head_font(weight: Weight) -> Font {
    Font {
        family: Family::Name(FONT_HEAD),
        weight,
        style: FontStyle::Normal,
        stretch: iced::font::Stretch::Normal,
    }
}

#[allow(dead_code)] // design-system toolkit
pub fn icon_font() -> Font {
    Font {
        family: Family::Name(FONT_ICON),
        weight: Weight::Normal,
        style: FontStyle::Normal,
        stretch: iced::font::Stretch::Normal,
    }
}

#[allow(dead_code)] // design-system toolkit
pub fn icon_font_filled() -> Font {
    Font {
        family: Family::Name(FONT_ICON_FILLED),
        weight: Weight::Normal,
        style: FontStyle::Normal,
        stretch: iced::font::Stretch::Normal,
    }
}

/// Text sizes (design pixels). The smallest labels from the prototype were
/// bumped by ~1px for projector-room legibility.
pub mod sz {
    pub const BASE: f32 = 13.0;
    #[allow(dead_code)] // kept for parity with the prototype
    pub const H1: f32 = 29.0;
    pub const H2: f32 = 19.0;

    /// Panel / card titles.
    pub const PANEL_TITLE: f32 = 13.0;
    /// Row primary text.
    pub const ROW: f32 = 14.0;
    /// Row secondary text, captions, subtitles.
    pub const SMALL: f32 = 12.0;
    /// Tags, eyebrows, kbd hints, footers.
    pub const TINY: f32 = 10.5;
    /// Micro labels (source tags, version).
    pub const MICRO: f32 = 9.5;
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

/// Linear interpolation between two colors (iced has no `Color::lerp`).
fn mix(a: Color, b: Color, t: f32) -> Color {
    let k = t.clamp(0.0, 1.0);
    Color::from_rgba(
        a.r + (b.r - a.r) * k,
        a.g + (b.g - a.g) * k,
        a.b + (b.b - a.b) * k,
        a.a,
    )
}

/// A copy of `color` with a different alpha (for hover / zebra overlays).
fn with_alpha(color: Color, a: f32) -> Color {
    Color { a, ..color }
}

/// All colors used by the redesign, resolved for the active theme.
///
/// `Theme::Dark` / `Theme::Light` map to the hand-tuned sage palettes; every
/// other theme picked in the settings dialog derives its full token set from
/// the iced palette ([`Tokens::from_palette`]), so the picker re-colors the
/// whole UI instead of only iced's built-in widgets.
#[derive(Debug, Clone)]
pub struct Tokens {
    pub is_light: bool,

    // Surfaces
    pub bg: Color,
    pub panel: Color,
    pub panel_deep: Color,
    pub panel_head: Color,
    pub border: Color,
    pub border_soft: Color,

    // Text
    pub text: Color,
    pub text_soft: Color,
    pub muted: Color,
    pub muted_head: Color,
    pub faint: Color,

    // Accent (sage green)
    pub accent: Color,
    pub accent_border: Color,
    pub accent_text: Color,
    pub accent_soft: Color,
    pub accent_bar: Color,
    pub accent_nav_bg: Color,
    pub accent_nav_text: Color,
    pub dance_headline: Color,
    #[allow(dead_code)] // kept for parity with the prototype
    pub focus: Color,

    // Switch
    pub switch_on: Color,
    pub switch_on_knob: Color,
    pub switch_off: Color,
    pub switch_knob: Color,

    // Buttons
    pub btn2_bg: Color,
    pub btn2_border: Color,
    pub btn2_text: Color,
    #[allow(dead_code)] // kept for parity with the prototype
    pub btn3_bg: Color,
    #[allow(dead_code)] // kept for parity with the prototype
    pub btn3_text: Color,
    pub active_bg: Color,
    pub active_border: Color,
    pub active_text: Color,
    pub outline_bg: Color,
    pub outline_border: Color,
    pub outline_text: Color,
    pub ghost_hover: Color,
    pub danger: Color,

    // Rows
    #[allow(dead_code)] // kept for parity with the prototype
    pub row_hover: Color,
    pub row_zebra: Color,
    pub row_current_bg: Color,
    pub row_current_border: Color,
    pub match_bg: Color,
    pub match_border: Color,

    // Tags / chips
    pub tag_border: Color,
    pub tag_text: Color,

    // Status
    pub dot_green: Color,
    #[allow(dead_code)] // kept for parity with the prototype
    pub dot_off: Color,
    pub led_connected: Color,
    pub led_partial: Color,
    pub led_off: Color,
    pub star: Color,

    // Toast
    pub toast_bg: Color,
    pub toast_border: Color,
    pub toast_text: Color,

    // Modal
    pub modal_border: Color,
    pub backdrop: Color,

    // Keyboard hints
    pub kbd_border: Color,
    pub kbd_bg: Color,
    pub kbd_text: Color,

    // Now-playing card
    pub now_border: Color,
    pub now_grad_a: Color,
    pub now_grad_b: Color,
    pub now_grad_c: Color,
}

impl Tokens {
    pub fn of(theme: &Theme) -> Self {
        match theme {
            Theme::Dark => Self::dark(),
            Theme::Light => Self::light(),
            other => Self::from_palette(other.palette()),
        }
    }

    pub fn dark() -> Self {
        Self {
            is_light: false,
            bg: rgb(0x11, 0x14, 0x13),
            panel: rgb(0x19, 0x1D, 0x1B),
            panel_deep: rgb(0x14, 0x18, 0x15),
            panel_head: rgb(0x17, 0x1B, 0x18),
            border: rgb(0x2B, 0x30, 0x2D),
            border_soft: rgb(0x25, 0x2B, 0x26),

            text: rgb(0xE9, 0xEB, 0xE9),
            text_soft: rgb(0xBD, 0xC5, 0xBF),
            muted: rgb(0x81, 0x8B, 0x84),
            muted_head: rgb(0x75, 0x83, 0x79),
            faint: rgb(0x6B, 0x76, 0x6E),

            accent: rgb(0xB9, 0xD1, 0xA7),
            accent_border: rgb(0xA9, 0xC4, 0x96),
            accent_text: rgb(0x18, 0x22, 0x19),
            accent_soft: rgb(0xB2, 0xD2, 0xA1),
            accent_bar: rgb(0xB1, 0xCE, 0x96),
            accent_nav_bg: rgb(0x28, 0x31, 0x2A),
            accent_nav_text: rgb(0xC4, 0xD8, 0xB7),
            dance_headline: rgb(0xD2, 0xE2, 0xC7),
            focus: rgb(0xAF, 0xD3, 0xA9),

            switch_on: rgb(0xB7, 0xCF, 0x9F),
            switch_on_knob: rgb(0x27, 0x32, 0x21),
            switch_off: rgb(0x39, 0x44, 0x3A),
            switch_knob: rgb(0xA3, 0xB1, 0x9E),

            btn2_bg: rgb(0x21, 0x29, 0x1F),
            btn2_border: rgb(0x35, 0x40, 0x33),
            btn2_text: rgb(0xB8, 0xC6, 0xB1),
            btn3_bg: rgb(0x1A, 0x20, 0x1B),
            btn3_text: rgb(0x91, 0xA0, 0x8A),
            active_bg: rgb(0x41, 0x50, 0x39),
            active_border: rgb(0x9A, 0xB5, 0x82),
            active_text: rgb(0xD9, 0xE6, 0xCF),
            outline_bg: rgb(0x19, 0x1D, 0x1B),
            outline_border: rgb(0x36, 0x3E, 0x36),
            outline_text: rgb(0xC9, 0xD0, 0xC9),
            ghost_hover: color!(0xFFFFFF, 0.03),
            danger: rgb(0xC2, 0x5E, 0x4C),

            row_hover: color!(0xFFFFFF, 0.02),
            row_zebra: color!(0xFFFFFF, 0.004),
            row_current_bg: rgb(0x29, 0x36, 0x29),
            row_current_border: rgb(0x43, 0x54, 0x3D),
            match_bg: rgb(0x23, 0x2D, 0x23),
            match_border: rgb(0x86, 0xA2, 0x71),

            tag_border: rgb(0x4B, 0x58, 0x47),
            tag_text: rgb(0x93, 0xA2, 0x8C),

            dot_green: rgb(0xB2, 0xD2, 0xA1),
            dot_off: rgb(0x66, 0x70, 0x67),
            led_connected: rgb(0x7F, 0xB0, 0x69),
            led_partial: rgb(0xD4, 0xBD, 0x7E),
            led_off: rgb(0x4A, 0x55, 0x4B),
            star: rgb(0xD4, 0xBD, 0x7E),

            toast_bg: rgb(0x2C, 0x3B, 0x28),
            toast_border: rgb(0x65, 0x7D, 0x58),
            toast_text: rgb(0xD4, 0xE4, 0xC8),

            modal_border: rgb(0x40, 0x50, 0x39),
            backdrop: color!(0x050805, 0.72),

            kbd_border: rgb(0x3F, 0x4C, 0x3A),
            kbd_bg: rgb(0x22, 0x2C, 0x20),
            kbd_text: rgb(0xA8, 0xB5, 0xA2),

            now_border: rgb(0x35, 0x42, 0x31),
            now_grad_a: rgb(0x26, 0x32, 0x24),
            now_grad_b: rgb(0x20, 0x2C, 0x22),
            now_grad_c: rgb(0x1A, 0x24, 0x1D),
        }
    }

    pub fn light() -> Self {
        Self {
            is_light: true,
            bg: rgb(0xF2, 0xF4, 0xF0),
            panel: rgb(0xFF, 0xFF, 0xFF),
            panel_deep: rgb(0xEA, 0xEE, 0xE7),
            panel_head: rgb(0xEF, 0xF2, 0xEC),
            border: rgb(0xD9, 0xE0, 0xD8),
            border_soft: rgb(0xE4, 0xE8, 0xE2),

            text: rgb(0x24, 0x2C, 0x26),
            text_soft: rgb(0x4A, 0x55, 0x4B),
            muted: rgb(0x71, 0x80, 0x75),
            muted_head: rgb(0x82, 0x8D, 0x82),
            faint: rgb(0x99, 0xA3, 0x99),

            accent: rgb(0xB9, 0xD1, 0xA7),
            accent_border: rgb(0x9C, 0xB8, 0x87),
            accent_text: rgb(0x18, 0x22, 0x19),
            accent_soft: rgb(0x7F, 0xA8, 0x68),
            accent_bar: rgb(0x7B, 0xA2, 0x65),
            accent_nav_bg: rgb(0xE3, 0xEC, 0xDA),
            accent_nav_text: rgb(0x3D, 0x4F, 0x37),
            dance_headline: rgb(0x4A, 0x6B, 0x3F),
            focus: rgb(0x7F, 0xA8, 0x68),

            switch_on: rgb(0xA9, 0xC4, 0x96),
            switch_on_knob: rgb(0x2C, 0x3D, 0x24),
            switch_off: rgb(0xD6, 0xDD, 0xD2),
            switch_knob: rgb(0x8A, 0x96, 0x8A),

            btn2_bg: rgb(0xEE, 0xF1, 0xEC),
            btn2_border: rgb(0xD3, 0xDB, 0xD2),
            btn2_text: rgb(0x4A, 0x55, 0x4B),
            btn3_bg: rgb(0xE9, 0xED, 0xE6),
            btn3_text: rgb(0x71, 0x80, 0x75),
            active_bg: rgb(0xD5, 0xE3, 0xC8),
            active_border: rgb(0x86, 0xA2, 0x71),
            active_text: rgb(0x2C, 0x3D, 0x24),
            outline_bg: rgb(0xFF, 0xFF, 0xFF),
            outline_border: rgb(0xD3, 0xDB, 0xD2),
            outline_text: rgb(0x4A, 0x55, 0x4B),
            ghost_hover: color!(0x242C26, 0.05),
            danger: rgb(0xB0, 0x56, 0x3F),

            row_hover: color!(0x242C26, 0.03),
            row_zebra: color!(0x242C26, 0.0),
            row_current_bg: rgb(0xE4, 0xEE, 0xD9),
            row_current_border: rgb(0xC2, 0xD4, 0xB2),
            match_bg: rgb(0xEE, 0xF4, 0xE6),
            match_border: rgb(0x86, 0xA2, 0x71),

            tag_border: rgb(0xC8, 0xD2, 0xC3),
            tag_text: rgb(0x75, 0x83, 0x6F),

            dot_green: rgb(0x6F, 0x9A, 0x5C),
            dot_off: rgb(0xAA, 0xB2, 0xAA),
            led_connected: rgb(0x5C, 0x9A, 0x48),
            led_partial: rgb(0xC9, 0xA2, 0x3E),
            led_off: rgb(0xB4, 0xBC, 0xB2),
            star: rgb(0xB8, 0x92, 0x3E),

            toast_bg: rgb(0xE6, 0xEF, 0xDD),
            toast_border: rgb(0x93, 0xAD, 0x85),
            toast_text: rgb(0x33, 0x50, 0x2C),

            modal_border: rgb(0xC2, 0xD4, 0xB2),
            backdrop: color!(0x0F120F, 0.45),

            kbd_border: rgb(0xD3, 0xDB, 0xD2),
            kbd_bg: rgb(0xE9, 0xED, 0xE6),
            kbd_text: rgb(0x5A, 0x67, 0x5C),

            now_border: rgb(0xCF, 0xDC, 0xC7),
            now_grad_a: rgb(0xE9, 0xF0, 0xE0),
            now_grad_b: rgb(0xEE, 0xF2, 0xE9),
            now_grad_c: rgb(0xF4, 0xF6, 0xF1),
        }
    }

    /// Derives every token from an iced [`Palette`] so that any theme picked
    /// in the settings dialog re-colors the whole UI.
    ///
    /// The derivation reuses the roles of the sage design: panels are the
    /// background lifted toward white (light themes) or the text color (dark
    /// themes), borders are faint background→text blends, secondary text
    /// steps from text toward the background, and the accent family tints
    /// surfaces, borders and text with `primary`. `success` / `warning` /
    /// `danger` feed the status colors directly.
    pub fn from_palette(palette: iced::theme::Palette) -> Self {
        let bg = palette.background;
        let text = palette.text;
        let accent = palette.primary;
        let is_light = luminance(bg) >= 0.5;

        // Panels: lift the background toward white (light) or text (dark).
        let surface = if is_light { Color::WHITE } else { text };
        let up = |k: f32| mix(bg, surface, k);
        // Borders: faint background→text blends.
        let line = |k: f32| mix(bg, text, k);
        // Text stepped down toward the background.
        let dim = |k: f32| mix(text, bg, k);
        // Accent darkened for borders / strong states (both directions darken).
        let anchor = if is_light { text } else { bg };
        let accent_deep = |k: f32| mix(accent, anchor, k);
        // Text tinted toward the accent.
        let tint = |k: f32| mix(text, accent, k);

        let panel = if is_light { Color::WHITE } else { up(0.05) };
        let accent_border = accent_deep(0.12);

        Self {
            is_light,
            bg,
            panel,
            panel_deep: mix(bg, text, 0.03),
            panel_head: mix(bg, text, 0.04),
            border: line(0.12),
            border_soft: line(0.08),

            text,
            text_soft: dim(0.2),
            muted: dim(0.45),
            muted_head: dim(0.5),
            faint: dim(0.6),

            accent,
            accent_border,
            accent_text: mix(accent, Color::BLACK, 0.85),
            accent_soft: if is_light { accent_deep(0.3) } else { accent },
            accent_bar: if is_light { accent_deep(0.3) } else { accent },
            accent_nav_bg: mix(bg, accent, 0.12),
            accent_nav_text: tint(if is_light { 0.15 } else { 0.45 }),
            dance_headline: tint(if is_light { 0.2 } else { 0.5 }),
            focus: accent_deep(0.15),

            switch_on: if is_light { accent_border } else { accent },
            switch_on_knob: mix(accent, Color::BLACK, 0.8),
            switch_off: line(0.15),
            switch_knob: mix(dim(0.42), accent, 0.4),

            btn2_bg: mix(bg, accent, 0.08),
            btn2_border: line(0.16),
            btn2_text: dim(if is_light { 0.2 } else { 0.3 }),
            btn3_bg: if is_light { line(0.06) } else { up(0.03) },
            btn3_text: dim(0.4),
            active_bg: mix(bg, accent, 0.32),
            active_border: accent_deep(0.25),
            active_text: tint(if is_light { 0.08 } else { 0.3 }),
            outline_bg: panel,
            outline_border: line(0.14),
            outline_text: dim(0.16),
            ghost_hover: with_alpha(text, if is_light { 0.05 } else { 0.03 }),
            danger: palette.danger,

            row_hover: with_alpha(text, if is_light { 0.03 } else { 0.02 }),
            row_zebra: with_alpha(text, if is_light { 0.0 } else { 0.004 }),
            row_current_bg: mix(bg, accent, 0.2),
            row_current_border: mix(accent, bg, if is_light { 0.2 } else { 0.7 }),
            match_bg: mix(bg, accent, 0.11),
            match_border: accent_deep(0.3),

            tag_border: line(0.22),
            tag_text: dim(0.4),

            dot_green: palette.success,
            dot_off: dim(0.55),
            led_connected: mix(palette.success, anchor, 0.15),
            led_partial: palette.warning,
            led_off: line(0.25),
            star: palette.warning,

            toast_bg: mix(bg, accent, 0.18),
            toast_border: accent_deep(0.4),
            toast_text: tint(if is_light { 0.1 } else { 0.45 }),

            modal_border: mix(accent, bg, if is_light { 0.2 } else { 0.7 }),
            backdrop: with_alpha(
                mix(Color::BLACK, accent, 0.05),
                if is_light { 0.45 } else { 0.72 },
            ),

            kbd_border: line(0.15),
            kbd_bg: mix(bg, accent, 0.12),
            kbd_text: dim(0.28),

            now_border: mix(bg, accent, if is_light { 0.65 } else { 0.25 }),
            now_grad_a: mix(bg, accent, 0.15),
            now_grad_b: mix(bg, accent, 0.1),
            now_grad_c: mix(bg, accent, 0.05),
        }
    }
}

fn luminance(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

/// Four-sided padding (iced only implements `From` for scalars and pairs).
pub fn pad4(top: f32, right: f32, bottom: f32, left: f32) -> iced::Padding {
    iced::Padding {
        top,
        right,
        bottom,
        left,
    }
}

// ---------------------------------------------------------------------------
// Container styles
// ---------------------------------------------------------------------------

pub fn page_bg(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default().background(t.bg)
}

/// Standard card: panel background, hairline border, rounded corners.
pub fn card(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default()
        .background(t.panel)
        .border(Border {
            color: t.border,
            width: 1.0,
            radius: Radius::from(7.0),
        })
}

pub fn sidebar_bg(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default().background(t.bg).border(Border {
        color: t.border,
        width: 1.0,
        radius: Radius::default(),
    })
}

/// Gradient "now playing" hero card.
pub fn now_card(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default()
        .background(Background::Gradient(iced::Gradient::Linear(
            iced::gradient::Linear::new(0.0)
                .add_stop(0.0, t.now_grad_a)
                .add_stop(0.45, t.now_grad_b)
                .add_stop(1.0, t.now_grad_c),
        )))
        .border(Border {
            color: t.now_border,
            width: 1.0,
            radius: Radius::from(8.0),
        })
}

/// Strip below the show bar / above rows.
pub fn table_head(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default().background(t.panel_head)
}

pub fn showbar_bg(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default()
        .background(t.panel)
        .border(Border {
            color: t.border,
            width: 1.0,
            radius: Radius::default(),
        })
}

/// Visual states of a playlist/statics row.
#[derive(Debug, Clone, Copy, Default)]
pub struct RowLook {
    pub zebra: bool,
    pub current: bool,
    pub matched: bool,
    pub first: bool,
}

pub fn row(theme: &Theme, look: RowLook) -> container::Style {
    let t = Tokens::of(theme);
    let radius = if look.first {
        Radius {
            top_left: 7.0,
            top_right: 7.0,
            bottom_right: 0.0,
            bottom_left: 0.0,
        }
    } else {
        Radius::from(0.0)
    };

    let (bg, border_color) = if look.current {
        (t.row_current_bg, t.row_current_border)
    } else if look.matched {
        (t.match_bg, t.match_border)
    } else if look.zebra {
        (t.row_zebra, t.border_soft)
    } else {
        (Color::TRANSPARENT, t.border_soft)
    };

    container::Style::default().background(bg).border(Border {
        color: border_color,
        width: 1.0,
        radius,
    })
}

pub fn modal_card(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default()
        .background(t.panel)
        .border(Border {
            color: t.modal_border,
            width: 1.0,
            radius: Radius::from(10.0),
        })
}

pub fn backdrop(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default().background(t.backdrop)
}

pub fn toast_card(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default()
        .background(t.toast_bg)
        .border(Border {
            color: t.toast_border,
            width: 1.0,
            radius: Radius::from(6.0),
        })
        .shadow(iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 35.0,
        })
}

/// Neutral dance chip: a faint text-tinted pill, like the prototype
/// (`background:#ffffff04; border:#ffffff06`). The dance color is carried by
/// the dot and the dance name, not by the chip surface, so arbitrary
/// user-picked colors cannot wreck the chip's contrast.
pub fn chip() -> impl Fn(&Theme) -> container::Style {
    |theme: &Theme| {
        let t = Tokens::of(theme);
        container::Style::default()
            .background(Background::Color(with_alpha(t.text, 0.05)))
            .border(Border {
                color: with_alpha(t.text, 0.1),
                width: 1.0,
                radius: Radius::from(4.0),
            })
    }
}

/// Blends a user-picked color toward the theme text color when its contrast
/// against the background is too low. Dance colors are picked freely and are
/// often too dark (or too close to the background) to stay readable.
pub fn readable(color: Color, theme: &Theme) -> Color {
    let t = Tokens::of(theme);
    let (hi, lo) = if luminance(color) > luminance(t.bg) {
        (color, t.bg)
    } else {
        (t.bg, color)
    };
    let ratio = (luminance(hi) + 0.05) / (luminance(lo) + 0.05);
    if ratio < 2.6 {
        mix(color, t.text, 0.4)
    } else {
        color
    }
}

/// Placeholder cover art when a song has no embedded image.
pub fn cover_placeholder(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    let (a, b) = if t.is_light {
        (rgb(0xDE, 0xE6, 0xD5), rgb(0xC3, 0xD1, 0xBB))
    } else {
        (rgb(0x2C, 0x38, 0x2B), rgb(0x17, 0x21, 0x1C))
    };
    container::Style::default()
        .background(Background::Gradient(iced::Gradient::Linear(
            iced::gradient::Linear::new(1.2)
                .add_stop(0.0, a)
                .add_stop(1.0, b),
        )))
        .border(Border {
            color: t.border,
            width: 1.0,
            radius: Radius::from(4.0),
        })
}

/// Keyboard hint pill.
pub fn kbd(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default()
        .background(t.kbd_bg)
        .border(Border {
            color: t.kbd_border,
            width: 1.0,
            radius: Radius::from(3.0),
        })
}

// ---------------------------------------------------------------------------
// Button styles
// ---------------------------------------------------------------------------

fn hover(status: button::Status, base: Color, factor: f32) -> Option<Background> {
    match status {
        button::Status::Hovered => Some(Background::Color(Color {
            a: base.a,
            r: (base.r * factor).min(1.0),
            g: (base.g * factor).min(1.0),
            b: (base.b * factor).min(1.0),
        })),
        button::Status::Pressed => Some(Background::Color(Color {
            a: base.a,
            r: (base.r * factor).min(1.0),
            g: (base.g * factor).min(1.0),
            b: (base.b * factor).min(1.0),
        })),
        button::Status::Disabled => Some(Background::Color(Color { a: 0.4, ..base })),
        _ => Some(Background::Color(base)),
    }
}

pub fn btn_primary(theme: &Theme, status: button::Status) -> button::Style {
    let t = Tokens::of(theme);
    button::Style {
        background: hover(status, t.accent, 1.06),
        text_color: t.accent_text,
        border: Border {
            color: t.accent_border,
            width: 1.0,
            radius: Radius::from(5.0),
        },
        ..button::Style::default()
    }
}

pub fn btn_secondary(theme: &Theme, status: button::Status) -> button::Style {
    let t = Tokens::of(theme);
    button::Style {
        background: hover(status, t.btn2_bg, 1.08),
        text_color: t.btn2_text,
        border: Border {
            color: t.btn2_border,
            width: 1.0,
            radius: Radius::from(5.0),
        },
        ..button::Style::default()
    }
}

/// Outline button used in page headings.
pub fn btn_outline(theme: &Theme, status: button::Status) -> button::Style {
    let t = Tokens::of(theme);
    button::Style {
        background: hover(status, t.outline_bg, 1.04),
        text_color: t.outline_text,
        border: Border {
            color: t.outline_border,
            width: 1.0,
            radius: Radius::from(5.0),
        },
        ..button::Style::default()
    }
}

/// Transparent icon button with a subtle hover wash.
pub fn btn_ghost(theme: &Theme, status: button::Status) -> button::Style {
    let t = Tokens::of(theme);
    button::Style {
        background: match status {
            button::Status::Hovered => Some(Background::Color(t.ghost_hover)),
            button::Status::Pressed => Some(Background::Color(Color {
                a: 0.08,
                ..t.ghost_hover
            })),
            _ => None,
        },
        text_color: t.muted,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(5.0),
        },
        ..button::Style::default()
    }
}

pub fn btn_danger(theme: &Theme, status: button::Status) -> button::Style {
    let t = Tokens::of(theme);
    button::Style {
        background: hover(status, t.danger, 1.08),
        text_color: Color::WHITE,
        border: Border {
            color: t.danger,
            width: 1.0,
            radius: Radius::from(5.0),
        },
        ..button::Style::default()
    }
}

/// Active quick-show button (the currently displayed item).
pub fn btn_active(theme: &Theme, status: button::Status) -> button::Style {
    let t = Tokens::of(theme);
    button::Style {
        background: hover(status, t.active_bg, 1.06),
        text_color: t.active_text,
        border: Border {
            color: t.active_border,
            width: 1.0,
            radius: Radius::from(5.0),
        },
        ..button::Style::default()
    }
}

/// Sidebar navigation item.
pub fn btn_nav(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, status: button::Status| {
        let t = Tokens::of(theme);
        let base = if selected { t.accent_nav_bg } else { t.bg };
        button::Style {
            background: hover(status, base, 1.08),
            text_color: if selected { t.accent_nav_text } else { t.muted },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(5.0),
            },
            ..button::Style::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Text input styles
// ---------------------------------------------------------------------------

/// Borderless inline input used inside table rows (contentEditable-style).
pub fn input_plain(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let t = Tokens::of(theme);
    text_input::Style {
        background: input_plain_bg(theme, status),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(3.0),
        },
        icon: t.muted,
        placeholder: t.faint,
        value: match status {
            text_input::Status::Focused { .. } => t.text,
            _ => t.text_soft,
        },
        selection: Color { a: 0.3, ..t.accent },
    }
}

/// Muted variant of [`input_plain`] for secondary row fields (artist).
pub fn input_plain_dim(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let t = Tokens::of(theme);
    let mut style = input_plain(theme, status);
    style.value = match status {
        text_input::Status::Focused { .. } => t.text_soft,
        _ => t.muted,
    };
    style
}

fn input_plain_bg(theme: &Theme, status: text_input::Status) -> Background {
    let t = Tokens::of(theme);
    match status {
        text_input::Status::Focused { .. } => Background::Color(if t.is_light {
            color!(0x242C26, 0.04)
        } else {
            color!(0xFFFFFF, 0.04)
        }),
        _ => Background::Color(Color::TRANSPARENT),
    }
}

/// Boxed input (search, quick add, dialogs).
pub fn input_boxed(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let t = Tokens::of(theme);
    let border_color = match status {
        text_input::Status::Focused { .. } => t.focus,
        text_input::Status::Disabled => t.border_soft,
        _ => t.border,
    };
    text_input::Style {
        background: Background::Color(t.panel_deep),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: Radius::from(4.0),
        },
        icon: t.muted,
        placeholder: t.faint,
        value: t.text,
        selection: Color { a: 0.3, ..t.accent },
    }
}

// ---------------------------------------------------------------------------
// Misc widget styles
// ---------------------------------------------------------------------------

pub fn scrollable_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let t = Tokens::of(theme);
    let active = matches!(
        status,
        scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. }
    );
    let scrollbar = scrollable::Rail {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(3.0),
        },
        scroller: scrollable::Scroller {
            background: Background::Color(if active { t.accent_border } else { t.border }),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(3.0),
            },
        },
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollbar,
        horizontal_rail: scrollbar,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(Color { a: 0.8, ..t.panel }),
            border: Border {
                color: t.border,
                width: 1.0,
                radius: Radius::from(20.0),
            },
            shadow: iced::Shadow::default(),
            icon: t.text,
        },
    }
}

pub fn pick_list_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let t = Tokens::of(theme);
    let bg = match status {
        pick_list::Status::Hovered => Color {
            a: 1.0,
            ..t.btn2_bg
        },
        _ => t.panel_deep,
    };
    pick_list::Style {
        text_color: t.text_soft,
        placeholder_color: t.faint,
        handle_color: t.muted,
        background: Background::Color(bg),
        border: Border {
            color: t.border,
            width: 1.0,
            radius: Radius::from(4.0),
        },
    }
}

pub fn pick_list_menu_style(theme: &Theme) -> iced::widget::overlay::menu::Style {
    let t = Tokens::of(theme);
    iced::widget::overlay::menu::Style {
        background: Background::Color(t.panel),
        border: Border {
            color: t.border,
            width: 1.0,
            radius: Radius::from(6.0),
        },
        text_color: t.text_soft,
        selected_text_color: t.accent_nav_text,
        selected_background: Background::Color(t.accent_nav_bg),
        shadow: iced::Shadow::default(),
    }
}

pub fn toggler_style(theme: &Theme, status: toggler::Status) -> toggler::Style {
    let t = Tokens::of(theme);
    let active = match status {
        toggler::Status::Active { is_toggled }
        | toggler::Status::Hovered { is_toggled }
        | toggler::Status::Disabled { is_toggled } => is_toggled,
    };
    toggler::Style {
        background: Background::Color(if active { t.switch_on } else { t.switch_off }),
        background_border_width: 0.0,
        background_border_color: Color::TRANSPARENT,
        foreground: Background::Color(if active {
            t.switch_on_knob
        } else {
            t.switch_knob
        }),
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        text_color: None,
        border_radius: None,
        padding_ratio: 0.22,
    }
}

#[allow(dead_code)] // design-system toolkit
pub fn checkbox_style(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let t = Tokens::of(theme);
    let (bg, border_color) = match status {
        checkbox::Status::Disabled { .. } => (t.switch_off, t.border),
        checkbox::Status::Hovered { is_checked } if is_checked => (t.switch_on, t.switch_on),
        checkbox::Status::Hovered { .. } => (t.panel_deep, t.accent_border),
        checkbox::Status::Active { is_checked } if is_checked => (t.switch_on, t.switch_on),
        checkbox::Status::Active { .. } => (t.panel_deep, t.border),
    };
    checkbox::Style {
        background: Background::Color(bg),
        icon_color: t.switch_on_knob,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: Radius::from(4.0),
        },
        text_color: Some(t.text_soft),
    }
}

pub fn tooltip_style(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default()
        .background(t.panel)
        .border(Border {
            color: t.border,
            width: 1.0,
            radius: Radius::from(4.0),
        })
        .shadow(iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: iced::Vector::new(0.0, 3.0),
            blur_radius: 10.0,
        })
}

/// LED dot color for the DJ connection state.
pub fn led_color(theme: &Theme, state: crate::traktor_api::ConnectionState) -> Color {
    let t = Tokens::of(theme);
    use crate::traktor_api::ConnectionState as S;
    match state {
        S::Connected => t.led_connected,
        S::Traktor | S::CoverLoader => t.led_partial,
        S::Disconnected => t.led_off,
    }
}
