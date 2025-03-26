use crate::dataloading::songinfo::SongInfo;
use crate::traktor_api::{AppMessage, ChannelState, DeckState, MixerState, ServerMessage, State};
use iced::futures::channel::mpsc::UnboundedSender;
use std::net::SocketAddr;

pub const TRAKTOR_SERVER_DEFAULT_ADDR: &str = "127.0.0.1:8080";

pub struct TraktorDataProvider {
    pub is_enabled: bool,
    pub address: String,
    pub submitted_address: String,

    channel: Option<UnboundedSender<AppMessage>>,

    time_offset_ms: i64,
    state: Option<State>,
    cached_song_info: Option<SongInfo>,

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

            time_offset_ms: 0,
            state: None,
            cached_song_info: None,

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
        self.update_song_info();

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

    fn get_deck_score(&self, deck: &DeckState, channel: &ChannelState, mixer: &MixerState) -> f64 {
        if !deck.content.is_loaded || deck.play_state.speed == 0.0 {
            return 0.0;
        }

        let mut score = channel.volume;

        if channel.x_fader_left && mixer.x_fader > 0.5 {
            score *= (mixer.x_fader - 0.5) * 2.0;
        }

        if channel.x_fader_right && mixer.x_fader < 0.5 {
            score *= mixer.x_fader * 2.0;
        }

        score
    }

    fn update_song_info(&mut self) {
        self.cached_song_info = None;

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

        self.cached_song_info = Some(SongInfo::new(
            0,
            content.title.to_owned(),
            content.artist.to_owned(),
            content.comment.to_owned(),
            None,
        ));
    }

    pub fn process_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Ready(channel) => {
                self.channel = Some(channel);

                self.time_offset_ms = 0;
                self.state = None;
                self.update_song_info();

                self.reconnect();
            }
            ServerMessage::Connect {
                time_offset_ms,
                initial_state,
            } => {
                println!("{:?}", initial_state);

                self.time_offset_ms = time_offset_ms;
                self.state = Some(initial_state);
                self.update_song_info();
            }
            ServerMessage::Update(update) => {
                println!("{:?}", update);

                if let Some(state) = self.state.as_mut() {
                    state.apply_update(update);
                }

                self.update_song_info();
            }
            ServerMessage::Log(msg) => {
                if self.debug_logging {
                    self.log.push(msg);
                }
            }
        }
    }

    fn send_message(&mut self, message: AppMessage) {
        if let Some(channel) = self.channel.as_ref() {
            if channel.unbounded_send(message).is_err() {
                self.channel = None;
            }
        }
    }
}
