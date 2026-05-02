use crate::dataloading::dataprovider::song_data_provider::SongDataProvider;
use crate::traktor_api::{TRAKTOR_SERVER_DEFAULT_ADDR, TraktorNextMode, TraktorSyncMode};
use crate::ui::config_window::{
    label_message_button_fill_opt, labeled_message_checkbox, material_icon_sized_message_button,
};
use crate::ui::widget::canvas_toggle::CanvasToggle;
use crate::ui::widget::suggestion_text_input::SuggestionTextInput;
use crate::ui::widget::{power_button, restart_button, suggestion_text_input};
use crate::{DanceInterpreter, Message};
use iced::alignment::Vertical;
use iced::widget::{Row, canvas, column as col, container, pick_list, row, text};
use iced::{Alignment, Animation, Length, animation};
use network_interface::Addr::V4;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::time::Duration;

pub struct Sidebar {
    pub state: Animation<bool>,
    pub power_button_cache: canvas::Cache,
    pub restart_button_cache: canvas::Cache,
    server_address_presets: suggestion_text_input::State<String>,
    pub server_address_text: String,
}

#[derive(Debug, Clone)]
pub enum SidebarMessage {
    Toggle,
    UpdateAddressPresets,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            state: Animation::new(false)
                .duration(Duration::from_millis(100))
                .easing(animation::Easing::EaseInOut),
            power_button_cache: canvas::Cache::default(),
            restart_button_cache: canvas::Cache::default(),
            server_address_presets: suggestion_text_input::State::default(),
            server_address_text: String::new(),
        }
    }

    pub(crate) fn build<'a>(&'a self, dance_interpreter: &'a DanceInterpreter) -> Row<'a, Message> {
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

        row![
            material_icon_sized_message_button(
                if self.state.value() {
                    "right_panel_close"
                } else {
                    "right_panel_open"
                },
                20.0,
                Message::Sidebar(SidebarMessage::Toggle)
            )
            .padding([0, 4]),
            container(
                col![
                    text("Server Settings").size(24),
                    row![
                        col![
                            CanvasToggle::new(
                                dance_interpreter.data_provider.traktor_provider.is_enabled,
                                &self.power_button_cache
                            )
                            .on_toggle(Message::TraktorEnableServer)
                            .on_draw(power_button::draw),
                            text("Enable Server")
                        ],
                        col![
                            CanvasToggle::new(
                                dance_interpreter.data_provider.traktor_provider.is_enabled,
                                &self.restart_button_cache
                            )
                            .on_toggle(|_| Message::TraktorReconnect)
                            .on_draw(restart_button::draw),
                            text("Restart Server")
                        ]
                    ]
                    .spacing(10),
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
                container::Style::default()
                    .background(t.extended_palette().background.weakest.color)
            })
        ]
        .align_y(Vertical::Top)
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
