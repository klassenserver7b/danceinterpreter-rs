use crate::dataloading::dataprovider::ItemChange;
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, Message};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Button, Column, button, column as col, container, row, scrollable, text};
use iced::{Element, Font, Length, Theme, font};

pub(crate) fn build(dance_interpreter: &'_ DanceInterpreter) -> Column<'_, Message> {
    let statics_buttons = get_statics_buttons(dance_interpreter);

    let statics_scrollable = scrollable(row(statics_buttons).spacing(5))
        .direction(Direction::Horizontal(Scrollbar::new()))
        .spacing(5)
        .width(Length::Shrink);

    let statics_bar = container(statics_scrollable)
        .width(Length::Shrink)
        .style(|t: &Theme| {
            container::Style::default().background(t.extended_palette().background.weakest.color)
        });

    col![statics_bar]
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .spacing(5)
}

pub(crate) fn get_statics_buttons(
    dance_interpreter: &'_ DanceInterpreter,
) -> Vec<Element<'_, Message>> {
    let bold_font = Font {
        family: font::Family::SansSerif,
        weight: font::Weight::Bold,
        stretch: font::Stretch::Normal,
        style: font::Style::Normal,
    };

    let btn_blank: Button<Message> =
        button(text("Blank").align_y(Vertical::Center).font(bold_font))
            .style(button::secondary)
            .on_press(Message::ItemChanged(ItemChange::Blank));
    let btn_traktor: Button<Message> =
        button(text("Traktor").align_y(Vertical::Center).font(bold_font))
            .style(button::secondary)
            .on_press(Message::ItemChanged(ItemChange::Traktor));
    let mut statics: Vec<Element<_>> = dance_interpreter
        .data_provider
        .statics
        .iter()
        .enumerate()
        .filter(|(_, s)| s.is_favorite)
        .map(|(idx, s)| {
            with_tooltip(
                button(text(&s.name).font(bold_font))
                    .style(button::secondary)
                    .on_press(Message::ItemChanged(ItemChange::StaticAbsolute(idx))),
                format!("Show {} static", s.name),
            )
        })
        .collect();
    statics.insert(0, with_tooltip(btn_blank, "Show blank screen"));
    statics.insert(1, with_tooltip(btn_traktor, "Show current Traktor song"));
    statics
}
