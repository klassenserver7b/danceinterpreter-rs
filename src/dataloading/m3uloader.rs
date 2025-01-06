use std::io;
use std::io::Result;
use std::path::{Path, PathBuf};

use crate::dataloading::id3tagreader::read_song_info_from_filepath;
use crate::dataloading::songinfo::SongInfo;
use percent_encoding::percent_decode_str;
use url::Url;

pub fn load_tag_data_from_m3u(path: &Path) -> Result<Vec<SongInfo>> {
    let files = load_m3u_content_from_path(path)?;
    let mut songtags: Vec<SongInfo> = Vec::new();

    for file in files {
        let tag = read_song_info_from_filepath(&file).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Error reading tag from file: {}", e),
            )
        })?;
        songtags.push(tag);
    }

    Ok(songtags)
}

fn load_m3u_content_from_path(path: &Path) -> Result<Vec<PathBuf>> {
    let m3u_content = std::fs::read_to_string(path)?;
    let root = path.parent().unwrap();

    let entries = m3u_content.lines().filter(|x| !x.starts_with('#'));

    let absolute_paths = entries
        .map(|entry| parse_file_uri(entry).unwrap_or(parse_encoded_file_name(entry)))
        .map(|entry| root.join(entry));

    let accessible_files = absolute_paths.filter(|entry| entry.exists());

    Ok(accessible_files.collect::<Vec<PathBuf>>())
}

fn parse_file_uri(uri: &str) -> Option<PathBuf> {
    let uri = Url::parse(uri).ok()?;
    uri.to_file_path().ok()
}

fn parse_encoded_file_name(file: &str) -> PathBuf {
    PathBuf::from(percent_decode_str(file).decode_utf8_lossy().to_string())
}

#[cfg(test)]
#[macro_use]
mod tests {
    use std::path::Path;

    use crate::dataloading::m3uloader::{load_m3u_content_from_path, load_tag_data_from_m3u};
    use crate::test_file;

    #[test]
    fn m3u_path_parsing() {
        let result = load_m3u_content_from_path(Path::new(test_file!("m3u_validation_test.m3u")));
        assert!(result.is_ok());
        #[cfg(not(windows))]
        assert_eq!(result.unwrap().len(), 4);
        #[cfg(windows)]
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn m3u_files_id3_tag_loading() {
        let result = load_tag_data_from_m3u(Path::new(test_file!("id3_read_test.m3u")));
        assert!(result.is_ok());
        let res = result.unwrap()[0].clone();
        assert_eq!(res.title, "Sine Test");
        assert_eq!(res.artist, "K7");
        assert_eq!(res.dance, "Test Dance");
    }
}
