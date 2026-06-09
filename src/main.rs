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
use crate::traktor_api::{ServerMessage, StateUpdate, TraktorMessage, TraktorSyncAction};
use crate::ui::config_window::sidebar::SidebarMessage;
use crate::ui::config_window::{ConfigWindow, PLAYLIST_SCROLLABLE_ID};
use crate::ui::song_window::SongWindow;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2 as Matcher;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::operation::{scroll_by, snap_to};
use iced::widget::scrollable::{AbsoluteOffset, RelativeOffset};
use iced::widget::space::horizontal;
use iced::window::icon::from_file_data;
use iced::{Element, Size, Subscription, Task, Theme, exit, keyboard, system, theme, window};
use iced_aw::ICED_AW_FONT_BYTES;
use rfd::FileDialog;
use std::env::var;
use std::path::PathBuf;
use std::time::Instant;

fn main() -> iced::Result {
    iced::daemon(
        DanceInterpreter::new,
        DanceInterpreter::update,
        DanceInterpreter::view,
    )
    .title(DanceInterpreter::title)
    .theme(DanceInterpreter::theme)
    .font(ICED_AW_FONT_BYTES)
    .subscription(DanceInterpreter::subscription)
    .run()
}

pub trait Window {
    fn new(id: window::Id) -> Self;

    fn on_resize(&mut self, size: Size);
    fn on_close(&mut self);

    fn is_closed(&self) -> bool;
}

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

    ThemeChanged(theme::Mode),
    SetConfigTheme(Theme),

    ToggleFullscreen,
    SetFullscreen(bool),

    OpenPlaylist,
    ReloadStatics,
    AddSong(SongInfo),
    DeleteSong(SongDataSource),
    ScrollBy(f32),
    SnapTo(RelativeOffset),
    ToggleStaticsView,
    AddBlankSong(RelativeOffset),
    Sidebar(SidebarMessage),
    Animate,

    ToggleSearch,
    SearchChanged(String),
    ClearSearch,

    FileDropped(PathBuf),
    SongChanged(SongChange),
    SongDataEdit(usize, SongDataEdit),
    SetNextSong(SongDataSource),

    EnableImage(bool),
    EnableNextDance(bool),
    ChangeSongWindowScale(f32),
    EnableAutoscroll(bool),
    EnableFollowSystemTheme(bool),

    Traktor(TraktorMessage),
}

impl DanceInterpreter {
    pub fn new() -> (Self, Task<Message>) {
        let mut tasks = Vec::new();

        let icon = from_file_data(
            include_bytes!(res_file!("icon_light.png")),
            Some(image::ImageFormat::Png),
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

            data_provider: SongDataProvider::default(),
        };

        tasks.push(cw_opened);
        tasks.push(sw_opened);
        tasks.push(system::theme().map(Message::ThemeChanged));

        tasks.push(
            iced::font::load(include_bytes!(res_file!("symbols.ttf"))).map(|_| Message::Noop),
        );

        tasks.push(Task::done(Message::ReloadStatics));

        (state, Task::batch(tasks))
    }

    fn open_window<T: Window>(settings: window::Settings) -> (T, Task<Message>) {
        let (id, open) = window::open(settings);
        (T::new(id), open.map(Message::WindowOpened))
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
        if self.config_window.id == window_id {
            "Config Window".to_string()
        } else if self.song_window.id == window_id {
            "Song Window".to_string()
        } else {
            String::new()
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if self.config_window.id == window_id {
            self.config_window.view(self)
        } else if self.song_window.id == window_id {
            self.song_window.view(self)
        } else {
            horizontal().into()
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        self.config_window.sidebar.power_button_cache.clear();
        self.config_window.sidebar.restart_button_cache.clear();

        match message {
            Message::WindowOpened(_) => ().into(),
            Message::WindowResized((window_id, size)) => {
                if self.config_window.id == window_id {
                    self.config_window.on_resize(size);
                } else if self.song_window.id == window_id {
                    self.song_window.on_resize(size);
                }

                ().into()
            }
            Message::WindowClosed(window_id) => {
                if self.config_window.id == window_id {
                    self.config_window.on_close();

                    if self.song_window.is_closed() {
                        exit()
                    } else {
                        window::close(self.song_window.id)
                    }
                } else if self.song_window.id == window_id {
                    self.song_window.on_close();

                    if self.config_window.is_closed() {
                        exit()
                    } else {
                        window::close(self.config_window.id)
                    }
                } else {
                    ().into()
                }
            }

            Message::ThemeChanged(mode) => {
                if self.config_window.follow_system_theme {
                    self.config_window.theme = match mode {
                        theme::Mode::Light => Theme::Light,
                        theme::Mode::Dark | theme::Mode::None => Theme::Dark,
                    };
                }

                let icon = from_file_data(
                    match mode {
                        theme::Mode::Light | theme::Mode::None => {
                            include_bytes!(res_file!("icon_light.png"))
                        }
                        theme::Mode::Dark => include_bytes!(res_file!("icon_dark.png")),
                    },
                    Some(image::ImageFormat::Png),
                );

                if let Ok(icon) = icon {
                    Task::batch([
                        window::set_icon(self.song_window.id, icon.clone()),
                        window::set_icon(self.config_window.id, icon),
                    ])
                } else {
                    ().into()
                }
            }

            Message::SetConfigTheme(theme) => {
                self.config_window.theme = theme;
                self.config_window.follow_system_theme = false;
                ().into()
            }

            Message::ToggleFullscreen => {
                let song_window_id = self.song_window.id;

                window::mode(song_window_id)
                    .map(|mode| Message::SetFullscreen(mode != window::Mode::Fullscreen))
            }
            Message::SetFullscreen(fullscreen) => {
                let song_window_id = self.song_window.id;

                window::set_mode(
                    song_window_id,
                    if fullscreen {
                        window::Mode::Fullscreen
                    } else {
                        window::Mode::Windowed
                    },
                )
            }

            Message::OpenPlaylist => {
                #[cfg(target_os = "linux")]
                if var("container").is_ok() {
                    // Request folder access first in flatpak environment
                    let folder = FileDialog::new()
                        .set_title(
                            "Select folder containing m3u AND audio files (required in flatpak)",
                        )
                        .set_directory(
                            dirs::audio_dir()
                                .unwrap_or(dirs::home_dir().unwrap_or(PathBuf::from("."))),
                        )
                        .pick_folders();
                    if folder.is_none() {
                        return ().into();
                    }
                }

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

            Message::ToggleSearch => {
                self.config_window.search_visible = !self.config_window.search_visible;
                if !self.config_window.search_visible {
                    self.config_window.search_query.clear();
                }
                ().into()
            }

            Message::SearchChanged(query) => {
                self.config_window.search_query = query;
                self.scroll_to_first_search_match()
            }

            Message::ClearSearch => {
                self.config_window.search_query.clear();
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
                self.try_scroll_to_song()
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
                Task::done(Message::SnapTo(offset))
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

            Message::ChangeSongWindowScale(value) => {
                if (0.5..=3.0).contains(&value) {
                    self.song_window.scale = (value * 100.0).round() / 100.0;
                } else {
                    self.song_window.scale = (((self.song_window.scale + value) * 100.0).round()
                        / 100.0)
                        .clamp(0.5, 3.0);
                }
                ().into()
            }

            Message::EnableAutoscroll(state) => {
                self.config_window.enable_autoscroll = state;
                ().into()
            }

            Message::EnableFollowSystemTheme(state) => {
                self.config_window.follow_system_theme = state;

                if state {
                    system::theme().map(Message::ThemeChanged)
                } else {
                    ().into()
                }
            }

            Message::ScrollBy(frac) => scroll_by(
                PLAYLIST_SCROLLABLE_ID.clone(),
                AbsoluteOffset {
                    x: 0.0,
                    y: self.config_window.size.height / frac,
                },
            ),

            Message::SnapTo(offset) => snap_to(PLAYLIST_SCROLLABLE_ID.clone(), offset),

            Message::ToggleStaticsView => {
                self.config_window.is_statics_view = !self.config_window.is_statics_view;
                ().into()
            }

            Message::Sidebar(msg) => match msg {
                SidebarMessage::Toggle => {
                    self.config_window
                        .sidebar
                        .state
                        .go_mut(!self.config_window.sidebar.state.value(), Instant::now());
                    ().into()
                }
                SidebarMessage::UpdateAddressPresets => {
                    self.config_window
                        .sidebar
                        .update_network_interface_selection(&self.data_provider);
                    ().into()
                }
            },

            Message::Traktor(msg) => match msg {
                TraktorMessage::ServerMessage(msg) => {
                    self.data_provider.process_traktor_message(*msg);
                    self.run_traktor_sync_action();

                    self.try_scroll_to_song()
                }

                TraktorMessage::EnableServer(enabled) => {
                    self.data_provider.traktor_provider.set_enabled(enabled);
                    ().into()
                }

                TraktorMessage::ChangeAddress(addr) => {
                    self.data_provider.traktor_provider.address = addr;
                    ().into()
                }

                TraktorMessage::SubmitAddress => {
                    self.data_provider.traktor_provider.submitted_address =
                        self.data_provider.traktor_provider.address.clone();
                    ().into()
                }

                TraktorMessage::ChangeAndSubmitAddress(addr) => {
                    self.data_provider.traktor_provider.address = addr;
                    self.data_provider.traktor_provider.submitted_address =
                        self.data_provider.traktor_provider.address.clone();
                    ().into()
                }

                TraktorMessage::EnableDebugLogging(enabled) => {
                    self.data_provider.traktor_provider.debug_logging = enabled;
                    self.data_provider.traktor_provider.reconnect();
                    ().into()
                }

                TraktorMessage::Reconnect => {
                    self.data_provider.traktor_provider.reconnect();
                    ().into()
                }

                TraktorMessage::EnableSync(enabled) => {
                    self.data_provider.traktor_provider.sync = enabled;
                    self.traktor_provider_force_update()
                }

                TraktorMessage::SetSyncMode(mode) => {
                    self.data_provider.traktor_provider.sync_mode = mode;
                    self.traktor_provider_force_update()
                }

                TraktorMessage::SetNextMode(mode) => {
                    self.data_provider.traktor_provider.next_mode = mode;
                    self.traktor_provider_force_update()
                }

                TraktorMessage::SetNextModeFallback(mode) => {
                    self.data_provider.traktor_provider.next_mode_fallback = mode;
                    self.traktor_provider_force_update()
                }
            },

            Message::Animate => Task::none(),

            _ => ().into(),
        }
    }

    fn traktor_provider_force_update(&mut self) -> Task<Message> {
        // send fake state update message to enforce sync refresh
        if let Some(mixer_state) = self
            .data_provider
            .traktor_provider
            .state
            .as_ref()
            .map(|s| s.mixer)
        {
            self.data_provider
                .process_traktor_message(ServerMessage::Update(StateUpdate::Mixer(mixer_state)));
            self.run_traktor_sync_action();
            self.try_scroll_to_song()
        } else {
            ().into()
        }
    }

    fn scroll_to_first_search_match(&mut self) -> Task<Message> {
        if self.config_window.search_query.is_empty() {
            return ().into();
        }

        let matcher = Matcher::default();
        let query = &self.config_window.search_query;

        let first_match = self
            .data_provider
            .playlist_songs
            .iter()
            .enumerate()
            .find(|(_, song)| {
                matcher
                    .fuzzy_match(
                        &format!("{} {} {}", song.title, song.artist, song.dance),
                        query,
                    )
                    .is_some()
            });

        if let Some((index, _)) = first_match {
            let offset_y =
                index as f32 / std::cmp::max(1, self.data_provider.playlist_songs.len() - 1) as f32;

            Task::done(Message::SnapTo(RelativeOffset {
                x: 0.0,
                y: offset_y,
            }))
        } else {
            ().into()
        }
    }

    fn run_traktor_sync_action(&mut self) {
        let action = self.data_provider.traktor_provider.take_sync_action();
        if !self.data_provider.traktor_provider.sync {
            return;
        }

        match action {
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

    fn try_scroll_to_song(&mut self) -> Task<Message> {
        if let Some(index) = self.data_provider.take_scroll_index() {
            let offset_y =
                index as f32 / std::cmp::max(1, self.data_provider.playlist_songs.len() - 1) as f32;

            Task::done(Message::SnapTo(RelativeOffset {
                x: 0.0,
                y: offset_y,
            }))
        } else {
            ().into()
        }
    }

    fn theme(&self, window_id: window::Id) -> Theme {
        if self.song_window.id == window_id {
            Theme::Dark
        } else {
            self.config_window.theme.clone()
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
            keyboard::listen().filter_map(|event| {
                let keyboard::Event::KeyPressed { key, .. } = event else {
                    return None;
                };

                match key {
                    Key::Named(Named::ArrowRight) | Key::Named(Named::Space) => {
                        Some(Message::SongChanged(SongChange::Next))
                    }
                    Key::Named(Named::ArrowLeft) => {
                        Some(Message::SongChanged(SongChange::Previous))
                    }
                    Key::Named(Named::End) => {
                        Some(Message::SongChanged(SongChange::StaticAbsolute(0)))
                    }
                    Key::Named(Named::F11) => Some(Message::ToggleFullscreen),
                    Key::Named(Named::F5) => Some(Message::ReloadStatics),
                    Key::Named(Named::PageUp) => Some(Message::ScrollBy(-10.0)),
                    Key::Named(Named::PageDown) => Some(Message::ScrollBy(10.0)),
                    _ => None,
                }
            }),
            keyboard::listen().filter_map(|event| {
                let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
                    return None;
                };
                match (key.as_ref(), modifiers) {
                    (Key::Character("n"), Modifiers::CTRL) => {
                        Some(Message::AddBlankSong(RelativeOffset::END))
                    }
                    (Key::Character("+"), Modifiers::CTRL) => {
                        Some(Message::ChangeSongWindowScale(0.1))
                    }
                    (Key::Character("-"), Modifiers::CTRL) => {
                        Some(Message::ChangeSongWindowScale(-0.1))
                    }
                    (Key::Character("f"), Modifiers::CTRL) => Some(Message::ToggleSearch),
                    (Key::Character("c"), Modifiers::ALT) => {
                        Some(Message::Sidebar(SidebarMessage::Toggle))
                    }
                    _ => None,
                }
            }),
            system::theme_changes().map(Message::ThemeChanged),
            if self
                .config_window
                .sidebar
                .state
                .is_animating(Instant::now())
            {
                window::frames().map(|_| Message::Animate)
            } else {
                Subscription::none()
            },
        ];

        if let Some(addr) = self.data_provider.traktor_provider.get_socket_addr() {
            subscriptions.push(
                run_subscription_with(addr, |addr| traktor_api::run_server(*addr))
                    .map(|m| Message::Traktor(TraktorMessage::ServerMessage(Box::new(m)))),
            );
        }

        Subscription::batch(subscriptions)
    }
}
