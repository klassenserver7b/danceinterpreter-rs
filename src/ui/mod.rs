use iced::widget::Text;
use iced::widget::text::Shaping;
use iced::widget::{container, text, tooltip};
use iced::{Element, Font, Length, Pixels, Renderer, Theme};

pub mod config_window;
pub mod song_window;
pub mod widget;

/// Wraps a widget in a tooltip with a readable styled container.
pub fn with_tooltip<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    label: impl text::IntoFragment<'a>,
) -> Element<'a, Message> {
    tooltip(
        content,
        container(text(label)).padding([4, 8]).style(|t: &Theme| {
            container::Style::default()
                .background(t.extended_palette().background.strong.color)
                .border(iced::border::rounded(4))
        }),
        tooltip::Position::Bottom,
    )
    .gap(4)
    .into()
}

pub fn material_icon_sized(
    id: &'_ str,
    filled: bool,
    size: impl Into<Pixels>,
) -> Text<'_, Theme, Renderer> {
    material_icon(id, filled).size(size)
}

pub fn material_icon(id: &'_ str, _filled: bool) -> Text<'_, Theme, Renderer> {
    Text::new(id)
        .font(Font::with_name("Material Symbols Outlined"))
        .shaping(Shaping::Advanced)
        .width(Length::Shrink)
}
