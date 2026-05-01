use crate::dataloading::dataprovider::song_data_provider::{
    SongChange, SongDataEdit, SongDataProvider, SongDataSource,
};
use crate::traktor_api::{TRAKTOR_SERVER_DEFAULT_ADDR, TraktorNextMode, TraktorSyncMode};
use crate::ui::widget::dynamic_text_input::DynamicTextInput;
use crate::ui::widget::suggestion_text_input;
use crate::ui::widget::suggestion_text_input::SuggestionTextInput;
use crate::ui::{material_icon, material_icon_sized};
use crate::{DanceInterpreter, Message, Window};
use iced::advanced::Widget;
use iced::alignment::Vertical;
use iced::border::Radius;
use iced::widget::scrollable::{Direction, RelativeOffset, Scrollbar};
use iced::widget::space::horizontal;
use iced::widget::{
    Button, Column, Container, Row, Scrollable, Space, button, checkbox, column as col, container,
    pick_list, radio, row, scrollable, text,
};
use iced::{
    Alignment, Animation, Border, Color, Element, Font, Length, Pixels, Renderer, Size, Theme,
    animation, font, window,
};
use iced_aw::style::{Status, menu_bar::primary};
use iced_aw::widget::InnerBounds;
use iced_aw::{Menu, MenuBar, iced_aw_font, menu, menu_bar, menu_items, quad};
use network_interface::Addr::V4;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

pub struct ConfigWindow {
    pub id: window::Id,
    pub closed: bool,
    pub size: Size,
    pub enable_autoscroll: bool,
    pub sidebar_state: Animation<bool>,
    pub server_address_text: String,
    pub theme: Theme,
    server_address_presets: suggestion_text_input::State<String>,
}

pub static PLAYLIST_SCROLLABLE_ID: LazyLock<iced::widget::Id> =
    LazyLock::new(iced::widget::Id::unique);

impl Window for ConfigWindow {
    fn new(id: window::Id) -> Self {
        Self {
            id,
            closed: false,
            size: Size::default(),

            enable_autoscroll: true,
            sidebar_state: Animation::new(false)
                .duration(Duration::from_millis(100))
                .easing(animation::Easing::EaseInOut),
            server_address_presets: suggestion_text_input::State::default(),
            server_address_text: String::new(),
            theme: Theme::Dark,
        }
    }

    fn on_resize(&mut self, size: Size) {
        self.size = size;
    }

    fn on_close(&mut self) {
        self.closed = true;
    }

    fn is_closed(&self) -> bool {
        self.closed
    }
}

impl ConfigWindow {
    pub fn view<'a>(&'a self, dance_interpreter: &'a DanceInterpreter) -> Element<'a, Message> {
        let top_bar = self.build_top_bar(dance_interpreter);
        let playlist_view = self.build_playlist_view(dance_interpreter);
        let statics_view = self.build_statics_view(dance_interpreter);

        if self.sidebar_state.value() || self.sidebar_state.is_animating(Instant::now()) {
            let side_bar = self
                .build_server_sidebar(dance_interpreter)
                .width(self.sidebar_state.interpolate(0.0, 400.0, Instant::now()));
            let main_content = row![col![top_bar, playlist_view], side_bar];
            col![main_content, statics_view].spacing(5).into()
        } else {
            col![top_bar, playlist_view, statics_view].into()
        }
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

    fn build_server_sidebar<'a>(
        &'a self,
        dance_interpreter: &'a DanceInterpreter,
    ) -> Container<'a, Message> {
        let sync_options = vec![
            TraktorSyncMode::None,
            TraktorSyncMode::Relative,
            TraktorSyncMode::AbsoluteByNumber,
            TraktorSyncMode::AbsoluteByName,
        ];

        let next_options = vec![
            TraktorNextMode::None,
            TraktorNextMode::DeckByPosition,
            TraktorNextMode::DeckByNumber,
            TraktorNextMode::PlaylistByNumber,
            TraktorNextMode::PlaylistByName,
        ];

        container(
            col![
                text("Server Settings").size(24),
                labeled_message_checkbox(
                    "Enable Server",
                    dance_interpreter.data_provider.traktor_provider.is_enabled,
                    Message::TraktorEnableServer,
                ),
                col![
                    text("Server Address: "),
                    self.build_network_interface_combo_box(dance_interpreter)
                ],
                labeled_message_checkbox(
                    "Enable Debug Logging",
                    dance_interpreter
                        .data_provider
                        .traktor_provider
                        .debug_logging,
                    Message::TraktorEnableDebugLogging,
                ),
                label_message_button_fill_opt(
                    "Reset Connection",
                    dance_interpreter
                        .data_provider
                        .traktor_provider
                        .is_enabled
                        .then_some(Message::TraktorReconnect)
                ),
                col![
                    text("Sync Mode"),
                    pick_list(
                        sync_options.clone(),
                        Some(dance_interpreter.data_provider.traktor_provider.sync_mode),
                        Message::TraktorSetSyncMode
                    )
                ]
                .align_x(Alignment::Center),
                col![
                    text("Next Song Mode"),
                    pick_list(
                        next_options.clone(),
                        Some(dance_interpreter.data_provider.traktor_provider.next_mode),
                        Message::TraktorSetNextMode
                    )
                ]
                .align_x(Alignment::Center),
                col![
                    text("Next Song Mode (Fallback)"),
                    pick_list(
                        next_options.clone(),
                        Some(
                            dance_interpreter
                                .data_provider
                                .traktor_provider
                                .next_mode_fallback
                        ),
                        Message::TraktorSetNextModeFallback
                    )
                ]
                .align_x(Alignment::Center)
            ]
            .align_x(Alignment::Center)
            .spacing(10)
            .padding(10),
        )
        .height(Length::Fill)
        .style(|t| {
            container::Style::default().background(t.extended_palette().background.weakest.color)
        })
    }

    fn build_playlist_view(&'_ self, dance_interpreter: &DanceInterpreter) -> Column<'_, Message> {
        let trow: Row<_> = row![
            text!("#").width(Length::Fixed(24.0)),
            text!("Title").width(Length::Fill),
            text!("Artist").width(Length::Fill),
            text!("Dance").width(Length::Fill),
            Space::new().width(Length::Fill).height(Length::Shrink),
            Space::new()
                .width(Length::Fixed(10.0))
                .height(Length::Shrink),
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
                Space::new()
                    .width(Length::Fixed(24.0))
                    .height(Length::Shrink)
                    .into()
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
                    Space::new().width(Length::Fill).height(Length::Shrink),
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

    fn build_top_bar<'a>(&self, dance_interpreter: &'a DanceInterpreter) -> Row<'a, Message> {
        row![
            self.build_menu_bar(dance_interpreter),
            horizontal(),
            material_icon_sized_message_button(
                "right_panel_open",
                20.0,
                Message::Sidebar(SidebarMessage::Toggle)
            )
            .padding([0, 4])
        ]
    }

    fn build_menu_bar<'a>(
        &self,
        dance_interpreter: &'a DanceInterpreter,
    ) -> MenuBar<'a, Message, Theme, Renderer> {
        let menu_tpl_1 = |items| Menu::new(items).max_width(150.0).offset(15.0).spacing(5.0);

        #[rustfmt::skip]
        let mb = menu_bar!
        (
            (
                label_message_button_shrink("File", Message::Noop),
                menu_tpl_1(
                    menu_items!(
                        (label_message_button_fill("Open Playlist File", Message::OpenPlaylist)),
                        (label_message_button_fill("Exit", Message::WindowClosed(self.id))),
                    )
                )
                .spacing(5.0)
            ),
            (
                label_message_button_shrink("Edit", Message::Noop),
                menu_tpl_1(
                    menu_items!(
                        (labeled_message_checkbox("Autoscroll", self.enable_autoscroll, Message::EnableAutoscroll)),
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

        mb
    }

    fn build_network_interface_combo_box(
        &'_ self,
        dance_interpreter: &DanceInterpreter,
    ) -> SuggestionTextInput<'_, String, Message> {
        SuggestionTextInput::new(
            &self.server_address_presets,
            if !self.server_address_text.is_empty() {
                self.server_address_text.as_ref()
            } else {
                TRAKTOR_SERVER_DEFAULT_ADDR
            },
            Some(&dance_interpreter.data_provider.traktor_provider.address),
            Message::TraktorChangeAndSubmitAddress,
        )
        .on_open(Message::Sidebar(SidebarMessage::UpdateAddressPresets))
        .on_option_hovered(Message::TraktorChangeAddress)
        .on_input(Message::TraktorChangeAddress)
        .on_close(Message::TraktorSubmitAddress)
    }

    pub fn update_network_interface_selection(&mut self, song_data_provider: &SongDataProvider) {
        let mut detected_interfaces: Vec<String> =
            get_formatted_network_interfaces(song_data_provider)
                .into_iter()
                .map(|(_, _, formatted)| formatted)
                .collect();
        detected_interfaces.push(TRAKTOR_SERVER_DEFAULT_ADDR.to_owned());
        detected_interfaces.sort();

        self.server_address_presets = suggestion_text_input::State::with_selection(
            detected_interfaces,
            Some(&song_data_provider.traktor_provider.address.clone()),
        );
    }
}

#[derive(Debug, Clone)]
pub enum SidebarMessage {
    Toggle,
    UpdateAddressPresets,
}

fn get_network_interfaces() -> Vec<(String, String)> {
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

    interfaces
}

fn get_formatted_network_interfaces(
    song_data_provider: &'_ SongDataProvider,
) -> Vec<(String, String, String)> {
    let interfaces = get_network_interfaces();

    let original_addr = song_data_provider
        .traktor_provider
        .get_socket_addr()
        .unwrap_or(TRAKTOR_SERVER_DEFAULT_ADDR.parse().unwrap());
    let original_port = original_addr.port();

    interfaces
        .into_iter()
        .map(|(name, addr)| (name, addr.clone(), format!("{}:{}", addr, original_port)))
        .collect()
}

fn get_network_interface_menu(
    song_data_provider: &'_ SongDataProvider,
) -> Vec<Button<'_, Message>> {
    let interfaces = get_formatted_network_interfaces(song_data_provider);

    interfaces
        .into_iter()
        .map(|(name, addr, addr_with_port)| {
            label_message_button_fill(
                format!("{}: {}", name, addr),
                Message::TraktorChangeAndSubmitAddress(addr_with_port),
            )
        })
        .collect()
}

fn label_message_button_fill<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
) -> Button<'a, Message> {
    label_message_button(label, message).width(Length::Fill)
}

fn label_message_button_shrink<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
) -> Button<'a, Message> {
    label_message_button(label, message).width(Length::Shrink)
}

fn label_message_button<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
) -> Button<'a, Message> {
    button(text(label).align_y(Vertical::Center))
        .padding([4, 8])
        .style(button::secondary)
        .on_press(message)
}

fn submenu_button(label: &'_ str) -> Button<'_, Message, Theme, Renderer> {
    button(
        row![
            text(label).width(Length::Fill).align_y(Vertical::Center),
            iced_aw_font::right_open()
                .width(Length::Shrink)
                .align_y(Vertical::Center),
        ]
        .align_y(Alignment::Center),
    )
    .padding([4, 8])
    .style(button::text)
    .on_press(Message::Noop)
    .width(Length::Fill)
}

fn label_message_button_opt(label: &'_ str, message: Option<Message>) -> Button<'_, Message> {
    if let Some(message) = message {
        label_message_button(label, message)
    } else {
        button(text(label).align_y(Vertical::Center))
            .padding([4, 8])
            .style(button::secondary)
    }
}

fn label_message_button_fill_opt(label: &'_ str, message: Option<Message>) -> Button<'_, Message> {
    label_message_button_opt(label, message).width(Length::Fill)
}

fn material_icon_message_button(icon_id: &'_ str, message: Message) -> Button<'_, Message> {
    button(material_icon(icon_id))
        //.padding([4, 8])
        .style(button::secondary)
        .on_press(message)
        .width(Length::Shrink)
}

fn material_icon_sized_message_button(
    icon_id: &'_ str,
    size: impl Into<Pixels>,
    message: Message,
) -> Button<'_, Message> {
    button(material_icon_sized(icon_id, size))
        .style(button::secondary)
        .on_press(message)
        .width(Length::Shrink)
}

fn labeled_message_checkbox(
    label: &'_ str,
    checked: bool,
    message: fn(bool) -> Message,
) -> checkbox::Checkbox<'_, Message> {
    checkbox(checked)
        .label(label)
        .on_toggle(message)
        .width(Length::Fill)
    //.style(checkbox::secondary)
}

fn labeled_message_radio<T: Copy + Eq>(
    label: &'_ str,
    value: T,
    selection: T,
    message: fn(T) -> Message,
) -> radio::Radio<'_, Message> {
    radio(label, value, Some(selection), message).width(Length::Fill)
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
        checkbox(checked).label(label).width(Length::Fill)
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
