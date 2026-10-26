//! Left navigation sidebar: brand, session card, view navigation and the
//! settings / help entries.

use crate::ui::config_window::{ConfigWindow, DialogState, View};
use crate::ui::theme::{self, sz};
use crate::ui::widgets::{material_symbol, material_symbol_sized};
use crate::{DanceInterpreter, Message};
use iced::widget::{column as col, container, row, text};
use iced::{Alignment, Color, Element, Length, Theme};

/// Full width of the navigation sidebar when expanded.
pub const SIDEBAR_WIDTH: f32 = 218.0;

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
        // Breather between the session card and the nav items (the old
        // "WORKSPACE" section label is gone — it was unclickable filler).
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
        nav_item(
            "desktop_windows",
            "Song window",
            Some(
                material_symbol_sized("open_in_new", false, 13.0)
                    .style(|t: &Theme| iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).faint),
                    })
                    .into(),
            ),
            false,
            Message::FocusSongWindow,
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
        nav_item_with_kbd(
            "help",
            "Help & shortcuts",
            crate::ui::widgets::kbd("?").into(),
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
                .padding(crate::ui::theme::pad4(20.0, 0.0, 14.0, 0.0))
                .height(Length::Fill)
        ]
        .height(Length::Fill),
    )
    .width(Length::Fixed(width))
    .height(Length::Fill)
    .clip(true)
    .padding(crate::ui::theme::pad4(29.0, 17.0, 0.0, 17.0))
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
        .padding(crate::ui::theme::pad4(0.0, 3.0, 33.0, 3.0))
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
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).accent_nav_text),
            }),
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
                radius: 5.0.into(),
            })
    });

    container(
        row![
            icon_box,
            col![
                text(title)
                    .size(sz::SMALL)
                    .font(theme::body_font(iced::font::Weight::Semibold))
                    .style(|t: &Theme| iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).text),
                    }),
                text(subtitle)
                    .size(sz::MICRO)
                    .style(|t: &Theme| iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).muted),
                    }),
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
            radius: 7.0.into(),
        })
    })
    .width(Length::Fill)
    .into()
}

fn count_badge(count: usize) -> Element<'static, Message> {
    container(text(count.to_string()).size(sz::MICRO).style(|t: &Theme| {
        iced::widget::text::Style {
            color: Some(theme::Tokens::of(t).muted),
        }
    }))
    .padding([1, 6])
    .style(|t: &Theme| {
        let tk = theme::Tokens::of(t);
        container::Style::default()
            .background(tk.kbd_bg)
            .border(iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 3.0.into(),
            })
    })
    .into()
}

fn nav_item(
    icon: &'static str,
    label: &'static str,
    trailing: Option<Element<'static, Message>>,
    selected: bool,
    message: Message,
) -> Element<'static, Message> {
    nav_item_impl(icon, label, trailing, None, selected, message)
}

fn nav_item_with_kbd(
    icon: &'static str,
    label: &'static str,
    kbd: Element<'static, Message>,
    selected: bool,
    message: Message,
) -> Element<'static, Message> {
    nav_item_impl(icon, label, None, Some(kbd), selected, message)
}

fn nav_item_impl<'a>(
    icon: &'static str,
    label: &'a str,
    trailing: Option<Element<'a, Message>>,
    kbd: Option<Element<'a, Message>>,
    selected: bool,
    message: Message,
) -> Element<'a, Message> {
    let icon_el: Element<'a, Message> = material_symbol(icon, false)
        .style(move |t: &Theme| iced::widget::text::Style {
            color: Some(if selected {
                theme::Tokens::of(t).accent_nav_text
            } else {
                theme::Tokens::of(t).muted
            }),
        })
        .into();

    let label_el: Element<'a, Message> = text(label)
        .size(sz::BASE)
        .style(move |t: &Theme| iced::widget::text::Style {
            color: Some(if selected {
                theme::Tokens::of(t).accent_nav_text
            } else {
                theme::Tokens::of(t).muted
            }),
        })
        .into();

    let mut content = row![icon_el, label_el]
        .spacing(12)
        .height(Length::Fill)
        .align_y(Alignment::Center);
    content = match (trailing, kbd) {
        (Some(t), _) => content.push(t),
        (None, Some(k)) => content.push(k),
        _ => content,
    };

    iced::widget::button(content)
        .width(Length::Fill)
        .height(Length::Fixed(43.0))
        .padding([0, 12])
        .style(theme::btn_nav(selected))
        .on_press(message)
        .into()
}

pub fn dot(size: f32, color: impl Fn(&Theme) -> Color + 'static) -> Element<'static, Message> {
    container(iced::widget::Space::new())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |t: &Theme| {
            container::Style::default()
                .background(color(t))
                .border(iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: (size / 2.0).into(),
                })
        })
        .into()
}

fn version_row() -> Element<'static, Message> {
    container(
        row![
            text("di")
                .size(16.0)
                .font(theme::body_font(iced::font::Weight::Bold))
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).accent_soft),
                }),
            text("Danceinterpreter")
                .size(sz::MICRO)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).faint),
                }),
            container(
                text(concat!("v", env!("CARGO_PKG_VERSION")))
                    .size(sz::MICRO)
                    .style(|t: &Theme| iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).faint),
                    })
            )
            .width(Length::Fill)
            .align_x(Alignment::End),
        ]
        .spacing(7)
        .align_y(Alignment::Center),
    )
    .padding(crate::ui::theme::pad4(0.0, 0.0, 0.0, 3.0))
    .height(Length::Fixed(48.0))
    .align_y(Alignment::Center)
    .into()
}
