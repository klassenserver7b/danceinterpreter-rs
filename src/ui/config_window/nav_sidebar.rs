//! Left navigation sidebar: brand, session card, and the view navigation /
//! settings / help entries.

use crate::res_file;
use crate::ui::config_window::{ConfigWindow, DialogState, View};
use crate::ui::theme::{self, pad4, sz};
use crate::ui::widgets::{EllipsisText, kbd, material_symbol};
use crate::{DanceInterpreter, Message};
use iced::widget::{column as col, container, row, svg, text};
use iced::{Alignment, Color, Element, Length, Theme};

/// Full width of the navigation sidebar when expanded.
pub const SIDEBAR_WIDTH: f32 = 218.0;

// Geometry (design px).
const NAV_ITEM_HEIGHT: f32 = 43.0;

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
            Some(kbd("?", theme::kbd_style).into()),
            false,
            Message::OpenDialog(DialogState::Help),
        ),
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
    // Monochrome app icon, tinted via the svg color filter.
    let symbol = svg(svg::Handle::from_memory(include_bytes!(res_file!(
        "de.klassenserver7b.danceinterpreter-rs.svg"
    ))))
    .width(Length::Fixed(25.0))
    .height(Length::Fixed(25.0))
    .style(|t: &Theme, _| svg::Style {
        color: Some(theme::Tokens::of(t).accent_soft),
    });

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

    container(
        col![
            EllipsisText::new(title)
                .size(sz::SMALL)
                .font(theme::body_font(iced::font::Weight::Semibold))
                .style(theme::text_style(|tk| tk.text)),
            text(subtitle)
                .size(sz::MICRO)
                .style(theme::text_style(|tk| tk.muted)),
        ]
        .spacing(3)
        .width(Length::Fill),
    )
    .padding([10, 12])
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
