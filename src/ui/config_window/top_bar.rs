use crate::ui::config_window::sidebar::SidebarMessage;
use crate::ui::config_window::{
    ConfigWindow, label_message_button_fill, label_message_button_fill_opt,
    label_message_button_shrink, labeled_message_checkbox, material_icon_sized_message_button,
};
use crate::{DanceInterpreter, Message};
use iced::border::Radius;
use iced::widget::scrollable::RelativeOffset;
use iced::widget::space::horizontal;
use iced::widget::{Space, Stack, row, stack};
use iced::{Border, Length, Renderer, Theme};
use iced_aw::badge::Status;
use iced_aw::menu::primary;
use iced_aw::{Menu, menu, menu_bar, menu_items};

pub(crate) fn build<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Stack<'a, Message, Theme, Renderer> {
    let menu_tpl_1 = |items| Menu::new(items).max_width(150.0).offset(15.0).spacing(5.0);

    #[rustfmt::skip]
        let mb = menu_bar!
        (
            (
                label_message_button_shrink("File", Message::Noop),
                menu_tpl_1(
                    menu_items!(
                        (label_message_button_fill("Open Playlist File", Message::OpenPlaylist)),
                        (label_message_button_fill("Exit", Message::WindowClosed(config_window.id))),
                    )
                )
                .spacing(5.0)
            ),
            (
                label_message_button_shrink("Edit", Message::Noop),
                menu_tpl_1(
                    menu_items!(
                        (labeled_message_checkbox("Autoscroll", config_window.enable_autoscroll, Message::EnableAutoscroll)),
                        (label_message_button_fill("Reload Statics", Message::ReloadStatics)),
                        (label_message_button_fill("Add blank song", Message::AddBlankSong(RelativeOffset::END))),
                    )
                )
                .spacing(5.0)
            ),
            (
                label_message_button_shrink("SongWindow", Message::Noop),
                menu_tpl_1(
                    menu_items!(
                        (labeled_message_checkbox("Show Thumbnails", dance_interpreter.song_window.enable_image, Message::EnableImage)),
                        (labeled_message_checkbox("Show Next Dance", dance_interpreter.song_window.enable_next_dance, Message::EnableNextDance)),
                    )
                )
                .spacing(5.0)
            )
        )
        .spacing(5.0)
        .draw_path(menu::DrawPath::Backdrop)
            .style(|theme:&Theme, status: Status | menu::Style{
                path_border: Border{
                    radius: Radius::new(6.0),
                    ..Default::default()
                },
                ..primary(theme, status)
            });

    let playlist_button = label_message_button_fill_opt(
        "Playlist",
        if config_window.is_statics_view {
            Some(Message::ToggleStaticsView)
        } else {
            None
        },
    )
    .width(Length::Shrink);

    let statics_button = label_message_button_fill_opt(
        "Statics",
        if !config_window.is_statics_view {
            Some(Message::ToggleStaticsView)
        } else {
            None
        },
    )
    .width(Length::Shrink);

    let view_buttons = row![
        Space::new().width(Length::Fill),
        playlist_button,
        statics_button,
        Space::new().width(Length::Fill)
    ]
    .width(Length::Fill)
    .spacing(5);

    let sidebar_button = material_icon_sized_message_button(
        if config_window.sidebar.state.value() {
            "right_panel_close"
        } else {
            "right_panel_open"
        },
        20.0,
        Message::Sidebar(SidebarMessage::Toggle),
    )
    .padding([0, 4]);

    stack![
        row![mb, horizontal()].width(Length::Fill),
        row![horizontal(), view_buttons, horizontal()].width(Length::Fill),
        row![horizontal(), sidebar_button].width(Length::Fill)
    ]
    .width(Length::Fill)
}
