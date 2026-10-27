use crate::dataloading::displayable_data::DisplayableData;
use crate::dataloading::songinfo::SongInfo;
use crate::dataloading::staticinfo::StaticInfo;
use crate::traktor_api;
use crate::traktor_api::TraktorDataProvider;
use iced::Color;
use std::cmp::PartialEq;
use std::path::PathBuf;

pub enum DeletedItem {
    Playlist {
        index: usize,
        song: SongInfo,
        played: bool,
    },
    Static {
        name: String,
        static_info: StaticInfo,
    },
}

#[derive(Default, Debug, PartialEq, Clone)]
pub enum ItemSource {
    #[default]
    Blank,
    Traktor,
    Other(SongInfo),
    Static(String),
    Playlist(usize),
}

#[derive(Debug, Clone)]
pub enum ItemChange {
    Blank,
    Traktor,
    StaticAbsolute(String),
    PlaylistAbsolute(usize),
    Previous,
    Next,
}
#[derive(Debug, Clone)]
pub enum SongDataEdit {
    Title(String),
    Artist(String),
    Dance(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubmitStaticResult {
    Success,
    Unchanged,
    NotFound,
    NeedsMerge { old_name: String, new_name: String },
    Failed(&'static str),
}

use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistItem {
    pub song: SongInfo,
    pub played: bool,
}

/// Playback flags of a single playlist row, as the table renderer needs them.
#[derive(Debug, Clone, Copy, Default)]
pub struct RowPlayState {
    /// The row the audience display is showing.
    pub is_current: bool,
    /// The row queued to play after the current one.
    pub is_next: bool,
    /// The row matching Traktor's live deck.
    pub is_traktor: bool,
    /// The row has already been played.
    pub is_played: bool,
}

#[derive(Default)]
pub struct DataProvider {
    pub traktor_provider: TraktorDataProvider,

    playlist: Vec<PlaylistItem>,
    statics: IndexMap<String, StaticInfo>,

    deleted_items: Vec<DeletedItem>,

    current: ItemSource,
    next: Option<ItemSource>,

    should_scroll: bool,
}

impl DataProvider {
    pub fn set_vec(&mut self, vec: Vec<SongInfo>) {
        self.playlist = vec
            .into_iter()
            .map(|song| PlaylistItem {
                song,
                played: false,
            })
            .collect();

        for dance in self.playlist_dances() {
            self.ensure_static(&dance);
        }

        if !self.playlist.is_empty() {
            self.current = ItemSource::Playlist(0);
        } else {
            self.current = ItemSource::Blank;
        }
    }

    fn set_statics(&mut self, vec: Vec<StaticInfo>) {
        self.statics = vec.into_iter().map(|s| (s.name.clone(), s)).collect();
    }

    /// All dance names used by the playlist, in row order (collected first so
    /// the statics map can be mutated while iterating).
    fn playlist_dances(&self) -> Vec<String> {
        self.playlist
            .iter()
            .map(|item| item.song.dance.clone())
            .collect()
    }

    /// Maps an item source to its displayable data, with the Traktor song
    /// passed in (current vs. next deck lookup differs per call site).
    fn displayable_for(
        &self,
        source: &ItemSource,
        traktor_song: Option<&SongInfo>,
    ) -> Option<DisplayableData> {
        match source {
            ItemSource::Static(name) => self.statics.get(name).map(|s| s.into()),
            ItemSource::Playlist(i) => self.playlist.get(*i).map(|item| (&item.song).into()),
            ItemSource::Other(song) => Some(song.into()),
            ItemSource::Blank => None,
            ItemSource::Traktor => traktor_song.map(|s| s.into()),
        }
    }

    pub fn get_current_displayable_data(&self) -> Option<DisplayableData> {
        self.displayable_for(&self.current, self.traktor_provider.get_song_info())
    }
    pub fn get_next_displayable_data(&self) -> Option<DisplayableData> {
        if let Some(next) = self.next.as_ref() {
            return self.displayable_for(next, self.traktor_provider.get_next_song_info());
        }

        match self.current {
            ItemSource::Static(_) => None,
            ItemSource::Playlist(i) => self.playlist.get(i + 1).map(|item| (&item.song).into()),
            ItemSource::Other(ref song) => Some(song.into()),
            ItemSource::Blank => None,
            ItemSource::Traktor => self.traktor_provider.get_next_song_info().map(|s| s.into()),
        }
    }

    pub fn prev(&mut self) {
        self.should_scroll = true;

        let ItemSource::Playlist(current_index) = self.current else {
            return;
        };

        if current_index == 0 {
            return;
        }

        self.set_current_as_played();
        self.current = ItemSource::Playlist(current_index - 1);
    }

    pub fn next(&mut self) {
        self.should_scroll = true;

        if let Some(next) = self.next.take() {
            self.set_current_as_played();
            self.current = next;
            return;
        }

        let ItemSource::Playlist(current_index) = self.current else {
            return;
        };

        if current_index == self.playlist.len() - 1 {
            return;
        }

        self.set_current_as_played();
        self.current = ItemSource::Playlist(current_index + 1);
    }

    pub fn set_current(&mut self, n: ItemSource) {
        self.set_current_as_played();

        match n {
            ItemSource::Static(ref name) => {
                if self.statics.get(name).is_some() {
                    self.current = n;
                }
            }
            ItemSource::Playlist(i) => {
                if self.playlist.get(i).is_some() {
                    self.current = n;
                }
            }
            _ => self.current = n,
        }
    }

    pub fn set_next(&mut self, next: ItemSource) {
        self.next = Some(next);
    }

    pub fn append_song(&mut self, song: SongInfo) {
        self.ensure_static(&song.dance);
        self.playlist.push(PlaylistItem {
            song,
            played: false,
        });
    }

    fn ensure_static(&mut self, dance: &str) {
        if !dance.is_empty() && !self.statics.contains_key(dance) {
            self.statics
                .insert(dance.to_string(), StaticInfo::new(dance.to_string()));
        }
    }

    /// Inserts a placeholder static under an unused "New Static N" name.
    pub fn add_static(&mut self) {
        self.add_named_static("New Static");
    }

    /// Adds a static with the given name (uniquified when it already exists).
    pub fn add_named_static(&mut self, name: &str) {
        let name = name.trim();
        if name.is_empty() {
            return;
        }

        let mut final_name = name.to_string();
        let mut counter = 1;
        while self.statics.contains_key(&final_name) {
            counter += 1;
            final_name = format!("{} {}", name, counter);
        }
        self.statics
            .insert(final_name.clone(), StaticInfo::new(final_name));
        self.save_statics();
    }

    pub fn toggle_static_favorite(&mut self, name: &str) {
        if let Some(song) = self.statics.get_mut(name) {
            song.is_favorite = !song.is_favorite;
            self.save_statics();
        }
    }

    pub fn delete_item(&mut self, song: ItemSource) {
        if self.current == song {
            self.current = ItemSource::Blank;
        } else {
            self.next.take_if(|next| *next == song);
        }

        if let ItemSource::Playlist(i) = song {
            let item = self.playlist.remove(i);
            self.deleted_items.push(DeletedItem::Playlist {
                index: i,
                song: item.song,
                played: item.played,
            });
        } else if let ItemSource::Static(ref name) = song
            && let Some(static_info) = self.statics.shift_remove(name)
        {
            self.deleted_items.push(DeletedItem::Static {
                name: name.clone(),
                static_info,
            });

            self.save_statics();
        }
    }

    pub fn undo_delete(&mut self) {
        if let Some(item) = self.deleted_items.pop() {
            match item {
                DeletedItem::Playlist {
                    index,
                    song,
                    played,
                } => {
                    let insert_idx = index.min(self.playlist.len());
                    self.playlist
                        .insert(insert_idx, PlaylistItem { song, played });
                }
                DeletedItem::Static { name, static_info } => {
                    self.statics.insert(name, static_info);
                    self.save_statics();
                }
            }
        }
    }

    pub fn handle_item_change(&mut self, change: ItemChange) {
        match change {
            ItemChange::Blank => {
                self.set_current_as_played();
                self.current = ItemSource::Blank;
            }
            ItemChange::Traktor => {
                self.set_current_as_played();
                self.current = ItemSource::Traktor;
            }
            ItemChange::StaticAbsolute(index) => {
                self.traktor_provider.sync = false;
                self.set_current_as_played();
                self.current = ItemSource::Static(index);
            }
            ItemChange::PlaylistAbsolute(index) => {
                self.set_current_as_played();
                self.current = ItemSource::Playlist(index);
            }
            ItemChange::Previous => {
                self.prev();
            }
            ItemChange::Next => {
                self.next();
            }
        }
    }

    pub fn handle_song_data_edit(&mut self, idx: usize, edit: SongDataEdit) {
        if let Some(song) = self.playlist.get_mut(idx).map(|item| &mut item.song) {
            match edit {
                SongDataEdit::Title(title) => {
                    song.title = title;
                }
                SongDataEdit::Artist(artist) => {
                    song.artist = artist;
                }
                SongDataEdit::Dance(v) => {
                    song.dance = v;
                }
            }
        }
    }

    /// Restores a song's editable fields from a pre-edit snapshot (Escape
    /// rollback of an inline editing session). No-op when the index no
    /// longer exists.
    pub fn restore_song_edit(&mut self, idx: usize, title: &str, artist: &str, dance: &str) {
        if let Some(song) = self.playlist.get_mut(idx).map(|item| &mut item.song) {
            song.title = title.to_string();
            song.artist = artist.to_string();
            song.dance = dance.to_string();
        }
    }

    pub fn handle_song_data_submit(&mut self, idx: usize) {
        if let Some(item) = self.playlist.get(idx) {
            let dance = item.song.dance.clone();
            self.ensure_static(&dance);
            self.save_statics();
        }
    }

    pub fn process_traktor_message(&mut self, message: traktor_api::ServerMessage) {
        self.set_current_as_played();
        self.traktor_provider
            .process_message(message, &self.playlist);
    }

    fn get_current_traktor_index(&self) -> Option<usize> {
        self.traktor_provider.get_current_index(&self.playlist)
    }

    pub fn take_scroll_index(&mut self) -> Option<usize> {
        let should_scroll = self.should_scroll | self.traktor_provider.take_should_scroll();
        self.should_scroll = false;

        if !should_scroll {
            return None;
        }

        match self.current {
            ItemSource::Traktor => self.get_current_traktor_index(),
            ItemSource::Playlist(i) => Some(i),
            _ => None,
        }
    }

    pub fn clear_pending_scroll(&mut self) {
        self.should_scroll = false;
        self.traktor_provider.take_should_scroll();
    }

    pub fn get_play_state(&self, playlist_index: usize) -> RowPlayState {
        let mut state = RowPlayState {
            is_played: self
                .playlist
                .get(playlist_index)
                .map(|item| item.played)
                .unwrap_or(false),
            ..RowPlayState::default()
        };

        if let ItemSource::Playlist(i) = self.current {
            state.is_current = playlist_index == i;
            state.is_next = playlist_index == (i + 1);
        }

        if let Some(ItemSource::Playlist(i)) = self.next {
            state.is_next = playlist_index == i;
        }

        if matches!(self.current, ItemSource::Traktor)
            && let Some(index) = self.get_current_traktor_index()
        {
            state.is_traktor = playlist_index == index;
        }

        state
    }

    pub fn ensure_statics_for_playlist(&mut self) {
        for dance in self.playlist_dances() {
            self.ensure_static(&dance);
        }
        self.save_statics();
    }

    pub fn get_dance_color(&self, dance: &str) -> Option<Color> {
        self.statics.get(dance).and_then(|s| s.color)
    }

    pub fn rename_static(&mut self, old_name: &str, new_name: &str) -> Result<(), &'static str> {
        if let Some(static_info) = self.statics.get_mut(old_name) {
            static_info.name = new_name.to_string();
            Ok(())
        } else {
            Err("Static not found")
        }
    }

    pub fn process_static_name_submit(&mut self, key: &str) -> SubmitStaticResult {
        let static_info = match self.statics.get(key) {
            Some(info) => info,
            None => return SubmitStaticResult::NotFound,
        };

        let typed_name = static_info.name.clone();

        if key == typed_name {
            return SubmitStaticResult::Unchanged;
        }

        if self.statics.contains_key(&typed_name) {
            return SubmitStaticResult::NeedsMerge {
                old_name: key.to_string(),
                new_name: typed_name,
            };
        }

        // If it's a completely new name, do the rename/re-keying immediately.
        match self.submit_static_name(key, &typed_name) {
            Ok(()) => SubmitStaticResult::Success,
            Err(message) => SubmitStaticResult::Failed(message),
        }
    }

    /// Renames every playlist row tagged `old_name` to `new_name` (shared by
    /// rename and merge).
    fn rekey_playlist(&mut self, old_name: &str, new_name: &str) {
        for item in &mut self.playlist {
            if item.song.dance == old_name {
                item.song.dance = new_name.to_string();
            }
        }
    }

    fn submit_static_name(&mut self, old_name: &str, new_name: &str) -> Result<(), &'static str> {
        if self.statics.contains_key(new_name) {
            return Err("Static with new name already exists");
        }

        if let Some(mut static_info) = self.statics.shift_remove(old_name) {
            static_info.name = new_name.to_string();
            self.statics.insert(new_name.to_string(), static_info);
            self.rekey_playlist(old_name, new_name);
            self.save_statics();
            Ok(())
        } else {
            Err("Static not found")
        }
    }

    pub fn update_static_color(
        &mut self,
        name: &str,
        color: Option<Color>,
        save: bool,
    ) -> Result<(), &'static str> {
        if let Some(static_info) = self.statics.get_mut(name) {
            static_info.color = color;
            if save {
                self.save_statics();
            }
            Ok(())
        } else {
            Err("Static not found")
        }
    }

    pub fn merge_statics(&mut self, old_name: &str, new_name: &str) -> Result<(), &'static str> {
        if !self.statics.contains_key(new_name) {
            return Err("Target static does not exist");
        }

        if self.statics.shift_remove(old_name).is_some() {
            self.rekey_playlist(old_name, new_name);
            self.save_statics();
            Ok(())
        } else {
            Err("Source static not found")
        }
    }

    fn set_current_as_played(&mut self) {
        let i = match self.current {
            ItemSource::Playlist(i) => i,
            ItemSource::Traktor => {
                let Some(index) = self.get_current_traktor_index() else {
                    return;
                };
                index
            }
            _ => return,
        };

        if let Some(item) = self.playlist.get_mut(i) {
            item.played = true;
        }
    }

    fn save_statics(&self) {
        let values: Vec<&StaticInfo> = self.statics.values().collect();
        if let Ok(json) = serde_json::to_string_pretty(&values) {
            let _ = std::fs::write(Self::default_statics_path(), json);
        }
    }

    #[cfg(test)]
    fn default_statics_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("danceinterpreter_test_statics.json");
        path
    }

    #[cfg(not(test))]
    fn default_statics_path() -> PathBuf {
        if let Some(mut path) = dirs::config_dir() {
            path.push("danceinterpreter");
            let _ = std::fs::create_dir_all(&path);
            path.push("statics.json");
            path
        } else {
            PathBuf::from("./statics.json")
        }
    }

    pub fn load_statics(&mut self) {
        let statics: Vec<StaticInfo> = std::fs::read_to_string(Self::default_statics_path())
            .map(|file_content| serde_json::from_str(&file_content).ok())
            .unwrap_or_default()
            .unwrap_or_default();

        self.set_statics(statics);
        self.save_statics();
    }

    pub fn statics(&self) -> &IndexMap<String, StaticInfo> {
        &self.statics
    }

    pub fn playlist(&self) -> &Vec<PlaylistItem> {
        &self.playlist
    }

    /// The currently displayed item source.
    pub fn get_current_source(&self) -> &ItemSource {
        &self.current
    }

    /// The explicitly queued next item, if any.
    pub fn get_queued_next(&self) -> Option<&ItemSource> {
        self.next.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataloading::songinfo::SongInfo;

    fn test_provider() -> DataProvider {
        // Statics persistence targets std::env::temp_dir() in test builds
        // (default_statics_path), so no repo files are touched.
        DataProvider::default()
    }

    #[test]
    fn test_ensure_static_empty() {
        let mut provider = test_provider();
        provider.ensure_static("");
        assert!(provider.statics.is_empty());
    }

    #[test]
    fn test_ensure_static_new() {
        let mut provider = test_provider();
        provider.ensure_static("Waltz");
        assert!(provider.statics.contains_key("Waltz"));
    }

    #[test]
    fn test_ensure_static_existing() {
        let mut provider = test_provider();
        provider.ensure_static("Waltz");
        provider.statics.get_mut("Waltz").unwrap().is_favorite = true;

        provider.ensure_static("Waltz"); // Should not overwrite
        assert!(provider.statics.get("Waltz").unwrap().is_favorite);
    }

    #[test]
    fn test_playlist_dance_updates_triggering_static_creation() {
        let mut provider = test_provider();
        let song = SongInfo {
            dance: "Waltz".to_string(),
            ..Default::default()
        };
        provider.playlist.push(PlaylistItem {
            song,
            played: false,
        });

        provider.handle_song_data_edit(0, SongDataEdit::Dance("Tango".to_string()));
        provider.handle_song_data_submit(0);

        assert_eq!(provider.playlist[0].song.dance, "Tango");
        assert!(provider.statics.contains_key("Tango"));
    }

    #[test]
    fn test_ensure_statics_for_playlist() {
        let mut provider = test_provider();
        let song = SongInfo {
            dance: "Waltz".to_string(),
            ..Default::default()
        };
        provider.playlist.push(PlaylistItem {
            song,
            played: false,
        });

        provider.ensure_statics_for_playlist();
        assert!(provider.statics.contains_key("Waltz"));
    }

    #[test]
    fn test_submit_static_name() {
        let mut provider = test_provider();
        provider.ensure_static("OldDance");
        let song = SongInfo {
            dance: "OldDance".to_string(),
            ..Default::default()
        };
        provider.playlist.push(PlaylistItem {
            song,
            played: false,
        });

        assert!(provider.submit_static_name("OldDance", "NewDance").is_ok());

        assert!(!provider.statics.contains_key("OldDance"));
        assert!(provider.statics.contains_key("NewDance"));
        assert_eq!(provider.playlist[0].song.dance, "NewDance");
    }

    #[test]
    fn test_rename_static_display_name() {
        let mut provider = test_provider();
        provider.ensure_static("OldDance");

        assert!(provider.rename_static("OldDance", "NewDisplay").is_ok());

        // The key should remain the same
        assert!(provider.statics.contains_key("OldDance"));
        // The display name should change
        assert_eq!(provider.statics.get("OldDance").unwrap().name, "NewDisplay");
    }

    #[test]
    fn test_update_static_color() {
        let mut provider = test_provider();
        provider.ensure_static("ColorDance");

        let color = Color::from_rgb(1.0, 0.0, 0.0);
        assert!(
            provider
                .update_static_color("ColorDance", Some(color), false)
                .is_ok()
        );

        assert_eq!(
            provider.statics.get("ColorDance").unwrap().color,
            Some(color)
        );

        // Revert to None
        assert!(
            provider
                .update_static_color("ColorDance", None, false)
                .is_ok()
        );
        assert_eq!(provider.statics.get("ColorDance").unwrap().color, None);
    }

    #[test]
    fn test_toggle_static_favorite() {
        let mut provider = test_provider();
        provider.ensure_static("FavDance");

        assert!(!provider.statics.get("FavDance").unwrap().is_favorite);

        provider.toggle_static_favorite("FavDance");
        assert!(provider.statics.get("FavDance").unwrap().is_favorite);

        provider.toggle_static_favorite("FavDance");
        assert!(!provider.statics.get("FavDance").unwrap().is_favorite);
    }

    #[test]
    fn test_merge_statics() {
        let mut provider = test_provider();
        provider.ensure_static("SourceDance");
        provider.ensure_static("TargetDance");
        let song = SongInfo {
            dance: "SourceDance".to_string(),
            ..Default::default()
        };
        provider.playlist.push(PlaylistItem {
            song,
            played: false,
        });

        assert!(provider.merge_statics("SourceDance", "TargetDance").is_ok());

        assert!(!provider.statics.contains_key("SourceDance"));
        assert!(provider.statics.contains_key("TargetDance"));
        assert_eq!(provider.playlist[0].song.dance, "TargetDance");
    }
}
