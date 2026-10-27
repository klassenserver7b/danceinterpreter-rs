//! Toast notification bubble shown above the quick-show bar.

use crate::Message;
use crate::ui::config_window::Toast;
use crate::ui::theme::{self, pad4, sz};
use crate::ui::widgets::{material_symbol, material_symbol_sized};
use iced::alignment::Vertical;
use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length, Theme};

/// Builds the floating toast overlay, if a toast is active.
pub fn build(toast: &Option<Toast>) -> Option<Element<'_, Message>> {
    let toast = toast.as_ref()?;

    let close = button(
        material_symbol_sized("close", false, 14.0).style(theme::text_style(|tk| tk.toast_text)),
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
                .size(sz::ICON)
                .style(theme::text_style(|tk| tk.toast_text)),
            text(&toast.text)
                .size(sz::CAPTION)
                .style(theme::text_style(|tk| tk.toast_text)),
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
            .padding(pad4(
                0.0,
                0.0,
                crate::ui::config_window::show_bar::DOCK_CLEARANCE,
                0.0,
            ))
            .into(),
    )
}
