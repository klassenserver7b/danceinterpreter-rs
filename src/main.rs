mod async_utils;
mod dataloading;
mod macros;
mod traktor_api;
mod ui;

use crate::async_utils::run_subscription_with;
use crate::dataloading::dataprovider::song_data_provider::{
    SongChange, SongDataEdit, SongDataProvider, SongDataSource,
};
use crate::dataloading::id3tagreader::read_song_info_from_filepath;
use crate::dataloading::m3uloader::load_tag_data_from_m3u;
use crate::dataloading::songinfo::SongInfo;
use crate::traktor_api::{
    ServerMessage, StateUpdate, TraktorNextMode, TraktorSyncAction, TraktorSyncMode,
};
use crate::ui::config_window::{ConfigWindow, PLAYLIST_SCROLLABLE_ID};
use crate::ui::song_window::SongWindow;
use iced::advanced::graphics::image::image_rs::ImageFormat;
use iced::keyboard::key::Named;
use iced::keyboard::{on_key_press, Key, Modifiers};
use iced::widget::scrollable::{AbsoluteOffset, RelativeOffset};
use iced::widget::{horizontal_space, scrollable};
use iced::window::icon::from_file_data;
use iced::{exit, window, Element, Size, Subscription, Task, Theme};
use iced_aw::iced_fonts::REQUIRED_FONT_BYTES;
use rfd::FileDialog;
use std::path::PathBuf;

fn main() -> iced::Result {
    iced::daemon(
        DanceInterpreter::title,
        DanceInterpreter::update,
        DanceInterpreter::view,
    )
    .theme(DanceInterpreter::theme)
    .font(REQUIRED_FONT_BYTES)
    .subscription(DanceInterpreter::subscription)
    .run_with(DanceInterpreter::new)
}

pub trait Window {
    fn on_create(&mut self, id: window::Id);
    fn on_resize(&mut self, size: Size);
}

#[derive(Default)]
struct DanceInterpreter {
    config_window: ConfigWindow,
    song_window: SongWindow,

    data_provider: SongDataProvider,
}

#[derive(Debug, Clone)]
pub enum Message {
    Noop,

    WindowOpened(window::Id),
    WindowResized((window::Id, Size)),
    WindowClosed(window::Id),

    ToggleFullscreen,
    SetFullscreen(bool),

    OpenPlaylist,
    ReloadStatics,
    AddSong(SongInfo),
    DeleteSong(SongDataSource),
    ScrollBy(f32),
    AddBlankSong(RelativeOffset),

    FileDropped(PathBuf),
    SongChanged(SongChange),
    SongDataEdit(usize, SongDataEdit),
    SetNextSong(SongDataSource),

    EnableImage(bool),
    EnableNextDance(bool),

    TraktorMessage(traktor_api::ServerMessage),
    TraktorSetSyncMode(Option<TraktorSyncMode>),
    TraktorSetNextMode(Option<TraktorNextMode>),
    TraktorSetNextModeFallback(Option<TraktorNextMode>),
    TraktorEnableServer(bool),
    TraktorChangeAddress(String),
    TraktorSubmitAddress,
    TraktorChangeAndSubmitAddress(String),
    TraktorEnableDebugLogging(bool),
    TraktorReconnect,
}

impl DanceInterpreter {
    pub fn new() -> (Self, Task<Message>) {
        let mut tasks = Vec::new();

        let icon = from_file_data(
            match dark_light::detect() {
                dark_light::Mode::Dark => include_bytes!(res_file!("icon_dark.png")),
                _ => include_bytes!(res_file!("icon_light.png")),
            },
            Some(ImageFormat::Png),
        )
        .ok();

        let (config_window, cw_opened) = Self::open_window(window::Settings {
            platform_specific: Self::get_platform_specific(),
            icon: icon.clone(),
            ..Default::default()
        });
        let (song_window, sw_opened) = Self::open_window(window::Settings {
            platform_specific: Self::get_platform_specific(),
            icon: icon.clone(),
            ..Default::default()
        });

        let state = Self {
            config_window,
            song_window,

            ..Self::default()
        };

        tasks.push(cw_opened);
        tasks.push(sw_opened);

        tasks.push(
            iced::font::load(include_bytes!(res_file!("symbols.ttf"))).map(|_| Message::Noop),
        );

        tasks.push(Task::done(Message::ReloadStatics));

        (state, Task::batch(tasks))
    }

    fn open_window<T: Window + Default>(settings: window::Settings) -> (T, Task<Message>) {
        let (id, open) = window::open(settings);

        let mut window = T::default();
        window.on_create(id);

        (window, open.map(Message::WindowOpened))
    }

    fn get_platform_specific() -> window::settings::PlatformSpecific {
        #[cfg(target_os = "linux")]
        return window::settings::PlatformSpecific {
            application_id: "danceinterpreter".to_string(),
            ..Default::default()
        };

        #[cfg(not(target_os = "linux"))]
        return Default::default();
    }

    pub fn title(&self, window_id: window::Id) -> String {
        if self.config_window.id == Some(window_id) {
            "Config Window".to_string()
        } else if self.song_window.id == Some(window_id) {
            "Song Window".to_string()
        } else {
            String::new()
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if self.config_window.id == Some(window_id) {
            self.config_window.view(self)
        } else if self.song_window.id == Some(window_id) {
            self.song_window.view(self)
        } else {
            horizontal_space().into()
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(_) => ().into(),
            Message::WindowResized((window_id, size)) => {
                if self.config_window.id == Some(window_id) {
                    self.config_window.on_resize(size);
                } else if self.song_window.id == Some(window_id) {
                    self.song_window.on_resize(size);
                }

                ().into()
            }
            Message::WindowClosed(window_id) => {
                if self.config_window.id == Some(window_id) {
                    self.config_window.id = None;

                    match self.song_window.id {
                        Some(window_id) => window::close(window_id),
                        None => exit(),
                    }
                } else if self.song_window.id == Some(window_id) {
                    self.song_window.id = None;

                    match self.config_window.id {
                        Some(window_id) => window::close(window_id),
                        None => exit(),
                    }
                } else {
                    ().into()
                }
            }
            Message::ToggleFullscreen => {
                let Some(song_window_id) = self.song_window.id else {
                    return ().into();
                };

                window::get_mode(song_window_id)
                    .map(|mode| Message::SetFullscreen(mode != window::Mode::Fullscreen))
            }
            Message::SetFullscreen(fullscreen) => {
                let Some(song_window_id) = self.song_window.id else {
                    return ().into();
                };

                window::change_mode(
                    song_window_id,
                    if fullscreen {
                        window::Mode::Fullscreen
                    } else {
                        window::Mode::Windowed
                    },
                )
            }

            Message::OpenPlaylist => {
                // Open playlist file
                let file = FileDialog::new()
                    .add_filter("Playlist", &["m3u", "m3u8"])
                    .add_filter("Any(*)", &["*"])
                    .set_title("Select playlist file")
                    .set_directory(
                        dirs::audio_dir().unwrap_or(dirs::home_dir().unwrap_or(PathBuf::from("."))),
                    )
                    .pick_file();

                let Some(file) = file else {
                    return ().into();
                };
                println!("Selected file: {:?}", file);

                let Ok(playlist) = load_tag_data_from_m3u(&file) else {
                    return ().into();
                };

                self.data_provider.set_vec(playlist);

                ().into()
            }

            Message::ReloadStatics => {
                let file_content = std::fs::read_to_string("./statics.txt");
                let statics = file_content
                    .map(|c| {
                        c.trim()
                            .lines()
                            .filter_map(|l| {
                                let trimmed = l.trim();
                                (!trimmed.is_empty()).then_some(trimmed)
                            })
                            .map(|l| SongInfo::with_dance(l.to_owned()))
                            .collect()
                    })
                    .unwrap_or_default();

                self.data_provider.set_statics(statics);

                ().into()
            }

            Message::FileDropped(path) => {
                if let Ok(playlist) = load_tag_data_from_m3u(&path) {
                    self.data_provider.set_vec(playlist);
                } else if let Ok(song_info) = read_song_info_from_filepath(&path) {
                    self.data_provider.append_song(song_info);
                }

                ().into()
            }

            Message::SongChanged(song_change) => {
                self.data_provider.handle_song_change(song_change);
                ().into()
            }

            Message::SongDataEdit(i, edit) => {
                self.data_provider.handle_song_data_edit(i, edit);
                ().into()
            }

            Message::AddSong(song) => {
                self.data_provider.append_song(song);
                ().into()
            }

            Message::AddBlankSong(offset) => {
                self.data_provider.append_song(SongInfo::default());
                scrollable::snap_to(PLAYLIST_SCROLLABLE_ID.clone(), offset)
            }

            Message::DeleteSong(song) => {
                self.data_provider.delete_song(song);
                ().into()
            }

            Message::SetNextSong(i) => {
                self.data_provider.set_next(i);
                ().into()
            }

            Message::EnableImage(state) => {
                self.song_window.enable_image = state;
                ().into()
            }

            Message::EnableNextDance(state) => {
                self.song_window.enable_next_dance = state;
                ().into()
            }

            Message::ScrollBy(frac) => scrollable::scroll_by(
                PLAYLIST_SCROLLABLE_ID.clone(),
                AbsoluteOffset {
                    x: 0.0,
                    y: self.config_window.size.height / frac,
                },
            ),

            Message::TraktorMessage(msg) => {
                self.data_provider
                    .traktor_provider
                    .process_message(msg, &self.data_provider.playlist_songs);
                self.run_traktor_sync_action();

                ().into()
            }

            Message::TraktorEnableServer(enabled) => {
                self.data_provider.traktor_provider.is_enabled = enabled;
                ().into()
            }

            Message::TraktorChangeAddress(addr) => {
                self.data_provider.traktor_provider.address = addr;
                ().into()
            }

            Message::TraktorSubmitAddress => {
                self.data_provider.traktor_provider.submitted_address =
                    self.data_provider.traktor_provider.address.clone();
                ().into()
            }

            Message::TraktorChangeAndSubmitAddress(addr) => {
                self.data_provider.traktor_provider.address = addr;
                self.data_provider.traktor_provider.submitted_address =
                    self.data_provider.traktor_provider.address.clone();
                ().into()
            }

            Message::TraktorEnableDebugLogging(enabled) => {
                self.data_provider.traktor_provider.debug_logging = enabled;
                self.data_provider.traktor_provider.reconnect();
                ().into()
            }

            Message::TraktorReconnect => {
                self.data_provider.traktor_provider.reconnect();
                ().into()
            }

            Message::TraktorSetSyncMode(mode) => {
                self.data_provider.traktor_provider.sync_mode = mode;
                self.traktor_provider_force_update();

                ().into()
            }

            Message::TraktorSetNextMode(mode) => {
                self.data_provider.traktor_provider.next_mode = mode;
                self.traktor_provider_force_update();

                ().into()
            }

            Message::TraktorSetNextModeFallback(mode) => {
                self.data_provider.traktor_provider.next_mode_fallback = mode;
                self.traktor_provider_force_update();

                ().into()
            }

            _ => ().into(),
        }
    }

    fn traktor_provider_force_update(&mut self) {
        // send fake state update message to enforce sync refresh
        if let Some(mixer_state) = self
            .data_provider
            .traktor_provider
            .state
            .as_ref()
            .map(|s| s.mixer.clone())
        {
            self.data_provider.traktor_provider.process_message(
                ServerMessage::Update(StateUpdate::Mixer(mixer_state)),
                &self.data_provider.playlist_songs,
            );
            self.run_traktor_sync_action();
        }
    }

    fn run_traktor_sync_action(&mut self) {
        match self.data_provider.traktor_provider.take_sync_action() {
            TraktorSyncAction::Relative(offset) => {
                if offset >= 0 {
                    for _ in 0..offset {
                        self.data_provider.next();
                    }
                } else {
                    for _ in 0..(-offset) {
                        self.data_provider.prev();
                    }
                }
            }
            TraktorSyncAction::PlaylistAbsolute(pos) => {
                self.data_provider
                    .set_current(SongDataSource::Playlist(pos));
            }
        }
    }

    fn theme(&self, window_id: window::Id) -> Theme {
        if self.song_window.id.is_some_and(|id| id == window_id) {
            Theme::Dark
        } else {
            Theme::default()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            window::close_events().map(Message::WindowClosed),
            window::resize_events().map(Message::WindowResized),
            window::events().map(|(_, event)| match event {
                window::Event::FileDropped(path) => Message::FileDropped(path),
                _ => Message::Noop,
            }),
            on_key_press(|key: Key, _modifiers: Modifiers| match key {
                Key::Named(Named::ArrowRight) | Key::Named(Named::Space) => {
                    Some(Message::SongChanged(SongChange::Next))
                }
                Key::Named(Named::ArrowLeft) => Some(Message::SongChanged(SongChange::Previous)),
                Key::Named(Named::End) => Some(Message::SongChanged(SongChange::StaticAbsolute(0))),
                Key::Named(Named::F11) => Some(Message::ToggleFullscreen),
                Key::Named(Named::F5) => Some(Message::ReloadStatics),
                Key::Named(Named::PageUp) => Some(Message::ScrollBy(-10.0)),
                Key::Named(Named::PageDown) => Some(Message::ScrollBy(10.0)),
                _ => None,
            }),
            on_key_press(
                |key: Key, modifiers: Modifiers| match (key.as_ref(), modifiers) {
                    (Key::Character("n"), Modifiers::CTRL) => {
                        Some(Message::AddBlankSong(RelativeOffset::END))
                    }
                    (Key::Character("r"), Modifiers::CTRL) => Some(Message::ReloadStatics),
                    _ => None,
                },
            ),
        ];

        if let Some(addr) = self.data_provider.traktor_provider.get_socket_addr() {
            subscriptions.push(
                run_subscription_with(addr, |addr| traktor_api::run_server(*addr))
                    .map(Message::TraktorMessage),
            );
        }

        Subscription::batch(subscriptions)
    }
}
