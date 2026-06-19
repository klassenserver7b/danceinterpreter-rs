pub mod bottombar;
pub mod sidebar;
pub mod top_bar;

use crate::dataloading::dataprovider::song_data_provider::{
    SongChange, SongDataEdit, SongDataSource,
};
use crate::dataloading::songinfo::SongInfo;
use crate::ui::config_window::sidebar::{Sidebar, SidebarMessage};
use crate::ui::widget::dynamic_text_input::DynamicTextInput;
use crate::ui::widgets::buttons::material_symbol_message_button;
use crate::ui::widgets::{material_symbol, material_symbol_sized};
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, Message, Window};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2 as Matcher;
use iced::alignment::Vertical;
use iced::widget::{
    Column, Container, Row, Scrollable, Space, TextInput, button, column as col, container, row,
    scrollable, text,
};
use iced::{Alignment, Element, Length, Size, Theme, window};
use std::sync::LazyLock;
use std::time::Instant;

pub struct ConfigWindow {
    pub id: window::Id,
    pub closed: bool,
    pub size: Size,
    pub enable_autoscroll: bool,
    pub sidebar: Sidebar,
    pub is_statics_view: bool,
    pub theme: Theme,
    pub follow_system_theme: bool,

    pub search_visible: bool,
    pub search_query: String,
}

pub static PLAYLIST_SCROLLABLE_ID: LazyLock<iced::widget::Id> =
    LazyLock::new(iced::widget::Id::unique);

impl Window for ConfigWindow {
    fn new(id: window::Id) -> Self {
        Self {
            id,
            closed: false,
            size: Size::default(),

            enable_autoscroll: true,
            sidebar: Sidebar::new(),
            is_statics_view: false,
            theme: Theme::Dark,
            follow_system_theme: true,

            search_visible: false,
            search_query: String::new(),
        }
    }

    fn on_resize(&mut self, size: Size) {
        self.size = size;
    }

    fn on_close(&mut self) {
        self.closed = true;
    }

    fn is_closed(&self) -> bool {
        self.closed
    }
}

impl ConfigWindow {
    pub fn view<'a>(&'a self, dance_interpreter: &'a DanceInterpreter) -> Element<'a, Message> {
        let top_bar = top_bar::build(self, dance_interpreter);

        let mut main_column = col![top_bar];

        if self.search_visible {
            let search_bar = self.build_search_bar();
            main_column = main_column.push(search_bar);
        }

        let content_view = if self.is_statics_view {
            self.build_statics_view(dance_interpreter)
        } else {
            self.build_playlist_view(dance_interpreter)
        };

        main_column = main_column.push(content_view);

        let side_bar = self
            .sidebar
            .build(dance_interpreter)
            .width(self.sidebar.state.interpolate(
                0.0,
                (self.size.width / 5.0).min(400.0),
                Instant::now(),
            ));
        let bottom_bar = bottombar::build(dance_interpreter);

        col![row![main_column, side_bar], bottom_bar]
            .spacing(5)
            .into()
    }

    fn build_search_bar<'a>(&self) -> Row<'a, Message> {
        row![
            material_symbol("search", false)
                .width(Length::Fixed(24.0))
                .align_y(Vertical::Center),
            TextInput::new("Search...", &self.search_query)
                .on_input(Message::SearchChanged)
                .on_submit(Message::Noop)
                .width(Length::Fill)
                .padding([4, 8]),
            material_symbol_message_button("backspace", false, Message::ClearSearch),
            material_symbol_message_button("close", false, Message::ToggleSearch),
        ]
        .spacing(5)
        .padding([5, 5])
        .align_y(Alignment::Center)
    }

    fn build_playlist_view(&'_ self, dance_interpreter: &DanceInterpreter) -> Column<'_, Message> {
        if dance_interpreter.data_provider.playlist_songs.is_empty() {
            return self.build_empty_playlist_view();
        }

        let trow: Row<_> = row![
            text!("#").width(Length::Fixed(24.0)),
            text!("Title").width(Length::Fill),
            text!("Artist").width(Length::Fill),
            text!("Dance").width(Length::Fill),
            Space::new().width(Length::Fill).height(Length::Shrink),
            Space::new()
                .width(Length::Fixed(10.0))
                .height(Length::Shrink),
        ]
        .spacing(5);

        let mut playlist_column: Column<'_, _, _, _> = col!();

        for (i, song) in dance_interpreter
            .data_provider
            .playlist_songs
            .iter()
            .enumerate()
        {
            let is_match = Self::song_matches_search_query(&self.search_query, song);
            let song_row = Self::build_song_row(dance_interpreter, song, i);
            let sr_container = Self::build_song_row_container_styled(song_row, is_match, i);
            playlist_column = playlist_column.push(sr_container);
        }

        let playlist_scrollable: Scrollable<'_, Message> = scrollable(playlist_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(5)
            .id(PLAYLIST_SCROLLABLE_ID.clone());

        col!(trow, playlist_scrollable).spacing(5)
    }

    fn build_empty_playlist_view(&'_ self) -> Column<'_, Message> {
        let playlist_col = col![
            text("Load Playlist").size(20),
            button(
                col![
                    material_symbol_sized("folder_open", false, 64),
                    text("Open playlist file").size(16)
                ]
                .align_x(Alignment::Center)
            )
            .style(empty_folder_button_style)
            .on_press(Message::OpenPlaylist)
            .padding(20)
        ]
        .spacing(20)
        .align_x(Alignment::Center);

        let traktor_col = col![
            text("Use Traktor").size(20),
            button(
                col![
                    material_symbol_sized("agriculture", false, 64),
                    text("Open the sidebar").size(16)
                ]
                .align_x(Alignment::Center)
            )
            .style(empty_folder_button_style)
            .on_press(Message::Sidebar(SidebarMessage::Toggle))
            .padding(20)
        ]
        .spacing(20)
        .align_x(Alignment::Center);

        let empty_state_column = col![
            text("No playlist loaded.").size(24),
            row![playlist_col, traktor_col]
                .spacing(40)
                .align_y(Alignment::Center)
        ]
        .spacing(40)
        .align_x(Alignment::Center);

        col![
            Space::new().height(Length::Fill),
            empty_state_column,
            Space::new().height(Length::Fill)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
    }

    fn build_statics_view(&'_ self, dance_interpreter: &DanceInterpreter) -> Column<'_, Message> {
        let trow = row![
            text("Statics").size(24).width(Length::Fill),
            button(row![material_symbol("add", false), text("Add Static")].spacing(5))
                .on_press(Message::AddBlankStatic)
                .padding([5, 10])
                .style(button::primary),
        ]
        .spacing(5);

        let mut list_column = col!().spacing(5);

        for (i, song) in dance_interpreter.data_provider.statics.iter().enumerate() {
            let is_current = matches!(
                dance_interpreter.data_provider.current,
                SongDataSource::Static(idx) if idx == i
            );
            let static_row = Self::build_static_row(song, i);
            let row_container = Self::build_song_row_container_styled(static_row, is_current, i);
            list_column = list_column.push(row_container);
        }

        let statics_scrollable: Scrollable<'_, Message> = scrollable(list_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(5);

        col!(trow, statics_scrollable).spacing(5)
    }

    fn build_static_row<'a>(song: &SongInfo, idx: usize) -> Row<'a, Message> {
        row![
            material_symbol_message_button(
                "star",
                song.is_favorite,
                Message::ToggleStaticFavorite(idx)
            ),
            DynamicTextInput::<'_, Message>::new("Static Name", &song.dance)
                .width(Length::Fill)
                .on_change(move |v| Message::UpdateStaticName(idx, v)),
            row![
                Space::new().width(Length::Fill).height(Length::Shrink),
                with_tooltip(
                    material_symbol_message_button(
                        "smart_display",
                        false,
                        Message::SongChanged(SongChange::StaticAbsolute(idx))
                    ),
                    "Show now"
                ),
                with_tooltip(
                    material_symbol_message_button("delete", false, Message::DeleteStatic(idx)),
                    "Delete static"
                ),
            ]
            .spacing(5)
            .align_y(Alignment::Center)
            .width(Length::Fill),
        ]
        .align_y(Alignment::Center)
        .spacing(10)
    }

    fn build_song_row<'a>(
        dance_interpreter: &DanceInterpreter,
        song: &SongInfo,
        idx: usize,
    ) -> Row<'a, Message> {
        let (is_current, is_next, is_traktor, is_played) =
            dance_interpreter.data_provider.get_play_state(idx);
        let icon: Element<Message> = if is_traktor {
            material_symbol("agriculture", false)
                .width(Length::Fixed(24.0))
                .into()
        } else if is_current {
            material_symbol("play_arrow", false)
                .width(Length::Fixed(24.0))
                .into()
        } else if is_next {
            material_symbol("skip_next", false)
                .width(Length::Fixed(24.0))
                .into()
        } else if is_played {
            material_symbol("check", false)
                .width(Length::Fixed(24.0))
                .into()
        } else {
            Space::new()
                .width(Length::Fixed(24.0))
                .height(Length::Shrink)
                .into()
        };

        row![
            icon,
            DynamicTextInput::<'_, Message>::new("Title", &song.title)
                .width(Length::Fill)
                .on_change(move |v| Message::SongDataEdit(idx, SongDataEdit::Title(v))),
            DynamicTextInput::<'_, Message>::new("Artist", &song.artist)
                .width(Length::Fill)
                .on_change(move |v| Message::SongDataEdit(idx, SongDataEdit::Artist(v))),
            DynamicTextInput::<'_, Message>::new("Dance", &song.dance)
                .width(Length::Fill)
                .on_change(move |v| Message::SongDataEdit(idx, SongDataEdit::Dance(v))),
            row![
                Space::new().width(Length::Fill).height(Length::Shrink),
                with_tooltip(
                    material_symbol_message_button(
                        "smart_display",
                        false,
                        Message::SongChanged(SongChange::PlaylistAbsolute(idx))
                    ),
                    "Show now"
                ),
                with_tooltip(
                    material_symbol_message_button(
                        "queue_play_next",
                        false,
                        Message::SetNextSong(SongDataSource::Playlist(idx))
                    ),
                    "Set as next song"
                ),
                with_tooltip(
                    material_symbol_message_button(
                        "delete",
                        false,
                        Message::DeleteSong(SongDataSource::Playlist(idx))
                    ),
                    "Delete song"
                ),
            ]
            .spacing(5)
            .align_y(Alignment::Center)
            .width(Length::Fill),
        ]
        .align_y(Alignment::Center)
    }

    fn build_song_row_container_styled(
        song_row: Row<Message>,
        highlight: bool,
        idx: usize,
    ) -> Container<Message> {
        container(song_row)
            .style(move |t| {
                let palette = t.extended_palette();
                let color = if idx.is_multiple_of(2) {
                    palette.background.weakest.color
                } else {
                    palette.background.weaker.color
                };

                let mut style = container::Style::default().background(color);

                if highlight {
                    let accent = palette.primary.base.color;
                    style = style
                        .background(palette.primary.weak.color)
                        .border(iced::Border {
                            color: accent,
                            width: 2.0,
                            radius: 4.0.into(),
                        });
                }

                style
            })
            .padding([4, 6])
            .width(Length::Fill)
    }

    fn song_matches_search_query(search_query: &str, song: &SongInfo) -> bool {
        let matcher = Matcher::default();

        !search_query.is_empty()
            && matcher
                .fuzzy_match(
                    &format!(
                        "{} {} {}",
                        song.title.to_lowercase(),
                        song.artist.to_lowercase(),
                        song.dance.to_lowercase()
                    ),
                    &search_query.to_lowercase(),
                )
                .is_some()
    }
}

fn empty_folder_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let mut style = button::text(theme, status);
    style.text_color = palette.secondary.strong.color;
    style.background = Some(palette.background.weakest.color.into());
    style.border.radius = 12.0.into();
    style
}
