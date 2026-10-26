//! Modal dialogs: delete confirmation, static merge, settings, keyboard
//! help and the "add static" form.

use crate::dataloading::dataprovider::ItemSource;
use crate::ui::config_window::ConfigWindow;
use crate::ui::theme::{self, sz};
use crate::ui::widgets::{buttons, heading, kbd};
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, DialogMessage, Message, StaticsMessage};
use iced::widget::{Column, column as col, container, mouse_area, opaque, pick_list, row, space::Space, stack, text, text_input, toggler};
use iced::{Alignment, Element, Length, Theme};
use iced_aw::number_input;

#[derive(Debug, Clone)]
pub enum DialogState {
    Delete(ItemSource),
    MergeStatic { old_name: String, new_name: String },
    Settings,
    Help,
    AddStatic,
}

pub fn build_dialog<'a>(
    state: &'a DialogState,
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    let close = Message::Dialog(DialogMessage::Close);

    let (title, body, actions) = match state {
        DialogState::Delete(source) => {
            let target_name = match source {
                ItemSource::Playlist(i) => dance_interpreter
                    .data_provider
                    .playlist()
                    .get(*i)
                    .map(|item| item.song.title.clone())
                    .unwrap_or_default(),
                ItemSource::Static(name) => dance_interpreter
                    .data_provider
                    .statics()
                    .get(name)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| name.clone()),
                _ => String::new(),
            };

            let is_static = matches!(source, ItemSource::Static(_));
            let kind = if is_static { "static" } else { "song" };

            (
                format!("Delete {}?", kind),
                col![text(format!(
                    "Remove \u{201C}{}\u{201D} from the {}? You can undo with Ctrl+Z.",
                    target_name,
                    if is_static { "statics" } else { "playlist" }
                ))
                .size(sz::SMALL + 1.0)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).muted),
                })],
                row![
                    buttons::secondary("Cancel", Message::Dialog(DialogMessage::Close)),
                    buttons::danger(
                        format!("Delete {}", kind),
                        Message::Dialog(DialogMessage::ConfirmDelete),
                    ),
                ]
                .spacing(10)
                .into(),
            )
        }
        DialogState::MergeStatic { old_name, new_name } => (
            "Merge statics?".to_string(),
            col![text(format!(
                "A static named \u{201C}{}\u{201D} already exists.\nMerge \u{201C}{}\u{201D} into it? All playlist songs will be re-tagged.",
                new_name, old_name
            ))
            .size(sz::SMALL + 1.0)
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).muted),
            })],
            row![
                buttons::secondary("Cancel", Message::Dialog(DialogMessage::Cancel)),
                buttons::primary(
                    "Merge",
                    Message::Statics(StaticsMessage::ConfirmMerge(
                        old_name.clone(),
                        new_name.clone(),
                    )),
                ),
            ]
            .spacing(10)
            .into(),
        ),
        DialogState::Settings => build_settings(config_window, dance_interpreter),
        DialogState::Help => build_help(),
        DialogState::AddStatic => build_add_static(config_window),
    };

    let card = container(
        col![
            row![
                heading(title, sz::H2, iced::font::Weight::Medium).style(|t| {
                    iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).text),
                    }
                }),
                Space::new().width(Length::Fill),
                with_tooltip(
                    buttons::ghost_icon("close", false, 18.0, close),
                    "Close"
                ),
            ]
            .align_y(Alignment::Center),
            body,
            actions,
        ]
        .spacing(18)
        .width(Length::Fill),
    )
    .padding(25)
    .width(Length::Fixed(440.0))
    .style(theme::modal_card);

    let backdrop = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::backdrop),
    )
    .on_press(Message::Dialog(DialogMessage::Close));

    let centered = container(opaque(card))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

    opaque(stack![backdrop, centered])
}

/// A labeled settings row with a bottom hairline.
fn setting_row<'a>(
    label: &'a str,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        row![
            text(label)
                .size(sz::SMALL + 1.0)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).text_soft),
                })
                .width(Length::Fill),
            control,
        ]
        .align_y(Alignment::Center)
        .spacing(20),
    )
    .padding([13, 0])
    .width(Length::Fill)
    .style(|t: &Theme| {
        let tk = theme::Tokens::of(t);
        container::Style::default().border(iced::Border {
            color: tk.border_soft,
            width: 0.0,
            radius: 0.0.into(),
        })
    })
    .into()
}

fn section_label(label: &'static str) -> Element<'static, Message> {
    container(
        text(label)
            .size(sz::MICRO)
            .font(theme::body_font(iced::font::Weight::Semibold))
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).muted),
            }),
    )
    .padding(crate::ui::theme::pad4(16.0, 0.0, 2.0, 0.0))
    .into()
}

fn build_settings<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> (String, Column<'a, Message, Theme>, Element<'a, Message>) {
    let theme_pick = pick_list(
        Theme::ALL,
        Some(config_window.theme.clone()),
        Message::SetConfigTheme,
    )
    .width(Length::Fixed(160.0))
    .text_size(sz::SMALL + 1.0)
    .padding([5, 8])
    .style(theme::pick_list_style)
    .menu_style(theme::pick_list_menu_style);

    let scale_input = number_input(
        &dance_interpreter.song_window.scale,
        0.5..=3.0,
        |val| Message::SongWindow(crate::SongWindowMessage::ChangeScale(val)),
    )
    .step(0.1)
    .width(Length::Fixed(120.0));

    let switch = |checked: bool, msg: fn(bool) -> Message| {
        toggler(checked)
            .on_toggle(msg)
            .style(theme::toggler_style)
            .width(Length::Shrink)
            .into()
    };

    let body = col![
        setting_row(
            "Follow system theme",
            switch(config_window.follow_system_theme, Message::EnableFollowSystemTheme),
        ),
        setting_row("Theme", theme_pick.into()),
        setting_row(
            "Autoscroll to current song",
            switch(config_window.enable_autoscroll, Message::EnableAutoscroll),
        ),
        section_label("AUDIENCE WINDOW"),
        setting_row("Text scale", scale_input.into()),
        setting_row(
            "Show thumbnails",
            switch(
                dance_interpreter.song_window.enable_image,
                |v| Message::SongWindow(crate::SongWindowMessage::EnableImage(v)),
            ),
        ),
        setting_row(
            "Show next dance",
            switch(
                dance_interpreter.song_window.enable_next_dance,
                |v| Message::SongWindow(crate::SongWindowMessage::EnableNextDance(v)),
            ),
        ),
        setting_row(
            "Open audience window",
            buttons::secondary("Focus window", Message::FocusSongWindow).into(),
        ),
        text("The audience display always stays dark for maximum contrast.")
            .size(sz::TINY)
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).faint),
            }),
    ]
    .spacing(0);

    ("Settings".to_string(), body, Space::new().height(Length::Fixed(0.0)).into())
}

fn build_help() -> (String, Column<'static, Message, Theme>, Element<'static, Message>) {
    let shortcuts: &[(&str, &str)] = &[
        ("Next song", "Space / \u{2192}"),
        ("Previous song", "\u{2190}"),
        ("Blank display", "End"),
        ("Song window fullscreen", "F11"),
        ("Find a song", "Ctrl+F"),
        ("Add blank entry", "Ctrl+N"),
        ("Undo delete", "Ctrl+Z"),
        ("Reload statics", "F5"),
        ("Scroll playlist", "PgUp / PgDn"),
        ("Audience text scale", "Ctrl + / \u{2212}"),
        ("DJ integration panel", "Alt+C"),
        ("Navigation sidebar", "Ctrl+B"),
        ("Info column", "Ctrl+J"),
        ("This help", "?"),
    ];

    let mut body = col![text("A little less clicking. A little more dancing.")
        .size(sz::SMALL + 1.0)
        .style(|t: &Theme| iced::widget::text::Style {
            color: Some(theme::Tokens::of(t).muted),
        })]
    .spacing(4);

    for (action, keys) in shortcuts {
        body = body.push(container(
            row![
                text(*action)
                    .size(sz::SMALL + 1.0)
                    .style(|t: &Theme| iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).text_soft),
                    })
                    .width(Length::Fill),
                kbd(*keys),
            ]
            .align_y(Alignment::Center),
        )
        .padding([7, 0])
        .width(Length::Fill));
    }

    ("Keyboard shortcuts".to_string(), body, Space::new().height(Length::Fixed(0.0)).into())
}

fn build_add_static<'a>(
    config_window: &'a ConfigWindow,
) -> (String, Column<'a, Message, Theme>, Element<'a, Message>) {
    let name_input = text_input("Static name", &config_window.add_static_name)
        .size(sz::SMALL + 1.0)
        .on_input(|v| Message::Dialog(DialogMessage::ChangeAddStaticName(v)))
        .on_submit(Message::Statics(StaticsMessage::AddNamed(
            config_window.add_static_name.clone(),
        )))
        .style(theme::input_boxed)
        .padding([7, 9]);

    let body = col![
        text("A static is shown alone on the audience display \u{2014} a dance label or an announcement like \u{201C}Break\u{201D}.")
            .size(sz::SMALL + 1.0)
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).muted),
            }),
        name_input,
    ]
    .spacing(12);

    let actions = row![
        buttons::secondary("Cancel", Message::Dialog(DialogMessage::Close)),
        buttons::primary(
            "Add static",
            Message::Statics(StaticsMessage::AddNamed(config_window.add_static_name.clone())),
        ),
    ]
    .spacing(10);

    ("Add static".to_string(), body, actions.into())
}
