use std::collections::VecDeque;
use std::io;
use std::path::{Path, PathBuf};

use crate::dataloading::id3tagreader::read_song_info_from_filepath;
use crate::dataloading::songinfo::SongInfo;
use percent_encoding::percent_decode_str;
use url::Url;

/// Result of loading an m3u: the songs whose tags could be read plus every
/// skipped entry with its reason. Entries can legitimately be unreadable —
/// audio outside a flatpak sandbox grant, absolute paths pointing elsewhere
/// — and silently dropping them made empty playlists look like successful
/// loads, so they are reported instead.
#[derive(Debug, Default)]
pub struct PlaylistLoad {
    pub songs: Vec<SongInfo>,
    pub skipped: Vec<SkippedEntry>,
}

/// One playlist entry that could not be turned into a song.
#[derive(Debug)]
pub struct SkippedEntry {
    /// The entry as written in the m3u (after percent-decoding).
    pub entry: String,
    /// The path the entry resolved to.
    pub path: PathBuf,
    pub reason: SkipReason,
}

/// Why an entry was skipped.
#[derive(Debug)]
pub enum SkipReason {
    /// The resolved path does not exist or is not accessible — typically an
    /// entry pointing outside a flatpak sandbox grant.
    Inaccessible,
    /// The file exists but no ID3 tag could be read from it.
    Unreadable(String),
}

impl PlaylistLoad {
    /// Entries the playlist declares in total (loaded + skipped).
    pub fn total(&self) -> usize {
        self.songs.len() + self.skipped.len()
    }
}

impl SkipReason {
    /// Short human-readable cause for logs and toasts.
    pub fn describe(&self) -> String {
        match self {
            SkipReason::Inaccessible => "not accessible".to_string(),
            SkipReason::Unreadable(err) => format!("unreadable: {err}"),
        }
    }
}

pub fn load_tag_data_from_m3u(path: &Path) -> io::Result<PlaylistLoad> {
    let mut load = PlaylistLoad::default();

    for (entry, path) in load_m3u_entries(path)? {
        if !path.exists() {
            load.skipped.push(SkippedEntry {
                entry,
                path,
                reason: SkipReason::Inaccessible,
            });
            continue;
        }

        match read_song_info_from_filepath(&path) {
            Ok(tag) => load.songs.push(tag),
            Err(err) => load.skipped.push(SkippedEntry {
                entry,
                path,
                reason: SkipReason::Unreadable(err.to_string()),
            }),
        }
    }

    Ok(load)
}

/// Reads the m3u at `path` and resolves every entry against the m3u's
/// directory (`file://` URIs and percent-encoded names supported). Returns
/// all entries — accessibility is checked by the caller so it can be
/// reported per entry.
fn load_m3u_entries(path: &Path) -> io::Result<Vec<(String, PathBuf)>> {
    let m3u_content = std::fs::read_to_string(path)?;
    // `parent()` is `None` for paths without a directory component; entries
    // then resolve relative to the working directory.
    let root = path.parent().unwrap_or_else(|| Path::new(""));

    Ok(m3u_content
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|entry| {
            let resolved =
                parse_file_uri(entry).unwrap_or_else(|| parse_encoded_file_name(entry));
            (entry.to_string(), root.join(resolved))
        })
        .collect())
}

/// Finds playlists (`.m3u`, `.m3u8`) under `root`, sorted by path. Inside a
/// flatpak sandbox `root` is the folder granted through the document
/// portal — its fuse view exposes the whole subtree, so discovery works
/// like on a regular filesystem. The walk is bounded (depth and entry
/// budget) and skips hidden directories.
pub fn discover_playlists(root: &Path) -> Vec<PathBuf> {
    const MAX_DEPTH: usize = 4;
    const MAX_ENTRIES: usize = 4096;

    let mut found = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back((root.to_path_buf(), 0_usize));
    let mut visited = 0_usize;

    while let Some((dir, depth)) = queue.pop_front() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            visited += 1;
            if visited > MAX_ENTRIES {
                return found;
            }

            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };

            if file_type.is_dir() {
                let hidden = entry.file_name().to_string_lossy().starts_with('.');
                if depth < MAX_DEPTH && !hidden {
                    queue.push_back((path, depth + 1));
                }
            } else if matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("m3u") | Some("m3u8")
            ) {
                found.push(path);
            }
        }
    }

    found.sort();
    found
}

fn parse_file_uri(uri: &str) -> Option<PathBuf> {
    let uri = Url::parse(uri).ok()?;
    uri.to_file_path().ok()
}

fn parse_encoded_file_name(file: &str) -> PathBuf {
    PathBuf::from(percent_decode_str(file).decode_utf8_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::dataloading::m3uloader::{
        SkipReason, discover_playlists, load_m3u_entries, load_tag_data_from_m3u,
    };
    use crate::test_file;

    #[test]
    fn m3u_entry_parsing() {
        let entries = load_m3u_entries(Path::new(test_file!("m3u_validation_test.m3u")))
            .expect("playlist should be readable");
        assert_eq!(entries.len(), 5);

        let resolved: Vec<PathBuf> = entries.into_iter().map(|(_, path)| path).collect();
        // Percent-encoded and plain relative entries resolve against the
        // m3u's directory …
        assert!(resolved
            .iter()
            .any(|p| p.ends_with("Tanzmusik/01_Competition Fever; SAP Rockets.mp3")));
        assert!(resolved
            .iter()
            .any(|p| p.ends_with("Tanzmusik/02_10 Hours; BLOOD SUGAR.mp3")));
        // … `file://` URIs and absolute paths are taken as-is (the latter
        // only on unix) …
        #[cfg(not(windows))]
        assert!(resolved.iter().any(|p| p == Path::new("/dev/zero")));
        #[cfg(not(windows))]
        assert!(resolved.iter().any(|p| p == Path::new("/dev/urandom")));
        // … and the entry pointing at the missing test file stays in the
        // list so it can be reported.
        let accessible = resolved.iter().filter(|p| p.exists()).count();
        #[cfg(not(windows))]
        assert_eq!(accessible, 4);
        #[cfg(windows)]
        assert_eq!(accessible, 2);
    }

    #[test]
    fn m3u_files_id3_tag_loading() {
        let load = load_tag_data_from_m3u(Path::new(test_file!("id3_read_test.m3u")))
            .expect("playlist should be readable");
        assert!(load.skipped.is_empty());
        assert_eq!(load.songs.len(), 1);
        let song = &load.songs[0];
        assert_eq!(song.title, "Sine Test");
        assert_eq!(song.artist, "K7");
        assert_eq!(song.dance, "Test Dance");
    }

    #[test]
    fn m3u_reports_inaccessible_entries() {
        let playlist = std::env::temp_dir().join("danceinterpreter_m3u_inaccessible_test.m3u");
        std::fs::write(&playlist, "does_not_exist_anywhere.mp3\n").expect("write temp playlist");

        let load = load_tag_data_from_m3u(&playlist).expect("playlist should be readable");
        assert!(load.songs.is_empty());
        assert_eq!(load.total(), 1);
        assert_eq!(load.skipped.len(), 1);
        assert!(matches!(load.skipped[0].reason, SkipReason::Inaccessible));
        assert_eq!(load.skipped[0].entry, "does_not_exist_anywhere.mp3");

        let _ = std::fs::remove_file(&playlist);
    }

    #[test]
    fn discover_playlists_finds_m3u_and_m3u8() {
        let found = discover_playlists(Path::new(test_file!("")));
        assert!(found.contains(&PathBuf::from(test_file!("id3_read_test.m3u"))));
        assert!(found.contains(&PathBuf::from(test_file!("m3u_validation_test.m3u"))));
    }
}
