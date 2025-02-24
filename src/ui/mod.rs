use iced::widget::text::Shaping;
use iced::widget::Text;
use iced::{Font, Length, Renderer, Theme};

pub mod config_window;
pub mod song_window;
pub mod widget;

pub fn material_icon(id: &str) -> Text<Theme, Renderer> {
    Text::new(id)
        .font(Font::with_name("Material Symbols Outlined"))
        .shaping(Shaping::Advanced)
        .width(Length::Shrink)
}
