use crate::{DanceInterpreter, Message, Window};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, checkbox, text};
use iced::{window, Element, Length, Size};
use iced_aw::menu::Item;
use iced_aw::{menu_bar, menu_items, Menu};

#[derive(Default)]
pub struct ConfigWindow {
    pub id: Option<window::Id>,
    pub size: Size,
}

impl Window for ConfigWindow {
    fn on_create(&mut self, id: window::Id) {
        self.id = Some(id);
    }

    fn on_resize(&mut self, size: Size) {
        self.size = size;
    }
}

impl ConfigWindow {
    pub fn view(&self, state: &DanceInterpreter) -> Element<Message> {
        let menu_tpl_1 = |items| Menu::new(items).max_width(150.0).offset(15.0).spacing(5.0);
        let mb = menu_bar!(
            (button(
                text("File").align_y(Vertical::Center)
            ).padding([4, 8]).style(button::secondary),
            menu_tpl_1(
                menu_items!(
                    (button(
                        text("Open Playlist File").align_y(Vertical::Center).align_x(Horizontal::Left)
                    ).padding([4, 8]).on_press(Message::OpenPlaylist)
                    .width(Length::Fill)
                    .style(button::secondary))
                (button(
                        text("Exit").align_y(Vertical::Center).align_x(Horizontal::Left)
                    ).padding([4, 8])
                    .width(Length::Fill)
                    .style(button::secondary))
            )).spacing(5.0))
            (button(
                text("Edit").align_y(Vertical::Center)
            ).padding([4, 8]).style(button::secondary),
            menu_tpl_1(
                menu_items!(
                    (button(
                        text("Import Playlistview").align_y(Vertical::Center).align_x(Horizontal::Left)
                    ).padding([4, 8])
                    .width(Length::Fill)
                    .style(button::secondary))
                (button(
                        text("Export Playlistview").align_y(Vertical::Center).align_x(Horizontal::Left)
                    ).padding([4, 8])
                    .width(Length::Fill)
                    .style(button::secondary))
            )).spacing(5.0))
            (button(
                text("SongWindow").align_y(Vertical::Center)
            ).padding([4, 8]).style(button::secondary),
            menu_tpl_1(
                menu_items!(
                    (checkbox(
                         "Show Thumbnails", true
                        ).spacing(5.0)
                        .width(Length::Fill)
                        .style(checkbox::secondary))

                    (checkbox(
                         "Show Next Dance", true
                        ).spacing(5.0)
                        .width(Length::Fill)
                        .style(checkbox::secondary))

                     (button(
                        text("Refresh").align_y(Vertical::Center).align_x(Horizontal::Left)
                    ).padding([4, 8]).on_press(Message::Refresh)
                    .width(Length::Fill)
                    .style(button::secondary))
            )).spacing(5.0))
        ).spacing(5.0);

        mb.into()
    }
}
