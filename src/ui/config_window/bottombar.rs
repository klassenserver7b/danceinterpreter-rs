use crate::dataloading::dataprovider::song_data_provider::SongChange;
use crate::ui::config_window::material_icon_sized_message_button;
use crate::{DanceInterpreter, Message};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Button, Column, button, column as col, container, row, scrollable, text};
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
    ) -> Column<'a, Message> {
        let statics_buttons = self.get_statics_buttons(dance_interpreter);
        let btn_bottombar_extend = material_icon_sized_message_button(
            if self.state.value() {
                "bottom_panel_close"
            } else {
                "bottom_panel_open"
            },
            20.0,
            Message::Bottombar(BottomBarMessage::Toggle),
        )
        .padding([0, 4]);

        if self.state.value() {
            col![btn_bottombar_extend]
        } else {
            let statics_scrollable = scrollable(row(statics_buttons).spacing(5))
                .direction(Direction::Horizontal(Scrollbar::new()))
                .spacing(5)
                .width(Length::Fill);

            let statics_bar =
                container(statics_scrollable)
                    .width(Length::Fill)
                    .style(|t: &Theme| {
                        container::Style::default()
                            .background(t.extended_palette().background.weakest.color)
                    });

            col![btn_bottombar_extend, statics_bar]
                .align_x(Horizontal::Left)
                .spacing(5)
        }
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
