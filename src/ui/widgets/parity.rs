//! Prototype parity toolkit.
//!
//! Items ported 1:1 from the approved TSX prototype (`docs/src/App.tsx`) that
//! the redesigned UI does not call yet. They are kept deliberately so future
//! screens can adopt them without re-deriving styles from the prototype; this
//! single module is the one place where unused toolkit items are allowed.

#![allow(dead_code)]

use crate::Message;
use crate::ui::theme::{self, Tokens};
use crate::ui::widgets::material_symbol_sized;
use iced::border::Radius;
use iced::font::{Family, Style as FontStyle, Weight};
use iced::widget::button;
use iced::widget::{Button, checkbox, text};
use iced::{Background, Border, Color, Font, Length, Pixels, Renderer, Theme};

// ---------------------------------------------------------------------------
// Fonts
// ---------------------------------------------------------------------------

/// Outlined Material Symbols font.
pub fn icon_font() -> Font {
    Font {
        family: Family::Name(theme::FONT_ICON),
        weight: Weight::Normal,
        style: FontStyle::Normal,
        stretch: iced::font::Stretch::Normal,
    }
}

/// Filled Material Icons font.
pub fn icon_font_filled() -> Font {
    Font {
        family: Family::Name(theme::FONT_ICON_FILLED),
        weight: Weight::Normal,
        style: FontStyle::Normal,
        stretch: iced::font::Stretch::Normal,
    }
}

// ---------------------------------------------------------------------------
// Text helpers
// ---------------------------------------------------------------------------

/// Body text at the small caption size.
pub fn text_small<'a>(
    label: impl text::IntoFragment<'a>,
) -> iced::widget::Text<'a, Theme, Renderer> {
    text(label).size(theme::sz::SMALL)
}

/// Body text at the base size.
pub fn text_base<'a>(
    label: impl text::IntoFragment<'a>,
) -> iced::widget::Text<'a, Theme, Renderer> {
    text(label).size(theme::sz::BASE)
}

// ---------------------------------------------------------------------------
// Selection widgets
// ---------------------------------------------------------------------------

/// Small labeled checkbox styled with the design tokens.
pub fn labeled_message_checkbox(
    label: &'_ str,
    checked: bool,
    message: fn(bool) -> Message,
) -> checkbox::Checkbox<'_, Message> {
    checkbox(checked)
        .label(label)
        .on_toggle(message)
        .width(Length::Fill)
        .style(checkbox_style)
}

/// Small labeled radio button.
pub fn labeled_message_radio<T: Copy + Eq>(
    label: &'_ str,
    value: T,
    selection: T,
    message: fn(T) -> Message,
) -> iced::widget::Radio<'_, Message> {
    iced::widget::radio(label, value, Some(selection), message).width(Length::Fill)
}

/// Checkbox style matching the toggler colors.
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
            radius: Radius::from(theme::r::SM),
        },
        text_color: Some(t.text_soft),
    }
}

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------

/// Colored icon at an explicit size.
pub fn material_symbol_sized_colored(
    id: &'_ str,
    filled: bool,
    size: impl Into<Pixels>,
    color: impl Into<Color>,
) -> iced::widget::Text<'_, Theme, Renderer> {
    crate::ui::widgets::material_symbol(id, filled)
        .size(size)
        .color(color)
}

/// Outline button used in page headings.
pub fn outline<'a>(
    label: impl text::IntoFragment<'a> + 'a,
    message: Message,
) -> Button<'a, Message> {
    super::buttons::labelled(label, message, theme::btn_outline)
}

/// Ghost icon button that stays disabled (no message) when `message` is None.
pub fn ghost_icon_opt<'a>(
    icon_id: &'a str,
    filled: bool,
    size: impl Into<Pixels>,
    message: Option<Message>,
) -> Button<'a, Message> {
    let btn = button(material_symbol_sized(icon_id, filled, size))
        .padding(5)
        .style(theme::btn_ghost)
        .width(Length::Shrink);

    if let Some(message) = message {
        btn.on_press(message)
    } else {
        btn
    }
}
