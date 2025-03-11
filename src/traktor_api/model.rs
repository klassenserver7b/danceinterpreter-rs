use iced::futures::channel::mpsc;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone)]
pub enum AppMessage {
    Reconnect { debug_logging: bool },
}

#[derive(Debug, Clone)]
pub enum ServerMessage {
    Ready(mpsc::UnboundedSender<AppMessage>),
    Connect {
        time_offset_ms: u64,
        initial_state: State,
    },
    Update(StateUpdate),
    Log(String),
}

#[derive(Debug, Clone)]
pub enum ID {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Clone)]
pub enum StateUpdate {
    Mixer(MixerState),
    Channel(ID, ChannelState),
    DeckContent(ID, DeckContentState),
    DeckPlayState(ID, DeckPlayState),
}

#[derive(Debug, Clone)]
pub struct State {
    pub mixer: MixerState,
    pub channels: (ChannelState, ChannelState, ChannelState, ChannelState),
    pub decks: (DeckState, DeckState, DeckState, DeckState),
}

impl<'de> Deserialize<'de> for State {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct FlattenedState {
            mixer: MixerState,
            channel0: ChannelState,
            channel1: ChannelState,
            channel2: ChannelState,
            channel3: ChannelState,
            deck0content: DeckContentState,
            deck1content: DeckContentState,
            deck2content: DeckContentState,
            deck3content: DeckContentState,
            deck0playstate: DeckPlayState,
            deck1playstate: DeckPlayState,
            deck2playstate: DeckPlayState,
            deck3playstate: DeckPlayState,
        }

        let flattened_state: FlattenedState = Deserialize::deserialize(deserializer)?;

        Ok(State {
            mixer: flattened_state.mixer,
            channels: (
                flattened_state.channel0,
                flattened_state.channel1,
                flattened_state.channel2,
                flattened_state.channel3,
            ),
            decks: (
                DeckState {
                    content: flattened_state.deck0content,
                    play_state: flattened_state.deck0playstate,
                },
                DeckState {
                    content: flattened_state.deck1content,
                    play_state: flattened_state.deck1playstate,
                },
                DeckState {
                    content: flattened_state.deck2content,
                    play_state: flattened_state.deck2playstate,
                },
                DeckState {
                    content: flattened_state.deck3content,
                    play_state: flattened_state.deck3playstate,
                },
            ),
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MixerState {
    pub x_fader: f64,
    pub master_volume: f64,
    pub cue_volume: f64,
    pub cue_mix: f64,
    pub mic_volume: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChannelState {
    pub cue: bool,
    pub volume: f64,
    pub x_fader_left: bool,
    pub x_fader_right: bool,
}

#[derive(Debug, Clone)]
pub struct DeckState {
    pub content: DeckContentState,
    pub play_state: DeckPlayState,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeckContentState {
    pub is_loaded: bool,

    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub comment: String,
    pub comment2: String,
    pub label: String,

    pub key: String,
    pub file_path: String,
    pub track_length: f64,
    pub bpm: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeckPlayState {
    pub timestamp: u64,
    pub position: f64,
    pub speed: f64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(in crate::traktor_api) struct ConnectionResponse {
    pub session_id: String,
    pub debug_logging: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(in crate::traktor_api) struct InitializeRequest {
    pub session_id: String,
    pub timestamp: u64,
    pub state: State,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(in crate::traktor_api) struct UpdateRequest<T> {
    pub session_id: String,
    pub state: T,
}
