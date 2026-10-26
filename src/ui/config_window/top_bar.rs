//! Top bar: breadcrumb and the "open song window" action.

use crate::ui::config_window::{ConfigWindow, View};
use crate::ui::theme::{self, sz};
use crate::ui::widgets::material_symbol_sized;
use crate::{DanceInterpreter, Message};
use iced::widget::{Column, button, column as col, container, row, text};
use iced::{Alignment, Element, Length, Theme};

pub fn build<'a>(
    config_window: &'a ConfigWindow,
    _dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    let view_name = match config_window.view {
        View::Playlist => "Playlist",
        View::Statics => "Statics",
    };

    let breadcrumb = row![
        text("Workspace")
            .size(sz::SMALL)
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).muted),
            }),
        material_symbol_sized("chevron_right", false, 13.0).style(|t: &Theme| {
            iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).faint),
            }
        }),
        text(view_name)
            .size(sz::SMALL)
            .font(theme::body_font(iced::font::Weight::Medium))
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).text_soft),
            }),
    ]
    .spacing(13)
    .align_y(Alignment::Center);

    let open_song_window = button(
        row![
            material_symbol_sized("desktop_windows", false, 16.0),
            text("Open song window").size(sz::SMALL),
            material_symbol_sized("open_in_new", false, 13.0),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([6, 8])
    .style(theme::btn_ghost)
    .on_press(Message::FocusSongWindow);

    container(
        row![
            breadcrumb,
            container(open_song_window)
                .width(Length::Fill)
                .align_x(Alignment::End),
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center),
    )
    .padding([0, 31])
    .height(Length::Fixed(65.0))
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .style(theme::page_bg)
    .into()
}

/// A 1px horizontal divider spanning the full width.
pub fn divider() -> Element<'static, Message> {
    container(iced::widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(|t: &Theme| {
            container::Style::default().background(theme::Tokens::of(t).border)
        })
        .into()
}

/// Vertical stack helper used by the shell (bar + hairline).
pub fn bar_with_divider<'a>(
    bar: Element<'a, Message>,
) -> Column<'a, Message, Theme, iced::Renderer> {
    col![bar, divider()]
}
