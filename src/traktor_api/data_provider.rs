use crate::dataloading::songinfo::SongInfo;
use crate::traktor_api::{
    AppMessage, ChannelState, DeckContentState, DeckState, MixerState, ServerMessage, State,
    StateUpdate,
};
use iced::futures::channel::mpsc::UnboundedSender;
use iced::widget::image;
use std::collections::HashMap;
use std::mem;
use std::net::SocketAddr;

pub const TRAKTOR_SERVER_DEFAULT_ADDR: &str = "127.0.0.1:8080";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TraktorNextMode {
    DeckByPosition,
    DeckByNumber,
    PlaylistByNumber,
    PlaylistByName,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TraktorSyncMode {
    Relative,
    AbsoluteByNumber,
    AbsoluteByName,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TraktorSyncAction {
    Relative(isize),
    PlaylistAbsolute(usize),
}

pub struct TraktorDataProvider {
    pub is_enabled: bool,
    pub address: String,
    pub submitted_address: String,

    pub next_mode: Option<TraktorNextMode>,
    pub next_mode_fallback: Option<TraktorNextMode>,
    pub sync_mode: Option<TraktorSyncMode>,

    channel: Option<UnboundedSender<AppMessage>>,

    time_offset_ms: i64,
    pub state: Option<State>,
    covers: HashMap<String, image::Handle>,

    sync_x_fader_is_left: bool,

    cached_song_info: Option<SongInfo>,
    cached_next_song_info: Option<SongInfo>,
    cached_sync_action: TraktorSyncAction,

    pub debug_logging: bool,
    log: Vec<String>,
}

impl Default for TraktorDataProvider {
    fn default() -> Self {
        Self {
            is_enabled: false,
            address: String::new(),
            submitted_address: String::new(),
            channel: None,

            next_mode: Some(TraktorNextMode::DeckByNumber),
            next_mode_fallback: None,
            sync_mode: None,

            time_offset_ms: 0,
            state: None,
            covers: HashMap::new(),

            sync_x_fader_is_left: true,

            cached_song_info: None,
            cached_next_song_info: None,
            cached_sync_action: TraktorSyncAction::Relative(0),

            debug_logging: false,
            log: Vec::new(),
        }
    }
}

impl TraktorDataProvider {
    pub fn is_ready(&self) -> bool {
        self.is_enabled && self.channel.as_ref().is_some_and(|c| !c.is_closed())
    }

    pub fn get_log(&self) -> &[String] {
        &self.log
    }

    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    pub fn reconnect(&mut self) {
        self.time_offset_ms = 0;
        self.state = None;
        self.sync_x_fader_is_left = true;
        self.update_song_info(&[]);

        self.send_message(AppMessage::Reconnect {
            debug_logging: self.debug_logging,
        });
    }

    pub fn get_socket_addr(&self) -> Option<SocketAddr> {
        if !self.is_enabled {
            return None;
        }

        if self.submitted_address.is_empty() {
            return TRAKTOR_SERVER_DEFAULT_ADDR.parse().ok();
        }

        self.submitted_address.parse().ok()
    }

    pub fn get_song_info(&self) -> Option<&SongInfo> {
        if !self.is_ready() {
            return None;
        }

        self.cached_song_info.as_ref()
    }

    pub fn get_next_song_info(&self) -> Option<&SongInfo> {
        if !self.is_ready() {
            return None;
        }

        self.cached_next_song_info.as_ref()
    }

    fn get_deck_score(&self, deck: &DeckState, channel: &ChannelState, mixer: &MixerState) -> f64 {
        if !deck.content.is_loaded || deck.play_state.speed == 0.0 || channel.volume == 0.0 {
            return 0.0;
        }

        if channel.x_fader_left && mixer.x_fader > 0.5 {
            (1.0 - mixer.x_fader) * 2.0
        } else if channel.x_fader_right && mixer.x_fader < 0.5 {
            mixer.x_fader * 2.0
        } else {
            1.0
        }
    }

    fn update_song_info(&mut self, playlist: &[SongInfo]) {
        self.cached_song_info = None;
        self.cached_next_song_info = None;

        if !self.is_ready() {
            return;
        }

        let Some(state) = self.state.as_ref() else {
            return;
        };

        let scores = vec![
            self.get_deck_score(&state.decks.0, &state.channels.0, &state.mixer),
            self.get_deck_score(&state.decks.1, &state.channels.1, &state.mixer),
            self.get_deck_score(&state.decks.2, &state.channels.2, &state.mixer),
            self.get_deck_score(&state.decks.3, &state.channels.3, &state.mixer),
        ];

        let Some(max) = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
        else {
            return;
        };
        let max_index = if *max.1 > 0.0 {
            max.0
        } else {
            return;
        };

        let content = match max_index {
            0 => &state.decks.0.content,
            1 => &state.decks.1.content,
            2 => &state.decks.2.content,
            3 => &state.decks.3.content,
            _ => return,
        };

        let channel = match max_index {
            0 => &state.channels.0,
            1 => &state.channels.1,
            2 => &state.channels.2,
            3 => &state.channels.3,
            _ => return,
        };

        let current_song_info = self.copy_song_info_from_deck(&content, playlist);
        self.cached_song_info = Some(current_song_info.clone());
        self.cached_next_song_info = self
            .try_get_next_with_mode(self.next_mode, channel, playlist)
            .or_else(|| self.try_get_next_with_mode(self.next_mode_fallback, channel, playlist));

        match self.sync_mode {
            Some(TraktorSyncMode::AbsoluteByNumber) => {
                let current_index = playlist
                    .iter()
                    .position(|s| content.number == s.track_number);

                self.cached_sync_action = match current_index {
                    None => TraktorSyncAction::Relative(0),
                    Some(ci) => TraktorSyncAction::PlaylistAbsolute(ci),
                };
            }
            Some(TraktorSyncMode::AbsoluteByName) => {
                let current_index = playlist
                    .iter()
                    .position(|s| Self::songs_name_match(&current_song_info, s));

                self.cached_sync_action = match current_index {
                    None => TraktorSyncAction::Relative(0),
                    Some(ci) => TraktorSyncAction::PlaylistAbsolute(ci),
                };
            }
            _ => {}
        };
    }

    fn try_get_next_with_mode(
        &self,
        mode: Option<TraktorNextMode>,
        current_channel: &ChannelState,
        playlist: &[SongInfo],
    ) -> Option<SongInfo> {
        let Some(mode) = mode else {
            return None;
        };

        if !self.is_ready() {
            return None;
        }

        let Some(state) = self.state.as_ref() else {
            return None;
        };

        let Some(current_song_info) = self.cached_song_info.as_ref() else {
            return None;
        };

        match mode {
            TraktorNextMode::DeckByPosition => {
                let is_on_left = if current_channel.x_fader_left {
                    true
                } else if current_channel.x_fader_right {
                    false
                } else {
                    return None;
                };

                let other_side = vec![
                    &state.channels.0,
                    &state.channels.1,
                    &state.channels.2,
                    &state.channels.3,
                ]
                .into_iter()
                .position(|c| {
                    if is_on_left {
                        c.x_fader_right
                    } else {
                        c.x_fader_left
                    }
                });

                let deck = other_side
                    .map(|o| match o {
                        0 => Some(&state.decks.0),
                        1 => Some(&state.decks.1),
                        2 => Some(&state.decks.2),
                        3 => Some(&state.decks.3),
                        _ => None,
                    })
                    .flatten();

                deck.filter(|d| d.play_state.position < 0.5 * d.content.track_length)
                    .map(|d| self.copy_song_info_from_deck(&d.content, playlist))
            }
            TraktorNextMode::DeckByNumber => {
                let deck = vec![
                    &state.decks.0,
                    &state.decks.1,
                    &state.decks.2,
                    &state.decks.3,
                ]
                .into_iter()
                .find(|d| d.content.number == current_song_info.track_number + 1);

                deck.map(|d| self.copy_song_info_from_deck(&d.content, playlist))
            }
            TraktorNextMode::PlaylistByNumber => {
                let current_index = playlist
                    .iter()
                    .position(|s| current_song_info.track_number == s.track_number);

                current_index
                    .map(|ci| playlist.get(ci + 1).cloned())
                    .flatten()
            }
            TraktorNextMode::PlaylistByName => {
                let current_index = playlist
                    .iter()
                    .position(|s| Self::songs_name_match(current_song_info, s));

                current_index
                    .map(|ci| playlist.get(ci + 1).cloned())
                    .flatten()
            }
        }
    }

    fn copy_song_info_from_deck(
        &self,
        content: &DeckContentState,
        playlist: &[SongInfo],
    ) -> SongInfo {
        let mut song_info = SongInfo::new(
            content.number,
            content.title.to_owned(),
            content.artist.to_owned(),
            content.genre.to_owned(),
            self.covers.get(&content.file_path).cloned(),
        );

        if song_info.album_art.is_none() {
            song_info.album_art = playlist
                .iter()
                .find(|s| Self::songs_name_match(&song_info, s))
                .map(|s| s.album_art.clone())
                .flatten();
        }

        song_info
    }

    fn songs_name_match(a: &SongInfo, b: &SongInfo) -> bool {
        // TODO: maybe change this to levenshtein or sth
        a.artist == b.artist && a.title == b.title
    }

    fn get_loaded_files(&self) -> Vec<String> {
        let Some(state) = self.state.as_ref() else {
            return Vec::new();
        };

        let mut files: Vec<String> = vec![
            &state.decks.0.content.file_path,
            &state.decks.1.content.file_path,
            &state.decks.2.content.file_path,
            &state.decks.3.content.file_path,
        ]
        .into_iter()
        .filter_map(|f| (!f.is_empty()).then(|| f.to_owned()))
        .collect();
        files.dedup();

        files
    }

    pub fn process_message(&mut self, message: ServerMessage, playlist: &[SongInfo]) {
        match message {
            ServerMessage::Ready(channel) => {
                self.channel = Some(channel);

                self.time_offset_ms = 0;
                self.state = None;
                self.sync_x_fader_is_left = true;
                self.update_song_info(playlist);

                self.reconnect();
            }
            ServerMessage::Connect {
                time_offset_ms,
                initial_state,
            } => {
                println!("{:?}", initial_state);

                self.time_offset_ms = time_offset_ms;
                self.sync_x_fader_is_left = initial_state.mixer.x_fader < 0.5;
                self.state = Some(initial_state);
                self.update_song_info(playlist);
            }
            ServerMessage::Update(update) => {
                println!("{:?}", update);

                if let Some(state) = self.state.as_mut() {
                    if matches!(self.sync_mode, Some(TraktorSyncMode::Relative)) {
                        if let StateUpdate::Mixer(new_mixer_state) = &update {
                            let x_fader_old = state.mixer.x_fader;
                            let x_fader_new = new_mixer_state.x_fader;

                            let mut offset = 0;
                            if x_fader_old > 0.5 && x_fader_new <= 0.5 {
                                if self.sync_x_fader_is_left {
                                    offset -= 1;
                                } else {
                                    offset += 1;
                                }
                            } else if x_fader_old <= 0.5 && x_fader_new > 0.5 {
                                if self.sync_x_fader_is_left {
                                    offset += 1;
                                } else {
                                    offset -= 1;
                                }
                            }

                            if x_fader_new < 0.2 {
                                self.sync_x_fader_is_left = true;
                            } else if x_fader_new > 0.8 {
                                self.sync_x_fader_is_left = false;
                            }

                            self.cached_sync_action = match self.cached_sync_action {
                                TraktorSyncAction::Relative(prev) => {
                                    TraktorSyncAction::Relative(prev + offset)
                                }
                                TraktorSyncAction::PlaylistAbsolute(_) => {
                                    TraktorSyncAction::Relative(offset)
                                }
                            };
                        }
                    }

                    state.apply_update(update);
                }

                self.update_song_info(playlist);
            }
            ServerMessage::CoverImage { path, data } => {
                self.covers.insert(path, image::Handle::from_bytes(data));

                let loaded_files = self.get_loaded_files();
                self.covers.retain(|path, _| loaded_files.contains(path));
            }
            ServerMessage::Log(msg) => {
                if self.debug_logging {
                    self.log.push(msg);
                }
            }
        }
    }

    pub fn take_sync_action(&mut self) -> TraktorSyncAction {
        mem::replace(&mut self.cached_sync_action, TraktorSyncAction::Relative(0))
    }

    fn send_message(&mut self, message: AppMessage) {
        if let Some(channel) = self.channel.as_ref() {
            if channel.unbounded_send(message).is_err() {
                self.channel = None;
            }
        }
    }
}
