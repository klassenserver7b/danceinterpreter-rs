use crate::dataloading::dataprovider::DataProvider;
use crate::traktor_api::{
    ConnectionState, TRAKTOR_SERVER_DEFAULT_ADDR, TraktorMessage, TraktorNextMode, TraktorSyncMode,
};
use crate::ui::widget::canvas_toggle::CanvasToggle;
use crate::ui::widget::suggestion_text_input::SuggestionTextInput;
use crate::ui::widget::{power_button, restart_button, suggestion_text_input};
use crate::ui::widgets::labeled_message_toggler;
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, Message};
use iced::alignment::Vertical;
use iced::widget::{
    Column, Container, Space, canvas, column as col, container, pick_list, row, text,
};
use iced::{Alignment, Animation, Length, animation, border};
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

    pub fn build<'a>(&'a self, dance_interpreter: &'a DanceInterpreter) -> Container<'a, Message> {
        let sync_options = vec![
            TraktorSyncMode::Relative,
            TraktorSyncMode::AbsoluteByNumber,
            TraktorSyncMode::AbsoluteByName,
        ];

        let next_options = vec![
            TraktorNextMode::DeckByPosition,
            TraktorNextMode::DeckByNumber,
            TraktorNextMode::PlaylistByNumber,
            TraktorNextMode::PlaylistByName,
        ];

        container(
            col![
                text("Server Settings").size(24),
                row![
                    col![
                        with_tooltip(
                            CanvasToggle::new(
                                dance_interpreter.data_provider.traktor_provider.enabled(),
                                &self.power_button_cache
                            )
                            .on_toggle(|b| Message::Traktor(TraktorMessage::EnableServer(b)))
                            .on_draw(power_button::draw),
                            "Toggle Traktor Server"
                        ),
                        text("Enable Server")
                    ]
                    .align_x(Alignment::Center),
                    col![
                        with_tooltip(
                            CanvasToggle::new(
                                dance_interpreter.data_provider.traktor_provider.enabled(),
                                &self.restart_button_cache
                            )
                            .on_toggle(|_| Message::Traktor(TraktorMessage::Reconnect))
                            .on_draw(restart_button::draw),
                            "Restart Traktor Server"
                        ),
                        text("Restart Server")
                    ]
                    .align_x(Alignment::Center)
                ]
                .spacing(10),
                Self::build_client_status(dance_interpreter),
                col![
                    text("Server Address: "),
                    self.build_network_interface_combo_box(dance_interpreter)
                ],
                labeled_message_toggler(
                    "Enable Debug Logging",
                    dance_interpreter
                        .data_provider
                        .traktor_provider
                        .debug_logging,
                    |b| Message::Traktor(TraktorMessage::EnableDebugLogging(b))
                ),
                labeled_message_toggler(
                    "Enable Sync",
                    dance_interpreter.data_provider.traktor_provider.sync,
                    |b| Message::Traktor(TraktorMessage::EnableSync(b)),
                ),
                col![
                    text("Sync Mode"),
                    pick_list(
                        sync_options.clone(),
                        Some(dance_interpreter.data_provider.traktor_provider.sync_mode),
                        |m| Message::Traktor(TraktorMessage::SetSyncMode(m))
                    )
                    .width(Length::Fill)
                ]
                .align_x(Alignment::Center),
                col![
                    text("Next Song Mode"),
                    pick_list(
                        next_options.clone(),
                        Some(dance_interpreter.data_provider.traktor_provider.next_mode),
                        |m| Message::Traktor(TraktorMessage::SetNextMode(m))
                    )
                    .width(Length::Fill)
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
                        |m| Message::Traktor(TraktorMessage::SetNextModeFallback(m))
                    )
                    .width(Length::Fill)
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
        .align_y(Vertical::Top)
    }

    pub fn update_network_interface_selection(&mut self, data_provider: &DataProvider) {
        let mut detected_interfaces: Vec<String> = get_formatted_network_interfaces(data_provider)
            .into_iter()
            .map(|(_, _, formatted)| formatted)
            .collect();
        detected_interfaces.push(TRAKTOR_SERVER_DEFAULT_ADDR.to_owned());
        detected_interfaces.sort();

        self.server_address_presets = suggestion_text_input::State::with_selection(
            detected_interfaces,
            Some(&data_provider.traktor_provider.address.clone()),
        );
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
            |s| Message::Traktor(TraktorMessage::ChangeAndSubmitAddress(s)),
        )
        .on_open(Message::Sidebar(SidebarMessage::UpdateAddressPresets))
        .on_option_hovered(|s| Message::Traktor(TraktorMessage::ChangeAddress(s)))
        .on_input(|s| Message::Traktor(TraktorMessage::ChangeAddress(s)))
        .on_close(Message::Traktor(TraktorMessage::SubmitAddress))
    }

    /// A small LED + label showing whether a Traktor client is
    /// connected, listing client's IP available.
    fn build_client_status<'a>(dance_interpreter: &DanceInterpreter) -> Column<'a, Message> {
        let client_connection_state = dance_interpreter
            .data_provider
            .traktor_provider
            .get_connection_state();

        let led = container(Space::new())
            .width(Length::Fixed(12.0))
            .height(Length::Fixed(12.0))
            .style(move |t: &iced::Theme| {
                let palette = t.extended_palette();
                let color = match client_connection_state {
                    ConnectionState::Connected => palette.success.base.color,
                    ConnectionState::CoverLoader | ConnectionState::Traktor => {
                        palette.warning.base.color
                    }
                    ConnectionState::Disconnected => palette.background.strong.color,
                };
                container::Style {
                    background: Some(color.into()),
                    border: border::rounded(6.0),
                    ..container::Style::default()
                }
            });

        let summary = match client_connection_state {
            ConnectionState::Connected => "Traktor & Cover Loader connected",
            ConnectionState::Traktor => "Traktor connected",
            ConnectionState::CoverLoader => "Cover Loader connected",
            ConnectionState::Disconnected => "No client connected",
        };

        let mut column = col![
            row![led, text(summary)]
                .spacing(8)
                .align_y(Alignment::Center)
        ]
        .spacing(4)
        .width(Length::Fill)
        .align_x(Alignment::Center);

        let label = dance_interpreter
            .data_provider
            .traktor_provider
            .cover_loader_addr
            .map(|addr| addr.ip().to_string());

        if let Some(label) = label {
            column = column.push(text(label).size(12));
        }

        column
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
    data_provider: &'_ DataProvider,
) -> Vec<(String, String, String)> {
    let interfaces = get_network_interfaces();

    let original_addr = data_provider
        .traktor_provider
        .get_socket_addr()
        .unwrap_or(TRAKTOR_SERVER_DEFAULT_ADDR.parse().unwrap());
    let original_port = original_addr.port();

    interfaces
        .into_iter()
        .map(|(name, addr)| (name, addr.clone(), format!("{}:{}", addr, original_port)))
        .collect()
}
