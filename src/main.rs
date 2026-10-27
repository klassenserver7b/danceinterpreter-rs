mod async_utils;
mod dataloading;
mod macros;
mod traktor_api;
mod ui;

use crate::dataloading::dataprovider::{
    DataProvider, ItemChange, ItemSource, SongDataEdit, SubmitStaticResult,
};
use crate::dataloading::id3tagreader::read_song_info_from_filepath;
use crate::dataloading::m3uloader::{PlaylistLoad, discover_playlists, load_tag_data_from_m3u};
use crate::dataloading::songinfo::SongInfo;
use crate::traktor_api::{ServerMessage, StateUpdate, TraktorMessage, TraktorSyncAction};
use crate::ui::config_window::toast;
use crate::ui::config_window::{
    ConfigWindow, DialogState, EditField, EditSnapshot, PLAYLIST_SCROLLABLE_ID, Toast, View,
    edit_input_id, static_edit_input_id,
};
use crate::ui::song_window::{Backdrop, SongWindow};
use crate::ui::theme;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2 as Matcher;
use iced::font::Weight;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::operation::{focus, scroll_by, snap_to};
use iced::widget::scrollable::{AbsoluteOffset, RelativeOffset};
use iced::widget::space::horizontal;
use iced::window::icon::from_file_data;
use iced::{Color, Element, Size, Subscription, Task, Theme, event, exit, keyboard, time, window};
use iced_aw::ICED_AW_FONT_BYTES;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::env::var;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn main() -> iced::Result {
    iced::daemon(
        DanceInterpreter::new,
        DanceInterpreter::update,
        DanceInterpreter::view,
    )
    .title(DanceInterpreter::title)
    .theme(DanceInterpreter::theme)
    .default_font(theme::body_font(Weight::Normal))
    .font(ICED_AW_FONT_BYTES)
    .subscription(DanceInterpreter::subscription)
    .run()
}

pub trait Window {
    fn new(id: window::Id) -> Self;

    fn on_resize(&mut self, size: Size);
    fn on_close(&mut self);

    fn is_closed(&self) -> bool;
}

struct DanceInterpreter {
    config_window: ConfigWindow,
    song_window: SongWindow,

    data_provider: DataProvider,
    modifiers: Modifiers,
}

#[derive(Debug, Clone)]
pub enum SearchMessage {
    Toggle,
    Changed(String),
    Clear,
}

#[derive(Debug, Clone)]
pub enum StaticsMessage {
    Reload,
    ToggleFavorite(String),
    /// Opens the inline name editor for the static with this key.
    BeginEdit(String),
    /// Commits the draft (Enter or click-away) and runs the rename/merge
    /// flow.
    EndEdit,
    /// Closes the editor without writing anything (Escape).
    CancelEdit,
    DraftChanged(String),
    UpdateColor(String, iced::Color),
    PreviewColor(String, iced::Color),
    ToggleColorPicker(String),
    AddBlank,
    AddNamed(String),
    RequestMerge(String, String),
    ConfirmMerge(String, String),
}

#[derive(Debug, Clone)]
pub enum SongWindowMessage {
    EnableImage(bool),
    EnableNextDance(bool),
    ChangeScale(f32),
    SetBackdrop(Backdrop),
}

#[derive(Debug, Clone)]
pub enum QuickAddMessage {
    UpdateTitle(String),
    UpdateArtist(String),
    UpdateDance(String),
    Submit,
}

#[derive(Debug, Clone)]
pub enum DialogMessage {
    Close,
    RequestDelete(ItemSource),
    ConfirmDelete,
    UndoDelete,
    ChangeAddStaticName(String),
    /// Chooser: the selected playlist was picked in the pick list.
    SelectPlaylist(PathBuf),
    /// Chooser: load the selected playlist and close the dialog.
    ConfirmPlaylist,
}

#[derive(Debug, Clone)]
pub enum Message {
    Noop,

    WindowOpened(window::Id),
    WindowResized((window::Id, Size)),
    WindowClosed(window::Id),

    SetConfigTheme(Theme),

    ToggleFullscreen,
    SetFullscreen(bool),

    SetView(View),
    Traktor(TraktorMessage),
    Search(SearchMessage),
    Statics(StaticsMessage),
    SongWindow(SongWindowMessage),
    QuickAdd(QuickAddMessage),
    Dialog(DialogMessage),
    OpenDialog(DialogState),

    Toast(String),
    DismissToast,
    Tick,
    FocusSongWindow,
    EscapePressed,
    ToggleNavSidebar,
    ToggleInfoRail,

    EnableAutoscroll(bool),

    ScrollBy(f32),
    SnapTo(RelativeOffset),

    OpenPlaylist,
    AddBlankEntry,

    DeleteItem(ItemSource),
    AddBlankSong(RelativeOffset),
    BeginSongEdit(usize, EditField),
    EndSongEdit,
    EditDraftChanged(String),
    CancelSongEdit,

    AddTraktorSong,
    AddAudioFiles,

    UpdateTraktorPresets,

    FileDropped(PathBuf),
    ItemChanged(ItemChange),
    SetNextItem(ItemSource),

    Animate,
    ModifiersChanged(Modifiers),
}

impl DanceInterpreter {
    pub fn new() -> (Self, Task<Message>) {
        let mut tasks = Vec::new();

        let icon = from_file_data(
            include_bytes!(res_file!("icon_light.png")),
            Some(image::ImageFormat::Png),
        )
        .ok();

        let (config_window, cw_opened) = Self::open_window(window::Settings {
            platform_specific: Self::get_platform_specific(),
            icon: icon.clone(),
            ..Default::default()
        });
        let (song_window, sw_opened) = Self::open_window(window::Settings {
            platform_specific: Self::get_platform_specific(),
            icon: icon.clone(),
            ..Default::default()
        });

        let mut state = Self {
            config_window,
            song_window,

            data_provider: DataProvider::default(),
            modifiers: Modifiers::default(),
        };

        tasks.push(cw_opened);
        tasks.push(sw_opened);

        // Restore the theme chosen in a previous session (Dracula on first
        // run), then pick the window icons matching it.
        if let Some(theme) = load_saved_theme() {
            state.config_window.theme = theme;
        }
        tasks.push(state.theme_icon_task());

        // Loads each bundled font binary. `include_bytes!` needs literals, so
        // the list is expanded by a tiny macro instead of a runtime loop.
        macro_rules! load_fonts {
            ($($file:literal),* $(,)?) => {
                $(
                    tasks.push(
                        iced::font::load(include_bytes!(res_file!($file)))
                            .map(|_| Message::Noop),
                    );
                )*
            };
        }

        // Bundled design fonts: Material Symbols/Icons for UI icons, DM Sans
        // (body) and Manrope (headings) as static instances.
        load_fonts!(
            "material-symbols.ttf",
            "material-icons.ttf",
            "fonts/DMSans-Regular.ttf",
            "fonts/DMSans-Medium.ttf",
            "fonts/DMSans-SemiBold.ttf",
            "fonts/DMSans-Bold.ttf",
            "fonts/Manrope-Medium.ttf",
            "fonts/Manrope-SemiBold.ttf",
            "fonts/Manrope-Bold.ttf",
            "fonts/Manrope-ExtraBold.ttf",
        );

        tasks.push(Task::done(Message::Statics(StaticsMessage::Reload)));

        (state, Task::batch(tasks))
    }

    fn open_window<T: Window>(settings: window::Settings) -> (T, Task<Message>) {
        let (id, open) = window::open(settings);
        (T::new(id), open.map(Message::WindowOpened))
    }

    fn get_platform_specific() -> window::settings::PlatformSpecific {
        #[cfg(target_os = "linux")]
        return window::settings::PlatformSpecific {
            application_id: "danceinterpreter".to_string(),
            ..Default::default()
        };

        #[cfg(not(target_os = "linux"))]
        return Default::default();
    }

    /// Commits the open inline editor's draft to the playlist (like pressing
    /// Enter): writes the draft into the song and, for the dance field,
    /// matches it against the known statics. No-op when nothing is edited.
    fn commit_song_edit(&mut self) {
        if let Some((idx, field)) = self.config_window.editing.take() {
            let draft = std::mem::take(&mut self.config_window.edit_draft);
            let edit = match field {
                EditField::Title => SongDataEdit::Title(draft),
                EditField::Artist => SongDataEdit::Artist(draft),
                EditField::Dance => SongDataEdit::Dance(draft),
            };
            self.data_provider.handle_song_data_edit(idx, edit);

            if field == EditField::Dance {
                self.data_provider.handle_song_data_submit(idx);
            }
        }
    }

    /// Discards the open editor's draft without writing anything (the
    /// playlist mutating under the editor). Also expires the Escape rollback
    /// snapshot: with the playlist changing, the snapshotted index may end
    /// up pointing at another song.
    fn cancel_song_edit(&mut self) {
        self.config_window.editing = None;
        self.config_window.edit_draft.clear();
        self.config_window.edit_snapshot = None;
    }

    /// Appends a song and settles the UI around the mutation: the inline
    /// editor cannot survive a playlist change, and newly seen dances must
    /// become statics. Every playlist append goes through this so the arms
    /// cannot drift apart.
    fn append_song_and_finalize(&mut self, song: SongInfo) {
        self.cancel_song_edit();
        self.data_provider.append_song(song);
        self.data_provider.ensure_statics_for_playlist();
    }

    /// Replaces the whole playlist with a freshly loaded m3u and settles the
    /// UI (the inline editor would point into stale rows otherwise).
    fn load_playlist_file(&mut self, path: &Path, load: PlaylistLoad) -> Task<Message> {
        self.cancel_song_edit();
        let total = load.total();
        let count = load.songs.len();
        let skipped = load.skipped.len();
        self.data_provider.set_vec(load.songs);
        self.config_window.playlist_file = Some(path.to_path_buf());
        println!("Loaded playlist {path:?} with {count} of {total} songs");
        for skipped in &load.skipped {
            println!(
                "  skipped {} -> {}: {}",
                skipped.entry,
                skipped.path.display(),
                skipped.reason.describe()
            );
        }

        let mut summary = if skipped == 0 {
            format!("Playlist loaded ({count} songs)")
        } else {
            format!("Playlist loaded \u{2014} {count}/{total} tracks, {skipped} not accessible")
        };
        // A playlist under the document portal fuse means the sandbox has no
        // direct filesystem access here; skipped entries then live outside
        // the folder granted in the folder pick.
        if skipped > 0 && in_flatpak() && is_document_portal_path(path) {
            summary.push_str(" \u{2014} entries outside the granted folder cannot be read");
        }
        toast::task(summary)
    }

    /// Loads an m3u and installs it as the current playlist.
    fn load_playlist_from(&mut self, file: PathBuf) -> Task<Message> {
        match load_tag_data_from_m3u(&file) {
            Ok(load) => self.load_playlist_file(&file, load),
            Err(err) => {
                println!("Playlist load failed for {file:?}: {err}");
                toast::task(format!("Could not read playlist file: {err}"))
            }
        }
    }

    /// Flatpak flow: pick a folder (the pick grants sandbox access to its
    /// whole subtree through the document portal), discover the playlists
    /// inside it and load the chosen one. All paths stay inside the grant,
    /// so reads work regardless of whether the portal returned real paths
    /// or fuse paths.
    #[cfg(target_os = "linux")]
    fn open_playlist_via_folder_grant(&mut self) -> Task<Message> {
        let Some(folder) = FileDialog::new()
            .set_title("Select folder containing m3u AND audio files (this grants sandbox access)")
            .set_directory(default_browse_dir())
            .pick_folders()
            .and_then(|folders| folders.into_iter().next())
        else {
            return ().into();
        };

        match discover_playlists(&folder).as_slice() {
            [] => {
                println!("No playlist found in granted folder {folder:?}");
                let name = folder.file_name().map_or_else(
                    || folder.display().to_string(),
                    |name| name.to_string_lossy().into_owned(),
                );
                toast::task(format!("No .m3u playlist found in \u{201C}{name}\u{201D}"))
            }
            [only] => self.load_playlist_from(only.clone()),
            candidates => {
                self.config_window.playlist_candidates = candidates.to_vec();
                self.config_window.playlist_granted_folder = folder;
                self.config_window.playlist_selected = None;
                Task::done(Message::OpenDialog(DialogState::PlaylistChooser))
            }
        }
    }

    /// Writes a static's color, forcing the alpha to fully opaque (picked
    /// colors often arrive translucent). `save` persists to the statics
    /// file — only the final color pick saves, previews stay volatile.
    fn set_static_color(&mut self, name: &str, mut color: Color, save: bool) {
        color.a = 1.0;
        let _ = self
            .data_provider
            .update_static_color(name, Some(color), save);
    }

    /// Scrolls the playlist proportionally to `index` (0-based row).
    fn snap_to_index(&self, index: usize) -> Task<Message> {
        let offset_y =
            index as f32 / std::cmp::max(1, self.data_provider.playlist().len() - 1) as f32;
        Task::done(Message::SnapTo(RelativeOffset {
            x: 0.0,
            y: offset_y,
        }))
    }

    /// Sets the window icons to the variant matching the current theme
    /// (light icon for light themes, dark icon for dark ones).
    fn theme_icon_task(&self) -> Task<Message> {
        let icon = from_file_data(
            if theme::Tokens::of(&self.config_window.theme).is_light {
                include_bytes!(res_file!("icon_light.png"))
            } else {
                include_bytes!(res_file!("icon_dark.png"))
            },
            Some(image::ImageFormat::Png),
        );

        if let Ok(icon) = icon {
            Task::batch([
                window::set_icon(self.song_window.id, icon.clone()),
                window::set_icon(self.config_window.id, icon),
            ])
        } else {
            ().into()
        }
    }

    pub fn title(&self, window_id: window::Id) -> String {
        if self.config_window.id == window_id {
            "Danceinterpreter".to_string()
        } else if self.song_window.id == window_id {
            "Danceinterpreter · Audience".to_string()
        } else {
            String::new()
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if self.config_window.id == window_id {
            self.config_window.view(self)
        } else if self.song_window.id == window_id {
            self.song_window.view(self)
        } else {
            horizontal().into()
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        self.config_window.traktor_power_cache.clear();
        self.config_window.traktor_restart_cache.clear();

        match message {
            Message::WindowOpened(_) => ().into(),
            Message::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                ().into()
            }
            Message::WindowResized((window_id, size)) => {
                if self.config_window.id == window_id {
                    self.config_window.on_resize(size);
                } else if self.song_window.id == window_id {
                    self.song_window.on_resize(size);
                }

                ().into()
            }
            Message::WindowClosed(window_id) => {
                if self.config_window.id == window_id {
                    self.config_window.on_close();

                    if self.song_window.is_closed() {
                        exit()
                    } else {
                        window::close(self.song_window.id)
                    }
                } else if self.song_window.id == window_id {
                    self.song_window.on_close();

                    if self.config_window.is_closed() {
                        exit()
                    } else {
                        window::close(self.config_window.id)
                    }
                } else {
                    ().into()
                }
            }

            Message::SetConfigTheme(theme) => {
                save_theme(&theme);
                self.config_window.theme = theme;
                self.theme_icon_task()
            }

            Message::ToggleFullscreen => {
                let song_window_id = self.song_window.id;

                window::mode(song_window_id)
                    .map(|mode| Message::SetFullscreen(mode != window::Mode::Fullscreen))
            }
            Message::SetFullscreen(fullscreen) => {
                let song_window_id = self.song_window.id;

                window::set_mode(
                    song_window_id,
                    if fullscreen {
                        window::Mode::Fullscreen
                    } else {
                        window::Mode::Windowed
                    },
                )
            }

            Message::FocusSongWindow => window::gain_focus(self.song_window.id),

            Message::ToggleNavSidebar => {
                self.config_window.toggle_nav_sidebar();
                ().into()
            }
            Message::ToggleInfoRail => {
                self.config_window.toggle_info_rail();
                ().into()
            }

            Message::EscapePressed => {
                if self.config_window.editing.is_some() {
                    Task::done(Message::CancelSongEdit)
                } else if self.config_window.static_editing.is_some() {
                    Task::done(Message::Statics(StaticsMessage::CancelEdit))
                } else if self.config_window.search_visible {
                    Task::done(Message::Search(SearchMessage::Toggle))
                } else if self.config_window.active_dialog.is_some() {
                    self.close_active_dialog()
                } else {
                    ().into()
                }
            }

            Message::Toast(text) => {
                self.config_window.toast = Some(Toast::new(text));
                ().into()
            }
            Message::DismissToast => {
                self.config_window.toast = None;
                ().into()
            }
            Message::Tick => {
                if let Some(toast) = &self.config_window.toast
                    && toast.expired(Instant::now())
                {
                    self.config_window.toast = None;
                }
                ().into()
            }

            Message::OpenDialog(state) => {
                self.config_window.active_dialog = Some(state);
                ().into()
            }
            Message::OpenPlaylist => {
                #[cfg(target_os = "linux")]
                if var("container").is_ok() {
                    // Flatpak: the folder pick doubles as the sandbox access
                    // grant. Playlist discovery, entry resolution and tag
                    // reads all stay inside the granted folder afterwards,
                    // so they work through the document portal no matter
                    // whether it hands out real paths or fuse paths.
                    return self.open_playlist_via_folder_grant();
                }

                // Native: pick the playlist file directly.
                let Some(file) = FileDialog::new()
                    .add_filter("Playlist", &["m3u", "m3u8"])
                    .add_filter("Any(*)", &["*"])
                    .set_title("Select playlist file")
                    .set_directory(default_browse_dir())
                    .pick_file()
                else {
                    return ().into();
                };

                self.load_playlist_from(file)
            }

            Message::AddBlankEntry => {
                if self.config_window.view == View::Statics {
                    Task::done(Message::Statics(StaticsMessage::AddBlank))
                } else {
                    Task::done(Message::AddBlankSong(RelativeOffset::END))
                }
            }

            Message::Statics(msg) => match msg {
                StaticsMessage::Reload => {
                    // The editor cannot survive a reload; the key may be gone
                    // or carry different data afterwards.
                    self.config_window.static_editing = None;
                    self.data_provider.load_statics();
                    toast::task("Statics reloaded")
                }
                StaticsMessage::ToggleFavorite(name) => {
                    self.data_provider.toggle_static_favorite(&name);
                    ().into()
                }
                StaticsMessage::BeginEdit(name) => {
                    // The draft only lives in the editor until it is
                    // committed, so Escape just closes it — no snapshot
                    // needed for a single-field edit.
                    self.config_window.edit_draft = self
                        .data_provider
                        .statics()
                        .get(&name)
                        .map(|s| s.name.clone())
                        .unwrap_or_default();
                    self.config_window.static_editing = Some(name.clone());
                    focus(static_edit_input_id(&name))
                }
                StaticsMessage::EndEdit => {
                    if let Some(key) = self.config_window.static_editing.take() {
                        // Write the typed draft into the static, then run the
                        // rename/merge flow over the entry's key.
                        let draft = std::mem::take(&mut self.config_window.edit_draft);
                        let _ = self.data_provider.rename_static(&key, &draft);
                        match self.data_provider.process_static_name_submit(&key) {
                            SubmitStaticResult::NeedsMerge { old_name, new_name } => {
                                Task::done(new_name).map(move |new_name| {
                                    Message::Statics(StaticsMessage::RequestMerge(
                                        old_name.clone(),
                                        new_name,
                                    ))
                                })
                            }
                            SubmitStaticResult::Success => toast::task("Static renamed"),
                            SubmitStaticResult::Failed(message) => {
                                toast::task(format!("Rename failed: {message}"))
                            }
                            SubmitStaticResult::Unchanged | SubmitStaticResult::NotFound => {
                                ().into()
                            }
                        }
                    } else {
                        ().into()
                    }
                }
                StaticsMessage::CancelEdit => {
                    self.config_window.static_editing = None;
                    self.config_window.edit_draft.clear();
                    ().into()
                }
                StaticsMessage::DraftChanged(draft) => {
                    self.config_window.edit_draft = draft;
                    ().into()
                }
                StaticsMessage::UpdateColor(name, color) => {
                    self.set_static_color(&name, color, true);
                    self.config_window.color_picker_open = None;
                    self.config_window.color_picker_old_color = None;
                    ().into()
                }
                StaticsMessage::PreviewColor(name, color) => {
                    self.set_static_color(&name, color, false);
                    ().into()
                }
                StaticsMessage::ToggleColorPicker(name) => {
                    if self.config_window.color_picker_open.as_ref() == Some(&name) {
                        if let Some(old_color) = self.config_window.color_picker_old_color.take() {
                            let _ = self.data_provider.update_static_color(
                                &name,
                                Some(old_color),
                                false,
                            );
                        } else {
                            let _ = self.data_provider.update_static_color(&name, None, false);
                        }
                        self.config_window.color_picker_open = None;
                    } else {
                        if let Some(static_info) = self.data_provider.statics().get(&name) {
                            self.config_window.color_picker_old_color = static_info.color;
                        }
                        self.config_window.color_picker_open = Some(name);
                    }
                    ().into()
                }
                StaticsMessage::AddBlank => {
                    self.data_provider.add_static();
                    ().into()
                }
                StaticsMessage::AddNamed(name) => {
                    self.data_provider.add_named_static(&name);
                    self.config_window.add_static_name.clear();
                    self.config_window.active_dialog = None;
                    toast::task(format!("Static \u{201C}{}\u{201D} added", name.trim()))
                }
                StaticsMessage::RequestMerge(old_name, new_name) => {
                    self.config_window.active_dialog =
                        Some(DialogState::MergeStatic { old_name, new_name });
                    ().into()
                }
                StaticsMessage::ConfirmMerge(old_name, new_name) => {
                    self.config_window.active_dialog = None;
                    match self.data_provider.merge_statics(&old_name, &new_name) {
                        Ok(()) => toast::task(format!("Merged into \u{201C}{}\u{201D}", new_name)),
                        Err(err) => toast::task(format!("Merge failed: {err}")),
                    }
                }
            },

            Message::QuickAdd(msg) => match msg {
                QuickAddMessage::UpdateTitle(title) => {
                    self.config_window.quick_title = title;
                    ().into()
                }
                QuickAddMessage::UpdateArtist(artist) => {
                    self.config_window.quick_artist = artist;
                    ().into()
                }
                QuickAddMessage::UpdateDance(dance) => {
                    self.config_window.quick_dance = dance;
                    ().into()
                }
                QuickAddMessage::Submit => {
                    if self.config_window.quick_title.is_empty()
                        && self.config_window.quick_artist.is_empty()
                        && self.config_window.quick_dance.is_empty()
                    {
                        return ().into();
                    }

                    let song = SongInfo {
                        title: self.config_window.quick_title.clone(),
                        artist: self.config_window.quick_artist.clone(),
                        dance: self.config_window.quick_dance.clone(),
                        ..Default::default()
                    };
                    self.append_song_and_finalize(song);
                    self.config_window.quick_title.clear();
                    self.config_window.quick_artist.clear();
                    self.config_window.quick_dance.clear();
                    toast::task("Song added to playlist")
                }
            },

            Message::Search(msg) => match msg {
                SearchMessage::Toggle => {
                    self.config_window.search_visible = !self.config_window.search_visible;
                    if !self.config_window.search_visible {
                        self.config_window.search_query.clear();
                        ().into()
                    } else {
                        focus(self.config_window.search_input_id.clone())
                    }
                }
                SearchMessage::Changed(query) => {
                    self.config_window.search_query = query;
                    self.scroll_to_first_search_match()
                }
                SearchMessage::Clear => {
                    self.config_window.search_query.clear();
                    ().into()
                }
            },

            Message::Dialog(msg) => match msg {
                DialogMessage::Close => self.close_active_dialog(),
                DialogMessage::SelectPlaylist(path) => {
                    self.config_window.playlist_selected = Some(path);
                    ().into()
                }
                DialogMessage::ConfirmPlaylist => {
                    let selected = self
                        .config_window
                        .playlist_selected
                        .clone()
                        .or_else(|| self.config_window.playlist_candidates.first().cloned());
                    let close = self.close_active_dialog();
                    match selected {
                        Some(file) => Task::batch([close, self.load_playlist_from(file)]),
                        None => close,
                    }
                }
                DialogMessage::RequestDelete(item) => {
                    if self.modifiers.shift() {
                        return Task::done(Message::DeleteItem(item));
                    }

                    self.config_window.active_dialog = Some(DialogState::Delete(item));
                    ().into()
                }
                DialogMessage::ConfirmDelete => {
                    if let Some(DialogState::Delete(item)) = self.config_window.active_dialog.take()
                    {
                        let name = match &item {
                            ItemSource::Playlist(i) => self
                                .data_provider
                                .playlist()
                                .get(*i)
                                .map(|entry| entry.song.title.clone())
                                .unwrap_or_else(|| "Song".to_string()),
                            ItemSource::Static(name) => name.clone(),
                            _ => "Item".to_string(),
                        };

                        self.cancel_song_edit();
                        self.data_provider.delete_item(item);
                        toast::task(format!(
                            "Deleted \u{201C}{}\u{201D} \u{00B7} Ctrl+Z to undo",
                            name
                        ))
                    } else {
                        ().into()
                    }
                }
                DialogMessage::UndoDelete => {
                    self.cancel_song_edit();
                    self.data_provider.undo_delete();
                    toast::task("Delete undone")
                }
                DialogMessage::ChangeAddStaticName(name) => {
                    self.config_window.add_static_name = name;
                    ().into()
                }
            },

            Message::AddTraktorSong => {
                if let Some(song) = self.data_provider.traktor_provider.get_song_info() {
                    let in_playlist = self.data_provider.playlist().iter().any(|item| {
                        item.song.title == song.title && item.song.artist == song.artist
                    });
                    if !in_playlist {
                        // `song` borrows the provider, which the append below
                        // mutates — hence the clone.
                        self.append_song_and_finalize(song.clone());
                        Task::batch([
                            Task::done(Message::SnapTo(RelativeOffset::END)),
                            toast::task("Traktor track added"),
                        ])
                    } else {
                        toast::task("Track is already in the playlist")
                    }
                } else {
                    toast::task("No track loaded in Traktor")
                }
            }

            Message::AddAudioFiles => {
                // Dialog-based fallback for drag & drop, which winit does not
                // implement on native Wayland at all (see README). The
                // portal-backed dialog grants access to the picked files, so
                // they are always readable.
                let Some(files) = FileDialog::new()
                    .add_filter(
                        "Audio",
                        &[
                            "mp3", "wav", "flac", "ogg", "oga", "opus", "m4a", "aac", "wma", "aiff",
                        ],
                    )
                    .add_filter("Any(*)", &["*"])
                    .set_title("Select audio files")
                    .set_directory(default_browse_dir())
                    .pick_files()
                else {
                    return ().into();
                };

                let mut added = 0usize;
                for file in &files {
                    match read_song_info_from_filepath(file) {
                        Ok(song) => {
                            println!("Loaded song {file:?}");
                            self.append_song_and_finalize(song);
                            added += 1;
                        }
                        Err(err) => println!("Could not read audio file {file:?}: {err}"),
                    }
                }

                if added > 0 {
                    Task::batch([
                        Task::done(Message::SnapTo(RelativeOffset::END)),
                        toast::task(format!(
                            "{added} song{} added",
                            if added == 1 { "" } else { "s" }
                        )),
                    ])
                } else {
                    toast::task("No readable audio files selected")
                }
            }

            Message::FileDropped(path) => match load_tag_data_from_m3u(&path) {
                Ok(playlist) => self.load_playlist_file(&path, playlist),
                Err(_) => match read_song_info_from_filepath(&path) {
                    Ok(song_info) => {
                        println!("Loaded song {path:?}");
                        self.append_song_and_finalize(song_info);
                        toast::task("Song added to playlist")
                    }
                    Err(err) => {
                        // Surface the real cause (sandbox denial, unsupported
                        // container, corrupt tag) instead of a generic hint.
                        println!("File drop failed for {path:?}: {err}");
                        let name = path.file_name().map_or_else(
                            || path.display().to_string(),
                            |n| n.to_string_lossy().into_owned(),
                        );
                        let mut text = format!("Could not read \u{201C}{name}\u{201D}: {err}");
                        // Drag & drop delivers real host paths; inside the
                        // sandbox the file is only readable when its location
                        // is covered by the app's filesystem grants.
                        if in_flatpak() && !path.is_file() {
                            text.push_str(
                                " \u{2014} the sandbox cannot access it; use Open playlist and pick its folder",
                            );
                        }
                        toast::task(text)
                    }
                },
            },

            Message::ItemChanged(song_change) => {
                self.data_provider.handle_item_change(song_change);
                self.try_scroll_to_song()
            }

            Message::BeginSongEdit(idx, field) => {
                // A fresh session snapshots the song so Escape can roll the
                // whole session back. Opening another field within a session
                // commits the pending draft first (like Enter), so what was
                // typed is never lost on a field switch.
                if self.config_window.editing.is_none() {
                    self.config_window.edit_snapshot =
                        self.data_provider.playlist().get(idx).map(|item| {
                            (
                                idx,
                                EditSnapshot {
                                    title: item.song.title.clone(),
                                    artist: item.song.artist.clone(),
                                    dance: item.song.dance.clone(),
                                },
                            )
                        });
                }
                self.commit_song_edit();
                self.config_window.edit_draft = self
                    .data_provider
                    .playlist()
                    .get(idx)
                    .map(|item| match field {
                        EditField::Title => item.song.title.clone(),
                        EditField::Artist => item.song.artist.clone(),
                        EditField::Dance => item.song.dance.clone(),
                    })
                    .unwrap_or_default();
                self.config_window.editing = Some((idx, field));
                focus(edit_input_id(idx, field))
            }

            Message::EndSongEdit => {
                // Enter or click-away: write the draft into the song and end
                // the session, expiring the Escape rollback snapshot.
                self.commit_song_edit();
                self.config_window.edit_snapshot = None;
                ().into()
            }

            Message::EditDraftChanged(draft) => {
                self.config_window.edit_draft = draft;
                ().into()
            }

            Message::CancelSongEdit => {
                // Escape rolls the whole session back to the pre-edit values,
                // including drafts auto-committed by field switches.
                if let Some((idx, _)) = self.config_window.editing
                    && let Some((snapshot_idx, snapshot)) = self.config_window.edit_snapshot.take()
                    && snapshot_idx == idx
                {
                    self.data_provider.restore_song_edit(
                        idx,
                        &snapshot.title,
                        &snapshot.artist,
                        &snapshot.dance,
                    );
                }
                self.cancel_song_edit();
                ().into()
            }

            Message::AddBlankSong(offset) => {
                self.cancel_song_edit();
                self.data_provider.append_song(SongInfo::default());
                Task::done(Message::SnapTo(offset))
            }

            Message::DeleteItem(song) => {
                self.cancel_song_edit();
                self.data_provider.delete_item(song);
                ().into()
            }

            Message::SetNextItem(i) => {
                self.data_provider.set_next(i);
                ().into()
            }

            Message::SongWindow(msg) => match msg {
                SongWindowMessage::EnableImage(state) => {
                    self.song_window.enable_image = state;
                    ().into()
                }
                SongWindowMessage::EnableNextDance(state) => {
                    self.song_window.enable_next_dance = state;
                    ().into()
                }
                SongWindowMessage::ChangeScale(value) => {
                    if (0.5..=3.0).contains(&value) {
                        self.song_window.scale = (value * 100.0).round() / 100.0;
                    } else {
                        self.song_window.scale =
                            (((self.song_window.scale + value) * 100.0).round() / 100.0)
                                .clamp(0.5, 3.0);
                    }
                    ().into()
                }
                SongWindowMessage::SetBackdrop(backdrop) => {
                    self.song_window.backdrop = backdrop;
                    ().into()
                }
            },

            Message::EnableAutoscroll(state) => {
                self.config_window.enable_autoscroll = state;
                ().into()
            }

            Message::ScrollBy(frac) => scroll_by(
                PLAYLIST_SCROLLABLE_ID.clone(),
                AbsoluteOffset {
                    x: 0.0,
                    y: self.config_window.size.height / frac,
                },
            ),

            Message::SnapTo(offset) => snap_to(PLAYLIST_SCROLLABLE_ID.clone(), offset),

            Message::SetView(view) => {
                self.config_window.view = view;
                ().into()
            }

            Message::UpdateTraktorPresets => {
                self.config_window
                    .update_traktor_presets(&self.data_provider);
                ().into()
            }

            Message::Traktor(msg) => match msg {
                TraktorMessage::ServerMessage(msg) => {
                    self.data_provider.process_traktor_message(*msg);
                    self.run_traktor_sync_action();

                    if self.config_window.enable_autoscroll {
                        self.try_scroll_to_song()
                    } else {
                        self.data_provider.clear_pending_scroll();
                        ().into()
                    }
                }

                TraktorMessage::EnableServer(enabled) => {
                    self.data_provider.traktor_provider.set_enabled(enabled);
                    ().into()
                }

                TraktorMessage::ChangeAddress(addr) => {
                    self.data_provider.traktor_provider.address = addr;
                    ().into()
                }

                TraktorMessage::SubmitAddress => {
                    self.data_provider.traktor_provider.submitted_address =
                        self.data_provider.traktor_provider.address.clone();
                    ().into()
                }

                TraktorMessage::ChangeAndSubmitAddress(addr) => {
                    let provider = &mut self.data_provider.traktor_provider;
                    provider.address = addr.clone();
                    provider.submitted_address = addr;
                    ().into()
                }

                TraktorMessage::EnableDebugLogging(enabled) => {
                    self.data_provider.traktor_provider.debug_logging = enabled;
                    self.data_provider.traktor_provider.reconnect();
                    ().into()
                }

                TraktorMessage::Reconnect => {
                    self.data_provider.traktor_provider.reconnect();
                    ().into()
                }

                TraktorMessage::EnableSync(enabled) => {
                    self.data_provider.traktor_provider.sync = enabled;
                    self.traktor_provider_force_update()
                }

                TraktorMessage::SetSyncMode(mode) => {
                    self.data_provider.traktor_provider.sync_mode = mode;
                    self.traktor_provider_force_update()
                }

                TraktorMessage::SetNextMode(mode) => {
                    self.data_provider.traktor_provider.next_mode = mode;
                    self.traktor_provider_force_update()
                }

                TraktorMessage::SetNextModeFallback(mode) => {
                    self.data_provider.traktor_provider.next_mode_fallback = mode;
                    self.traktor_provider_force_update()
                }
            },

            Message::Animate => Task::none(),

            Message::Noop => ().into(),
        }
    }

    /// Closes the active dialog, restoring pending state for the merge dialog.
    fn close_active_dialog(&mut self) -> Task<Message> {
        if let Some(DialogState::MergeStatic { old_name, .. }) = &self.config_window.active_dialog {
            // Undo the in-progress rename (the entry's display name was edited
            // inline while typing; reset it back to its map key).
            let _ = self.data_provider.rename_static(old_name, old_name);
        }
        self.config_window.active_dialog = None;
        ().into()
    }

    fn traktor_provider_force_update(&mut self) -> Task<Message> {
        // send fake state update message to enforce sync refresh
        if let Some(mixer_state) = self
            .data_provider
            .traktor_provider
            .state
            .as_ref()
            .map(|s| s.mixer)
        {
            self.data_provider
                .process_traktor_message(ServerMessage::Update(StateUpdate::Mixer(mixer_state)));
            self.run_traktor_sync_action();
            self.try_scroll_to_song()
        } else {
            ().into()
        }
    }

    fn scroll_to_first_search_match(&mut self) -> Task<Message> {
        if self.config_window.search_query.is_empty() {
            return ().into();
        }

        let matcher = Matcher::default();
        let query = &self.config_window.search_query;

        let first_match = self
            .data_provider
            .playlist()
            .iter()
            .enumerate()
            .find(|(_, item)| {
                let song = &item.song;
                matcher
                    .fuzzy_match(
                        &format!("{} {} {}", song.title, song.artist, song.dance),
                        query,
                    )
                    .is_some()
            });

        if let Some((index, _)) = first_match {
            self.snap_to_index(index)
        } else {
            ().into()
        }
    }

    fn run_traktor_sync_action(&mut self) {
        let action = self.data_provider.traktor_provider.take_sync_action();
        if !self.data_provider.traktor_provider.sync {
            return;
        }

        match action {
            TraktorSyncAction::Relative(offset) => {
                if offset >= 0 {
                    for _ in 0..offset {
                        self.data_provider.next();
                    }
                } else {
                    for _ in 0..(-offset) {
                        self.data_provider.prev();
                    }
                }
            }
            TraktorSyncAction::PlaylistAbsolute(pos) => {
                self.data_provider.set_current(ItemSource::Playlist(pos));
            }
        }
    }

    fn try_scroll_to_song(&mut self) -> Task<Message> {
        if let Some(index) = self.data_provider.take_scroll_index() {
            self.snap_to_index(index)
        } else {
            ().into()
        }
    }

    fn theme(&self, window_id: window::Id) -> Theme {
        if self.song_window.id == window_id {
            Theme::Dark
        } else {
            self.config_window.theme.clone()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            window::close_events().map(Message::WindowClosed),
            window::resize_events().map(Message::WindowResized),
            window::events().map(|(_, event)| match event {
                window::Event::FileDropped(path) => Message::FileDropped(path),
                _ => Message::Noop,
            }),
            keyboard::listen().filter_map(|event| {
                if let keyboard::Event::ModifiersChanged(modifiers) = event {
                    return Some(Message::ModifiersChanged(modifiers));
                }
                let keyboard::Event::KeyPressed {
                    key,
                    modified_key,
                    modifiers,
                    ..
                } = event
                else {
                    return None;
                };

                // Solo shortcuts (no Ctrl/Alt). `modified_key` follows the
                // keyboard layout with shift applied — e.g. "?" on Shift+"/" —
                // so character shortcuts match what the user actually typed.
                if modifiers.control() || modifiers.alt() {
                    return None;
                }
                let pressed = match modified_key.as_ref() {
                    Key::Character(_) => modified_key.as_ref(),
                    _ => key.as_ref(),
                };

                match pressed {
                    Key::Named(Named::Escape) => Some(Message::EscapePressed),
                    Key::Character("?") => Some(Message::OpenDialog(DialogState::Help)),
                    _ => match key {
                        Key::Named(Named::ArrowRight) | Key::Named(Named::Space) => {
                            Some(Message::ItemChanged(ItemChange::Next))
                        }
                        Key::Named(Named::ArrowLeft) => {
                            Some(Message::ItemChanged(ItemChange::Previous))
                        }
                        Key::Named(Named::End) => Some(Message::ItemChanged(ItemChange::Blank)),
                        Key::Named(Named::F11) => Some(Message::ToggleFullscreen),
                        Key::Named(Named::F5) => Some(Message::Statics(StaticsMessage::Reload)),
                        Key::Named(Named::PageUp) => Some(Message::ScrollBy(-10.0)),
                        Key::Named(Named::PageDown) => Some(Message::ScrollBy(10.0)),
                        _ => None,
                    },
                }
            }),
            keyboard::listen().filter_map(|event| {
                let keyboard::Event::KeyPressed {
                    key,
                    modified_key,
                    modifiers,
                    ..
                } = event
                else {
                    return None;
                };

                // Modifier shortcuts (Ctrl/Alt). When the platform reports no
                // typed character for the combo (e.g. Alt on X11), fall back
                // to the layout key without modifiers.
                if !modifiers.control() && !modifiers.alt() {
                    return None;
                }
                let pressed = match modified_key.as_ref() {
                    Key::Character(_) => modified_key.as_ref(),
                    _ => key.as_ref(),
                };

                match (pressed, modifiers) {
                    (Key::Character("n"), Modifiers::CTRL) => Some(Message::AddBlankEntry),
                    (Key::Character("z"), Modifiers::CTRL) => {
                        Some(Message::Dialog(DialogMessage::UndoDelete))
                    }
                    (Key::Character("+"), Modifiers::CTRL) => {
                        Some(Message::SongWindow(SongWindowMessage::ChangeScale(0.1)))
                    }
                    (Key::Character("-"), Modifiers::CTRL) => {
                        Some(Message::SongWindow(SongWindowMessage::ChangeScale(-0.1)))
                    }
                    (Key::Character("f"), Modifiers::CTRL) => {
                        Some(Message::Search(SearchMessage::Toggle))
                    }
                    (Key::Character("b"), Modifiers::CTRL) => Some(Message::ToggleNavSidebar),
                    (Key::Character("j"), Modifiers::CTRL) => Some(Message::ToggleInfoRail),
                    _ => None,
                }
            }),
            if self.config_window.nav_anim.is_animating(Instant::now())
                || self.config_window.rail_anim.is_animating(Instant::now())
            {
                window::frames().map(|_| Message::Animate)
            } else {
                Subscription::none()
            },
            // Escape pressed while a text input has keyboard focus never
            // reaches `keyboard::listen()`: iced's text_input captures Escape
            // to unfocus itself. This listener picks up exactly those
            // captured presses so closing the inline editor with Escape keeps
            // working; `EscapePressed` decides what to do based on the UI
            // state.
            event::listen_with(|event, status, _window| match (status, event) {
                (
                    event::Status::Captured,
                    event::Event::Keyboard(keyboard::Event::KeyPressed {
                        key: Key::Named(Named::Escape),
                        ..
                    }),
                ) => Some(Message::EscapePressed),
                _ => None,
            }),
            if self.config_window.toast.is_some() {
                time::every(Duration::from_millis(500)).map(|_| Message::Tick)
            } else {
                Subscription::none()
            },
        ];

        if let Some(addr) = self.data_provider.traktor_provider.get_socket_addr() {
            subscriptions.push(
                Subscription::run_with(addr, |addr| traktor_api::run_server(*addr))
                    .map(|m| Message::Traktor(TraktorMessage::ServerMessage(Box::new(m)))),
            );
        }

        Subscription::batch(subscriptions)
    }
}

/// Settings persisted across sessions (so far only the theme).
#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedSettings {
    theme: Option<String>,
}

/// Name of the settings directory inside a config directory.
const SETTINGS_DIR: &str = "danceinterpreter";

/// Directory the file dialogs start in: the user's audio folder, falling back
/// to the home directory and then the working directory.
fn default_browse_dir() -> PathBuf {
    dirs::audio_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
}

/// True when running inside a flatpak sandbox: flatpak sets the `container`
/// environment variable to the app id.
fn in_flatpak() -> bool {
    var("container").is_ok()
}

/// True for document-portal fuse paths (`/run/user/<uid>/doc/...`): the
/// sandbox has no direct filesystem access at that location and only the
/// granted file or folder is reachable.
fn is_document_portal_path(path: &Path) -> bool {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .is_some_and(|runtime| path.starts_with(runtime.join("doc")))
}

/// Directory that holds `settings.json`. The XDG config directory (via
/// `dirs::config_dir`, which also covers the platform equivalents on
/// Windows and macOS) always has priority: it respects `XDG_CONFIG_HOME`,
/// which flatpak redirects to the app's own persistent per-app directory,
/// so the same lookup works native and sandboxed. Only when that directory
/// cannot be determined or created do we fall back to a fixed `~/.config`
/// path.
fn settings_dir() -> Option<PathBuf> {
    let xdg = dirs::config_dir().map(|dir| dir.join(SETTINGS_DIR));

    if let Some(dir) = &xdg
        && (dir.is_dir() || std::fs::create_dir_all(dir).is_ok())
    {
        return Some(dir.clone());
    }

    if let Some(dir) = &xdg {
        println!("Cannot use {dir:?} for settings, falling back to ~/.config");
    }

    dirs::home_dir().map(|home| home.join(".config").join(SETTINGS_DIR))
}

/// Persists the chosen theme (stored as its Debug variant name) so it
/// survives a restart. Failures are logged but never fatal — the app keeps
/// running with the in-memory theme.
fn save_theme(theme: &Theme) {
    let Some(dir) = settings_dir() else {
        return;
    };

    if let Err(err) = std::fs::create_dir_all(&dir) {
        println!("Could not create settings directory {dir:?}: {err}");
        return;
    }

    let json = match serde_json::to_string_pretty(&PersistedSettings {
        theme: Some(format!("{theme:?}")),
    }) {
        Ok(json) => json,
        Err(err) => {
            println!("Could not serialize settings: {err}");
            return;
        }
    };

    if let Err(err) = std::fs::write(dir.join("settings.json"), json) {
        println!("Could not save settings: {err}");
    }
}

/// Loads the theme persisted by a previous session, if any.
fn load_saved_theme() -> Option<Theme> {
    let path = settings_dir()?.join("settings.json");
    let json = std::fs::read_to_string(path).ok()?;
    let settings: PersistedSettings = serde_json::from_str(&json).ok()?;
    settings.theme.as_deref().and_then(theme_from_name)
}

/// Parses a persisted theme name back into a theme. Unknown names (e.g.
/// written by a newer version) yield `None` and the default theme is kept.
fn theme_from_name(name: &str) -> Option<Theme> {
    let theme = match name {
        "Light" => Theme::Light,
        "Dark" => Theme::Dark,
        "Dracula" => Theme::Dracula,
        "Nord" => Theme::Nord,
        "SolarizedLight" => Theme::SolarizedLight,
        "SolarizedDark" => Theme::SolarizedDark,
        "GruvboxLight" => Theme::GruvboxLight,
        "GruvboxDark" => Theme::GruvboxDark,
        "CatppuccinLatte" => Theme::CatppuccinLatte,
        "CatppuccinFrappe" => Theme::CatppuccinFrappe,
        "CatppuccinMacchiato" => Theme::CatppuccinMacchiato,
        "CatppuccinMocha" => Theme::CatppuccinMocha,
        "TokyoNight" => Theme::TokyoNight,
        "TokyoNightStorm" => Theme::TokyoNightStorm,
        "TokyoNightLight" => Theme::TokyoNightLight,
        "KanagawaWave" => Theme::KanagawaWave,
        "KanagawaDragon" => Theme::KanagawaDragon,
        "KanagawaLotus" => Theme::KanagawaLotus,
        "Moonfly" => Theme::Moonfly,
        "Nightfly" => Theme::Nightfly,
        "Oxocarbon" => Theme::Oxocarbon,
        "Ferra" => Theme::Ferra,
        _ => return None,
    };
    Some(theme)
}
