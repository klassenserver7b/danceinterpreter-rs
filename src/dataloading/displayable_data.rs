use crate::dataloading::songinfo::SongInfo;
use crate::dataloading::staticinfo::StaticInfo;

#[derive(Default, Debug, Clone)]
pub struct DisplayableData {
    pub headline: String,
    pub subline_upper: String,
    pub subline_lower: String,
    pub subline_image: Option<iced::widget::image::Handle>,
}

impl DisplayableData {
    #[allow(dead_code)]
    pub fn new(
        headline: String,
        subline_left: String,
        subline_right: String,
        subline_image: Option<iced::widget::image::Handle>,
    ) -> Self {
        Self {
            headline,
            subline_upper: subline_left,
            subline_lower: subline_right,
            subline_image,
        }
    }
}

impl From<SongInfo> for DisplayableData {
    fn from(song_info: SongInfo) -> Self {
        Self {
            headline: song_info.dance,
            subline_upper: song_info.title,
            subline_lower: song_info.artist,
            subline_image: song_info.album_art,
        }
    }
}

impl From<&SongInfo> for DisplayableData {
    fn from(song_info: &SongInfo) -> Self {
        let song_info = song_info.clone();
        Self::from(song_info)
    }
}

impl From<StaticInfo> for DisplayableData {
    fn from(static_info: StaticInfo) -> Self {
        Self {
            headline: static_info.name,
            subline_upper: String::new(),
            subline_lower: String::new(),
            subline_image: None,
        }
    }
}

impl From<&StaticInfo> for DisplayableData {
    fn from(static_info: &StaticInfo) -> Self {
        let static_info = static_info.clone();
        Self::from(static_info)
    }
}
