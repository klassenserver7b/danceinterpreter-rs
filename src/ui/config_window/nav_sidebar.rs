//! Left navigation sidebar: brand, session card, view navigation and the
//! settings / help entries.

use crate::ui::config_window::{ConfigWindow, DialogState, View};
use crate::ui::theme::{self, pad4, sz};
use crate::ui::widgets::{kbd, material_symbol};
use crate::{DanceInterpreter, Message};
use iced::widget::{column as col, container, row, text};
use iced::{Alignment, Color, Element, Length, Theme};

/// Full width of the navigation sidebar when expanded.
pub const SIDEBAR_WIDTH: f32 = 218.0;

// Geometry (design px).
const NAV_ITEM_HEIGHT: f32 = 43.0;
const VERSION_ROW_HEIGHT: f32 = 48.0;

pub fn build<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
    width: f32,
) -> Element<'a, Message> {
    let playlist_count = dance_interpreter.data_provider.playlist().len();
    let statics_count = dance_interpreter.data_provider.statics().len();

    let nav = col![
        brand(),
        session_card(config_window),
        // Breather between the session card and the nav items.
        iced::widget::Space::new().height(Length::Fixed(10.0)),
        nav_item(
            "format_list_bulleted",
            "Playlist",
            Some(count_badge(playlist_count)),
            config_window.view == View::Playlist,
            Message::SetView(View::Playlist),
        ),
        nav_item(
            "grid_view",
            "Statics",
            Some(count_badge(statics_count)),
            config_window.view == View::Statics,
            Message::SetView(View::Statics),
        ),
    ]
    .spacing(3);

    let bottom = col![
        nav_item(
            "settings",
            "Settings",
            None,
            false,
            Message::OpenDialog(DialogState::Settings),
        ),
        nav_item(
            "help",
            "Help & shortcuts",
            Some(kbd("?", theme::kbd).into()),
            false,
            Message::OpenDialog(DialogState::Help),
        ),
        version_row(),
    ]
    .spacing(3);

    container(
        col![
            nav,
            container(bottom)
                .padding(pad4(20.0, 0.0, 14.0, 0.0))
                .height(Length::Fill)
        ]
        .height(Length::Fill),
    )
    .width(Length::Fixed(width))
    .height(Length::Fill)
    .clip(true)
    .padding(pad4(29.0, 17.0, 0.0, 17.0))
    .style(theme::sidebar_bg)
    .into()
}

fn brand() -> Element<'static, Message> {
    let bars = [14.0, 25.0, 20.0, 10.0]
        .into_iter()
        .map(|h| {
            container(iced::widget::Space::new())
                .width(Length::Fixed(3.0))
                .height(Length::Fixed(h))
                .style(|t: &Theme| {
                    container::Style::default().background(theme::Tokens::of(t).accent_soft)
                })
                .into()
        })
        .collect::<Vec<Element<'static, Message, Theme>>>();

    let symbol = row(bars)
        .spacing(3)
        .height(Length::Fixed(25.0))
        .align_y(Alignment::Center);

    let name = row![
        text("dance")
            .size(14.0)
            .font(theme::body_font(iced::font::Weight::Bold)),
        text("interpreter")
            .size(14.0)
            .font(theme::body_font(iced::font::Weight::Normal)),
    ]
    .spacing(0);

    container(row![symbol, name].spacing(9).align_y(Alignment::Center))
        .padding(pad4(0.0, 3.0, 33.0, 3.0))
        .into()
}

fn session_card<'a>(config_window: &'a ConfigWindow) -> Element<'a, Message> {
    let (title, subtitle) = match &config_window.playlist_file {
        Some(path) => (
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Playlist loaded".to_string()),
            "Event playlist".to_string(),
        ),
        None => (
            "No playlist loaded".to_string(),
            "Drop or open an .m3u".to_string(),
        ),
    };

    let icon_box = container(
        material_symbol("graphic_eq", false)
            .size(15.0)
            .style(theme::text_style(|tk| tk.accent_nav_text)),
    )
    .width(Length::Fixed(30.0))
    .height(Length::Fixed(32.0))
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(|t: &Theme| {
        let tk = theme::Tokens::of(t);
        container::Style::default()
            .background(tk.accent_nav_bg)
            .border(iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: theme::r::MD.into(),
            })
    });

    container(
        row![
            icon_box,
            col![
                text(title)
                    .size(sz::SMALL)
                    .font(theme::body_font(iced::font::Weight::Semibold))
                    .style(theme::text_style(|tk| tk.text)),
                text(subtitle)
                    .size(sz::MICRO)
                    .style(theme::text_style(|tk| tk.muted)),
            ]
            .spacing(3),
        ]
        .spacing(9)
        .align_y(Alignment::Center),
    )
    .padding(12)
    .style(|t: &Theme| {
        let tk = theme::Tokens::of(t);
        container::Style::default().border(iced::Border {
            color: tk.border,
            width: 1.0,
            radius: theme::r::XL.into(),
        })
    })
    .width(Length::Fill)
    .into()
}

fn count_badge(count: usize) -> Element<'static, Message> {
    container(
        text(count.to_string())
            .size(sz::MICRO)
            .style(theme::text_style(|tk| tk.muted)),
    )
    .padding([1, 6])
    .style(|t: &Theme| {
        let tk = theme::Tokens::of(t);
        container::Style::default()
            .background(tk.kbd_bg)
            .border(iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: theme::r::XS.into(),
            })
    })
    .into()
}

fn nav_item<'a>(
    icon: &'static str,
    label: &'a str,
    trailing: Option<Element<'a, Message>>,
    selected: bool,
    message: Message,
) -> Element<'a, Message> {
    let icon_color: fn(&theme::Tokens) -> Color = if selected {
        |tk| tk.accent_nav_text
    } else {
        |tk| tk.muted
    };

    let content = row![
        material_symbol(icon, false).style(theme::text_style(icon_color)),
        text(label)
            .size(sz::BASE)
            .style(theme::text_style(icon_color)),
    ]
    .spacing(12)
    .height(Length::Fill)
    .align_y(Alignment::Center);
    let content = match trailing {
        Some(trailing) => content.push(trailing),
        None => content,
    };

    iced::widget::button(content)
        .width(Length::Fill)
        .height(Length::Fixed(NAV_ITEM_HEIGHT))
        .padding([0, 12])
        .style(theme::btn_nav(selected))
        .on_press(message)
        .into()
}

fn version_row() -> Element<'static, Message> {
    container(
        row![
            text("di")
                .size(sz::ICON)
                .font(theme::body_font(iced::font::Weight::Bold))
                .style(theme::text_style(|tk| tk.accent_soft)),
            text("Danceinterpreter")
                .size(sz::MICRO)
                .style(theme::text_style(|tk| tk.faint)),
            container(
                text(concat!("v", env!("CARGO_PKG_VERSION")))
                    .size(sz::MICRO)
                    .style(theme::text_style(|tk| tk.faint))
            )
            .width(Length::Fill)
            .align_x(Alignment::End),
        ]
        .spacing(7)
        .align_y(Alignment::Center),
    )
    .padding(pad4(0.0, 0.0, 0.0, 3.0))
    .height(Length::Fixed(VERSION_ROW_HEIGHT))
    .align_y(Alignment::Center)
    .into()
}
