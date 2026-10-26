//! Toast notification bubble shown above the quick-show bar.

use crate::ui::config_window::Toast;
use crate::ui::theme::{self, sz};
use crate::ui::widgets::{material_symbol, material_symbol_sized};
use crate::Message;
use iced::alignment::Vertical;
use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length, Theme};

/// Builds the floating toast overlay, if a toast is active.
pub fn build(toast: &Option<Toast>) -> Option<Element<'_, Message>> {
    let toast = toast.as_ref()?;

    let close = button(
        material_symbol_sized("close", false, 14.0).style(|t: &Theme| {
            iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).toast_text),
            }
        }),
    )
    .padding(2)
    .style(|t: &Theme, s: button::Status| {
        let tk = theme::Tokens::of(t);
        let mut style = theme::btn_ghost(t, s);
        style.text_color = tk.toast_text;
        style
    })
    .on_press(Message::DismissToast);

    let bubble = container(
        row![
            material_symbol("check_circle", false)
                .size(16.0)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).toast_text),
                }),
            text(&toast.text)
                .size(sz::SMALL + 0.5)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).toast_text),
                }),
            close,
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([12, 16])
    .style(theme::toast_card);

    Some(
        container(bubble)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Vertical::Bottom)
            .padding(crate::ui::theme::pad4(0.0, 0.0, 92.0, 0.0))
            .into(),
    )
}
