//! The "On the floor" hero card: what the audience display is showing right
//! now, with transport controls.

use crate::dataloading::dataprovider::ItemSource;
use crate::dataloading::displayable_data::DisplayableData;
use crate::ui::config_window::{ConfigWindow, source_tag};
use crate::ui::theme::{self, sz};
use crate::ui::widgets::{
    buttons, heading, material_symbol, material_symbol_sized,
};
use crate::{DanceInterpreter, Message};
use iced::alignment::Vertical;
use iced::widget::{column as col, container, image, row, text};
use iced::widget::space::Space;
use iced::{Alignment, Color, ContentFit, Element, Length, Theme};

pub fn build<'a>(
    _config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    let provider = &dance_interpreter.data_provider;
    let current = provider.get_current_source();

    let (source_tag_text, is_blank) = match current {
        ItemSource::Playlist(_) => ("PLAYLIST", false),
        ItemSource::Static(_) => ("STATIC", false),
        ItemSource::Traktor => ("DJ LIVE", false),
        ItemSource::Other(_) => ("MANUAL", false),
        ItemSource::Blank => ("BLANK", true),
    };

    let displayable: Option<DisplayableData> = provider.get_current_displayable_data();

    let dance_name = displayable
        .as_ref()
        .map(|d| d.headline.clone())
        .unwrap_or_else(|| "Display is blank".to_string());

    let (title, artist) = match (&current, &displayable) {
        (ItemSource::Playlist(_), Some(d))
        | (ItemSource::Traktor, Some(d))
        | (ItemSource::Other(_), Some(d)) => (d.subline_upper.clone(), d.subline_lower.clone()),
        _ => ("Ready when you are".to_string(), "Audience display".to_string()),
    };

    let cover: Element<'a, Message> = match displayable.as_ref().and_then(|d| d.subline_image.clone())
    {
        Some(handle) => container(
            image(handle)
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Cover),
        )
        .width(Length::Fixed(64.0))
        .height(Length::Fixed(64.0))
        .clip(true)
        .style(theme::cover_placeholder)
        .into(),
        None => container(
            material_symbol("music_note", false)
                .size(26.0)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(Color {
                        a: 0.35,
                        ..theme::Tokens::of(t).accent_soft
                    }),
                }),
        )
        .width(Length::Fixed(64.0))
        .height(Length::Fixed(64.0))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(theme::cover_placeholder)
        .into(),
    };

    // Playlist position indicator.
    let playlist_len = provider.playlist().len();
    let position = match current {
        ItemSource::Playlist(i) => format!("{:02} / {} songs", i + 1, playlist_len),
        _ => format!("{} songs", playlist_len),
    };

    let kicker = row![
        crate::ui::config_window::nav_sidebar::dot(6.0, |t| theme::Tokens::of(t).dot_green),
        text("ON THE FLOOR")
            .size(sz::TINY)
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).muted),
            }),
        source_tag(source_tag_text),
    ]
    .spacing(7)
    .align_y(Alignment::Center);

    let dance_headline = heading(dance_name, sz::H2, iced::font::Weight::Semibold).style(
        |t: &Theme| iced::widget::text::Style {
            color: Some(theme::Tokens::of(t).dance_headline),
        },
    );

    let info = col![
        kicker,
        dance_headline,
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
        transport(is_blank, position),
    ]
    .spacing(5)
    .width(Length::Fill);

    container(
        row![cover, info, decoration()].spacing(16).align_y(Alignment::Center),
    )
    .padding(crate::ui::theme::pad4(14.0, 18.0, 14.0, 18.0))
    .width(Length::Fill)
    .style(theme::now_card)
    .into()
}

fn transport<'a>(is_blank: bool, position: String) -> Element<'a, Message> {
    let prev = buttons::ghost_icon("skip_previous", false, 16.0, Message::ItemChanged(
        crate::dataloading::dataprovider::ItemChange::Previous,
    ));

    let next_button = iced::widget::button(
        row![
            material_symbol_sized("skip_next", false, 15.0),
            text("Next song").size(sz::BASE),
            kbd_on_accent("Space"),
        ]
        .spacing(8)
        .align_y(Vertical::Center),
    )
    .padding([7, 12])
    .style(theme::btn_secondary)
    .on_press(Message::ItemChanged(
        crate::dataloading::dataprovider::ItemChange::Next,
    ));

    let position_text = text(position)
        .size(sz::SMALL)
        .style(move |t: &Theme| iced::widget::text::Style {
            color: Some(if is_blank {
                theme::Tokens::of(t).faint
            } else {
                theme::Tokens::of(t).muted
            }),
        });

    container(
        row![prev, next_button, position_text]
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .padding(crate::ui::theme::pad4(8.0, 0.0, 0.0, 0.0))
    .into()
}

/// Small kbd pill tinted for use on colored surfaces.
fn kbd_on_accent<'a>(label: &'a str) -> Element<'a, Message> {
    container(text(label).size(sz::MICRO))
        .padding([1, 5])
        .style(|t: &Theme| {
            let tk = theme::Tokens::of(t);
            container::Style::default().border(iced::Border {
                color: Color { a: 0.4, ..tk.btn2_border },
                width: 1.0,
                radius: 3.0.into(),
            })
        })
        .into()
}

/// Decorative equalizer bars on the right edge of the card.
fn decoration() -> Element<'static, Message> {
    let heights = [12.0, 22.0, 17.0, 27.0, 14.0];
    let bars: Vec<Element<'static, Message, Theme>> = heights
        .into_iter()
        .map(|h| {
            container(Space::new())
                .width(Length::Fixed(3.0))
                .height(Length::Fixed(h))
                .style(|t: &Theme| {
                    container::Style::default().background(Color {
                        a: 0.35,
                        ..theme::Tokens::of(t).accent_soft
                    })
                })
                .into()
        })
        .collect();

    row(bars)
        .spacing(5)
        .align_y(Alignment::End)
        .width(Length::Shrink)
        .into()
}