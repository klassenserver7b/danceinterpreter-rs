//! Slide-in "DJ integration" panel (server settings for the Traktor
//! websocket integration), restyled for the redesign.

use crate::dataloading::dataprovider::DataProvider;
use crate::traktor_api::{
    TRAKTOR_SERVER_DEFAULT_ADDR, TraktorMessage, TraktorNextMode, TraktorSyncMode,
};
use crate::ui::config_window::source_tag_right;
use crate::ui::theme::{self, sz};
use crate::ui::widgets::canvas::canvas_toggle::CanvasToggle;
use crate::ui::widgets::canvas::{power_button, restart_button};
use crate::ui::widgets::suggestion_text_input;
use crate::ui::widgets::suggestion_text_input::SuggestionTextInput;
use crate::ui::widgets::{buttons, eyebrow, heading, icon_muted, labeled_message_toggler};
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, Message};
use iced::alignment::Vertical;
use iced::widget::{Column, Space, canvas, column as col, container, pick_list, row, text};
use iced::{Alignment, Animation, Element, Length, Theme, animation};
use network_interface::Addr::V4;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

/// Full width of the slide-in DJ integration panel when expanded.
pub const PANEL_WIDTH: f32 = 400.0;

/// Sync modes offered in the DJ panel.
const SYNC_MODES: [TraktorSyncMode; 3] = [
    TraktorSyncMode::Relative,
    TraktorSyncMode::AbsoluteByNumber,
    TraktorSyncMode::AbsoluteByName,
];

/// Next-song modes offered in the DJ panel.
const NEXT_MODES: [TraktorNextMode; 4] = [
    TraktorNextMode::DeckByPosition,
    TraktorNextMode::DeckByNumber,
    TraktorNextMode::PlaylistByNumber,
    TraktorNextMode::PlaylistByName,
];

pub struct Sidebar {
    pub state: Animation<bool>,
    pub power_button_cache: canvas::Cache,
    pub restart_button_cache: canvas::Cache,
    server_address_presets: suggestion_text_input::State<String>,
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
                .duration(crate::ui::config_window::SLIDE_DURATION)
                .easing(animation::Easing::EaseInOut),
            power_button_cache: canvas::Cache::default(),
            restart_button_cache: canvas::Cache::default(),
            server_address_presets: suggestion_text_input::State::default(),
        }
    }

    pub fn build<'a>(
        &'a self,
        dance_interpreter: &'a DanceInterpreter,
        width: f32,
    ) -> Element<'a, Message> {
        let provider = &dance_interpreter.data_provider.traktor_provider;

        let header = row![
            icon_muted("link", sz::ICON),
            heading("DJ integration", sz::H2, iced::font::Weight::Medium)
                .style(theme::text_style(|tk| tk.text)),
            source_tag_right("TRAKTOR PRO"),
            buttons::ghost_icon(
                "close",
                false,
                sz::ICON,
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

        let address = captioned_control(
            "Server Address",
            self.build_network_interface_combo_box(dance_interpreter),
        )
        .width(Length::Fill);

        let sync_mode = mode_pick_list("Sync Mode", &SYNC_MODES, provider.sync_mode, |m| {
            Message::Traktor(TraktorMessage::SetSyncMode(m))
        });

        let next_mode = mode_pick_list("Next Song Mode", &NEXT_MODES, provider.next_mode, |m| {
            Message::Traktor(TraktorMessage::SetNextMode(m))
        });

        let next_mode_fallback = mode_pick_list(
            "Next Song Mode (Fallback)",
            &NEXT_MODES,
            provider.next_mode_fallback,
            |m| Message::Traktor(TraktorMessage::SetNextModeFallback(m)),
        );

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
                        .style(theme::text_style(|tk| tk.faint)),
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
        let mut detected_interfaces: Vec<String> = get_formatted_network_interfaces(data_provider);
        detected_interfaces.push(TRAKTOR_SERVER_DEFAULT_ADDR.to_owned());
        detected_interfaces.sort();

        self.server_address_presets = suggestion_text_input::State::with_selection(
            detected_interfaces,
            Some(&data_provider.traktor_provider.address),
        );
    }

    fn build_network_interface_combo_box(
        &'_ self,
        dance_interpreter: &DanceInterpreter,
    ) -> SuggestionTextInput<'_, String, Message> {
        // Always show the provider's configured address (the default until
        // something was submitted).
        let configured = &dance_interpreter.data_provider.traktor_provider.address;
        let value = if configured.is_empty() {
            TRAKTOR_SERVER_DEFAULT_ADDR
        } else {
            configured.as_str()
        };

        SuggestionTextInput::new(
            &self.server_address_presets,
            value,
            Some(&dance_interpreter.data_provider.traktor_provider.address),
            |s| Message::Traktor(TraktorMessage::ChangeAndSubmitAddress(s)),
        )
        .on_open(Message::Sidebar(SidebarMessage::UpdateAddressPresets))
        .on_option_hovered(|s| Message::Traktor(TraktorMessage::ChangeAddress(s)))
        .on_input(|s| Message::Traktor(TraktorMessage::ChangeAddress(s)))
        .on_close(Message::Traktor(TraktorMessage::SubmitAddress))
        .size(sz::BASE)
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

        let led =
            crate::ui::widgets::dot(8.0, move |t| theme::led_color(t, client_connection_state));

        let summary = client_connection_state.summary();

        let mut column = col![
            row![
                led,
                text(summary)
                    .size(sz::SMALL)
                    .style(theme::text_style(|tk| tk.text_soft))
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
            column = column.push(
                text(label)
                    .size(sz::MICRO)
                    .style(theme::text_style(|tk| tk.faint)),
            );
        }

        container(column)
            .padding(10)
            .width(Length::Fill)
            .style(|t: &Theme| theme::inset_card(t, theme::r::MD))
            .into()
    }
}

fn control_caption(label: &'static str) -> Element<'static, Message> {
    eyebrow(label).into()
}

/// A control with its micro caption stacked above it.
fn captioned_control<'a>(
    caption: &'static str,
    control: impl Into<Element<'a, Message>>,
) -> Column<'a, Message> {
    col![control_caption(caption), control.into()].spacing(6)
}

/// Captioned mode pick list (sync mode, next-song mode and its fallback).
fn mode_pick_list<'a, T>(
    caption: &'static str,
    options: &'a [T],
    selected: T,
    on_selected: fn(T) -> Message,
) -> Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
{
    captioned_control(
        caption,
        pick_list(options, Some(selected), on_selected)
            .width(Length::Fill)
            .text_size(sz::BASE)
            .padding([5, 8])
            .style(theme::pick_list_style)
            .menu_style(theme::pick_list_menu_style),
    )
    .into()
}

/// Rounded panel like the info rail, plus a drop shadow; while the panel is
/// collapsed (animated width ~0) paint nothing, so no border hairline
/// survives at the window edge.
fn dj_panel_style(t: &iced::Theme, width: f32) -> container::Style {
    let mut style = theme::panel_collapsible(t, width);
    if width >= 1.0 {
        style.shadow = iced::Shadow {
            color: theme::Tokens::of(t).shadow,
            offset: iced::Vector::new(-4.0, 0.0),
            blur_radius: 18.0,
        };
    }
    style
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

fn get_formatted_network_interfaces(data_provider: &DataProvider) -> Vec<String> {
    let original_addr = data_provider
        .traktor_provider
        .get_socket_addr()
        .unwrap_or(TRAKTOR_SERVER_DEFAULT_ADDR.parse().unwrap());
    let original_port = original_addr.port();

    get_network_interfaces()
        .into_iter()
        .map(|(_, addr)| format!("{}:{}", addr, original_port))
        .collect()
}
