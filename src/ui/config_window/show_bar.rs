//! Bottom quick-show bar: blank / DJ live / favorite statics — the
//! operator's one-click show control during an event.

use crate::dataloading::dataprovider::{ItemChange, ItemSource};
use crate::ui::config_window::{ConfigWindow, View};
use crate::ui::theme::{self, sz};
use crate::ui::widgets::{kbd, material_symbol, material_symbol_sized};
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, Message};
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{button, container, row, scrollable, text};
use iced::{Alignment, Element, Length, Theme};

pub fn build<'a>(
    _config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    let current = dance_interpreter.data_provider.get_current_source().clone();

    let label = row![
        material_symbol("graphic_eq", false)
            .size(17.0)
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).accent_soft),
            }),
        text("Quick show")
            .size(sz::SMALL + 0.5)
            .font(theme::body_font(iced::font::Weight::Medium))
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).text),
            }),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let blank = quick_button(
        Some("crop_free"),
        "Blank",
        Some(kbd("End").into()),
        current == ItemSource::Blank,
        Message::ItemChanged(ItemChange::Blank),
    );

    let dj_live = quick_button(
        Some("graphic_eq"),
        "DJ live",
        None,
        current == ItemSource::Traktor,
        Message::ItemChanged(ItemChange::Traktor),
    );

    let mut buttons_row = row![label, blank, dj_live]
        .spacing(8)
        .align_y(Alignment::Center);

    // Divider between sources and favorite statics.
    let divider = container(iced::widget::Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(24.0))
        .style(|t: &Theme| container::Style::default().background(theme::Tokens::of(t).border));

    buttons_row = buttons_row.push(divider);

    for (name, static_info) in dance_interpreter
        .data_provider
        .statics()
        .iter()
        .filter(|(_, s)| s.is_favorite)
    {
        let active = current == ItemSource::Static(name.clone());
        let color = static_info
            .color
            .unwrap_or_else(|| theme::Tokens::dark().muted);

        let dot = container(iced::widget::Space::new())
            .width(Length::Fixed(5.0))
            .height(Length::Fixed(5.0))
            .style(move |_: &Theme| {
                container::Style::default()
                    .background(color)
                    .border(iced::Border {
                        color: iced::Color::TRANSPARENT,
                        width: 0.0,
                        radius: 3.0.into(),
                    })
            });

        let btn = button(
            row![dot, text(&static_info.name).size(sz::TINY)]
                .spacing(7)
                .height(Length::Fill)
                .align_y(Alignment::Center),
        )
        .padding([0, 11])
        .height(Length::Fixed(33.0))
        .style(move |t: &Theme, s: button::Status| {
            if active {
                theme::btn_active(t, s)
            } else {
                theme::btn_secondary(t, s)
            }
        })
        .on_press(Message::ItemChanged(ItemChange::StaticAbsolute(
            name.clone(),
        )));

        buttons_row = buttons_row.push(with_tooltip(
            btn,
            format!("Show {} static", static_info.name),
        ));
    }

    let manage = with_tooltip(
        button(
            container(
                material_symbol_sized("add", false, 16.0).style(|t: &Theme| {
                    iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).muted),
                    }
                }),
            )
            .height(Length::Fill)
            .align_y(Alignment::Center),
        )
        .padding([0, 10])
        .height(Length::Fixed(33.0))
        .style(theme::btn_outline)
        .on_press(Message::SetView(View::Statics)),
        "Manage favorite statics",
    );

    buttons_row = buttons_row.push(manage);

    let scroll = scrollable(buttons_row)
        .direction(Direction::Horizontal(
            Scrollbar::new().width(6.0).scroller_width(6.0),
        ))
        .width(Length::Fill)
        .height(Length::Shrink)
        .style(theme::scrollable_style);

    container(scroll)
        .padding([15, 28])
        .height(Length::Fixed(77.0))
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .style(theme::showbar_bg)
        .into()
}

/// A quick-show pill button with optional icon and keyboard hint.
fn quick_button<'a>(
    icon: Option<&'a str>,
    label: &'a str,
    hint: Option<Element<'a, Message>>,
    active: bool,
    message: Message,
) -> Element<'a, Message> {
    let mut content = row![]
        .spacing(7)
        .height(Length::Fill)
        .align_y(Alignment::Center);

    if let Some(icon) = icon {
        content = content.push(
            material_symbol_sized(icon, false, 15.0).style(move |t: &Theme| {
                iced::widget::text::Style {
                    color: Some(if active {
                        theme::Tokens::of(t).active_text
                    } else {
                        theme::Tokens::of(t).btn2_text
                    }),
                }
            }),
        );
    }

    content = content.push(text(label).size(sz::TINY).style(move |t: &Theme| {
        iced::widget::text::Style {
            color: Some(if active {
                theme::Tokens::of(t).active_text
            } else {
                theme::Tokens::of(t).btn2_text
            }),
        }
    }));

    if let Some(hint) = hint {
        content = content.push(hint);
    }

    let btn = button(content)
        .padding([0, 11])
        .height(Length::Fixed(33.0))
        .style(move |t: &Theme, s: button::Status| {
            if active {
                theme::btn_active(t, s)
            } else {
                theme::btn_secondary(t, s)
            }
        });

    if active {
        // The active item is already on screen; pressing again is a no-op.
        btn.into()
    } else {
        btn.on_press(message).into()
    }
}
