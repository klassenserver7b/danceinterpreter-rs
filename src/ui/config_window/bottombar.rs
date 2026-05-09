use crate::dataloading::dataprovider::song_data_provider::SongChange;
use crate::{DanceInterpreter, Message};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Button, Container, button, container, row, scrollable, text};
use iced::{Animation, Element, Font, Length, Theme, animation, font};
use std::time::Duration;

pub struct Bottombar {
    pub state: Animation<bool>,
}

#[derive(Debug, Clone)]
pub enum BottomBarMessage {
    Toggle,
}

impl Bottombar {
    pub fn new() -> Self {
        Self {
            state: Animation::new(false)
                .duration(Duration::from_millis(100))
                .easing(animation::Easing::EaseInOut),
        }
    }

    pub(crate) fn build<'a>(
        &'a self,
        dance_interpreter: &'a DanceInterpreter,
    ) -> Container<'a, Message> {
        let statics_buttons = self.get_statics_buttons(dance_interpreter);
        let statics_scrollable = scrollable(row(statics_buttons).spacing(5))
            .direction(Direction::Horizontal(Scrollbar::new()))
            .spacing(5)
            .width(Length::Fill);

        let statics_bar = container(statics_scrollable)
            .width(Length::Shrink)
            .style(|t: &Theme| {
                container::Style::default()
                    .background(t.extended_palette().background.weakest.color)
            });

        statics_bar.align_x(Horizontal::Left)
    }

    pub(crate) fn get_statics_buttons<'a>(
        &self,
        dance_interpreter: &'a DanceInterpreter,
    ) -> Vec<Element<'a, Message>> {
        let bold_font = Font {
            family: font::Family::SansSerif,
            weight: font::Weight::Bold,
            stretch: font::Stretch::Normal,
            style: font::Style::Normal,
        };

        let btn_blank: Button<Message> =
            button(text("Blank").align_y(Vertical::Center).font(bold_font))
                .style(button::secondary)
                .on_press(Message::SongChanged(SongChange::Blank));
        let btn_traktor: Button<Message> =
            button(text("Traktor").align_y(Vertical::Center).font(bold_font))
                .style(button::secondary)
                .on_press(Message::SongChanged(SongChange::Traktor));
        let mut statics: Vec<Element<_>> = dance_interpreter
            .data_provider
            .statics
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                button(text(&s.dance).font(bold_font))
                    .style(button::secondary)
                    .on_press(Message::SongChanged(SongChange::StaticAbsolute(idx)))
                    .into()
            })
            .collect();
        statics.insert(0, btn_blank.into());
        statics.insert(1, btn_traktor.into());
        statics
    }
}
