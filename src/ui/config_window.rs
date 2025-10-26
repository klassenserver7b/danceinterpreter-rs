use crate::dataloading::dataprovider::song_data_provider::{
    SongChange, SongDataEdit, SongDataSource,
};
use crate::traktor_api::{TraktorNextMode, TraktorSyncMode, TRAKTOR_SERVER_DEFAULT_ADDR};
use crate::ui::material_icon;
use crate::ui::widget::dynamic_text_input::DynamicTextInput;
use crate::{DanceInterpreter, Message, Window};
use iced::advanced::Widget;
use iced::alignment::Vertical;
use iced::border::Radius;
use iced::widget::scrollable::{Direction, RelativeOffset, Scrollbar};
use iced::widget::{
    button, checkbox, column as col, radio, row, scrollable, text, Button, Column, Row,
    Scrollable, Space,
};
use iced::{font, window, Border, Color, Element, Font, Length, Renderer, Size, Theme};
use iced_aw::iced_fonts::required::{icon_to_string, RequiredIcons};
use iced_aw::iced_fonts::REQUIRED_FONT;
use iced_aw::menu::Item;
use iced_aw::style::{menu_bar::primary, Status};
use iced_aw::widget::InnerBounds;
use iced_aw::{menu, menu_bar, menu_items, quad, Menu, MenuBar};
use network_interface::Addr::V4;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::sync::LazyLock;

#[derive(Default)]
pub struct ConfigWindow {
    pub id: Option<window::Id>,
    pub size: Size,
}

pub static PLAYLIST_SCROLLABLE_ID: LazyLock<scrollable::Id> = LazyLock::new(scrollable::Id::unique);

impl Window for ConfigWindow {
    fn on_create(&mut self, id: window::Id) {
        self.id = Some(id);
    }

    fn on_resize(&mut self, size: Size) {
        self.size = size;
    }
}

impl ConfigWindow {
    pub fn view<'a>(&'a self, dance_interpreter: &'a DanceInterpreter) -> Element<'a, Message> {
        let menu_bar = self.build_menu_bar(dance_interpreter);
        let playlist_view = self.build_playlist_view(dance_interpreter);
        let statics_view = self.build_statics_view(dance_interpreter);

        let content = col![menu_bar, playlist_view, statics_view];
        content.into()
    }

    fn get_play_state(
        &self,
        dance_interpreter: &DanceInterpreter,
        playlist_index: usize,
    ) -> (bool, bool, bool, bool) {
        let mut is_current = false;
        let mut is_next = false;
        let mut is_traktor = false;
        let is_played = dance_interpreter
            .data_provider
            .playlist_played
            .get(playlist_index)
            .copied()
            .unwrap_or(false);

        if let SongDataSource::Playlist(i) = dance_interpreter.data_provider.current {
            is_current = playlist_index == i;
            is_next = playlist_index == (i + 1);
        }

        if let Some(SongDataSource::Playlist(i)) = dance_interpreter.data_provider.next {
            is_next = playlist_index == i;
        }

        if matches!(
            dance_interpreter.data_provider.current,
            SongDataSource::Traktor
        ) && let Some(index) = dance_interpreter.data_provider.get_current_traktor_index()
        {
            is_traktor = playlist_index == index;
        }

        (is_current, is_next, is_traktor, is_played)
    }

    fn build_playlist_view(&'_ self, dance_interpreter: &DanceInterpreter) -> Column<'_, Message> {
        let trow: Row<_> = row![
            text!("#").width(Length::Fixed(24.0)),
            text!("Title").width(Length::Fill),
            text!("Artist").width(Length::Fill),
            text!("Dance").width(Length::Fill),
            Space::new(Length::Fill, Length::Shrink),
            Space::new(Length::Fixed(10.0), Length::Shrink),
        ]
        .spacing(5);

        let mut playlist_column: Column<'_, _, _, _> = col!().spacing(5);

        for (i, song) in dance_interpreter
            .data_provider
            .playlist_songs
            .iter()
            .enumerate()
        {
            let (is_current, is_next, is_traktor, is_played) =
                self.get_play_state(dance_interpreter, i);
            let icon: Element<Message> = if is_traktor {
                material_icon("agriculture")
                    .width(Length::Fixed(24.0))
                    .into()
            } else if is_current {
                material_icon("play_arrow")
                    .width(Length::Fixed(24.0))
                    .into()
            } else if is_next {
                material_icon("skip_next").width(Length::Fixed(24.0)).into()
            } else if is_played {
                material_icon("check").width(Length::Fixed(24.0)).into()
            } else {
                Space::new(Length::Fixed(24.0), Length::Shrink).into()
            };

            let song_row = row![
                icon,
                DynamicTextInput::<'_, Message>::new("Title", &song.title)
                    .width(Length::Fill)
                    .on_change(move |v| Message::SongDataEdit(i, SongDataEdit::Title(v))),
                DynamicTextInput::<'_, Message>::new("Artist", &song.artist)
                    .width(Length::Fill)
                    .on_change(move |v| Message::SongDataEdit(i, SongDataEdit::Artist(v))),
                DynamicTextInput::<'_, Message>::new("Dance", &song.dance)
                    .width(Length::Fill)
                    .on_change(move |v| Message::SongDataEdit(i, SongDataEdit::Dance(v))),
                row![
                    Space::new(Length::Fill, Length::Shrink),
                    material_icon_message_button(
                        "smart_display",
                        Message::SongChanged(SongChange::PlaylistAbsolute(i))
                    ),
                    material_icon_message_button(
                        "queue_play_next",
                        Message::SetNextSong(SongDataSource::Playlist(i))
                    ),
                    material_icon_message_button(
                        "delete",
                        Message::DeleteSong(SongDataSource::Playlist(i))
                    ),
                ]
                .spacing(5)
                .width(Length::Fill),
            ]
            .spacing(5);

            if !playlist_column.children().is_empty() {
                playlist_column = playlist_column.push(separator());
            }

            playlist_column = playlist_column.push(song_row);
        }

        let playlist_scrollable: Scrollable<'_, Message> = scrollable(playlist_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(5)
            .id(PLAYLIST_SCROLLABLE_ID.clone());

        col!(trow, playlist_scrollable).spacing(5)
    }

    fn build_statics_view<'a>(
        &self,
        dance_interpreter: &'a DanceInterpreter,
    ) -> Scrollable<'a, Message> {
        let bold_font = Font {
            family: font::Family::SansSerif,
            weight: font::Weight::Bold,
            stretch: font::Stretch::Normal,
            style: font::Style::Normal,
        };

        let btn_blank: Button<Message> =
            button(text("Blank").align_y(Vertical::Center).font(bold_font))
                .style(button::secondary)
                .on_press(Message::SongChanged(SongChange::Blank));
        let btn_traktor: Button<Message> =
            button(text("Traktor").align_y(Vertical::Center).font(bold_font))
                .style(button::secondary)
                .on_press(Message::SongChanged(SongChange::Traktor));
        let mut statics: Vec<Element<_>> = dance_interpreter
            .data_provider
            .statics
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                button(text(&s.dance).font(bold_font))
                    .style(button::secondary)
                    .on_press(Message::SongChanged(SongChange::StaticAbsolute(idx)))
                    .into()
            })
            .collect();

        statics.insert(0, btn_blank.into());
        statics.insert(1, btn_traktor.into());

        scrollable(row(statics).spacing(5))
            .direction(Direction::Horizontal(Scrollbar::new()))
            .spacing(5)
            .width(Length::Fill)
    }

    fn build_menu_bar<'a>(
        &self,
        dance_interpreter: &'a DanceInterpreter,
    ) -> MenuBar<'a, Message, Theme, Renderer> {
        let menu_tpl_1 = |items| Menu::new(items).max_width(150.0).offset(15.0).spacing(5.0);
        let menu_tpl_2 = |items| Menu::new(items).max_width(150.0).offset(0.0).spacing(5.0);

        #[rustfmt::skip]
        let mb = menu_bar!
        (
            (
                label_message_button_shrink("File", Message::Noop),
                menu_tpl_1(
                    menu_items!(
                        (label_message_button_fill("Open Playlist File", Message::OpenPlaylist))
                        (label_message_button_fill("Exit", Message::WindowClosed(self.id.unwrap())))
                    )
                )
                .spacing(5.0)
            )
            (
                label_message_button_shrink("Edit", Message::Noop),
                menu_tpl_1(
                    menu_items!(
                        (label_message_button_fill("Reload Statics", Message::ReloadStatics))
                        (label_message_button_fill("Add blank song", Message::AddBlankSong(RelativeOffset::END)))
                    )
                )
                .spacing(5.0)
            )
            (
                label_message_button_shrink("SongWindow", Message::Noop),
                menu_tpl_1(
                    menu_items!(
                        (labeled_message_checkbox("Show Thumbnails", dance_interpreter.song_window.enable_image, Message::EnableImage))
                        (labeled_message_checkbox("Show Next Dance", dance_interpreter.song_window.enable_next_dance, Message::EnableNextDance))
                    )
                )
                .spacing(5.0)
            )
            (
                label_message_button_shrink("Traktor", Message::Noop),
                menu_tpl_1(
                    menu_items!(
                        (labeled_message_checkbox("Enable Server", dance_interpreter.data_provider.traktor_provider.is_enabled, Message::TraktorEnableServer))
                        (
                            labeled_dynamic_text_input("Server Address", TRAKTOR_SERVER_DEFAULT_ADDR, dance_interpreter.data_provider.traktor_provider.address.as_str(),
                                Message::TraktorChangeAddress, Some(Message::TraktorSubmitAddress)),
                            menu_tpl_2(get_network_interface_menu(dance_interpreter))
                        )
                        (separator())
                        (labeled_message_checkbox_opt("Enable Debug Logging", dance_interpreter.data_provider.traktor_provider.debug_logging,
                            dance_interpreter.data_provider.traktor_provider.is_enabled.then_some(Message::TraktorEnableDebugLogging)))
                        (label_message_button_fill_opt("Reset Connection", dance_interpreter.data_provider.traktor_provider.is_enabled.then_some(Message::TraktorReconnect)))
                        (separator())
                        (
                            submenu_button("Sync Mode"),
                            menu_tpl_2(
                                menu_items!(
                                    (labeled_message_radio("None", true,
                                        Some(dance_interpreter.data_provider.traktor_provider.sync_mode.is_none()), |_| Message::TraktorSetSyncMode(None)))
                                    (labeled_message_radio("X Fader", TraktorSyncMode::Relative,
                                        dance_interpreter.data_provider.traktor_provider.sync_mode, |v| Message::TraktorSetSyncMode(Some(v))))
                                    (labeled_message_radio("By Track Number", TraktorSyncMode::AbsoluteByNumber,
                                        dance_interpreter.data_provider.traktor_provider.sync_mode, |v| Message::TraktorSetSyncMode(Some(v))))
                                    (labeled_message_radio("By Title / Artist", TraktorSyncMode::AbsoluteByName,
                                        dance_interpreter.data_provider.traktor_provider.sync_mode, |v| Message::TraktorSetSyncMode(Some(v))))
                                )
                            )
                        )
                        (
                            submenu_button("Next Song Mode"),
                            menu_tpl_2(
                                menu_items!(
                                    (labeled_message_radio("None", true,
                                        Some(dance_interpreter.data_provider.traktor_provider.next_mode.is_none()), |_| Message::TraktorSetNextMode(None)))
                                    (labeled_message_radio("From other Deck (by Position)", TraktorNextMode::DeckByPosition,
                                        dance_interpreter.data_provider.traktor_provider.next_mode, |v| Message::TraktorSetNextMode(Some(v))))
                                    (labeled_message_radio("From other Deck (by Track Number)", TraktorNextMode::DeckByNumber,
                                        dance_interpreter.data_provider.traktor_provider.next_mode, |v| Message::TraktorSetNextMode(Some(v))))
                                    (labeled_message_radio("From Playlist (by Track Number)", TraktorNextMode::PlaylistByNumber,
                                        dance_interpreter.data_provider.traktor_provider.next_mode, |v| Message::TraktorSetNextMode(Some(v))))
                                    (labeled_message_radio("From Playlist (by Title / Artist)", TraktorNextMode::PlaylistByName,
                                        dance_interpreter.data_provider.traktor_provider.next_mode, |v| Message::TraktorSetNextMode(Some(v))))
                                )
                            )
                        )
                        (
                            submenu_button("Next Song Mode (Fallback)"),
                            menu_tpl_2(
                                menu_items!(
                                    (labeled_message_radio("None", true,
                                        Some(dance_interpreter.data_provider.traktor_provider.next_mode_fallback.is_none()), |_| Message::TraktorSetNextModeFallback(None)))
                                    (labeled_message_radio("From other Deck (by Position)", TraktorNextMode::DeckByPosition,
                                        dance_interpreter.data_provider.traktor_provider.next_mode_fallback, |v| Message::TraktorSetNextModeFallback(Some(v))))
                                    (labeled_message_radio("From other Deck (by Track Number)", TraktorNextMode::DeckByNumber,
                                        dance_interpreter.data_provider.traktor_provider.next_mode_fallback, |v| Message::TraktorSetNextModeFallback(Some(v))))
                                    (labeled_message_radio("From Playlist (by Track Number)", TraktorNextMode::PlaylistByNumber,
                                        dance_interpreter.data_provider.traktor_provider.next_mode_fallback, |v| Message::TraktorSetNextModeFallback(Some(v))))
                                    (labeled_message_radio("From Playlist (by Title / Artist)", TraktorNextMode::PlaylistByName,
                                        dance_interpreter.data_provider.traktor_provider.next_mode_fallback, |v| Message::TraktorSetNextModeFallback(Some(v))))
                                )
                            )
                        )
                    )
                )
                .spacing(5.0)
            )
        )
        .spacing(5.0)
        .draw_path(menu::DrawPath::Backdrop)
            .style(|theme:&iced::Theme, status: Status | menu::Style{
                path_border: Border{
                    radius: Radius::new(6.0),
                    ..Default::default()
                },
                ..primary(theme, status)
            });

        mb
    }
}

fn label_message_button_fill<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
) -> button::Button<'a, Message> {
    label_message_button(label, message).width(Length::Fill)
}

fn label_message_button_shrink<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
) -> button::Button<'a, Message> {
    label_message_button(label, message).width(Length::Shrink)
}

fn label_message_button<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
) -> button::Button<'a, Message> {
    button(text(label).align_y(Vertical::Center))
        .padding([4, 8])
        .style(button::secondary)
        .on_press(message)
}

fn submenu_button(label: &'_ str) -> button::Button<'_, Message, iced::Theme, iced::Renderer> {
    button(
        row![
            text(label).width(Length::Fill).align_y(Vertical::Center),
            text(icon_to_string(RequiredIcons::CaretRightFill))
                .font(REQUIRED_FONT)
                .width(Length::Shrink)
                .align_y(Vertical::Center),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 8])
    .style(button::text)
    .on_press(Message::Noop)
    .width(Length::Fill)
}

fn label_message_button_opt(
    label: &'_ str,
    message: Option<Message>,
) -> button::Button<'_, Message> {
    if let Some(message) = message {
        label_message_button(label, message)
    } else {
        button(text(label).align_y(Vertical::Center))
            .padding([4, 8])
            .style(button::secondary)
    }
}

fn label_message_button_fill_opt(
    label: &'_ str,
    message: Option<Message>,
) -> button::Button<'_, Message> {
    label_message_button_opt(label, message).width(Length::Fill)
}

fn material_icon_message_button(icon_id: &'_ str, message: Message) -> button::Button<'_, Message> {
    button(material_icon(icon_id))
        .padding([4, 8])
        .style(button::secondary)
        .on_press(message)
        .width(Length::Shrink)
}

fn labeled_message_checkbox(
    label: &'_ str,
    checked: bool,
    message: fn(bool) -> Message,
) -> checkbox::Checkbox<'_, Message> {
    checkbox(label, checked)
        .on_toggle(message)
        .width(Length::Fill)
    //.style(checkbox::secondary)
}

fn labeled_message_radio<T: Copy + Eq>(
    label: &'_ str,
    value: T,
    selection: Option<T>,
    message: fn(T) -> Message,
) -> radio::Radio<'_, Message> {
    radio(label, value, selection, message).width(Length::Fill)
    //.style(checkbox::secondary)
}

fn labeled_message_checkbox_opt(
    label: &'_ str,
    checked: bool,
    message: Option<fn(bool) -> Message>,
) -> checkbox::Checkbox<'_, Message> {
    if let Some(message) = message {
        labeled_message_checkbox(label, checked, message)
    } else {
        checkbox(label, checked).width(Length::Fill)
        //.style(checkbox::secondary)
    }
}

fn labeled_dynamic_text_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    message: fn(String) -> Message,
    submit_message: Option<Message>,
) -> Column<'a, Message> {
    let mut input = DynamicTextInput::<Message>::new(placeholder, value)
        .width(Length::Fill)
        .on_change(message);

    if let Some(submit_message) = submit_message {
        input = input.on_submit(submit_message);
    }

    col!(text(label).width(Length::Fill), input,).width(Length::Fill)
}

fn separator() -> quad::Quad {
    quad::Quad {
        quad_color: Color::from([0.5; 3]).into(),
        quad_border: Border {
            radius: Radius::new(2.0),
            ..Default::default()
        },
        inner_bounds: InnerBounds::Ratio(1.0, 0.2),
        height: Length::Fixed(5.0),
        width: Length::Fill,
        ..Default::default()
    }
}

fn get_network_interface_menu(
    dance_interpreter: &'_ DanceInterpreter,
) -> Vec<Item<'_, Message, Theme, Renderer>> {
    let mut interfaces = vec![("any".to_owned(), "0.0.0.0".to_owned())];

    if let Ok(network_interfaces) = NetworkInterface::show() {
        for i in network_interfaces {
            for addr in i.addr {
                let V4(ipv4_addr) = addr else {
                    continue;
                };

                interfaces.push((i.name.clone(), ipv4_addr.ip.to_string()));
            }
        }
    }

    let original_addr = dance_interpreter
        .data_provider
        .traktor_provider
        .get_socket_addr()
        .unwrap_or(TRAKTOR_SERVER_DEFAULT_ADDR.parse().unwrap());
    let original_port = original_addr.port();

    let interfaces = interfaces
        .into_iter()
        .map(|(name, addr)| (name, addr.clone(), format!("{}:{}", addr, original_port)));

    interfaces
        .into_iter()
        .map(|(name, addr, addr_with_port)| {
            Item::new(label_message_button_fill(
                format!("{}: {}", name, addr),
                Message::TraktorChangeAndSubmitAddress(addr_with_port),
            ))
        })
        .collect()
}
