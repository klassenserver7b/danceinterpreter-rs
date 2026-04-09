use crate::dataloading::songinfo::SongInfo;
use crate::traktor_api::{
    AppMessage, ChannelState, DeckContentState, DeckState, MixerState, ServerMessage, State,
    StateUpdate,
};
use iced::futures::channel::mpsc::UnboundedSender;
use iced::widget::image;
use std::collections::HashMap;
use std::fmt::Display;
use std::mem;
use std::net::SocketAddr;

pub const TRAKTOR_SERVER_DEFAULT_ADDR: &str = "127.0.0.1:8080";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TraktorNextMode {
    None,
    DeckByPosition,
    DeckByNumber,
    PlaylistByNumber,
    PlaylistByName,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TraktorSyncMode {
    None,
    Relative,
    AbsoluteByNumber,
    AbsoluteByName,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TraktorSyncAction {
    Relative(isize),
    PlaylistAbsolute(usize),
}

impl Display for TraktorNextMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for TraktorSyncMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for TraktorSyncAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct TraktorDataProvider {
    pub is_enabled: bool,
    pub address: String,
    pub submitted_address: String,

    pub next_mode: TraktorNextMode,
    pub next_mode_fallback: TraktorNextMode,
    pub sync_mode: TraktorSyncMode,

    channel: Option<UnboundedSender<AppMessage>>,

    time_offset_ms: i64,
    pub state: Option<State>,
    covers: HashMap<String, image::Handle>,

    sync_x_fader_is_left: bool,

    cached_song_info: Option<SongInfo>,
    cached_next_song_info: Option<SongInfo>,
    cached_sync_action: TraktorSyncAction,
    should_scroll: bool,

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

            next_mode: TraktorNextMode::DeckByNumber,
            next_mode_fallback: TraktorNextMode::None,
            sync_mode: TraktorSyncMode::None,

            time_offset_ms: 0,
            state: None,
            covers: HashMap::new(),

            sync_x_fader_is_left: true,

            cached_song_info: None,
            cached_next_song_info: None,
            cached_sync_action: TraktorSyncAction::Relative(0),
            should_scroll: false,

            debug_logging: false,
            log: Vec::new(),
        }
    }
}

impl TraktorDataProvider {
    pub fn is_ready(&self) -> bool {
        self.is_enabled && self.channel.as_ref().is_some_and(|c| !c.is_closed())
    }

    #[allow(dead_code)]
    pub fn get_log(&self) -> &[String] {
        &self.log
    }

    #[allow(dead_code)]
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
        let old_song_info = self.cached_song_info.take();
        self.cached_next_song_info = None;

        if !self.is_ready() {
            return;
        }

        let Some(state) = self.state.as_ref() else {
            return;
        };

        let scores = (0..4)
            .map(|i| self.get_deck_score(&state.decks[i], &state.channels[i], &state.mixer))
            .collect::<Vec<f64>>();

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

        let content = &state.decks[max_index].content;
        let channel = &state.channels[max_index];

        let current_song_info = self.copy_song_info_from_deck(content, playlist);
        self.cached_song_info = Some(current_song_info.clone());

        if old_song_info != self.cached_song_info {
            self.should_scroll = true;
        }

        self.cached_next_song_info = self
            .try_get_next_with_mode(false, channel, playlist)
            .or_else(|| self.try_get_next_with_mode(true, channel, playlist));

        match self.sync_mode {
            TraktorSyncMode::AbsoluteByNumber => {
                let current_index = playlist
                    .iter()
                    .position(|s| content.number == s.track_number);

                self.cached_sync_action = match current_index {
                    None => TraktorSyncAction::Relative(0),
                    Some(ci) => TraktorSyncAction::PlaylistAbsolute(ci),
                };
            }
            TraktorSyncMode::AbsoluteByName => {
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
        fallback: bool,
        current_channel: &ChannelState,
        playlist: &[SongInfo],
    ) -> Option<SongInfo> {
        let mode = if !fallback {
            self.next_mode
        } else {
            self.next_mode_fallback
        };

        if !self.is_ready() {
            return None;
        }

        let state = self.state.as_ref()?;

        let current_song_info = self.cached_song_info.as_ref()?;

        match mode {
            TraktorNextMode::DeckByPosition => {
                let is_on_left = if current_channel.x_fader_left {
                    true
                } else if current_channel.x_fader_right {
                    false
                } else {
                    return None;
                };

                let other_side = state.channels.iter().position(|c| {
                    if is_on_left {
                        c.x_fader_right
                    } else {
                        c.x_fader_left
                    }
                });

                let deck = other_side.map(|o| &state.decks[o]);
                deck.filter(|d| d.play_state.position < 0.5 * d.content.track_length)
                    .map(|d| self.copy_song_info_from_deck(&d.content, playlist))
            }
            TraktorNextMode::DeckByNumber => {
                let deck = state
                    .decks
                    .iter()
                    .find(|d| d.content.number == current_song_info.track_number + 1);

                deck.map(|d| self.copy_song_info_from_deck(&d.content, playlist))
            }
            TraktorNextMode::PlaylistByNumber => {
                let current_index = playlist
                    .iter()
                    .position(|s| current_song_info.track_number == s.track_number);

                current_index.and_then(|ci| playlist.get(ci + 1).cloned())
            }
            TraktorNextMode::PlaylistByName => {
                let current_index = playlist
                    .iter()
                    .position(|s| Self::songs_name_match(current_song_info, s));

                current_index.and_then(|ci| playlist.get(ci + 1).cloned())
            }
            TraktorNextMode::None => None,
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
                .and_then(|s| s.album_art.clone());
        }

        song_info
    }

    pub fn songs_name_match(a: &SongInfo, b: &SongInfo) -> bool {
        // TODO: maybe change this to levenshtein or sth
        a.artist == b.artist && a.title == b.title
    }

    fn get_loaded_files(&self) -> Vec<String> {
        let Some(state) = self.state.as_ref() else {
            return Vec::new();
        };

        let mut files: Vec<String> = state
            .decks
            .iter()
            .map(|d| &d.content.file_path)
            .filter(|&f| !f.is_empty())
            .map(|f| f.to_owned())
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
                self.time_offset_ms = time_offset_ms;
                self.sync_x_fader_is_left = initial_state.mixer.x_fader < 0.5;
                self.state = Some(*initial_state);
                self.update_song_info(playlist);
            }
            ServerMessage::Update(update) => {
                if let Some(state) = self.state.as_mut() {
                    if matches!(self.sync_mode, TraktorSyncMode::Relative)
                        && let StateUpdate::Mixer(new_mixer_state) = &update
                    {
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

    pub fn take_should_scroll(&mut self) -> bool {
        let should_scroll = self.should_scroll;
        self.should_scroll = false;
        should_scroll
    }

    pub fn get_current_index(&self, playlist: &[SongInfo]) -> Option<usize> {
        let traktor_song = self.get_song_info()?;

        playlist
            .iter()
            .enumerate()
            .find(|(_i, s)| TraktorDataProvider::songs_name_match(s, traktor_song))
            .map(|(i, _s)| i)
    }

    fn send_message(&mut self, message: AppMessage) {
        if let Some(channel) = self.channel.as_ref()
            && channel.unbounded_send(message).is_err()
        {
            self.channel = None;
        }
    }
}
