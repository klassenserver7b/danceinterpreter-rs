//! Right rail: the "now playing", "coming up next" and "Traktor" cards.

use crate::dataloading::dataprovider::{ItemChange, ItemSource};
use crate::ui::config_window::source_tag_right;
use crate::ui::config_window::traktor_card;
use crate::ui::theme::{self, pad4, sz};
use crate::ui::widgets::{EllipsisText, buttons, dot, kbd, material_symbol_sized};
use crate::{ConfigWindow, DanceInterpreter, Message};
use iced::widget::{button, column as col, container, row, scrollable, text};
use iced::{Alignment, Element, Length, Theme};

/// Full width of the right info rail when expanded.
pub const RAIL_WIDTH: f32 = 292.0;

pub fn build<'a>(
    dance_interpreter: &'a DanceInterpreter,
    config_window: &'a ConfigWindow,
    width: f32,
) -> Element<'a, Message> {
    let content = col![
        build_now_playing(dance_interpreter),
        build_up_next(dance_interpreter),
        traktor_card::build(dance_interpreter, config_window),
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
// Now playing
// ---------------------------------------------------------------------------

fn build_now_playing<'a>(dance_interpreter: &'a DanceInterpreter) -> Element<'a, Message> {
    let provider = &dance_interpreter.data_provider;
    let current = provider.get_current_source();

    let (tag, is_blank) = match current {
        ItemSource::Playlist(_) => ("PLAYLIST", false),
        ItemSource::Static(_) => ("STATIC", false),
        ItemSource::Traktor => ("DJ LIVE", false),
        ItemSource::Other(_) => ("MANUAL", false),
        ItemSource::Blank => ("BLANK", true),
    };

    let displayable = provider.get_current_displayable_data();

    let dance_name = displayable
        .as_ref()
        .map(|d| d.headline.clone())
        .unwrap_or_else(|| "Display is blank".to_string());

    let (title, artist) = match (&current, &displayable) {
        (ItemSource::Playlist(_), Some(d))
        | (ItemSource::Traktor, Some(d))
        | (ItemSource::Other(_), Some(d)) => (d.subline_upper.clone(), d.subline_lower.clone()),
        _ => (
            "Ready when you are".to_string(),
            "Audience display".to_string(),
        ),
    };

    let dance_color = provider.get_dance_color(&dance_name);

    let kicker = row![
        dot(6.0, |t| theme::Tokens::of(t).dot_green),
        text("NOW PLAYING")
            .size(sz::TINY)
            .style(theme::text_style(|tk| tk.muted)),
        source_tag_right(tag),
    ]
    .spacing(7)
    .align_y(Alignment::Center);

    // Playlist position indicator.
    let playlist_len = provider.playlist().len();
    let position = match current {
        ItemSource::Playlist(i) => format!("{:02} / {} songs", i + 1, playlist_len),
        _ => format!("{} songs", playlist_len),
    };

    let prev = buttons::ghost_icon(
        "skip_previous",
        false,
        sz::ICON,
        Message::ItemChanged(ItemChange::Previous),
    );

    let next_button = button(
        row![
            material_symbol_sized("skip_next", false, sz::ICON_SM),
            text("Next song").size(sz::SMALL),
            kbd("Space", theme::kbd_style),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([5, 10])
    .style(theme::btn_secondary)
    .on_press(Message::ItemChanged(ItemChange::Next));

    let position_text = text(position)
        .size(sz::TINY)
        .style(theme::text_style(move |tk| {
            if is_blank { tk.faint } else { tk.muted }
        }));

    let transport = container(
        row![prev, next_button, position_text]
            .spacing(4)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(pad4(4.0, 0.0, 0.0, 0.0));

    container(
        col![
            kicker,
            EllipsisText::new(dance_name)
                .size(sz::H2)
                .font(theme::head_font(iced::font::Weight::Semibold))
                .style(theme::text_style(move |tk| {
                    if is_blank {
                        tk.faint
                    } else {
                        dance_color.unwrap_or(tk.dance_headline)
                    }
                })),
            EllipsisText::new(title)
                .size(sz::BASE)
                .font(theme::body_font(iced::font::Weight::Medium))
                .style(theme::text_style(move |tk| if is_blank {
                    tk.muted
                } else {
                    tk.text
                })),
            EllipsisText::new(artist)
                .size(sz::CAPTION)
                .style(theme::text_style(|tk| tk.muted)),
            transport,
        ]
        .spacing(6),
    )
    .padding(15)
    .width(Length::Fill)
    .style(theme::card)
    .into()
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
            EllipsisText::new(dance)
                .size(sz::H2)
                .font(theme::head_font(iced::font::Weight::Semibold))
                .style(theme::text_style(move |tk| dance_color.unwrap_or(tk.text),)),
            EllipsisText::new(title)
                .size(sz::BASE)
                .font(theme::body_font(iced::font::Weight::Medium))
                .style(theme::text_style(|tk| tk.text)),
            EllipsisText::new(artist)
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
