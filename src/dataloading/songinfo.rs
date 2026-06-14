use iced::widget::image;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SongInfo {
    pub track_number: u32,
    pub title: String,
    pub artist: String,
    pub dance: String,
    #[serde(skip)]
    pub album_art: Option<image::Handle>,
    pub is_favorite: bool,
}

impl SongInfo {
    pub fn with_dance(dance: String) -> Self {
        SongInfo {
            dance,
            ..Default::default()
        }
    }

    pub fn new(
        track_number: u32,
        title: String,
        artist: String,
        dance: String,
        album_art: Option<image::Handle>,
    ) -> Self {
        SongInfo {
            track_number,
            title,
            artist,
            dance,
            album_art,
            is_favorite: false,
        }
    }
}
