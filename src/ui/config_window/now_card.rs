//! The "On the floor" hero card: what the audience display is showing right
//! now, with transport controls.

use crate::dataloading::dataprovider::ItemSource;
use crate::dataloading::displayable_data::DisplayableData;
use crate::ui::config_window::{ConfigWindow, source_tag};
use crate::ui::theme::{self, sz};
use crate::ui::widgets::{buttons, heading, material_symbol, material_symbol_sized};
use crate::{DanceInterpreter, Message};
use iced::alignment::Vertical;
use iced::widget::{column as col, container, image, row, text};
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
        _ => (
            "Ready when you are".to_string(),
            "Audience display".to_string(),
        ),
    };

    let cover: Element<'a, Message> =
        match displayable.as_ref().and_then(|d| d.subline_image.clone()) {
            Some(handle) => container(
                image(handle)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(ContentFit::Cover),
            )
            .width(Length::Fixed(56.0))
            .height(Length::Fill)
            .clip(true)
            .style(theme::cover_placeholder)
            .into(),
            None => container(material_symbol("music_note", false).size(22.0).style(
                |t: &Theme| iced::widget::text::Style {
                    color: Some(Color {
                        a: 0.35,
                        ..theme::Tokens::of(t).accent_soft
                    }),
                },
            ))
            .width(Length::Fixed(56.0))
            .height(Length::Fill)
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
            .style(theme::text_style(|tk| tk.muted)),
        source_tag(source_tag_text),
    ]
    .spacing(7)
    .align_y(Alignment::Center);

    // One line: dance · title · artist. All segments share an explicit line
    // height so the mixed font sizes sit on the same baseline, keep their
    // intrinsic width (adjacent, nothing pushed to the far edge) and the
    // whole line clips before the transport controls when it runs long.
    let lh = iced::widget::text::LineHeight::Absolute(iced::Pixels(19.0));
    let mut line = row![].spacing(7).align_y(Alignment::Center);
    line = line.push(
        heading(dance_name, 15.0, iced::font::Weight::Semibold)
            .line_height(lh)
            .style(theme::text_style(|tk| tk.dance_headline)),
    );
    if !title.is_empty() {
        line = line.push(
            text("\u{B7}")
                .size(sz::SMALL)
                .line_height(lh)
                .style(theme::text_style(|tk| tk.muted)),
        );
        line = line.push(
            text(title)
                .size(sz::BASE)
                .line_height(lh)
                .font(theme::body_font(iced::font::Weight::Medium))
                .style(theme::text_style(|tk| tk.text)),
        );
        if !artist.is_empty() {
            line = line.push(
                text("\u{B7}")
                    .size(sz::SMALL)
                    .line_height(lh)
                    .style(theme::text_style(|tk| tk.muted)),
            );
            line = line.push(
                text(artist)
                    .size(sz::SMALL)
                    .line_height(lh)
                    .style(theme::text_style(|tk| tk.text_soft)),
            );
        }
    }
    let main_line = container(line).width(Length::Fill).clip(true);

    // Fixed content height: gives the row a definite height (no compression),
    // so the cover's Fill height expands to the full card content while the
    // card itself still hugs this row.
    let body = row![
        cover,
        col![kicker, main_line].spacing(5).width(Length::Fill),
        transport(is_blank, position)
    ]
    .spacing(14)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .height(Length::Fixed(40.0));

    container(body)
        .padding(crate::ui::theme::pad4(14.0, 18.0, 14.0, 18.0))
        .width(Length::Fill)
        .clip(true)
        .style(theme::now_card)
        .into()
}

fn transport<'a>(is_blank: bool, position: String) -> Element<'a, Message> {
    let prev = buttons::ghost_icon(
        "skip_previous",
        false,
        sz::ICON,
        Message::ItemChanged(crate::dataloading::dataprovider::ItemChange::Previous),
    );

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

    let position_text =
        text(position)
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
    .into()
}

/// Small kbd pill tinted for use on colored surfaces.
fn kbd_on_accent<'a>(label: &'a str) -> Element<'a, Message> {
    container(text(label).size(sz::MICRO))
        .padding([1, 5])
        .style(|t: &Theme| {
            let tk = theme::Tokens::of(t);
            container::Style::default().border(iced::Border {
                color: Color {
                    a: 0.4,
                    ..tk.btn2_border
                },
                width: 1.0,
                radius: 3.0.into(),
            })
        })
        .into()
}
