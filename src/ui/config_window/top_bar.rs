use crate::ui::config_window::ConfigWindow;
use crate::ui::config_window::sidebar::SidebarMessage;
use crate::ui::widgets::buttons::{
    label_message_button_fill, label_message_button_fill_opt, label_message_button_shrink,
    material_symbol_sized_message_button,
};
use crate::ui::widgets::labeled_message_checkbox;
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, Message};
use iced::alignment::Vertical;
use iced::border::Radius;
use iced::widget::scrollable::RelativeOffset;
use iced::widget::space::horizontal;
use iced::widget::{Space, Stack, pick_list, row, stack, text};
use iced::{Border, Length, Renderer, Theme};
use iced_aw::style::Status;
use iced_aw::style::menu_bar::primary;
use iced_aw::{Menu, menu, menu_bar, menu_items, number_input};

pub(crate) fn build<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Stack<'a, Message, Theme, Renderer> {
    let menu_tpl = |items| Menu::new(items).max_width(160.0).offset(15.0).spacing(5.0);
    let menu_tpl_wide = |items| Menu::new(items).max_width(256.0).offset(15.0).spacing(5.0);

    #[rustfmt::skip]
        let mb = menu_bar!
        (
            (
                label_message_button_shrink("File", Message::Noop),
                menu_tpl(
                    menu_items!(
                        (label_message_button_fill("Open Playlist File", Message::OpenPlaylist)),
                        (label_message_button_fill("Exit", Message::WindowClosed(config_window.id))),
                    )
                )
                .spacing(5.0)
            ),
            (
                label_message_button_shrink("Edit", Message::Noop),
                menu_tpl_wide(
                    menu_items!(
                        (labeled_message_checkbox("Autoscroll", config_window.enable_autoscroll, Message::EnableAutoscroll)),
                        (labeled_message_checkbox(
                            "Follow system theme",
                            config_window.follow_system_theme,
                            Message::EnableFollowSystemTheme
                        )),
                        (labeled_message_checkbox(
                            "Search",
                            config_window.search_visible,
                            |_| Message::ToggleSearch
                        )),
                        (row![
                            text("Theme").width(Length::Shrink),
                            Space::new().width(Length::Fill),
                            pick_list(
                                Theme::ALL,
                                Some(config_window.theme.clone()),
                                Message::SetConfigTheme
                            )
                        ]
                        .align_y(Vertical::Center)
                        .spacing(5.0)
                        .width(Length::Fill)),
                        (label_message_button_fill("Reload Statics", Message::ReloadStatics)),
                        (label_message_button_fill("Add blank song", Message::AddBlankSong(RelativeOffset::END))),
                    )
                )
                .spacing(5.0)
            ),
            (
                label_message_button_shrink("SongWindow", Message::Noop),
                menu_tpl(
                    menu_items!(
                        (row![
                            text("Scale").width(Length::Fill),
                            number_input(&dance_interpreter.song_window.scale, 0.5..=3.0 , Message::ChangeSongWindowScale)
                                .step(0.1)
                                .width(Length::Fill)
                        ].align_y(Vertical::Center)
                        .spacing(5.0)
                        .width(Length::Fill)),
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
        with_tooltip(playlist_button, "Show Playlist"),
        with_tooltip(statics_button, "Show Statics"),
        Space::new().width(Length::Fill)
    ]
    .width(Length::Fill)
    .spacing(5);

    let sidebar_button = with_tooltip(
        material_symbol_sized_message_button(
            if config_window.sidebar.state.value() {
                "right_panel_close"
            } else {
                "right_panel_open"
            },
            false,
            20.0,
            Message::Sidebar(SidebarMessage::Toggle),
        )
        .padding([0, 4]),
        if config_window.sidebar.state.value() {
            "Expand Traktor Panel"
        } else {
            "Collapse Traktor Panel"
        },
    );

    stack![
        row![mb, horizontal()].width(Length::Fill),
        row![horizontal(), view_buttons, horizontal()].width(Length::Fill),
        row![horizontal(), sidebar_button].width(Length::Fill)
    ]
    .width(Length::Fill)
}
