use crate::Window;
use crate::{DanceInterpreter, Message};
use iced::advanced::text::Shaping;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::text::LineHeight;
use iced::widget::{column, horizontal_space, image, row, stack, Text};
use iced::Size;
use iced::{window, Element, Length};

pub struct SongWindow {
    pub id: Option<window::Id>,
    pub size: Size,

    pub enable_image: bool,
    pub enable_next_dance: bool,
}

impl Window for SongWindow {
    fn on_create(&mut self, id: window::Id) {
        self.id = Some(id);
    }

    fn on_resize(&mut self, size: Size) {
        self.size = size;
    }
}

impl Default for SongWindow {
    fn default() -> Self {
        Self {
            id: None,
            size: Size::new(1.0, 1.0),
            enable_image: true,
            enable_next_dance: true,
        }
    }
}

impl SongWindow {
    pub fn view<'a>(&self, state: &'a DanceInterpreter) -> Element<'a, Message> {
        let Some(song_info) = state.data_provider.get_current_song_info() else {
            return horizontal_space().into();
        };

        let dance_size = self.size.height / 8.0;
        let title_size = self.size.height / 20.0;
        let artist_size = self.size.height / 25.0;
        let next_dance_size = self.size.height / 25.0;
        let next_dance_label_size = self.size.height / 30.0;

        let dance_spacing = self.size.height / 35.0;
        let song_spacing = self.size.height / 150.0;

        let cover_height = LineHeight::default().to_absolute(title_size.into())
            + song_spacing
            + LineHeight::default().to_absolute(artist_size.into());

        let text_dance = Text::new(&song_info.dance)
            .size(dance_size)
            .height(Length::Fill)
            .align_y(Vertical::Bottom)
            .shaping(Shaping::Advanced);

        let column_title_artist = column![
            Text::new(&song_info.title)
                .size(title_size)
                .shaping(Shaping::Advanced),
            Text::new(&song_info.artist)
                .size(artist_size)
                .shaping(Shaping::Advanced),
        ]
        .spacing(song_spacing);

        let row_bottom = (if self.enable_image {
            if let Some(image_handle) = song_info.album_art.as_ref() {
                row![
                    image(image_handle).height(cover_height),
                    column_title_artist
                ]
            } else {
                row![column_title_artist]
            }
        } else {
            row![column_title_artist]
        })
        .height(Length::Fill)
        .align_y(Vertical::Top)
        .spacing(song_spacing);

        let column_center = column![text_dance, row_bottom]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .spacing(dance_spacing);

        (if self.enable_next_dance {
            if let Some(next_song_info) = state.data_provider.get_next_song_info() {
                stack![
                    column_center,
                    row![
                        horizontal_space(),
                        column![
                            Text::new("Nächster Tanz")
                                .size(next_dance_label_size)
                                .height(Length::Fill)
                                .align_x(Horizontal::Right)
                                .align_y(Vertical::Bottom)
                                .shaping(Shaping::Advanced),
                            Text::new(&next_song_info.dance)
                                .size(next_dance_size)
                                .align_x(Horizontal::Right)
                                .align_y(Vertical::Bottom)
                                .shaping(Shaping::Advanced)
                        ]
                        .width(Length::Shrink)
                        .spacing(dance_spacing / 3.0)
                    ]
                ]
            } else {
                stack![column_center]
            }
        } else {
            stack![column_center]
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
