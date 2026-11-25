//! Design system for the redesigned Danceinterpreter UI.
//!
//! The look is ported from the approved design prototype (`docs/src/App.tsx`
//! plus `docs/src/index.css`): a calm, gray-blue "music studio" theme with
//! `Manrope` for headings and `DM Sans` for body text (bundled as static
//! instances, see `resources/main/fonts/`).

#![allow(clippy::unreadable_literal)]

use iced::border::Radius;
use iced::font::{Family, Style as FontStyle, Weight};
use iced::widget::{button, container, pick_list, scrollable, text_input, toggler};
use iced::{Background, Border, Color, Font, Theme, color};

/// Body font family (DM Sans statics are bundled).
pub const FONT_BODY: &str = "DM Sans";
/// Heading font family (Manrope statics are bundled).
pub const FONT_HEAD: &str = "Manrope";
/// Outlined icon font (Material Symbols).
pub const FONT_ICON: &str = "Material Symbols Outlined";
/// Filled icon font (Material Icons).
pub const FONT_ICON_FILLED: &str = "Material Icons";

/// Horizontal page gutter of the primary content column and top bar.
pub const PAGE_GUTTER: f32 = 31.0;

/// Vertical/horizontal padding of boxed text inputs and pick lists.
pub const INPUT_PAD: [f32; 2] = [5.0, 8.0];

/// Body font (DM Sans) with the given weight.
pub const fn body_font(weight: Weight) -> Font {
    Font {
        family: Family::Name(FONT_BODY),
        weight,
        style: FontStyle::Normal,
        stretch: iced::font::Stretch::Normal,
    }
}

/// Heading font (Manrope) with the given weight.
pub const fn head_font(weight: Weight) -> Font {
    Font {
        family: Family::Name(FONT_HEAD),
        weight,
        style: FontStyle::Normal,
        stretch: iced::font::Stretch::Normal,
    }
}

/// Text sizes (design pixels). The smallest labels from the prototype were
/// bumped by ~1px for projector-room legibility.
pub mod text_size {
    /// Body text, dialog copy.
    pub const BASE: f32 = 13.0;
    /// Section and dialog titles.
    pub const H2: f32 = 19.0;

    /// Panel / card titles.
    pub const PANEL_TITLE: f32 = 13.0;
    /// Row primary text.
    pub const ROW: f32 = 14.0;
    /// Row secondary text, subtitles.
    pub const SMALL: f32 = 12.0;
    /// Captions a notch above [`SMALL`] (toast text, show-bar buttons).
    pub const CAPTION: f32 = 12.5;
    /// Tags, eyebrows, kbd hints, footers.
    pub const TINY: f32 = 10.5;
    /// Micro labels (source tags, version).
    pub const MICRO: f32 = 9.5;
    /// Default icon size.
    pub const ICON: f32 = 16.0;
    /// Panel icons a notch under [`ICON`] (search, section and nav icons).
    pub const ICON_SM: f32 = 15.0;
    /// Row action icons (delete, play, skip, backspace, toast close).
    pub const ICON_XS: f32 = 14.0;
    /// Icons inline with small text (status rows, chevrons, button trailing
    /// icons).
    pub const ICON_XXS: f32 = 13.0;
}

/// Corner radius scale (design pixels), mirroring the prototype.
pub mod radius {
    /// Inline inputs, kbd hints.
    pub const XS: f32 = 3.0;
    /// Chips, boxed inputs, tooltips.
    pub const SM: f32 = 4.0;
    /// Buttons.
    pub const MD: f32 = 5.0;
    /// Toasts.
    pub const LG: f32 = 6.0;
    /// Cards.
    pub const XL: f32 = 7.0;
    /// Modals.
    pub const MODAL: f32 = 10.0;
    /// Pill-shaped elements.
    pub const PILL: f32 = 20.0;
}

/// Builds a [`iced::widget::text::Style`] closure from a token-derived color,
/// e.g. `text(text).style(theme::text_style(|tk| tk.muted))`.
pub fn text_style(
    color: impl Fn(&Tokens) -> Color,
) -> impl Fn(&Theme) -> iced::widget::text::Style {
    move |theme: &Theme| iced::widget::text::Style {
        color: Some(color(&Tokens::of(theme))),
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

/// Linear interpolation between two colors (iced has no `Color::lerp`).
fn mix(a: Color, b: Color, t: f32) -> Color {
    let k = t.clamp(0.0, 1.0);
    Color::from_rgba(
        (b.r - a.r).mul_add(k, a.r),
        (b.g - a.g).mul_add(k, a.g),
        (b.b - a.b).mul_add(k, a.b),
        a.a,
    )
}

/// A copy of `color` with a different alpha (for hover / zebra overlays).
pub const fn with_alpha(color: Color, a: f32) -> Color {
    Color { a, ..color }
}

/// All colors used by the redesign, resolved for the active theme.
///
/// `Theme::Dark` / `Theme::Light` map to the hand-tuned blue palettes; every
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

    // Accent (blue)
    pub accent: Color,
    pub accent_border: Color,
    pub accent_text: Color,
    pub accent_soft: Color,
    pub accent_bar: Color,
    pub accent_nav_bg: Color,
    pub accent_nav_text: Color,
    pub dance_headline: Color,
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
    pub active_bg: Color,
    pub active_border: Color,
    pub active_text: Color,
    pub outline_bg: Color,
    pub outline_border: Color,
    pub outline_text: Color,
    pub ghost_hover: Color,
    pub danger: Color,
    /// Text on danger-filled surfaces.
    pub danger_text: Color,

    // Rows
    pub row_zebra: Color,
    pub row_current_bg: Color,
    pub row_current_border: Color,
    pub match_bg: Color,
    pub match_border: Color,
    /// Text selection highlight.
    pub selection: Color,

    // Tags / chips
    pub tag_border: Color,
    pub tag_text: Color,

    // Status
    pub dot_green: Color,
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

    // Inputs
    /// Wash behind focused inline inputs.
    pub input_focus_bg: Color,
}

impl Tokens {
    /// Reads the token set for the active iced theme.
    pub fn of(theme: &Theme) -> Self {
        match theme {
            Theme::Dark => Self::dark(),
            Theme::Light => Self::light(),
            other => Self::from_palette(other.palette()),
        }
    }

    /// The hand-tuned dark palette (`Theme::Dark`).
    pub fn dark() -> Self {
        Self {
            is_light: false,
            bg: rgb(0x0D, 0x11, 0x17),
            panel: rgb(0x16, 0x1B, 0x22),
            panel_deep: rgb(0x11, 0x15, 0x1C),
            panel_head: rgb(0x13, 0x17, 0x1F),
            border: rgb(0x30, 0x36, 0x3D),
            border_soft: rgb(0x21, 0x26, 0x2D),

            text: rgb(0xE6, 0xED, 0xF3),
            text_soft: rgb(0xC9, 0xD1, 0xD9),
            muted: rgb(0x8B, 0x94, 0x9E),
            muted_head: rgb(0x76, 0x81, 0x8B),
            faint: rgb(0x6E, 0x76, 0x80),

            accent: rgb(0x58, 0xA6, 0xFF),
            accent_border: rgb(0x79, 0xB8, 0xFF),
            accent_text: rgb(0x05, 0x1D, 0x40),
            accent_soft: rgb(0xB2, 0xD2, 0xF0),
            accent_bar: rgb(0x79, 0xC0, 0xFF),
            accent_nav_bg: rgb(0x1C, 0x2A, 0x3A),
            accent_nav_text: rgb(0xA8, 0xC8, 0xE8),
            dance_headline: rgb(0xC9, 0xD1, 0xD9),
            focus: rgb(0x79, 0xB8, 0xFF),

            switch_on: rgb(0x58, 0xA6, 0xFF),
            switch_on_knob: rgb(0x05, 0x1D, 0x40),
            switch_off: rgb(0x30, 0x36, 0x3D),
            switch_knob: rgb(0x8B, 0x94, 0x9E),

            btn2_bg: rgb(0x21, 0x26, 0x2D),
            btn2_border: rgb(0x30, 0x36, 0x3D),
            btn2_text: rgb(0xC9, 0xD1, 0xD9),
            active_bg: rgb(0x1C, 0x2A, 0x3A),
            active_border: rgb(0x58, 0xA6, 0xFF),
            active_text: rgb(0xE6, 0xED, 0xF3),
            outline_bg: rgb(0x0D, 0x11, 0x17),
            outline_border: rgb(0x30, 0x36, 0x3D),
            outline_text: rgb(0xE6, 0xED, 0xF3),
            ghost_hover: color!(0xFFFFFF, 0.05),
            danger: rgb(0xF8, 0x51, 0x49),
            danger_text: Color::WHITE,

            row_zebra: color!(0xFFFFFF, 0.004),
            row_current_bg: rgb(0x1C, 0x2A, 0x3A),
            row_current_border: rgb(0x58, 0xA6, 0xFF),
            match_bg: rgb(0x16, 0x1B, 0x22),
            match_border: rgb(0x30, 0x36, 0x3D),
            selection: Color {
                a: 0.3,
                ..rgb(0x58, 0xA6, 0xFF)
            },

            tag_border: rgb(0x30, 0x36, 0x3D),
            tag_text: rgb(0x8B, 0x94, 0x9E),

            dot_green: rgb(0x3F, 0xB9, 0x50),
            led_connected: rgb(0x3F, 0xB9, 0x50),
            led_partial: rgb(0xD2, 0x99, 0x22),
            led_off: rgb(0x6E, 0x76, 0x80),
            star: rgb(0xE3, 0xB3, 0x41),

            toast_bg: rgb(0x16, 0x1B, 0x22),
            toast_border: rgb(0x30, 0x36, 0x3D),
            toast_text: rgb(0xE6, 0xED, 0xF3),

            modal_border: rgb(0x30, 0x36, 0x3D),
            backdrop: color!(0x05, 0x12, 0x26, 0.72),

            kbd_border: rgb(0x30, 0x36, 0x3D),
            kbd_bg: rgb(0x16, 0x1B, 0x22),
            kbd_text: rgb(0xC9, 0xD1, 0xD9),

            input_focus_bg: color!(0xFFFFFF, 0.04),
        }
    }

    /// The hand-tuned light palette (`Theme::Light`).
    pub fn light() -> Self {
        Self {
            is_light: true,
            bg: rgb(0xF6, 0xF8, 0xFA),
            panel: rgb(0xFF, 0xFF, 0xFF),
            panel_deep: rgb(0xEF, 0xF2, 0xF5),
            panel_head: rgb(0xF6, 0xF8, 0xFA),
            border: rgb(0xD0, 0xD7, 0xDE),
            border_soft: rgb(0xE4, 0xE9, 0xEF),

            text: rgb(0x1F, 0x23, 0x28),
            text_soft: rgb(0x57, 0x60, 0x6A),
            muted: rgb(0x6E, 0x77, 0x81),
            muted_head: rgb(0x81, 0x89, 0x93),
            faint: rgb(0x8C, 0x95, 0x9F),

            accent: rgb(0x09, 0x69, 0xDA),
            accent_border: rgb(0x54, 0xAE, 0xFF),
            accent_text: Color::WHITE,
            accent_soft: rgb(0x09, 0x69, 0xDA),
            accent_bar: rgb(0x54, 0xAE, 0xFF),
            accent_nav_bg: rgb(0xDD, 0xF2, 0xFF),
            accent_nav_text: rgb(0x09, 0x69, 0xDA),
            dance_headline: rgb(0x57, 0x60, 0x6A),
            focus: rgb(0x54, 0xAE, 0xFF),

            switch_on: rgb(0x09, 0x69, 0xDA),
            switch_on_knob: rgb(0x05, 0x1D, 0x40),
            switch_off: rgb(0xD0, 0xD7, 0xDE),
            switch_knob: rgb(0x81, 0x89, 0x93),

            btn2_bg: rgb(0xF6, 0xF8, 0xFA),
            btn2_border: rgb(0xD0, 0xD7, 0xDE),
            btn2_text: rgb(0x57, 0x60, 0x6A),
            active_bg: rgb(0xDD, 0xF2, 0xFF),
            active_border: rgb(0x09, 0x69, 0xDA),
            active_text: rgb(0x1F, 0x23, 0x28),
            outline_bg: rgb(0xFF, 0xFF, 0xFF),
            outline_border: rgb(0xD0, 0xD7, 0xDE),
            outline_text: rgb(0x1F, 0x23, 0x28),
            ghost_hover: color!(0x1F2328, 0.06),
            danger: rgb(0xCF, 0x22, 0x00),
            danger_text: Color::WHITE,

            row_zebra: color!(0x1F2328, 0.0),
            row_current_bg: rgb(0xDD, 0xF2, 0xFF),
            row_current_border: rgb(0x09, 0x69, 0xDA),
            match_bg: rgb(0xF6, 0xF8, 0xFA),
            match_border: rgb(0xD0, 0xD7, 0xDE),
            selection: Color {
                a: 0.3,
                ..rgb(0x09, 0x69, 0xDA)
            },

            tag_border: rgb(0xD0, 0xD7, 0xDE),
            tag_text: rgb(0x6E, 0x77, 0x81),

            dot_green: rgb(0x1A, 0x7F, 0x37),
            led_connected: rgb(0x1A, 0x7F, 0x37),
            led_partial: rgb(0x9A, 0x67, 0x07),
            led_off: rgb(0x8C, 0x95, 0x9F),
            star: rgb(0x9A, 0x67, 0x07),

            toast_bg: rgb(0xF6, 0xF8, 0xFA),
            toast_border: rgb(0xD0, 0xD7, 0xDE),
            toast_text: rgb(0x1F, 0x23, 0x28),

            modal_border: rgb(0xD0, 0xD7, 0xDE),
            backdrop: color!(0x1F2328, 0.45),

            kbd_border: rgb(0xD0, 0xD7, 0xDE),
            kbd_bg: rgb(0xF6, 0xF8, 0xFA),
            kbd_text: rgb(0x57, 0x60, 0x6A),

            input_focus_bg: color!(0x1F2328, 0.04),
        }
    }

    /// Derives every token from an iced [`Palette`] so that any theme picked
    /// in the settings dialog re-colors the whole UI.
    ///
    /// The derivation reuses the roles of the design: panels are the
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
            border: line(0.16),
            border_soft: line(0.11),

            text,
            text_soft: dim(0.12),
            muted: dim(0.32),
            muted_head: dim(0.4),
            faint: dim(0.5),

            accent,
            accent_border,
            accent_text: mix(accent, Color::BLACK, 0.85),
            accent_soft: if is_light { accent_deep(0.3) } else { accent },
            accent_bar: if is_light { accent_deep(0.3) } else { accent },
            accent_nav_bg: mix(bg, accent, 0.12),
            accent_nav_text: tint(if is_light { 0.08 } else { 0.55 }),
            dance_headline: tint(if is_light { 0.12 } else { 0.6 }),
            focus: accent_deep(0.15),

            switch_on: if is_light { accent_border } else { accent },
            switch_on_knob: mix(accent, Color::BLACK, 0.8),
            switch_off: line(0.18),
            switch_knob: mix(dim(0.42), accent, 0.4),

            btn2_bg: mix(bg, accent, 0.08),
            btn2_border: line(0.2),
            btn2_text: dim(if is_light { 0.12 } else { 0.18 }),
            active_bg: mix(bg, accent, 0.32),
            active_border: accent_deep(0.25),
            active_text: tint(if is_light { 0.04 } else { 0.42 }),
            outline_bg: panel,
            outline_border: line(0.18),
            outline_text: dim(0.08),
            ghost_hover: with_alpha(text, if is_light { 0.06 } else { 0.05 }),
            danger: palette.danger,
            danger_text: Color::WHITE,

            row_zebra: with_alpha(text, if is_light { 0.0 } else { 0.004 }),
            row_current_bg: mix(bg, accent, 0.2),
            row_current_border: mix(accent, bg, if is_light { 0.2 } else { 0.6 }),
            match_bg: mix(bg, accent, 0.11),
            match_border: if is_light {
                accent_deep(0.3)
            } else {
                accent_deep(0.2)
            },
            selection: with_alpha(accent, 0.3),

            tag_border: line(0.26),
            tag_text: dim(0.3),

            dot_green: palette.success,
            led_connected: mix(palette.success, anchor, 0.15),
            led_partial: palette.warning,
            led_off: line(0.3),
            star: palette.warning,

            toast_bg: mix(bg, accent, 0.18),
            toast_border: accent_deep(0.3),
            toast_text: tint(if is_light { 0.05 } else { 0.55 }),

            modal_border: mix(accent, bg, if is_light { 0.2 } else { 0.6 }),
            backdrop: with_alpha(
                mix(Color::BLACK, accent, 0.05),
                if is_light { 0.45 } else { 0.72 },
            ),

            kbd_border: line(0.19),
            kbd_bg: mix(bg, accent, 0.12),
            kbd_text: dim(0.18),

            input_focus_bg: with_alpha(text, 0.04),
        }
    }
}

fn luminance(c: Color) -> f32 {
    0.0722f32.mul_add(c.b, 0.7152f32.mul_add(c.g, 0.2126 * c.r))
}

/// Four-sided padding (iced only implements `From` for scalars and pairs).
pub const fn pad4(top: f32, right: f32, bottom: f32, left: f32) -> iced::Padding {
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

/// Flat page background (window base).
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
            radius: Radius::from(radius::XL),
        })
}

/// Theme-aware hairline border of the given colour and radius. Use this
/// instead of hand-rolling `container::Style::default().border(...)` in the
/// view layer so border widths and radii stay consistent.
pub fn bordered(
    theme: &Theme,
    color: fn(&Tokens) -> Color,
    radius: impl Into<Radius>,
) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default().border(Border {
        color: color(&t),
        width: 1.0,
        radius: radius.into(),
    })
}

/// Inset card: deep panel with a hairline border (search box shells, status
/// cards, source rows).
pub fn inset_card(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default()
        .background(t.panel_deep)
        .border(Border {
            color: t.border,
            width: 1.0,
            radius: Radius::from(radius::SM),
        })
}

/// Sidebar surface: page background with a hairline edge.
pub fn sidebar_bg(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default().background(t.bg).border(Border {
        color: t.border,
        width: 1.0,
        radius: Radius::default(),
    })
}

/// Strip below the show bar / above rows.
pub fn table_head(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default().background(t.panel_head)
}

/// Quick-show bar background (head panel above the rows).
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
}

impl RowLook {
    /// Standard table-row look: zebra striping, current highlight, and a
    /// search-match wash for non-current rows.
    pub const fn table(idx: usize, is_current: bool, is_match: bool) -> Self {
        Self {
            zebra: idx % 2 == 1,
            current: is_current,
            matched: is_match && !is_current,
        }
    }
}

/// Playlist/statics stripe style for the given [`RowLook`].
pub fn row(theme: &Theme, look: RowLook) -> container::Style {
    let t = Tokens::of(theme);

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
        radius: Radius::from(0.0),
    })
}

/// Modal dialog card: panel background with the modal border.
pub fn modal_card(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default()
        .background(t.panel)
        .border(Border {
            color: t.modal_border,
            width: 1.0,
            radius: Radius::from(radius::MODAL),
        })
}

/// Dimmed page backdrop behind modal dialogs.
pub fn backdrop(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default().background(t.backdrop)
}

/// Toast bubble card.
pub fn toast_card(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default()
        .background(t.toast_bg)
        .border(Border {
            color: t.toast_border,
            width: 1.0,
            radius: Radius::from(radius::LG),
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
                radius: Radius::from(radius::SM),
            })
    }
}

/// Collapsible side panel (info rail, DJ panel): rounded panel card that
/// paints nothing while collapsed (animated width ~0) so no border hairline
/// survives at the window edge. Callers needing a shadow add it to the
/// returned style.
pub fn panel_collapsible(theme: &Theme, width: f32) -> container::Style {
    if width < 1.0 {
        return container::Style::default();
    }
    let t = Tokens::of(theme);
    container::Style::default()
        .background(t.panel)
        .border(Border {
            color: t.border,
            width: 1.0,
            radius: Radius::from(radius::MODAL),
        })
}

/// Blends a user-picked color toward the theme text color when its contrast
/// against the background is too low. Dance colors are picked freely and are
/// often too dark (or too close to the background) to stay readable.
pub fn readable(color: Color, t: &Tokens) -> Color {
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

/// Keyboard hint pill.
pub fn kbd_style(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style {
        text_color: Some(t.kbd_text),
        ..container::Style::default()
    }
    .background(t.kbd_bg)
    .border(Border {
        color: t.kbd_border,
        width: 1.0,
        radius: Radius::from(radius::XS),
    })
}

// ---------------------------------------------------------------------------
// Button styles
// ---------------------------------------------------------------------------

/// Hover/press shift for a button background. Dark themes brighten the fill;
/// light themes darken it (brightening washes out near-white fills there).
/// `factor` is the brightness factor on dark themes (e.g. `1.06`).
fn shift(status: button::Status, is_light: bool, base: Color, factor: f32) -> Background {
    let tinted = if is_light {
        let k = 2.0 - factor;
        Color {
            a: base.a,
            r: base.r * k,
            g: base.g * k,
            b: base.b * k,
        }
    } else {
        Color {
            a: base.a,
            r: (base.r * factor).min(1.0),
            g: (base.g * factor).min(1.0),
            b: (base.b * factor).min(1.0),
        }
    };

    match status {
        button::Status::Hovered | button::Status::Pressed => Background::Color(tinted),
        button::Status::Disabled => Background::Color(Color { a: 0.4, ..base }),
        button::Status::Active => Background::Color(base),
    }
}

/// Filled primary button.
pub fn btn_primary(theme: &Theme, status: button::Status) -> button::Style {
    let t = Tokens::of(theme);
    button::Style {
        background: Some(shift(status, t.is_light, t.accent, 1.06)),
        text_color: t.accent_text,
        border: Border {
            color: t.accent_border,
            width: 1.0,
            radius: Radius::from(radius::MD),
        },
        ..button::Style::default()
    }
}

/// Soft secondary button (panel tint with border).
pub fn btn_secondary(theme: &Theme, status: button::Status) -> button::Style {
    let t = Tokens::of(theme);
    button::Style {
        background: Some(shift(status, t.is_light, t.btn2_bg, 1.08)),
        text_color: t.btn2_text,
        border: Border {
            color: t.btn2_border,
            width: 1.0,
            radius: Radius::from(radius::MD),
        },
        ..button::Style::default()
    }
}

/// Outline button used in page headings.
pub fn btn_outline(theme: &Theme, status: button::Status) -> button::Style {
    let t = Tokens::of(theme);
    button::Style {
        background: Some(shift(status, t.is_light, t.outline_bg, 1.04)),
        text_color: t.outline_text,
        border: Border {
            color: t.outline_border,
            width: 1.0,
            radius: Radius::from(radius::MD),
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
            radius: Radius::from(radius::MD),
        },
        ..button::Style::default()
    }
}

/// Filled red destructive button.
pub fn btn_danger(theme: &Theme, status: button::Status) -> button::Style {
    let t = Tokens::of(theme);
    button::Style {
        background: Some(shift(status, t.is_light, t.danger, 1.08)),
        text_color: t.danger_text,
        border: Border {
            color: t.danger,
            width: 1.0,
            radius: Radius::from(radius::MD),
        },
        ..button::Style::default()
    }
}

/// Active quick-show button (the currently displayed item).
pub fn btn_active(theme: &Theme, status: button::Status) -> button::Style {
    let t = Tokens::of(theme);
    button::Style {
        background: Some(shift(status, t.is_light, t.active_bg, 1.06)),
        text_color: t.active_text,
        border: Border {
            color: t.active_border,
            width: 1.0,
            radius: Radius::from(radius::MD),
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
            background: Some(shift(status, t.is_light, base, 1.08)),
            text_color: if selected { t.accent_nav_text } else { t.muted },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(radius::MD),
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
            radius: Radius::from(radius::XS),
        },
        icon: t.muted,
        placeholder: t.faint,
        value: match status {
            text_input::Status::Focused { .. } => t.text,
            _ => t.text_soft,
        },
        selection: t.selection,
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
        text_input::Status::Focused { .. } => Background::Color(t.input_focus_bg),
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
            radius: Radius::from(radius::SM),
        },
        icon: t.muted,
        placeholder: t.faint,
        value: t.text,
        selection: t.selection,
    }
}

// ---------------------------------------------------------------------------
// Misc widget styles
// ---------------------------------------------------------------------------

/// Hairline track with a rounded soft thumb.
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
            radius: Radius::from(radius::XS),
        },
        scroller: scrollable::Scroller {
            background: Background::Color(if active { t.accent_border } else { t.border }),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(radius::XS),
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
                radius: Radius::from(radius::PILL),
            },
            shadow: iced::Shadow::default(),
            icon: t.text,
        },
    }
}

/// Boxed pick list, mirroring [`input_boxed`].
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
            radius: Radius::from(radius::SM),
        },
    }
}

/// Dropdown menu shared by pick lists and suggestion inputs.
pub fn pick_list_menu_style(theme: &Theme) -> iced::widget::overlay::menu::Style {
    let t = Tokens::of(theme);
    iced::widget::overlay::menu::Style {
        background: Background::Color(t.panel),
        border: Border {
            color: t.border,
            width: 1.0,
            radius: Radius::from(radius::LG),
        },
        text_color: t.text_soft,
        selected_text_color: t.accent_nav_text,
        selected_background: Background::Color(t.accent_nav_bg),
        shadow: iced::Shadow::default(),
    }
}

/// Toggler (switch) with the blue "on" state.
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

/// Tooltip card: panel with border and drop shadow.
pub fn tooltip_style(theme: &Theme) -> container::Style {
    let t = Tokens::of(theme);
    container::Style::default()
        .background(t.panel)
        .border(Border {
            color: t.border,
            width: 1.0,
            radius: Radius::from(radius::SM),
        })
}
