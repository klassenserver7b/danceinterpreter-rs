use crate::dataloading::displayable_data::DisplayableData;
use crate::dataloading::songinfo::SongInfo;
use crate::dataloading::staticinfo::StaticInfo;
use crate::traktor_api;
use crate::traktor_api::TraktorDataProvider;
use std::cmp::PartialEq;

pub enum DeletedItem {
    Playlist {
        index: usize,
        song: SongInfo,
        played: bool,
    },
    Static {
        index: usize,
        static_info: StaticInfo,
    },
}

#[derive(Default, Debug, PartialEq, Clone)]
pub enum ItemSource {
    #[default]
    Blank,
    Traktor,
    Other(SongInfo),
    Static(usize),
    Playlist(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum ItemChange {
    Blank,
    Traktor,
    StaticAbsolute(usize),
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

#[derive(Debug, Clone)]
pub enum StaticDataEdit {
    Name(String),
}

#[derive(Default)]
pub struct DataProvider {
    pub playlist_songs: Vec<SongInfo>,
    pub playlist_played: Vec<bool>,

    pub statics: Vec<StaticInfo>,

    pub deleted_items: Vec<DeletedItem>,

    pub traktor_provider: TraktorDataProvider,

    pub current: ItemSource,
    pub next: Option<ItemSource>,

    should_scroll: bool,
}

impl DataProvider {
    pub fn set_vec(&mut self, vec: Vec<SongInfo>) {
        self.playlist_songs = vec;
        self.playlist_played = vec![false; self.playlist_songs.len()];

        if !self.playlist_songs.is_empty() {
            self.current = ItemSource::Playlist(0);
        } else {
            self.current = ItemSource::Blank;
        }
    }

    pub fn set_statics(&mut self, vec: Vec<StaticInfo>) {
        self.statics = vec;
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

        if let Some(v) = self.playlist_played.get_mut(i) {
            *v = true;
        }
    }

    pub fn get_current_displayable_data(&self) -> Option<DisplayableData> {
        match self.current {
            ItemSource::Static(i) => self.statics.get(i).map(|s| s.into()),
            ItemSource::Playlist(i) => self.playlist_songs.get(i).map(|s| s.into()),
            ItemSource::Other(ref song) => Some(song.into()),
            ItemSource::Blank => None,
            ItemSource::Traktor => self.traktor_provider.get_song_info().map(|s| s.into()),
        }
    }
    pub fn get_next_displayable_data(&self) -> Option<DisplayableData> {
        if let Some(next) = self.next.as_ref() {
            return match next {
                ItemSource::Static(i) => self.statics.get(*i).map(|s| s.into()),
                ItemSource::Playlist(i) => self.playlist_songs.get(*i).map(|s| s.into()),
                ItemSource::Other(song) => Some(song.into()),
                ItemSource::Blank => None,
                ItemSource::Traktor => self.traktor_provider.get_next_song_info().map(|s| s.into()),
            };
        }

        match self.current {
            ItemSource::Static(_) => None,
            ItemSource::Playlist(i) => self.playlist_songs.get(i + 1).map(|s| s.into()),
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

        if current_index == self.playlist_songs.len() - 1 {
            return;
        }

        self.set_current_as_played();
        self.current = ItemSource::Playlist(current_index + 1);
    }

    pub fn set_current(&mut self, n: ItemSource) {
        self.set_current_as_played();

        match n {
            ItemSource::Static(i) => {
                if self.playlist_songs.get(i).is_some() {
                    self.current = n;
                }
            }
            ItemSource::Playlist(i) => {
                if self.playlist_songs.get(i).is_some() {
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
        self.playlist_songs.push(song);
        self.playlist_played.push(false);
    }

    pub fn add_static(&mut self) {
        self.statics.push(StaticInfo::default());
    }

    pub fn toggle_static_favorite(&mut self, index: usize) {
        if let Some(song) = self.statics.get_mut(index) {
            song.is_favorite = !song.is_favorite;
        }
    }

    pub fn handle_static_data_edit(&mut self, index: usize, edit: StaticDataEdit) {
        if let Some(s) = self.statics.get_mut(index) {
            match edit {
                StaticDataEdit::Name(name) => s.name = name,
            }
        }
    }

    pub fn delete_item(&mut self, song: ItemSource) {
        if self.current == song {
            self.current = ItemSource::Blank;
        } else if self.next == Some(song.clone()) {
            self.next = None;
        }

        if let ItemSource::Playlist(i) = song {
            let song_info = self.playlist_songs.remove(i);
            let played = self.playlist_played.remove(i);
            self.deleted_items.push(DeletedItem::Playlist {
                index: i,
                song: song_info,
                played,
            });
        } else if let ItemSource::Static(i) = song {
            let static_info = self.statics.remove(i);
            self.deleted_items.push(DeletedItem::Static {
                index: i,
                static_info,
            });
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
                    let insert_idx = index.min(self.playlist_songs.len());
                    self.playlist_songs.insert(insert_idx, song);
                    self.playlist_played.insert(insert_idx, played);
                }
                DeletedItem::Static { index, static_info } => {
                    let insert_idx = index.min(self.statics.len());
                    self.statics.insert(insert_idx, static_info);
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

    pub fn handle_song_data_edit(&mut self, i: usize, edit: SongDataEdit) {
        if let Some(song) = self.playlist_songs.get_mut(i) {
            match edit {
                SongDataEdit::Title(title) => {
                    song.title = title;
                }
                SongDataEdit::Artist(artist) => {
                    song.artist = artist;
                }
                SongDataEdit::Dance(dance) => {
                    song.dance = dance;
                }
            }
        }
    }

    pub fn process_traktor_message(&mut self, message: traktor_api::ServerMessage) {
        self.set_current_as_played();
        self.traktor_provider
            .process_message(message, &self.playlist_songs);
    }

    pub fn get_current_traktor_index(&self) -> Option<usize> {
        self.traktor_provider
            .get_current_index(&self.playlist_songs)
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

    pub fn get_play_state(&self, playlist_index: usize) -> (bool, bool, bool, bool) {
        let mut is_current = false;
        let mut is_next = false;
        let mut is_traktor = false;
        let is_played = self
            .playlist_played
            .get(playlist_index)
            .copied()
            .unwrap_or(false);

        if let ItemSource::Playlist(i) = self.current {
            is_current = playlist_index == i;
            is_next = playlist_index == (i + 1);
        }

        if let Some(ItemSource::Playlist(i)) = self.next {
            is_next = playlist_index == i;
        }

        if matches!(self.current, ItemSource::Traktor)
            && let Some(index) = self.get_current_traktor_index()
        {
            is_traktor = playlist_index == index;
        }

        (is_current, is_next, is_traktor, is_played)
    }
}
