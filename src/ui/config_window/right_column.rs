//! Right rail: "Coming up next" card, compact DJ-integration card and the
//! keyboard tip. The full DJ settings live in the slide-in panel.

use crate::dataloading::dataprovider::ItemChange;
use crate::traktor_api::TraktorMessage;
use crate::ui::config_window::DialogState;
use crate::ui::config_window::dj_panel::SidebarMessage;
use crate::ui::config_window::source_tag_right;
use crate::ui::theme::{self, pad4, sz};
use crate::ui::widgets::{
    buttons, heading, kbd, material_symbol, material_symbol_sized, push_right,
};
use crate::{DanceInterpreter, Message};
use iced::widget::{button, column as col, container, row, scrollable, text, toggler};
use iced::{Alignment, Color, Element, Length, Theme};

/// Full width of the right info rail when expanded.
pub const RAIL_WIDTH: f32 = 292.0;

pub fn build<'a>(dance_interpreter: &'a DanceInterpreter, width: f32) -> Element<'a, Message> {
    let content = col![
        build_up_next(dance_interpreter),
        build_dj_card(dance_interpreter),
        build_tip(),
    ]
    .spacing(19)
    .width(Length::Fill);

    let rail = container(scrollable(content).style(theme::scrollable_style))
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .clip(true)
        .padding(pad4(14.0, 14.0, 16.0, 14.0))
        .style(move |t: &Theme| theme::panel_collapsible(t, width));

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
        material_symbol_sized("skip_next", false, sz::ICON_XXS)
            .style(theme::text_style(|tk| tk.muted)),
        text("COMING UP NEXT")
            .size(sz::TINY)
            .style(theme::text_style(|tk| tk.muted)),
        source_tag_right(tag),
    ]
    .spacing(7)
    .align_y(Alignment::Center);

    let show_now = button(
        row![
            text("Show now").size(sz::SMALL),
            material_symbol_sized("chevron_right", false, sz::ICON_XXS),
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
            heading(dance, sz::H2, iced::font::Weight::Semibold)
                .style(theme::text_style(move |tk| dance_color.unwrap_or(tk.text),)),
            text(title)
                .size(sz::BASE)
                .font(theme::body_font(iced::font::Weight::Medium))
                .style(theme::text_style(|tk| tk.text)),
            text(artist)
                .size(sz::CAPTION)
                .style(theme::text_style(|tk| tk.muted)),
            container(show_now).padding(pad4(4.0, 0.0, 0.0, 0.0)),
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

fn build_dj_card<'a>(dance_interpreter: &'a DanceInterpreter) -> Element<'a, Message> {
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

    let summary = connection.summary();

    let heading_row = row![
        material_symbol_sized("link", false, sz::ICON_SM).style(theme::text_style(|tk| tk.muted)),
        heading(
            "DJ integration",
            sz::PANEL_TITLE,
            iced::font::Weight::Semibold
        )
        .style(theme::text_style(|tk| tk.text)),
        source_tag_right(state_tag),
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
                .background(tk.accent_nav_bg)
                .border(iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: theme::r::SM.into(),
                })
        });

    let source_row = container(
        row![
            symbol_box,
            col![
                text("Traktor Pro")
                    .size(sz::CAPTION)
                    .font(theme::body_font(iced::font::Weight::Medium))
                    .style(theme::text_style(|tk| tk.text)),
                text(if enabled {
                    "Source enabled"
                } else {
                    "Source disabled"
                })
                .size(sz::TINY)
                .style(theme::text_style(|tk| tk.muted)),
            ]
            .spacing(2)
            .width(Length::Fill),
            buttons::ghost_icon(
                "expand_more",
                false,
                sz::ICON,
                Message::Sidebar(SidebarMessage::Toggle),
            ),
        ]
        .spacing(9)
        .align_y(Alignment::Center),
    )
    .padding(9)
    .width(Length::Fill)
    .style(|t: &Theme| theme::inset_card(t, theme::r::MD));

    let connection_row = row![
        container(crate::ui::widgets::dot(6.0, move |t| {
            theme::led_color(t, connection)
        })),
        text(summary)
            .size(sz::SMALL)
            .style(theme::text_style(|tk| tk.text_soft)),
        push_right(
            text(address.as_str())
                .size(sz::TINY)
                .style(theme::text_style(|tk| tk.faint)),
        ),
    ]
    .spacing(5)
    .align_y(Alignment::Center);

    let sync_row = row![
        col![
            text("Follow DJ playback")
                .size(sz::CAPTION)
                .font(theme::body_font(iced::font::Weight::Medium))
                .style(theme::text_style(|tk| tk.text)),
            text("Automatically update the display")
                .size(sz::TINY)
                .style(theme::text_style(|tk| tk.muted)),
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
        foot = foot.push(buttons::secondary(
            "Add current song",
            Message::AddTraktorSong,
        ));
    }

    foot = foot.push(
        row![
            text(if sync {
                "Manual control off"
            } else {
                "Manual control active"
            })
            .size(sz::TINY)
            .style(theme::text_style(|tk| tk.muted)),
            push_right(
                button(
                    row![
                        text("Configure").size(sz::TINY),
                        material_symbol_sized("chevron_right", false, 12.0),
                    ]
                    .spacing(3)
                    .align_y(Alignment::Center)
                )
                .padding([4, 6])
                .style(theme::btn_ghost)
                .on_press(Message::Sidebar(SidebarMessage::Toggle)),
            ),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    );

    container(col![heading_row, source_row, connection_row, sync_row, foot].spacing(12))
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
                    .size(sz::ICON_SM)
                    .style(theme::text_style(|tk| tk.accent_soft)),
                row![
                    text("Press")
                        .size(sz::SMALL)
                        .style(theme::text_style(|tk| tk.muted)),
                    kbd("Space", theme::kbd),
                    text("to move to the next song.")
                        .size(sz::SMALL)
                        .style(theme::text_style(|tk| tk.muted)),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            container(
                button(
                    row![
                        text("View keyboard shortcuts").size(sz::SMALL),
                        material_symbol_sized("open_in_new", false, 11.0),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center)
                )
                .padding([4, 6])
                .style(theme::btn_ghost)
                .on_press(Message::OpenDialog(DialogState::Help))
            )
            .padding(pad4(2.0, 0.0, 0.0, 23.0)),
        ]
        .spacing(2),
    )
    .padding([8, 5])
    .width(Length::Fill)
    .into()
}
