//! Page heading: eyebrow, title, subtitle and the "Open playlist" action.

use crate::ui::config_window::{ConfigWindow, View};
use crate::ui::theme::{self, sz};
use crate::ui::widgets::{heading, material_symbol_sized};
use crate::{DanceInterpreter, Message};
use iced::alignment::Vertical;
use iced::widget::{container, row, text};
use iced::{Alignment, Element, Length, Theme};

pub fn build<'a>(
    config_window: &'a ConfigWindow,
    _dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    let title = match config_window.view {
        View::Playlist => "The dance floor is yours.",
        View::Statics => "A moment. A message.",
    };

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
        row![
            heading(title, sz::H1, iced::font::Weight::Semibold).style(|t: &Theme| {
                iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).text),
                }
            }),
            container(open_button)
                .width(Length::Fill)
                .align_x(Alignment::End),
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .into()
}
