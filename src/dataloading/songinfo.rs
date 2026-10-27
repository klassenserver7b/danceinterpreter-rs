use iced::widget::image;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct SongInfo {
    pub track_number: u32,
    pub title: String,
    pub artist: String,
    pub dance: String,
    pub album_art: Option<image::Handle>,
}

impl SongInfo {
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
        }
    }
}
