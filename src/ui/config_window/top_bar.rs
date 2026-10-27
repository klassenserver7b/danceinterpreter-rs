//! Top bar: sidebar toggles, breadcrumb and the "open song window" action.

use crate::Message;
use crate::ui::config_window::{ConfigWindow, View};
use crate::ui::theme::{self, pad4, sz};
use crate::ui::widgets::{buttons, divider, icon_button_tooltip, push_right};
use iced::widget::{Column, column as col, container, row, text};
use iced::{Alignment, Element, Length, Theme};

pub fn build<'a>(config_window: &'a ConfigWindow) -> Element<'a, Message> {
    let view_name = match config_window.view {
        View::Playlist => "Playlist",
        View::Statics => "Statics",
    };

    let nav_toggle = icon_button_tooltip(
        if config_window.nav_open() {
            "menu_open"
        } else {
            "menu"
        },
        sz::ICON,
        Message::ToggleNavSidebar,
        "Toggle navigation sidebar (Ctrl+B)",
    );

    let breadcrumb = text(view_name)
        .size(sz::SMALL)
        .font(theme::body_font(iced::font::Weight::Medium))
        .style(theme::text_style(|tk| tk.text_soft));

    let open_playlist =
        buttons::ghost_labelled("folder", "Open playlist", None, Message::OpenPlaylist);

    let open_song_window = buttons::ghost_labelled(
        "desktop_windows",
        "Open song window",
        Some("open_in_new"),
        Message::FocusSongWindow,
    );

    // The icon mirrors the nav toggle: a plain panel while collapsed, and
    // while the rail is expanded the close glyph — same panel, arrow exactly
    // reversed, pointing in the collapse direction.
    let rail_toggle = icon_button_tooltip(
        if config_window.rail_open() {
            "right_panel_close"
        } else {
            "view_sidebar"
        },
        sz::ICON,
        Message::ToggleInfoRail,
        "Toggle info column (Ctrl+J)",
    );

    container(
        row![
            container(
                row![nav_toggle, breadcrumb]
                    .spacing(10)
                    .align_y(Alignment::Center)
            ),
            push_right(
                row![rail_toggle, open_playlist, open_song_window]
                    .spacing(10)
                    .align_y(Alignment::Center),
            ),
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center),
    )
    .padding(pad4(18.0, theme::PAGE_GUTTER, 18.0, theme::PAGE_GUTTER))
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .style(theme::page_bg)
    .into()
}

/// Vertical stack helper used by the shell (bar + hairline).
pub fn bar_with_divider<'a>(
    bar: Element<'a, Message>,
) -> Column<'a, Message, Theme, iced::Renderer> {
    col![bar, divider()]
}
