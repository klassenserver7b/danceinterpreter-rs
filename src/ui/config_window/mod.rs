pub mod bottombar;
pub mod sidebar;
pub mod top_bar;

use crate::dataloading::dataprovider::{ItemChange, ItemSource, SongDataEdit};
use crate::dataloading::displayable_data::DisplayableData;
use crate::dataloading::songinfo::SongInfo;
use crate::dataloading::staticinfo::StaticInfo;
use crate::ui::config_window::sidebar::{Sidebar, SidebarMessage};
use crate::ui::widget::dynamic_text_input::DynamicTextInput;
use crate::ui::widgets::buttons::{
    material_symbol_message_button, material_symbol_message_button_colored,
};
use crate::ui::widgets::{material_symbol, material_symbol_sized};
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, Message, Window};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2 as Matcher;
use iced::alignment::Vertical;
use iced::widget::{
    Column, Container, Row, Scrollable, Space, TextInput, button, column as col, container,
    mouse_area, opaque, row, scrollable, stack, text,
};
use iced::{Alignment, Color, Element, Length, Size, Theme, window};
use std::sync::LazyLock;
use std::time::Instant;

pub enum DialogState {
    Delete(ItemSource),
    MergeStatic { old_name: String, new_name: String },
}

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
    pub active_dialog: Option<DialogState>,
    pub color_picker_open: Option<String>,
    pub color_picker_old_color: Option<Color>,

    pub dummy_song_title: String,
    pub dummy_song_artist: String,
    pub dummy_song_dance: String,
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
            active_dialog: None,
            color_picker_open: None,
            color_picker_old_color: None,

            dummy_song_title: String::new(),
            dummy_song_artist: String::new(),
            dummy_song_dance: String::new(),
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
        let main_view = col![row![main_column, side_bar], bottom_bar].spacing(5);

        if let Some(state) = &self.active_dialog {
            let dialog = match state {
                DialogState::Delete(source) => {
                    let target_name = match source {
                        ItemSource::Playlist(i) => dance_interpreter
                            .data_provider
                            .playlist
                            .get(*i)
                            .map(|item| item.song.title.clone())
                            .unwrap_or_default(),
                        ItemSource::Static(name) => dance_interpreter
                            .data_provider
                            .statics()
                            .get(name)
                            .map(|s| s.name.clone())
                            .unwrap_or_else(|| name.clone()),
                        _ => String::new(),
                    };
                    self.build_action_dialog(
                        "Delete item?".to_string(),
                        target_name,
                        "Delete".to_string(),
                        button::danger,
                        Message::CancelDelete,
                        Message::ConfirmDelete,
                    )
                }
                DialogState::MergeStatic { old_name, new_name } => self.build_action_dialog(
                    "Merge Statics?".to_string(),
                    format!(
                        "A static named '{}' already exists.\nMerge '{}' into it?",
                        new_name, old_name
                    ),
                    "Merge".to_string(),
                    button::primary,
                    Message::CancelDialog,
                    Message::ConfirmStaticMerge(old_name.clone(), new_name.clone()),
                ),
            };
            stack![main_view, dialog].into()
        } else {
            main_view.into()
        }
    }

    fn build_action_dialog(
        &self,
        title: String,
        target: String,
        confirm_text: String,
        confirm_style: impl Fn(&Theme, button::Status) -> button::Style + 'static,
        cancel_msg: Message,
        confirm_msg: Message,
    ) -> Element<'_, Message> {
        let cancel_msg_clone = cancel_msg.clone();
        let card = container(
            col![
                text(title).size(20),
                text(target),
                row![
                    button(text("Cancel").align_x(Alignment::Center))
                        .padding([4, 8])
                        .style(button::secondary)
                        .on_press(cancel_msg)
                        .width(Length::Fill),
                    button(text(confirm_text).align_x(Alignment::Center))
                        .padding([4, 8])
                        .style(confirm_style)
                        .on_press(confirm_msg)
                        .width(Length::Fill),
                ]
                .spacing(10),
            ]
            .spacing(15)
            .width(Length::Fixed(360.0)),
        )
        .padding(20)
        .style(|t: &Theme| {
            let palette = t.extended_palette();
            container::Style::default()
                .background(palette.background.base.color)
                .border(iced::Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 8.0.into(),
                })
        });

        let backdrop = mouse_area(
            container(Space::new().width(Length::Fill).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_t: &Theme| {
                    container::Style::default().background(Color::from_rgba(0.0, 0.0, 0.0, 0.5))
                }),
        )
        .on_press(cancel_msg_clone);

        let centered_card = container(opaque(card))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        opaque(stack![backdrop, centered_card])
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
        if dance_interpreter.data_provider.playlist.is_empty() {
            return self.build_empty_playlist_view(dance_interpreter);
        }

        let mut header_items: Vec<Element<'_, Message>> = vec![
            text!("#").width(Length::Fixed(24.0)).into(),
            text!("Title").width(Length::Fill).into(),
            text!("Artist").width(Length::Fill).into(),
            text!("Dance").width(Length::Fill).into(),
            Space::new()
                .width(Length::Fill)
                .height(Length::Shrink)
                .into(),
        ];

        if dance_interpreter
            .data_provider
            .traktor_provider
            .get_song_info()
            .is_some()
        {
            header_items.push(
                button(row![material_symbol("add", false), text("Add Traktor")].spacing(5))
                    .on_press(Message::AddTraktorSong)
                    .padding([5, 10])
                    .style(button::primary)
                    .into(),
            );
        } else {
            header_items.push(
                Space::new()
                    .width(Length::Fixed(10.0))
                    .height(Length::Shrink)
                    .into(),
            );
        }

        let trow = container(
            Row::with_children(header_items)
                .spacing(5)
                .align_y(Vertical::Bottom),
        )
        .padding([4, 6])
        .width(Length::Fill);

        let mut playlist_column: Column<'_, _, _, _> = col!();

        for (i, item) in dance_interpreter.data_provider.playlist.iter().enumerate() {
            let song = &item.song;
            let is_match = Self::data_matches_search_query(&self.search_query, song.into());
            let song_row = Self::build_song_row(dance_interpreter, song, i);
            let accent_color = dance_interpreter.data_provider.get_dance_color(&song.dance);
            let sr_container =
                Self::build_item_row_container_styled(song_row, is_match, i, accent_color);
            playlist_column = playlist_column.push(sr_container);
        }

        let dummy_row = row![
            text!("*").width(Length::Fixed(24.0)),
            iced::widget::text_input("Title...", &self.dummy_song_title)
                .width(Length::Fill)
                .on_input(Message::UpdateDummySongTitle)
                .on_submit(Message::SubmitDummySong),
            iced::widget::text_input("Artist...", &self.dummy_song_artist)
                .width(Length::Fill)
                .on_input(Message::UpdateDummySongArtist)
                .on_submit(Message::SubmitDummySong),
            iced::widget::text_input("Dance...", &self.dummy_song_dance)
                .width(Length::Fill)
                .on_input(Message::UpdateDummySongDance)
                .on_submit(Message::SubmitDummySong),
            Space::new().width(Length::Fill).height(Length::Shrink),
            Space::new()
                .width(Length::Fixed(10.0))
                .height(Length::Shrink),
        ]
        .spacing(5)
        .align_y(Alignment::Center);

        playlist_column = playlist_column.push(
            container(row![
                container(Space::new().width(4).height(Length::Fill)),
                dummy_row,
            ])
            .padding([4, 6])
            .width(Length::Fill),
        );

        let playlist_scrollable: Scrollable<'_, Message> = scrollable(playlist_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(5)
            .id(PLAYLIST_SCROLLABLE_ID.clone());

        col!(trow, playlist_scrollable).spacing(5)
    }

    fn build_empty_playlist_view(
        &'_ self,
        dance_interpreter: &DanceInterpreter,
    ) -> Column<'_, Message> {
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

        let traktor_col = if dance_interpreter
            .data_provider
            .traktor_provider
            .get_song_info()
            .is_some()
        {
            col![
                text("Song Playing!").size(20),
                button(
                    col![
                        material_symbol_sized("add", false, 64),
                        text("Add Traktor Song").size(16)
                    ]
                    .align_x(Alignment::Center)
                )
                .style(empty_folder_button_style)
                .on_press(Message::AddTraktorSong)
                .padding(20)
            ]
            .spacing(20)
            .align_x(Alignment::Center)
        } else {
            col![
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
            .align_x(Alignment::Center)
        };

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
        let trow = container(
            row![
                text("Fav").width(Length::Fixed(42.0)),
                text!("Name").width(Length::Fill),
                Space::new().width(Length::Fill).height(Length::Shrink),
                button(row![material_symbol("add", false), text("Add Static")].spacing(5))
                    .on_press(Message::AddBlankStatic)
                    .padding([5, 10])
                    .style(button::primary),
            ]
            .align_y(Vertical::Bottom)
            .spacing(5),
        )
        .padding([4, 6])
        .width(Length::Fill);

        let mut list_column = col!().spacing(5);

        for (i, (name, static_info)) in dance_interpreter.data_provider.statics().iter().enumerate()
        {
            let is_match = Self::data_matches_search_query(&self.search_query, static_info.into());
            let static_row = Self::build_static_row(
                static_info,
                name.clone(),
                self.theme.extended_palette().primary.weak.color,
                self.color_picker_open.as_ref() == Some(name),
            );
            let accent_color = static_info.color;
            let row_container =
                Self::build_item_row_container_styled(static_row, is_match, i, accent_color);
            list_column = list_column.push(row_container);
        }

        let statics_scrollable: Scrollable<'_, Message> = scrollable(list_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(5);

        col!(trow, statics_scrollable).spacing(5)
    }

    fn build_static_row<'a>(
        static_info: &StaticInfo,
        name: String,
        color: impl Into<Color>,
        color_picker_open: bool,
    ) -> Row<'a, Message> {
        let swatch_color = static_info.color.unwrap_or(Color::TRANSPARENT);
        let name_clone1 = name.clone();
        let name_clone2 = name.clone();
        let name_clone3 = name.clone();
        let name_clone4 = name.clone();
        let name_clone5 = name.clone();

        let color_swatch = crate::ui::widgets::color_swatch::color_swatch(
            swatch_color,
            Message::ToggleStaticColorPicker(name.clone()),
        );

        let color_element: Element<'a, Message> = if color_picker_open {
            iced_aw::ColorPicker::new(
                true,
                swatch_color,
                color_swatch,
                Message::ToggleStaticColorPicker(name_clone1),
                move |c| Message::UpdateStaticColor(name_clone2.clone(), c),
            )
            .on_color_change(move |c| Message::PreviewStaticColor(name_clone3.clone(), c))
            .into()
        } else {
            color_swatch
        };

        row![
            if static_info.is_favorite {
                material_symbol_message_button_colored(
                    "star",
                    static_info.is_favorite,
                    Message::ToggleStaticFavorite(name.clone()),
                    color,
                )
            } else {
                material_symbol_message_button(
                    "star",
                    static_info.is_favorite,
                    Message::ToggleStaticFavorite(name.clone()),
                )
            },
            Space::new().width(5),
            color_element,
            DynamicTextInput::<'_, Message>::new("Static Name", &static_info.name)
                .width(Length::Fill)
                .on_change(move |v| Message::UpdateStaticName(name_clone4.clone(), v))
                .on_submit(Message::SubmitStaticName(name_clone5.clone())),
            row![
                Space::new().width(Length::Fill).height(Length::Shrink),
                with_tooltip(
                    material_symbol_message_button(
                        "smart_display",
                        false,
                        Message::ItemChanged(ItemChange::StaticAbsolute(name.clone()))
                    ),
                    "Show now"
                ),
                with_tooltip(
                    material_symbol_message_button(
                        "delete",
                        false,
                        Message::RequestDelete(ItemSource::Static(name.clone()))
                    ),
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
                .on_change(move |v| Message::SongDataEdit(idx, SongDataEdit::Dance(v)))
                .on_submit(Message::SubmitPlaylistDance(idx)),
            row![
                Space::new().width(Length::Fill).height(Length::Shrink),
                with_tooltip(
                    material_symbol_message_button(
                        "smart_display",
                        false,
                        Message::ItemChanged(ItemChange::PlaylistAbsolute(idx))
                    ),
                    "Show now"
                ),
                with_tooltip(
                    material_symbol_message_button(
                        "queue_play_next",
                        false,
                        Message::SetNextItem(ItemSource::Playlist(idx))
                    ),
                    "Set as next song"
                ),
                with_tooltip(
                    material_symbol_message_button(
                        "delete",
                        false,
                        Message::RequestDelete(ItemSource::Playlist(idx))
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

    fn build_item_row_container_styled(
        song_row: Row<Message>,
        highlight: bool,
        idx: usize,
        accent_color: Option<Color>,
    ) -> Container<Message> {
        let accent = accent_color;
        container(
            row![
                container(Space::new().width(4).height(40.0))
                    .height(Length::Fixed(40.0))
                    .style(move |_t: &Theme| {
                        container::Style::default().background(accent.unwrap_or(Color::TRANSPARENT))
                    }),
                song_row,
            ]
            .align_y(Alignment::Center)
            .spacing(6),
        )
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

    fn data_matches_search_query(search_query: &str, data: DisplayableData) -> bool {
        let matcher = Matcher::default();

        !search_query.is_empty()
            && matcher
                .fuzzy_match(
                    format!(
                        "{} {} {}",
                        data.headline.to_lowercase(),
                        data.subline_upper.to_lowercase(),
                        data.subline_lower.to_lowercase()
                    )
                    .trim(),
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
