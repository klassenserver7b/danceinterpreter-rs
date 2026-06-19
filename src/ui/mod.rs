use iced::widget::{container, text, tooltip};
use iced::{Element, Theme};

pub mod config_window;
pub mod song_window;
pub mod widget;
pub mod widgets;

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
