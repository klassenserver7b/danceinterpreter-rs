use iced::widget::Text;
use iced::widget::text::Shaping;
use iced::{Font, Length, Pixels, Renderer, Theme};

pub mod config_window;
pub mod song_window;
pub mod widget;

pub fn material_icon_sized(id: &'_ str, size: impl Into<Pixels>) -> Text<'_, Theme, Renderer> {
    Text::new(id)
        .font(Font::with_name("Material Symbols Outlined"))
        .size(size)
        .shaping(Shaping::Advanced)
        .width(Length::Shrink)
}

pub fn material_icon(id: &'_ str) -> Text<'_, Theme, Renderer> {
    Text::new(id)
        .font(Font::with_name("Material Symbols Outlined"))
        .shaping(Shaping::Advanced)
        .width(Length::Shrink)
}
