use iced::widget::button::Status;
use iced::widget::button::Style;
use iced::{Background, Color, Element, Length, Theme, widget::Button};

pub fn color_swatch<'a, Message: Clone + 'a>(
    color: Color,
    on_press: Message,
) -> Element<'a, Message, Theme, iced::Renderer> {
    Button::new(
        iced::widget::Space::new()
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0)),
    )
    .on_press(on_press)
    .style(move |_theme: &Theme, _status: Status| Style {
        background: Some(Background::Color(color)),
        border: iced::Border {
            color: Color::WHITE,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Style::default()
    })
    .padding(4)
    .into()
}
