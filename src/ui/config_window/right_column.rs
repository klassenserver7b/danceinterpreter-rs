//! Right rail: "Coming up next" card, compact DJ-integration card and the
//! keyboard tip. The full DJ settings live in the slide-in panel.

use crate::dataloading::dataprovider::ItemChange;
use crate::traktor_api::TraktorMessage;
use crate::ui::config_window::sidebar::SidebarMessage;
use crate::ui::config_window::{ConfigWindow, source_tag};
use crate::ui::theme::{self, sz};
use crate::ui::widgets::{
    buttons, heading, kbd, material_symbol, material_symbol_sized,
};
use crate::{DanceInterpreter, Message};
use iced::widget::{button, column as col, container, row, scrollable, text, toggler};
use iced::{Alignment, Color, Element, Length, Theme};

const RAIL_WIDTH: f32 = 292.0;

pub fn build<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    let content = col![
        build_up_next(dance_interpreter),
        build_dj_card(config_window, dance_interpreter),
        build_tip(),
    ]
    .spacing(19)
    .width(Length::Fill);

    let rail = container(scrollable(content).style(theme::scrollable_style))
        .width(Length::Fixed(RAIL_WIDTH))
        .height(Length::Fill)
        .padding(crate::ui::theme::pad4(0.0, 24.0, 22.0, 0.0))
        .style(theme::page_bg);

    rail.into()
}

// ---------------------------------------------------------------------------
// Up next
// ---------------------------------------------------------------------------

fn build_up_next<'a>(dance_interpreter: &'a DanceInterpreter) -> Element<'a, Message> {
    let provider = &dance_interpreter.data_provider;
    let queued = provider.get_queued_next().is_some();
    let next = provider.get_next_displayable_data();

    let tag = if queued { "QUEUED" } else { "PLAYLIST" };

    let (dance, title, artist) = match &next {
        Some(d) => (
            d.headline.clone(),
            d.subline_upper.clone(),
            d.subline_lower.clone(),
        ),
        None => (
            "End of playlist".to_string(),
            "Enjoy the evening".to_string(),
            String::new(),
        ),
    };

    let dance_color = provider.get_dance_color(&dance);

    let kicker = row![
        material_symbol_sized("skip_next", false, 13.0).style(|t: &Theme| {
            iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).muted),
            }
        }),
        text("COMING UP NEXT")
            .size(sz::TINY)
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).muted),
            }),
        container(source_tag(tag)).width(Length::Fill).align_x(Alignment::End),
    ]
    .spacing(7)
    .align_y(Alignment::Center);

    let show_now = button(
        row![
            text("Show now").size(sz::TINY),
            material_symbol_sized("chevron_right", false, 13.0),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    )
    .padding([6, 11])
    .style(theme::btn_secondary)
    .on_press(Message::ItemChanged(ItemChange::Next));

    container(
        col![
            kicker,
            heading(dance, 20.0, iced::font::Weight::Semibold).style(move |t: &Theme| {
                iced::widget::text::Style {
                    color: Some(dance_color.unwrap_or_else(|| theme::Tokens::of(t).text)),
                }
            }),
            text(title)
                .size(sz::BASE)
                .font(theme::body_font(iced::font::Weight::Medium))
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).text),
                }),
            text(artist)
                .size(sz::SMALL)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).muted),
                }),
            container(show_now).padding(crate::ui::theme::pad4(4.0, 0.0, 0.0, 0.0)),
        ]
        .spacing(6),
    )
    .padding(15)
    .width(Length::Fill)
    .style(theme::card)
    .into()
}

// ---------------------------------------------------------------------------
// DJ integration card
// ---------------------------------------------------------------------------

fn build_dj_card<'a>(
    _config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    let provider = &dance_interpreter.data_provider.traktor_provider;
    let enabled = provider.enabled();
    let connection = provider.get_connection_state();
    let address = &provider.address;
    let sync = provider.sync;

    let state_tag = match (enabled, connection) {
        (false, _) => "OFF",
        (true, crate::traktor_api::ConnectionState::Connected) => "LIVE",
        (true, _) => "IDLE",
    };

    let summary = match connection {
        crate::traktor_api::ConnectionState::Connected => "Traktor & Cover Loader",
        crate::traktor_api::ConnectionState::Traktor => "Traktor connected",
        crate::traktor_api::ConnectionState::CoverLoader => "Cover Loader connected",
        crate::traktor_api::ConnectionState::Disconnected => "No client connected",
    };

    let heading_row = row![
        material_symbol_sized("link", false, 15.0).style(|t: &Theme| {
            iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).muted),
            }
        }),
        heading("DJ integration", sz::PANEL_TITLE, iced::font::Weight::Semibold).style(|t| {
            iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).text),
            }
        }),
        container(source_tag(state_tag))
            .width(Length::Fill)
            .align_x(Alignment::End),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let symbol_box = container(material_symbol("graphic_eq", false).size(17.0))
        .width(Length::Fixed(29.0))
        .height(Length::Fixed(29.0))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(|t: &Theme| {
            let tk = theme::Tokens::of(t);
            container::Style::default()
                .background(if tk.is_light {
                    Color::from_rgb8(0xE3, 0xEC, 0xDA)
                } else {
                    Color::from_rgb8(0x30, 0x37, 0x2F)
                })
                .border(iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                })
        });

    let source_row = container(
        row![
            symbol_box,
            col![
                text("Traktor Pro")
                    .size(sz::SMALL + 0.5)
                    .font(theme::body_font(iced::font::Weight::Medium))
                    .style(|t: &Theme| iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).text),
                    }),
                text(if enabled {
                    "Source enabled"
                } else {
                    "Source disabled"
                })
                .size(sz::MICRO)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).muted),
                }),
            ]
            .spacing(2)
            .width(Length::Fill),
            crate::ui::widgets::buttons::ghost_icon(
                "expand_more",
                false,
                16.0,
                Message::Sidebar(SidebarMessage::Toggle),
            ),
        ]
        .spacing(9)
        .align_y(Alignment::Center),
    )
    .padding(9)
    .width(Length::Fill)
    .style(|t: &Theme| {
        let tk = theme::Tokens::of(t);
        container::Style::default()
            .background(tk.panel_deep)
            .border(iced::Border {
                color: tk.border,
                width: 1.0,
                radius: 5.0.into(),
            })
    });

    let connection_row = row![
        container(crate::ui::config_window::nav_sidebar::dot(6.0, move |t| {
            theme::led_color(t, connection)
        })),
        text(summary)
            .size(sz::MICRO)
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).text_soft),
            }),
        container(
            text(address.as_str())
                .size(sz::MICRO)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).faint),
                })
        )
        .width(Length::Fill)
        .align_x(Alignment::End),
    ]
    .spacing(5)
    .align_y(Alignment::Center);

    let sync_row = row![
        col![
            text("Follow DJ playback")
                .size(sz::SMALL + 0.5)
                .font(theme::body_font(iced::font::Weight::Medium))
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).text),
                }),
            text("Automatically update the display")
                .size(sz::MICRO)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).muted),
                }),
        ]
        .spacing(3)
        .width(Length::Fill),
        toggler(sync)
            .on_toggle(|b| Message::Traktor(TraktorMessage::EnableSync(b)))
            .style(theme::toggler_style)
            .width(Length::Shrink),
    ]
    .align_y(Alignment::Center);

    let mut foot = col![].spacing(8);

    if provider.get_song_info().is_some() {
        foot = foot.push(
            buttons::secondary(
                "Add current song",
                Message::AddTraktorSong,
            ),
        );
    }

    foot = foot.push(
        row![
            text(if sync {
                "Manual control off"
            } else {
                "Manual control active"
            })
            .size(sz::MICRO)
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).muted),
            }),
            container(
                button(
                    row![
                        text("Configure").size(sz::MICRO),
                        material_symbol_sized("chevron_right", false, 12.0),
                    ]
                    .spacing(3)
                    .align_y(Alignment::Center)
                )
                .padding([4, 6])
                .style(theme::btn_ghost)
                .on_press(Message::Sidebar(SidebarMessage::Toggle))
            )
            .width(Length::Fill)
            .align_x(Alignment::End),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    );

    container(
        col![heading_row, source_row, connection_row, sync_row, foot]
            .spacing(12),
    )
    .padding(15)
    .width(Length::Fill)
    .style(theme::card)
    .into()
}

// ---------------------------------------------------------------------------
// Tip
// ---------------------------------------------------------------------------

fn build_tip() -> Element<'static, Message> {
    container(
        col![
            row![
                material_symbol("help", false)
                    .size(15.0)
                    .style(|t: &Theme| iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).accent_soft),
                    }),
                text("Keep your eyes on the floor.")
                    .size(sz::SMALL + 0.5)
                    .font(theme::body_font(iced::font::Weight::Medium))
                    .style(|t: &Theme| iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).text_soft),
                    }),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            // Subtitle + action hang under the heading text, matching the
            // prototype's 24px indent (icon 15 + gap 8).
            container(
                col![
                    row![
                        text("Press")
                            .size(sz::TINY)
                            .style(|t: &Theme| iced::widget::text::Style {
                                color: Some(theme::Tokens::of(t).muted),
                            }),
                        kbd("Space"),
                        text("to move to the next song.")
                            .size(sz::TINY)
                            .style(|t: &Theme| iced::widget::text::Style {
                                color: Some(theme::Tokens::of(t).muted),
                            }),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                    text("We\u{2019}ll take care of the display.")
                        .size(sz::TINY)
                        .style(|t: &Theme| iced::widget::text::Style {
                            color: Some(theme::Tokens::of(t).muted),
                        }),
                ]
                .spacing(3)
            )
            .padding(crate::ui::theme::pad4(0.0, 0.0, 0.0, 23.0)),
            container(
                button(
                    row![
                        text("View keyboard shortcuts").size(sz::TINY),
                        material_symbol_sized("open_in_new", false, 11.0),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center)
                )
                .padding([4, 6])
                .style(theme::btn_ghost)
                .on_press(Message::OpenDialog(crate::ui::config_window::DialogState::Help))
            )
            .padding(crate::ui::theme::pad4(4.0, 0.0, 0.0, 23.0)),
        ]
        .spacing(6),
    )
    .padding([8, 5])
    .width(Length::Fill)
    .into()
}
