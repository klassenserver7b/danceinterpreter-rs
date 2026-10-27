//! The "Traktor" card in the right rail: connection status and the DJ
//! integration settings.

use crate::dataloading::dataprovider::DataProvider;
use crate::traktor_api::{
    TRAKTOR_SERVER_DEFAULT_ADDR, TraktorMessage, TraktorNextMode, TraktorSyncMode,
};
use crate::ui::config_window::led_color;
use crate::ui::config_window::source_tag_right;
use crate::ui::theme::{self, sz};
use crate::ui::widgets::canvas::canvas_toggle::CanvasToggle;
use crate::ui::widgets::canvas::{power_button, restart_button};
use crate::ui::widgets::suggestion_text_input::SuggestionTextInput;
use crate::ui::widgets::{buttons, dot, eyebrow, heading, icon_muted};
use crate::ui::with_tooltip;
use crate::{ConfigWindow, DanceInterpreter, Message};
use iced::widget::{Column, column as col, container, pick_list, row, text};
use iced::{Alignment, Element, Length, Theme};
use network_interface::Addr::V4;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

/// Sync modes offered in the Traktor card.
const SYNC_MODES: [TraktorSyncMode; 3] = [
    TraktorSyncMode::Relative,
    TraktorSyncMode::AbsoluteByNumber,
    TraktorSyncMode::AbsoluteByName,
];

/// Next-song modes offered in the Traktor card.
const NEXT_MODES: [TraktorNextMode; 4] = [
    TraktorNextMode::DeckByPosition,
    TraktorNextMode::DeckByNumber,
    TraktorNextMode::PlaylistByNumber,
    TraktorNextMode::PlaylistByName,
];

pub fn build<'a>(
    dance_interpreter: &'a DanceInterpreter,
    config_window: &'a ConfigWindow,
) -> Element<'a, Message> {
    let provider = &dance_interpreter.data_provider.traktor_provider;
    let enabled = provider.enabled();
    let connection = provider.get_connection_state();

    let state_tag = match (enabled, connection) {
        (false, _) => "OFF",
        (true, crate::traktor_api::ConnectionState::Connected) => "LIVE",
        (true, _) => "IDLE",
    };

    let header = row![
        icon_muted("agriculture", sz::ICON_SM),
        heading("Traktor", sz::PANEL_TITLE, iced::font::Weight::Semibold)
            .style(theme::text_style(|tk| tk.text)),
        source_tag_right(state_tag),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let server_controls = row![
        col![
            with_tooltip(
                CanvasToggle::new(enabled, &config_window.traktor_power_cache)
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
                CanvasToggle::new(enabled, &config_window.traktor_restart_cache)
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
        build_network_interface_combo_box(dance_interpreter, config_window),
    )
    .width(Length::Fill);

    let mut body = col![
        header,
        connection_card(dance_interpreter),
        container(server_controls)
            .width(Length::Fill)
            .align_x(Alignment::Center),
        address
    ]
    .spacing(12);

    body = body.push(
        row![
            col![
                text("Follow DJ playback")
                    .size(sz::CAPTION)
                    .font(theme::body_font(iced::font::Weight::Medium))
                    .style(theme::text_style(|tk| tk.text)),
                text("Automatically update the display")
                    .size(sz::TINY)
                    .style(theme::text_style(|tk| tk.muted)),
            ]
            .spacing(3)
            .width(Length::Fill),
            toggler_row(provider.sync),
        ]
        .align_y(Alignment::Center),
    );

    body = body.push(mode_pick_list(
        "Sync Mode",
        &SYNC_MODES,
        provider.sync_mode,
        |m| Message::Traktor(TraktorMessage::SetSyncMode(m)),
    ));

    body = body.push(mode_pick_list(
        "Next Song Mode",
        &NEXT_MODES,
        provider.next_mode,
        |m| Message::Traktor(TraktorMessage::SetNextMode(m)),
    ));

    body = body.push(mode_pick_list(
        "Next Song Mode (Fallback)",
        &NEXT_MODES,
        provider.next_mode_fallback,
        |m| Message::Traktor(TraktorMessage::SetNextModeFallback(m)),
    ));

    if provider.get_song_info().is_some() {
        body = body.push(buttons::secondary(
            "Add current song",
            Message::AddTraktorSong,
        ));
    }

    body = body.push(
        text("Traktor connects to this address through the QML API plugin bundled in traktor-api/. The cover-loader sidecar delivers cover art.")
            .size(sz::MICRO)
            .style(theme::text_style(|tk| tk.faint)),
    );

    container(body)
        .padding(15)
        .width(Length::Fill)
        .style(theme::card)
        .into()
}

/// LED + summary, plus the cover loader's IP when present.
fn connection_card<'a>(dance_interpreter: &'a DanceInterpreter) -> Element<'a, Message> {
    let provider = &dance_interpreter.data_provider.traktor_provider;
    let connection = provider.get_connection_state();

    let led = dot(8.0, move |t| led_color(t, connection));

    let status_row = row![
        led,
        text(connection.summary())
            .size(sz::SMALL)
            .style(theme::text_style(|tk| tk.text_soft)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let mut column = col![status_row].spacing(4).width(Length::Fill);

    if let Some(cover_loader) = provider.cover_loader_addr {
        column = column.push(
            text(format!("Cover loader \u{B7} {}", cover_loader.ip()))
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

fn toggler_row<'a>(checked: bool) -> Element<'a, Message> {
    iced::widget::toggler(checked)
        .on_toggle(|b| Message::Traktor(TraktorMessage::EnableSync(b)))
        .style(theme::toggler_style)
        .width(Length::Shrink)
        .into()
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
            .padding(theme::INPUT_PAD)
            .style(theme::pick_list_style)
            .menu_style(theme::pick_list_menu_style),
    )
    .into()
}

fn build_network_interface_combo_box<'a>(
    dance_interpreter: &'a DanceInterpreter,
    config_window: &'a ConfigWindow,
) -> SuggestionTextInput<'a, String, Message> {
    // Always show the provider's configured address (the default until
    // something was submitted).
    let provider = &dance_interpreter.data_provider.traktor_provider;
    let value = if provider.address.is_empty() {
        TRAKTOR_SERVER_DEFAULT_ADDR
    } else {
        provider.address.as_str()
    };

    SuggestionTextInput::new(
        &config_window.traktor_address_presets,
        value,
        Some(&provider.address),
        |s| Message::Traktor(TraktorMessage::ChangeAndSubmitAddress(s)),
    )
    .on_open(Message::UpdateTraktorPresets)
    .on_option_hovered(|s| Message::Traktor(TraktorMessage::ChangeAddress(s)))
    .on_input(|s| Message::Traktor(TraktorMessage::ChangeAddress(s)))
    .on_close(Message::Traktor(TraktorMessage::SubmitAddress))
    .size(sz::BASE)
    .padding([7, 9])
    .input_style(theme::input_boxed)
    .menu_style(theme::pick_list_menu_style)
}

/// Address presets for the server address input: every local interface with
/// the configured port, plus the default address.
pub fn address_presets(data_provider: &DataProvider) -> Vec<String> {
    let mut detected_interfaces: Vec<String> = get_formatted_network_interfaces(data_provider);
    detected_interfaces.push(TRAKTOR_SERVER_DEFAULT_ADDR.to_owned());
    detected_interfaces.sort();
    detected_interfaces
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
