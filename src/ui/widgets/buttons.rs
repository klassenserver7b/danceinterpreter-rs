//! Button builders for the redesigned UI.
//!
//! All styles come from [`crate::ui::theme`]; the builders here only exist to
//! avoid repeating padding / alignment / icon boilerplate.

use crate::Message;
use crate::ui::theme::{self, sz};
use crate::ui::widgets::material_symbol_sized;
use iced::alignment::Vertical;
use iced::widget::text::IntoFragment;
use iced::widget::{Button, button, text};
use iced::{Length, Pixels, Renderer, Theme};

fn label_text<'a>(label: impl IntoFragment<'a>) -> iced::widget::Text<'a, Theme, Renderer> {
    text(label).size(sz::BASE).align_y(Vertical::Center)
}

/// Filled sage-green primary button.
pub fn primary<'a>(
    label: impl IntoFragment<'a> + 'a,
    message: Message,
) -> Button<'a, Message> {
    button(label_text(label))
        .padding([8, 13])
        .style(theme::btn_primary)
        .on_press(message)
}

/// Small filled primary button (table toolbars).
pub fn primary_small<'a>(
    label: impl IntoFragment<'a> + 'a,
    message: Message,
) -> Button<'a, Message> {
    button(label_text(label))
        .padding([5, 10])
        .style(theme::btn_primary)
        .on_press(message)
}

/// Dark outlined secondary button.
pub fn secondary<'a>(
    label: impl IntoFragment<'a> + 'a,
    message: Message,
) -> Button<'a, Message> {
    button(label_text(label))
        .padding([8, 13])
        .style(theme::btn_secondary)
        .on_press(message)
}

/// Outline button used in page headings.
#[allow(dead_code)] // design-system toolkit
pub fn outline<'a>(
    label: impl IntoFragment<'a> + 'a,
    message: Message,
) -> Button<'a, Message> {
    button(label_text(label))
        .padding([8, 13])
        .style(theme::btn_outline)
        .on_press(message)
}

pub fn danger<'a>(label: impl IntoFragment<'a> + 'a, message: Message) -> Button<'a, Message> {
    button(label_text(label))
        .padding([8, 13])
        .style(theme::btn_danger)
        .on_press(message)
}

/// Icon-only button with the ghost style (transparent, hover wash).
pub fn ghost_icon<'a>(
    icon_id: &'a str,
    filled: bool,
    size: impl Into<Pixels>,
    message: Message,
) -> Button<'a, Message> {
    button(material_symbol_sized(icon_id, filled, size))
        .padding(5)
        .style(theme::btn_ghost)
        .on_press(message)
        .width(Length::Shrink)
}

/// Ghost icon button that stays disabled (no message) when `message` is None.
#[allow(dead_code)] // design-system toolkit
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
