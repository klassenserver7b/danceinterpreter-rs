use crate::dataloading::songinfo::SongInfo;
use iced::widget::image::Handle;
use id3::frame::PictureType;
use id3::Result;
use id3::{Tag, TagLike};
use std::path::Path;

pub fn read_song_info_from_filepath(file: impl AsRef<Path>) -> Result<SongInfo> {
    let tag = Tag::read_from_path(file)?;

    let album_art = tag
        .pictures()
        // use front cover if available
        .find(|pic| pic.picture_type == PictureType::CoverFront)
        // otherwise try to use any picture
        .or(tag.pictures().next())
        .cloned();

    Ok(SongInfo::new(
        tag.track().unwrap_or(0),
        tag.title().unwrap_or("").to_string(),
        tag.artist().unwrap_or("").to_string(),
        tag.genre().unwrap_or("").to_string(),
        album_art.map(|img| Handle::from_bytes(img.data)),
    ))
}
#[allow(dead_code)]
pub fn read_song_info_from_files(file_list: &[impl AsRef<Path>]) -> Vec<Result<SongInfo>> {
    file_list.iter().map(read_song_info_from_filepath).collect()
}
