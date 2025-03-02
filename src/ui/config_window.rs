use crate::dataloading::dataprovider::song_data_provider::{
    SongChange, SongDataEdit, SongDataSource,
};
use crate::ui::material_icon;
use crate::ui::widget::dynamic_text_input::DynamicTextInput;
use crate::{DanceInterpreter, Message, Window};
use iced::advanced::Widget;
use iced::alignment::Vertical;
use iced::border::Radius;
use iced::widget::scrollable::{Direction, RelativeOffset, Scrollbar};
use iced::widget::{
    button, checkbox, column as col, row, scrollable, text, Button, Column, Row, Scrollable, Space,
};
use iced::{font, window, Border, Color, Element, Font, Length, Renderer, Size, Theme};
use iced_aw::menu::Item;
use iced_aw::style::{menu_bar::primary, Status};
use iced_aw::widget::InnerBounds;
use iced_aw::{menu, menu_bar, menu_items, quad, Menu, MenuBar};
use std::sync::LazyLock;

#[derive(Default)]
pub struct ConfigWindow {
    pub id: Option<window::Id>,
    pub size: Size,
}

pub static PLAYLIST_SCROLLABLE_ID: LazyLock<scrollable::Id> = LazyLock::new(scrollable::Id::unique);

impl Window for ConfigWindow {
    fn on_create(&mut self, id: window::Id) {
        self.id = Some(id);
    }

    fn on_resize(&mut self, size: Size) {
        self.size = size;
    }
}

impl ConfigWindow {
    pub fn view<'a>(&'a self, dance_interpreter: &'a DanceInterpreter) -> Element<'a, Message> {
        let menu_bar = self.build_menu_bar(dance_interpreter);
        let playlist_view = self.build_playlist_view(dance_interpreter);
        let statics_view = self.build_statics_view(dance_interpreter);

        let content = col![menu_bar, playlist_view, statics_view];
        content.into()
    }

    fn get_play_state(
        &self,
        dance_interpreter: &DanceInterpreter,
        playlist_index: usize,
    ) -> (bool, bool, bool) {
        let mut is_current = false;
        let mut is_next = false;
        let is_played = dance_interpreter
            .data_provider
            .playlist_played
            .get(playlist_index)
            .copied()
            .unwrap_or(false);

        if let SongDataSource::Playlist(i) = dance_interpreter.data_provider.current {
            is_current = playlist_index == i;
            is_next = playlist_index == (i + 1);
        }

        if let Some(SongDataSource::Playlist(i)) = dance_interpreter.data_provider.next {
            is_next = playlist_index == i;
        }

        (is_current, is_next, is_played)
    }

    fn build_playlist_view(&self, dance_interpreter: &DanceInterpreter) -> Column<Message> {
        let trow: Row<_> = row![
            text!("#").width(Length::Fixed(24.0)),
            text!("Title").width(Length::Fill),
            text!("Artist").width(Length::Fill),
            text!("Dance").width(Length::Fill),
            Space::new(Length::Fill, Length::Shrink),
            Space::new(Length::Fixed(10.0), Length::Shrink),
        ]
        .spacing(5);

        let mut playlist_column: Column<'_, _, _, _> = col!().spacing(5);

        for (i, song) in dance_interpreter
            .data_provider
            .playlist_songs
            .iter()
            .enumerate()
        {
            let (is_current, is_next, is_played) = self.get_play_state(dance_interpreter, i);
            let icon: Element<Message> = if is_current {
                material_icon("play_arrow")
                    .width(Length::Fixed(24.0))
                    .into()
            } else if is_next {
                material_icon("skip_next").width(Length::Fixed(24.0)).into()
            } else if is_played {
                material_icon("check").width(Length::Fixed(24.0)).into()
            } else {
                Space::new(Length::Fixed(24.0), Length::Shrink).into()
            };

            let song_row = row![
                icon,
                DynamicTextInput::<'_, Message>::new("Title", &song.title)
                    .width(Length::Fill)
                    .on_change(move |v| Message::SongDataEdit(i, SongDataEdit::Title(v))),
                DynamicTextInput::<'_, Message>::new("Artist", &song.artist)
                    .width(Length::Fill)
                    .on_change(move |v| Message::SongDataEdit(i, SongDataEdit::Artist(v))),
                DynamicTextInput::<'_, Message>::new("Dance", &song.dance)
                    .width(Length::Fill)
                    .on_change(move |v| Message::SongDataEdit(i, SongDataEdit::Dance(v))),
                row![
                    Space::new(Length::Fill, Length::Shrink),
                    material_icon_message_button(
                        "smart_display",
                        Message::SongChanged(SongChange::PlaylistAbsolute(i))
                    ),
                    material_icon_message_button(
                        "queue_play_next",
                        Message::SetNextSong(SongDataSource::Playlist(i))
                    ),
                    material_icon_message_button(
                        "delete",
                        Message::DeleteSong(SongDataSource::Playlist(i))
                    ),
                ]
                .spacing(5)
                .width(Length::Fill),
            ]
            .spacing(5);

            if !playlist_column.children().is_empty() {
                playlist_column = playlist_column.push(separator());
            }

            playlist_column = playlist_column.push(song_row);
        }

        let playlist_scrollable: Scrollable<'_, Message> = scrollable(playlist_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(5)
            .id(PLAYLIST_SCROLLABLE_ID.clone());

        col!(trow, playlist_scrollable).spacing(5)
    }

    fn build_statics_view<'a>(
        &self,
        dance_interpreter: &'a DanceInterpreter,
    ) -> Scrollable<'a, Message> {
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

        scrollable(row(statics).spacing(5))
            .direction(Direction::Horizontal(Scrollbar::new()))
            .spacing(5)
            .width(Length::Fill)
    }

    fn build_menu_bar<'a>(
        &self,
        dance_interpreter: &'a DanceInterpreter,
    ) -> MenuBar<'a, Message, Theme, Renderer> {
        let menu_tpl_1 = |items| Menu::new(items).max_width(150.0).offset(15.0).spacing(5.0);

        #[rustfmt::skip]
        let mb = menu_bar!
        (
            (
                label_message_button_shrink("File", Message::Noop),
                menu_tpl_1(
                    menu_items!(
                        (label_message_button_fill("Open Playlist File", Message::OpenPlaylist))
                        (label_message_button_fill("Exit", Message::WindowClosed(self.id.unwrap())))
                    )
                )
                .spacing(5.0)
            )
            (
                label_message_button_shrink("Edit", Message::Noop),
                menu_tpl_1(
                    menu_items!(
                        (label_message_button_fill("Reload Statics", Message::ReloadStatics))
                        (label_message_button_fill("Add blank song", Message::AddBlankSong(RelativeOffset::END)))
                    )
                )
                .spacing(5.0)
            )
            (
                label_message_button_shrink("SongWindow", Message::Noop),
                menu_tpl_1(
                    menu_items!(
                        (labeled_message_checkbox("Show Thumbnails", dance_interpreter.song_window.enable_image, Message::EnableImage))
                        (labeled_message_checkbox("Show Next Dance", dance_interpreter.song_window.enable_next_dance, Message::EnableNextDance))
                    )
                )
                .spacing(5.0)
            )
        )
        .spacing(5.0)
        .draw_path(menu::DrawPath::Backdrop)
            .style(|theme:&iced::Theme, status: Status | menu::Style{
                path_border: Border{
                    radius: Radius::new(6.0),
                    ..Default::default()
                },
                ..primary(theme, status)
            });

        mb
    }
}

fn label_message_button_fill(label: &str, message: Message) -> button::Button<Message> {
    label_message_button(label, message).width(Length::Fill)
}

fn label_message_button_shrink(label: &str, message: Message) -> button::Button<Message> {
    label_message_button(label, message).width(Length::Shrink)
}

fn label_message_button(label: &str, message: Message) -> button::Button<Message> {
    button(text(label).align_y(Vertical::Center))
        .padding([4, 8])
        .style(button::secondary)
        .on_press(message)
}

fn material_icon_message_button(icon_id: &str, message: Message) -> button::Button<Message> {
    button(material_icon(icon_id))
        .padding([4, 8])
        .style(button::secondary)
        .on_press(message)
        .width(Length::Shrink)
}

fn labeled_message_checkbox(
    label: &str,
    checked: bool,
    message: fn(bool) -> Message,
) -> checkbox::Checkbox<Message> {
    checkbox(label, checked)
        .on_toggle(message)
        .width(Length::Fill)
    //.style(checkbox::secondary)
}

fn separator() -> quad::Quad {
    quad::Quad {
        quad_color: Color::from([0.5; 3]).into(),
        quad_border: Border {
            radius: Radius::new(2.0),
            ..Default::default()
        },
        inner_bounds: InnerBounds::Ratio(1.0, 0.2),
        height: Length::Fixed(5.0),
        width: Length::Fill,
        ..Default::default()
    }
}
