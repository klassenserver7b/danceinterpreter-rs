//! Slide-in "DJ integration" panel (server settings for the Traktor
//! websocket integration), restyled for the redesign.

use crate::dataloading::dataprovider::DataProvider;
use crate::traktor_api::{
    ConnectionState, TRAKTOR_SERVER_DEFAULT_ADDR, TraktorMessage, TraktorNextMode, TraktorSyncMode,
};
use crate::ui::config_window::source_tag;
use crate::ui::theme::{self, sz};
use crate::ui::widget::canvas_toggle::CanvasToggle;
use crate::ui::widget::suggestion_text_input::SuggestionTextInput;
use crate::ui::widget::{power_button, restart_button, suggestion_text_input};
use crate::ui::widgets::{buttons, heading, labeled_message_toggler, material_symbol};
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, Message};
use iced::alignment::Vertical;
use iced::widget::{Space, canvas, column as col, container, pick_list, row, text};
use iced::{Alignment, Animation, Element, Length, Theme, animation, border, color};
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

    pub fn build<'a>(
        &'a self,
        dance_interpreter: &'a DanceInterpreter,
        width: f32,
    ) -> Element<'a, Message> {
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

        let provider = &dance_interpreter.data_provider.traktor_provider;

        let header = row![
            material_symbol("link", false)
                .size(16.0)
                .style(|t: &Theme| iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).muted),
                }),
            heading("DJ integration", sz::H2, iced::font::Weight::Medium).style(|t| {
                iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).text),
                }
            }),
            container(source_tag("TRAKTOR PRO"))
                .width(Length::Fill)
                .align_x(Alignment::End),
            buttons::ghost_icon(
                "close",
                false,
                16.0,
                Message::Sidebar(SidebarMessage::Toggle)
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let server_controls = row![
            col![
                with_tooltip(
                    CanvasToggle::new(provider.enabled(), &self.power_button_cache)
                        .on_toggle(|b| Message::Traktor(TraktorMessage::EnableServer(b)))
                        .on_draw(power_button::draw),
                    "Toggle DJ server"
                ),
                control_caption("Enable Server"),
            ]
            .align_x(Alignment::Center)
            .spacing(6),
            col![
                with_tooltip(
                    CanvasToggle::new(provider.enabled(), &self.restart_button_cache)
                        .on_toggle(|_| Message::Traktor(TraktorMessage::Reconnect))
                        .on_draw(restart_button::draw),
                    "Restart DJ server"
                ),
                control_caption("Restart"),
            ]
            .align_x(Alignment::Center)
            .spacing(6),
        ]
        .spacing(24);

        let address = col![
            control_caption("Server Address"),
            self.build_network_interface_combo_box(dance_interpreter),
        ]
        .spacing(6)
        .width(Length::Fill);

        let sync_mode = col![
            control_caption("Sync Mode"),
            pick_list(
                sync_options,
                Some(provider.sync_mode),
                |m| Message::Traktor(TraktorMessage::SetSyncMode(m))
            )
            .width(Length::Fill)
            .text_size(sz::SMALL + 1.0)
            .padding([5, 8])
            .style(theme::pick_list_style)
            .menu_style(theme::pick_list_menu_style),
        ]
        .spacing(6);

        let next_mode = col![
            control_caption("Next Song Mode"),
            pick_list(next_options.clone(), Some(provider.next_mode), |m| {
                Message::Traktor(TraktorMessage::SetNextMode(m))
            })
            .width(Length::Fill)
            .text_size(sz::SMALL + 1.0)
            .padding([5, 8])
            .style(theme::pick_list_style)
            .menu_style(theme::pick_list_menu_style),
        ]
        .spacing(6);

        let next_mode_fallback = col![
            control_caption("Next Song Mode (Fallback)"),
            pick_list(next_options, Some(provider.next_mode_fallback), |m| {
                Message::Traktor(TraktorMessage::SetNextModeFallback(m))
            })
            .width(Length::Fill)
            .text_size(sz::SMALL + 1.0)
            .padding([5, 8])
            .style(theme::pick_list_style)
            .menu_style(theme::pick_list_menu_style),
        ]
        .spacing(6);

        container(
            col![
                header,
                container(server_controls).width(Length::Fill).align_x(Alignment::Center),
                Self::build_client_status(dance_interpreter),
                address,
                labeled_message_toggler(
                    "Enable Debug Logging",
                    provider.debug_logging,
                    |b| Message::Traktor(TraktorMessage::EnableDebugLogging(b))
                ),
                labeled_message_toggler("Enable Sync", provider.sync, |b| {
                    Message::Traktor(TraktorMessage::EnableSync(b))
                }),
                sync_mode,
                next_mode,
                next_mode_fallback,
                Space::new().height(Length::Fill),
                col![
                    text("Traktor connects to this address through the QML API plugin bundled in traktor-api/. The cover-loader sidecar delivers cover art.")
                        .size(sz::MICRO)
                        .style(|t: &Theme| iced::widget::text::Style {
                            color: Some(theme::Tokens::of(t).faint),
                        }),
                ]
            ]
            .spacing(14)
            .padding([18, 16])
            .width(Length::Fill),
        )
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .clip(true)
        .style(move |t: &Theme| dj_panel_style(t, width))
        .align_y(Vertical::Top)
        .into()
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
        .size(sz::SMALL + 1.0)
        .padding([7, 9])
        .input_style(theme::input_boxed)
        .menu_style(theme::pick_list_menu_style)
    }

    /// A small LED + label showing whether a Traktor client is
    /// connected, listing client's IP available.
    fn build_client_status<'a>(dance_interpreter: &'a DanceInterpreter) -> Element<'a, Message> {
        let client_connection_state = dance_interpreter
            .data_provider
            .traktor_provider
            .get_connection_state();

        let led = container(Space::new())
            .width(Length::Fixed(8.0))
            .height(Length::Fixed(8.0))
            .style(move |t: &iced::Theme| container::Style {
                background: Some(theme::led_color(t, client_connection_state).into()),
                border: border::rounded(4.0),
                ..container::Style::default()
            });

        let summary = match client_connection_state {
            ConnectionState::Connected => "Traktor & Cover Loader connected",
            ConnectionState::Traktor => "Traktor connected",
            ConnectionState::CoverLoader => "Cover Loader connected",
            ConnectionState::Disconnected => "No client connected",
        };

        let mut column = col![
            row![
                led,
                text(summary)
                    .size(sz::SMALL)
                    .style(|t: &Theme| iced::widget::text::Style {
                        color: Some(theme::Tokens::of(t).text_soft),
                    })
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(4)
        .width(Length::Fill);

        let label = dance_interpreter
            .data_provider
            .traktor_provider
            .cover_loader_addr
            .map(|addr| addr.ip().to_string());

        if let Some(label) = label {
            column = column.push(text(label).size(sz::MICRO).style(|t: &Theme| {
                iced::widget::text::Style {
                    color: Some(theme::Tokens::of(t).faint),
                }
            }));
        }

        container(column)
            .padding(10)
            .width(Length::Fill)
            .style(|t: &Theme| {
                let tk = theme::Tokens::of(t);
                container::Style::default()
                    .background(tk.panel_deep)
                    .border(iced::Border {
                        color: tk.border,
                        width: 1.0,
                        radius: 5.0.into(),
                    })
            })
            .into()
    }
}

fn control_caption(label: &'static str) -> Element<'static, Message> {
    text(label)
        .size(sz::MICRO)
        .font(theme::body_font(iced::font::Weight::Semibold))
        .style(|t: &Theme| iced::widget::text::Style {
            color: Some(theme::Tokens::of(t).muted_head),
        })
        .into()
}

/// Rounded panel like the info rail. While the panel is collapsed (animated
/// width ~0) paint nothing, so no border hairline survives at the window edge.
fn dj_panel_style(t: &iced::Theme, width: f32) -> container::Style {
    if width < 1.0 {
        return container::Style::default();
    }
    let tk = theme::Tokens::of(t);
    container::Style::default()
        .background(tk.panel)
        .border(iced::Border {
            color: tk.border,
            width: 1.0,
            radius: 10.0.into(),
        })
        .shadow(iced::Shadow {
            color: color!(0x000000, 0.25),
            offset: iced::Vector::new(-4.0, 0.0),
            blur_radius: 18.0,
        })
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
