//! Button builders for the redesigned UI.
//!
//! All styles come from [`crate::ui::theme`]; the builders here only exist to
//! avoid repeating padding / alignment / icon boilerplate.

use crate::Message;
use crate::ui::theme::{self, sz};
use crate::ui::widgets::material_symbol_sized;
use iced::alignment::Vertical;
use iced::widget::text::IntoFragment;
use iced::widget::{Button, button, row, text};
use iced::{Length, Pixels, Renderer, Theme};

fn label_text<'a>(label: impl IntoFragment<'a>) -> iced::widget::Text<'a, Theme, Renderer> {
    text(label).size(sz::BASE).align_y(Vertical::Center)
}

/// Shared body of the labelled button builders.
fn labelled<'a>(
    label: impl IntoFragment<'a> + 'a,
    message: Message,
    style: fn(&Theme, button::Status) -> button::Style,
) -> Button<'a, Message> {
    button(label_text(label))
        .padding([8, 13])
        .style(style)
        .on_press(message)
}

/// Filled sage-green primary button.
pub fn primary<'a>(label: impl IntoFragment<'a> + 'a, message: Message) -> Button<'a, Message> {
    labelled(label, message, theme::btn_primary)
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
pub fn secondary<'a>(label: impl IntoFragment<'a> + 'a, message: Message) -> Button<'a, Message> {
    labelled(label, message, theme::btn_secondary)
}

/// Filled red destructive button.
pub fn danger<'a>(label: impl IntoFragment<'a> + 'a, message: Message) -> Button<'a, Message> {
    labelled(label, message, theme::btn_danger)
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

/// Ghost button with a leading icon and a small label (top bar actions),
/// optionally with a small trailing icon.
pub fn ghost_labelled<'a>(
    icon_id: &'a str,
    label: impl IntoFragment<'a> + 'a,
    trailing_icon: Option<&'a str>,
    message: Message,
) -> Button<'a, Message> {
    let mut content = row![
        material_symbol_sized(icon_id, false, sz::ICON),
        text(label).size(sz::SMALL).align_y(Vertical::Center),
    ]
    .spacing(8)
    .align_y(Vertical::Center);

    if let Some(trailing_icon) = trailing_icon {
        content = content.push(material_symbol_sized(trailing_icon, false, sz::ICON_XXS));
    }

    button(content)
        .padding([6, 8])
        .style(theme::btn_ghost)
        .on_press(message)
        .width(Length::Shrink)
}
