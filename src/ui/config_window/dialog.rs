use crate::dataloading::dataprovider::ItemSource;
use crate::{DanceInterpreter, Message};
use iced::alignment::Alignment;
use iced::widget::{Space, button, column as col, container, mouse_area, opaque, row, stack, text};
use iced::{Color, Element, Length, Theme};

pub enum DialogState {
    Delete(ItemSource),
    MergeStatic { old_name: String, new_name: String },
}

pub fn build_action_dialog<'a>(
    title: String,
    target: String,
    confirm_text: String,
    confirm_style: impl Fn(&Theme, button::Status) -> button::Style + 'static,
    cancel_msg: Message,
    confirm_msg: Message,
) -> Element<'a, Message> {
    let cancel_msg_clone = cancel_msg.clone();
    let card = container(
        col![
            text(title).size(20),
            text(target),
            row![
                button(text("Cancel").align_x(Alignment::Center))
                    .padding([4, 8])
                    .style(button::secondary)
                    .on_press(cancel_msg)
                    .width(Length::Fill),
                button(text(confirm_text).align_x(Alignment::Center))
                    .padding([4, 8])
                    .style(confirm_style)
                    .on_press(confirm_msg)
                    .width(Length::Fill),
            ]
            .spacing(10),
        ]
        .spacing(15)
        .width(Length::Fixed(360.0)),
    )
    .padding(20)
    .style(|t: &Theme| {
        let palette = t.extended_palette();
        container::Style::default()
            .background(palette.background.base.color)
            .border(iced::Border {
                color: palette.background.strong.color,
                width: 1.0,
                radius: 8.0.into(),
            })
    });

    let backdrop = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_t: &Theme| {
                container::Style::default().background(Color::from_rgba(0.0, 0.0, 0.0, 0.5))
            }),
    )
    .on_press(cancel_msg_clone);

    let centered_card = container(opaque(card))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

    opaque(stack![backdrop, centered_card])
}

pub fn build_dialog<'a>(
    state: &'a DialogState,
    dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    match state {
        DialogState::Delete(source) => {
            let target_name = match source {
                ItemSource::Playlist(i) => dance_interpreter
                    .data_provider
                    .playlist()
                    .get(*i)
                    .map(|item| item.song.title.clone())
                    .unwrap_or_default(),
                ItemSource::Static(name) => dance_interpreter
                    .data_provider
                    .statics()
                    .get(name)
                    .map(|s| s.name.clone())
                    .unwrap_or(name.clone()),
                _ => String::new(),
            };
            build_action_dialog(
                "Delete item?".to_string(),
                target_name,
                "Delete".to_string(),
                button::danger,
                Message::CancelDelete,
                Message::ConfirmDelete,
            )
        }
        DialogState::MergeStatic { old_name, new_name } => build_action_dialog(
            "Merge Statics?".to_string(),
            format!(
                "A static named '{}' already exists.\nMerge '{}' into it?",
                new_name, old_name
            ),
            "Merge".to_string(),
            button::primary,
            Message::CancelDialog,
            Message::ConfirmStaticMerge(old_name.clone(), new_name.clone()),
        ),
    }
}
