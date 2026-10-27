use crate::Message;
use crate::ui::theme;
use iced::advanced::text::Shaping;
use iced::widget::{Text, text, toggler};
use iced::{Color, Font, Length, Pixels, Renderer, Theme};

pub mod buttons;
pub mod color_swatch;
pub mod parity;

/// Small labeled toggler (switch) styled with the design tokens.
pub fn labeled_message_toggler(
    label: &'_ str,
    checked: bool,
    message: fn(bool) -> Message,
) -> toggler::Toggler<'_, Message> {
    toggler(checked)
        .label(label)
        .on_toggle(message)
        .width(Length::Fill)
        .text_size(theme::sz::SMALL)
        .style(theme::toggler_style)
}

pub fn material_symbol_colored(
    id: &'_ str,
    filled: bool,
    color: impl Into<Color>,
) -> Text<'_, Theme, Renderer> {
    material_symbol(id, filled).color(color)
}

pub fn material_symbol_sized(
    id: &'_ str,
    filled: bool,
    size: impl Into<Pixels>,
) -> Text<'_, Theme, Renderer> {
    material_symbol(id, filled).size(size)
}

pub fn material_symbol(id: &'_ str, filled: bool) -> Text<'_, Theme, Renderer> {
    let font_name = if filled {
        theme::FONT_ICON_FILLED
    } else {
        theme::FONT_ICON
    };

    Text::new(id)
        .font(Font::with_name(font_name))
        .shaping(Shaping::Advanced)
        .width(Length::Shrink)
}

/// Keyboard hint pill, e.g. `Space`, `Ctrl+F`.
pub fn kbd<'a>(
    label: impl text::IntoFragment<'a>,
) -> iced::widget::Container<'a, Message, Theme, Renderer> {
    iced::widget::container(
        text(label)
            .size(theme::sz::MICRO)
            .style(theme::text_style(|tk| tk.kbd_text)),
    )
    .padding([1, 5])
    .style(theme::kbd)
}

/// Body text at the micro size (tags, eyebrows, hints).
pub fn text_micro<'a>(label: impl text::IntoFragment<'a>) -> Text<'a, Theme, Renderer> {
    text(label).size(theme::sz::MICRO)
}

/// Manrope heading text.
pub fn heading<'a>(
    label: impl text::IntoFragment<'a>,
    size: impl Into<Pixels>,
    weight: iced::font::Weight,
) -> Text<'a, Theme, Renderer> {
    text(label).size(size).font(theme::head_font(weight))
}
