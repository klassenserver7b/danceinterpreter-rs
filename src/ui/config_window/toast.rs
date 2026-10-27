//! Toast notification bubble shown above the quick-show bar.

use crate::Message;
use crate::ui::config_window::Toast;
use crate::ui::config_window::show_bar::DOCK_CLEARANCE;
use crate::ui::theme::{self, pad4, sz};
use crate::ui::widgets::{EllipsisText, material_symbol, material_symbol_sized};
use iced::alignment::Vertical;
use iced::widget::{button, container, row};
use iced::{Alignment, Element, Length, Task, Theme};

/// Task that shows a toast with `text` — the standard way `update`
/// surfaces short-lived feedback.
pub fn task(text: impl Into<String>) -> Task<crate::Message> {
    Task::done(crate::Message::Toast(text.into()))
}

/// Widest the toast bubble may get; longer messages truncate with a `…`
/// instead of stretching the bubble past the window edges.
const BUBBLE_MAX_WIDTH: f32 = 560.0;

/// Builds the floating toast overlay, if a toast is active.
pub fn build(toast: &Option<Toast>) -> Option<Element<'_, Message>> {
    let toast = toast.as_ref()?;

    let close = button(
        material_symbol_sized("close", false, sz::ICON_XS)
            .style(theme::text_style(|tk| tk.toast_text)),
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
            EllipsisText::new(&toast.text)
                .width(Length::Shrink)
                .size(sz::CAPTION)
                .style(theme::text_style(|tk| tk.toast_text)),
            close,
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding([12, 16])
    .max_width(BUBBLE_MAX_WIDTH)
    .style(theme::toast_card);

    Some(
        container(bubble)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Vertical::Bottom)
            .padding(pad4(0.0, 0.0, DOCK_CLEARANCE, 0.0))
            .into(),
    )
}
