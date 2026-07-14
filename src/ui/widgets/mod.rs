use crate::Message;
use crate::ui::widget::dynamic_text_input::DynamicTextInput;
use iced::advanced::text::Shaping;
use iced::widget::{Column, Text, checkbox, column as col, radio, text, toggler};
use iced::{Color, Font, Length, Pixels, Renderer, Theme};

pub mod buttons;
pub mod color_swatch;
pub fn labeled_message_checkbox(
    label: &'_ str,
    checked: bool,
    message: fn(bool) -> Message,
) -> checkbox::Checkbox<'_, Message> {
    checkbox(checked)
        .label(label)
        .on_toggle(message)
        .width(Length::Fill)
    //.style(checkbox::secondary)
}

pub fn labeled_message_toggler(
    label: &'_ str,
    checked: bool,
    message: fn(bool) -> Message,
) -> toggler::Toggler<'_, Message> {
    toggler(checked)
        .label(label)
        .on_toggle(message)
        .width(Length::Fill)
}

#[allow(dead_code)]
pub fn labeled_message_radio<T: Copy + Eq>(
    label: &'_ str,
    value: T,
    selection: T,
    message: fn(T) -> Message,
) -> radio::Radio<'_, Message> {
    radio(label, value, Some(selection), message).width(Length::Fill)
    //.style(checkbox::secondary)
}

#[allow(dead_code)]
pub fn labeled_message_checkbox_opt(
    label: &'_ str,
    checked: bool,
    message: Option<fn(bool) -> Message>,
) -> checkbox::Checkbox<'_, Message> {
    if let Some(message) = message {
        labeled_message_checkbox(label, checked, message)
    } else {
        checkbox(checked).label(label).width(Length::Fill)
        //.style(checkbox::secondary)
    }
}

#[allow(dead_code)]
pub fn labeled_dynamic_text_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    message: fn(String) -> Message,
    submit_message: Option<Message>,
) -> Column<'a, Message> {
    let mut input = DynamicTextInput::<Message>::new(placeholder, value)
        .width(Length::Fill)
        .on_change(message);

    if let Some(submit_message) = submit_message {
        input = input.on_submit(submit_message);
    }

    col!(text(label).width(Length::Fill), input,).width(Length::Fill)
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
        "Material Icons"
    } else {
        "Material Symbols Outlined"
    };

    Text::new(id)
        .font(Font::with_name(font_name))
        .shaping(Shaping::Advanced)
        .width(Length::Shrink)
}
