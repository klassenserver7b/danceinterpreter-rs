use crate::Message;
use crate::ui::widgets::{material_symbol, material_symbol_colored, material_symbol_sized};
use iced::alignment::Vertical;
use iced::widget::{Button, button, row, text};
use iced::{Alignment, Color, Length, Pixels, Renderer, Theme};
use iced_aw::iced_aw_font;

pub fn label_message_button<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
) -> Button<'a, Message> {
    button(text(label).align_y(Vertical::Center))
        .padding([4, 8])
        .style(button::secondary)
        .on_press(message)
}

pub fn label_message_button_fill<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
) -> Button<'a, Message> {
    label_message_button(label, message).width(Length::Fill)
}

pub fn label_message_button_shrink<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
) -> Button<'a, Message> {
    label_message_button(label, message).width(Length::Shrink)
}

fn label_message_button_opt(label: &'_ str, message: Option<Message>) -> Button<'_, Message> {
    if let Some(message) = message {
        label_message_button(label, message)
    } else {
        button(text(label).align_y(Vertical::Center))
            .padding([4, 8])
            .style(button::primary)
    }
}

pub fn label_message_button_fill_opt(
    label: &'_ str,
    message: Option<Message>,
) -> Button<'_, Message> {
    label_message_button_opt(label, message).width(Length::Fill)
}

pub fn material_symbol_message_button_colored(
    icon_id: &'_ str,
    filled: bool,
    message: Message,
    color: impl Into<Color>,
) -> Button<'_, Message> {
    button(material_symbol_colored(icon_id, filled, color))
        //.padding([4, 8])
        .style(button::secondary)
        .on_press(message)
        .width(Length::Shrink)
}

pub fn material_symbol_message_button(
    icon_id: &'_ str,
    filled: bool,
    message: Message,
) -> Button<'_, Message> {
    button(material_symbol(icon_id, filled))
        //.padding([4, 8])
        .style(button::secondary)
        .on_press(message)
        .width(Length::Shrink)
}

pub fn material_symbol_sized_message_button(
    icon_id: &'_ str,
    filled: bool,
    size: impl Into<Pixels>,
    message: Message,
) -> Button<'_, Message> {
    button(material_symbol_sized(icon_id, filled, size))
        .style(button::secondary)
        .on_press(message)
        .width(Length::Shrink)
}

#[allow(dead_code)]
pub fn submenu_button(label: &'_ str) -> Button<'_, Message, Theme, Renderer> {
    button(
        row![
            text(label).width(Length::Fill).align_y(Vertical::Center),
            iced_aw_font::right_open()
                .width(Length::Shrink)
                .align_y(Vertical::Center),
        ]
        .align_y(Alignment::Center),
    )
    .padding([4, 8])
    .style(button::text)
    .on_press(Message::Noop)
    .width(Length::Fill)
}
