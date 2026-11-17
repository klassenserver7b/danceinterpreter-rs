use iced::Element;
use iced::widget::{container, text, tooltip};

pub mod config_window;
pub mod song_window;
pub mod theme;

pub mod widgets;

/// Wraps a widget in a tooltip with a readable styled container.
pub fn with_tooltip<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    label: impl text::IntoFragment<'a>,
) -> Element<'a, Message> {
    tooltip(
        content,
        container(text(label).size(theme::sz::SMALL))
            .padding([4, 8])
            .style(theme::tooltip_style),
        tooltip::Position::Bottom,
    )
    .gap(4)
    .into()
}
