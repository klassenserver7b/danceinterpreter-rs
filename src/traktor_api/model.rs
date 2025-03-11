use iced::futures::channel::mpsc;
use serde::{Deserialize, Deserializer};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum AppMessage {
    Reconnect { debug_logging: bool },
}

#[derive(Debug, Clone)]
pub enum ServerMessage {
    Ready(mpsc::UnboundedSender<AppMessage>),
    Connect {
        time_base: SystemTime,
        initial_state: State,
    },
    Update(StateUpdate),
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
    mixer: MixerState,
    channels: (ChannelState, ChannelState, ChannelState, ChannelState),
    decks: (DeckState, DeckState, DeckState, DeckState),
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
    x_fader: f64,
    master_volume: f64,
    cue_volume: f64,
    cue_mix: f64,
    mic_volume: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChannelState {
    cue: bool,
    volume: f64,
    x_fader_left: bool,
    x_fader_right: bool,
}

#[derive(Debug, Clone)]
pub struct DeckState {
    content: DeckContentState,
    play_state: DeckPlayState,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeckContentState {
    is_loaded: bool,

    title: String,
    artist: String,
    album: String,
    genre: String,
    comment: String,
    comment2: String,
    label: String,

    key: String,
    file_path: String,
    track_length: f64,
    bpm: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeckPlayState {
    #[serde(deserialize_with = "deserialize_system_time")]
    timestamp: SystemTime,
    position: f64,
    speed: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(in crate::traktor_api) struct ConnectionResponse {
    session_id: String,
    debug_logging: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(in crate::traktor_api) struct InitializeRequest {
    session_id: String,
    #[serde(deserialize_with = "deserialize_system_time")]
    timestamp: SystemTime,
    state: State,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(in crate::traktor_api) struct UpdateRequest<T> {
    session_id: String,
    state: T,
}

fn deserialize_system_time<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
where
    D: Deserializer<'de>,
{
    let secs_since_epoch: u64 = Deserialize::deserialize(deserializer)?;
    Ok(UNIX_EPOCH + Duration::from_secs(secs_since_epoch))
}
