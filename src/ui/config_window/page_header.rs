//! Page heading: eyebrow, title, subtitle and the "Open playlist" action.

use crate::ui::config_window::{ConfigWindow, View};
use crate::ui::theme::{self, sz};
use crate::ui::widgets::{heading, material_symbol_sized};
use crate::{DanceInterpreter, Message};
use iced::alignment::Vertical;
use iced::widget::{column as col, container, row, text};
use iced::{Alignment, Element, Length, Theme};

pub fn build<'a>(
    config_window: &'a ConfigWindow,
    _dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    let (title, subtitle) = match config_window.view {
        View::Playlist => (
            "The dance floor is yours.",
            "A little preparation. A seamless night of dancing.",
        ),
        View::Statics => (
            "A moment. A message.",
            "Your favorite dances and announcements, ready in one click.",
        ),
    };

    let heading_block = col![
        text("YOUR EVENING, IN RHYTHM")
            .size(sz::MICRO)
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).muted),
            }),
        heading(title, sz::H1, iced::font::Weight::Semibold).style(|t: &Theme| {
            iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).text),
            }
        }),
        text(subtitle)
            .size(sz::SMALL)
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).muted),
            }),
    ]
    .spacing(6);

    let open_button = iced::widget::button(
        row![
            material_symbol_sized("folder", false, 16.0),
            text("Open playlist").size(sz::BASE),
        ]
        .spacing(8)
        .align_y(Vertical::Center),
    )
    .padding([8, 13])
    .style(theme::btn_outline)
    .on_press(Message::OpenPlaylist);

    container(
        row![heading_block, container(open_button).align_x(Alignment::End)]
            .width(Length::Fill)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .into()
}
