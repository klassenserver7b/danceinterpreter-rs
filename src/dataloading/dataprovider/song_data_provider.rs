use crate::dataloading::songinfo::SongInfo;
use std::cmp::PartialEq;
use crate::traktor_api::TraktorDataProvider;

#[derive(Default, Debug, PartialEq, Clone)]
pub enum SongDataSource {
    #[default]
    Blank,
    Traktor,
    Other(SongInfo),
    Static(usize),
    Playlist(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum SongChange {
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

#[derive(Default)]
pub struct SongDataProvider {
    pub playlist_songs: Vec<SongInfo>,
    pub playlist_played: Vec<bool>,

    pub statics: Vec<SongInfo>,

    pub traktor_provider: TraktorDataProvider,

    pub current: SongDataSource,
    pub next: Option<SongDataSource>,
}

impl SongDataProvider {
    pub fn set_vec(&mut self, vec: Vec<SongInfo>) {
        self.playlist_songs = vec;
        self.playlist_played = vec![false; self.playlist_songs.len()];

        if !self.playlist_songs.is_empty() {
            self.current = SongDataSource::Playlist(0);
        } else {
            self.current = SongDataSource::Blank;
        }
    }

    pub fn set_statics(&mut self, vec: Vec<SongInfo>) {
        self.statics = vec;
    }

    fn set_current_as_played(&mut self) {
        let SongDataSource::Playlist(i) = self.current else {
            return;
        };

        if let Some(v) = self.playlist_played.get_mut(i) {
            *v = true;
        }
    }

    pub fn get_current_song_info(&self) -> Option<&SongInfo> {
        match self.current {
            SongDataSource::Static(i) => self.statics.get(i),
            SongDataSource::Playlist(i) => self.playlist_songs.get(i),
            SongDataSource::Other(ref song) => Some(song),
            SongDataSource::Blank => None,
            SongDataSource::Traktor => self.traktor_provider.get_song_info(),
        }
    }
    pub fn get_next_song_info(&self) -> Option<&SongInfo> {
        if let Some(next) = self.next.as_ref() {
            return match next {
                SongDataSource::Static(i) => self.statics.get(*i),
                SongDataSource::Playlist(i) => self.playlist_songs.get(*i),
                SongDataSource::Other(ref song) => Some(song),
                SongDataSource::Blank => None,
                SongDataSource::Traktor => None,
            };
        }

        match self.current {
            SongDataSource::Static(_) => None,
            SongDataSource::Playlist(i) => self.playlist_songs.get(i + 1),
            SongDataSource::Other(ref song) => Some(song),
            SongDataSource::Blank => None,
            SongDataSource::Traktor => None,
        }
    }

    pub fn prev(&mut self) {
        let SongDataSource::Playlist(current_index) = self.current else {
            return;
        };

        if current_index == 0 {
            return;
        }

        self.set_current_as_played();
        self.current = SongDataSource::Playlist(current_index - 1);
    }

    pub fn next(&mut self) {
        if let Some(next) = self.next.take() {
            self.set_current_as_played();
            self.current = next;
            return;
        }

        let SongDataSource::Playlist(current_index) = self.current else {
            return;
        };

        if current_index == self.playlist_songs.len() - 1 {
            return;
        }

        self.set_current_as_played();
        self.current = SongDataSource::Playlist(current_index + 1);
    }

    #[allow(dead_code)]
    pub fn set_current(&mut self, n: SongDataSource) {
        self.set_current_as_played();

        match n {
            SongDataSource::Static(i) => {
                if self.playlist_songs.get(i).is_some() {
                    self.current = n;
                }
            }
            SongDataSource::Playlist(i) => {
                if self.playlist_songs.get(i).is_some() {
                    self.current = n;
                }
            }
            _ => self.current = n,
        }
    }

    pub fn set_next(&mut self, next: SongDataSource) {
        self.next = Some(next);
    }

    pub fn append_song(&mut self, song: SongInfo) {
        self.playlist_songs.push(song);
        self.playlist_played.push(false);
    }

    pub fn delete_song(&mut self, song: SongDataSource) {
        if let SongDataSource::Playlist(i) = song {
            self.playlist_songs.remove(i);
            self.playlist_played.remove(i);
        } else if let SongDataSource::Static(i) = song {
            self.statics.remove(i);
        }
    }

    pub fn handle_song_change(&mut self, change: SongChange) {
        match change {
            SongChange::Blank => {
                self.set_current_as_played();
                self.current = SongDataSource::Blank;
            }
            SongChange::Traktor => {
                self.set_current_as_played();
                self.current = SongDataSource::Traktor;
            }
            SongChange::StaticAbsolute(index) => {
                self.set_current_as_played();
                self.current = SongDataSource::Static(index);
            }
            SongChange::PlaylistAbsolute(index) => {
                self.set_current_as_played();
                self.current = SongDataSource::Playlist(index);
            }
            SongChange::Previous => {
                self.prev();
            }
            SongChange::Next => {
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
}
