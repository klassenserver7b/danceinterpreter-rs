use crate::Message;
use iced::widget::{Container, Row, Space, button, container, row};
use iced::{Alignment, Color, Length, Theme};

pub fn build_item_row_container_styled(
    song_row: Row<Message>,
    highlight: bool,
    idx: usize,
    accent_color: Option<Color>,
) -> Container<Message> {
    let accent = accent_color;
    container(
        row![
            container(Space::new().width(4).height(40.0))
                .height(Length::Fixed(40.0))
                .style(move |_t: &Theme| {
                    container::Style::default().background(accent.unwrap_or(Color::TRANSPARENT))
                }),
            song_row,
        ]
        .align_y(Alignment::Center)
        .spacing(6),
    )
    .style(move |t| {
        let palette = t.extended_palette();
        let color = if idx.is_multiple_of(2) {
            palette.background.weakest.color
        } else {
            palette.background.weaker.color
        };

        let mut style = container::Style::default().background(color);

        if highlight {
            let accent = palette.primary.base.color;
            style = style
                .background(palette.primary.weak.color)
                .border(iced::Border {
                    color: accent,
                    width: 2.0,
                    radius: 4.0.into(),
                });
        }

        style
    })
    .padding([4, 6])
    .width(Length::Fill)
}

pub fn empty_folder_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let mut style = button::text(theme, status);
    style.text_color = palette.secondary.strong.color;
    style.background = Some(palette.background.weakest.color.into());
    style.border.radius = 12.0.into();
    style
}
