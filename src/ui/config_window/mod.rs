pub mod bottombar;
pub mod sidebar;
pub mod top_bar;

use crate::dataloading::dataprovider::song_data_provider::{
    SongChange, SongDataEdit, SongDataSource,
};
use crate::ui::config_window::sidebar::{Sidebar, SidebarMessage};
use crate::ui::widget::dynamic_text_input::DynamicTextInput;
use crate::ui::{material_icon, material_icon_sized, with_tooltip};
use crate::{DanceInterpreter, Message, Window};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2 as Matcher;
use iced::alignment::Vertical;
use iced::widget::{
    Button, Column, Row, Scrollable, Space, TextInput, button, checkbox, column as col, container,
    radio, row, scrollable, text, toggler,
};
use iced::{Alignment, Element, Length, Pixels, Renderer, Size, Theme, window};
use iced_aw::iced_aw_font;
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
            material_icon("search")
                .width(Length::Fixed(24.0))
                .align_y(Vertical::Center),
            TextInput::new("Search...", &self.search_query)
                .on_input(Message::SearchChanged)
                .on_submit(Message::Noop)
                .width(Length::Fill)
                .padding([4, 8]),
            material_icon_message_button("backspace", Message::ClearSearch),
            material_icon_message_button("close", Message::ToggleSearch),
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

        let matcher = Matcher::default();

        for (i, song) in dance_interpreter
            .data_provider
            .playlist_songs
            .iter()
            .enumerate()
        {
            let is_match = !self.search_query.is_empty()
                && matcher
                    .fuzzy_match(
                        &format!(
                            "{} {} {}",
                            song.title.to_lowercase(),
                            song.artist.to_lowercase(),
                            song.dance.to_lowercase()
                        ),
                        &self.search_query.to_lowercase(),
                    )
                    .is_some();

            let (is_current, is_next, is_traktor, is_played) =
                dance_interpreter.data_provider.get_play_state(i);
            let icon: Element<Message> = if is_traktor {
                material_icon("agriculture")
                    .width(Length::Fixed(24.0))
                    .into()
            } else if is_current {
                material_icon("play_arrow")
                    .width(Length::Fixed(24.0))
                    .into()
            } else if is_next {
                material_icon("skip_next").width(Length::Fixed(24.0)).into()
            } else if is_played {
                material_icon("check").width(Length::Fixed(24.0)).into()
            } else {
                Space::new()
                    .width(Length::Fixed(24.0))
                    .height(Length::Shrink)
                    .into()
            };

            let song_row = row![
                icon,
                DynamicTextInput::<'_, Message>::new("Title", &song.title)
                    .width(Length::Fill)
                    .on_change(move |v| Message::SongDataEdit(i, SongDataEdit::Title(v))),
                DynamicTextInput::<'_, Message>::new("Artist", &song.artist)
                    .width(Length::Fill)
                    .on_change(move |v| Message::SongDataEdit(i, SongDataEdit::Artist(v))),
                DynamicTextInput::<'_, Message>::new("Dance", &song.dance)
                    .width(Length::Fill)
                    .on_change(move |v| Message::SongDataEdit(i, SongDataEdit::Dance(v))),
                row![
                    Space::new().width(Length::Fill).height(Length::Shrink),
                    with_tooltip(
                        material_icon_message_button(
                            "smart_display",
                            Message::SongChanged(SongChange::PlaylistAbsolute(i))
                        ),
                        "Show now"
                    ),
                    with_tooltip(
                        material_icon_message_button(
                            "queue_play_next",
                            Message::SetNextSong(SongDataSource::Playlist(i))
                        ),
                        "Set as next song"
                    ),
                    with_tooltip(
                        material_icon_message_button(
                            "delete",
                            Message::DeleteSong(SongDataSource::Playlist(i))
                        ),
                        "Delete song"
                    ),
                ]
                .spacing(5)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            ]
            .align_y(Alignment::Center);

            let song_row = container(song_row)
                .style(move |t| {
                    let palette = t.extended_palette();
                    let color = if i % 2 == 0 {
                        palette.background.weakest.color
                    } else {
                        palette.background.weaker.color
                    };

                    let mut style = container::Style::default().background(color);

                    if is_match {
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
                .width(Length::Fill);

            playlist_column = playlist_column.push(song_row);
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
            text("Click the folder icon below to load a playlist (.m3u) file.")
                .size(16)
                .width(Length::Fixed(300.0))
                .align_x(Alignment::Center),
            button(material_icon_sized("folder_open", 64))
                .style(empty_folder_button_style)
                .on_press(Message::OpenPlaylist)
                .padding(20)
        ]
        .spacing(20)
        .align_x(Alignment::Center);

        let traktor_col = col![
            text("Use Traktor").size(20),
            text("Open the sidebar to start the Traktor server and sync automatically.")
                .size(16)
                .width(Length::Fixed(300.0))
                .align_x(Alignment::Center),
            button(material_icon_sized("agriculture", 64))
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

    fn build_statics_view(&'_ self, _dance_interpreter: &DanceInterpreter) -> Column<'_, Message> {
        col![].width(Length::Fill).height(Length::Fill)
    }
}

fn empty_folder_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let mut style = button::text(theme, status);
    style.text_color = palette.secondary.strong.color;
    style.background = Some(palette.background.weakest.color.into());
    style.border.radius = 12.0.into();
    style
}

fn label_message_button_fill<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
) -> Button<'a, Message> {
    label_message_button(label, message).width(Length::Fill)
}

fn label_message_button_shrink<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
) -> Button<'a, Message> {
    label_message_button(label, message).width(Length::Shrink)
}

fn label_message_button<'a>(
    label: impl text::IntoFragment<'a>,
    message: Message,
) -> Button<'a, Message> {
    button(text(label).align_y(Vertical::Center))
        .padding([4, 8])
        .style(button::secondary)
        .on_press(message)
}

#[allow(dead_code)]
fn submenu_button(label: &'_ str) -> Button<'_, Message, Theme, Renderer> {
    button(
        row![
            text(label).width(Length::Fill).align_y(Vertical::Center),
            iced_aw_font::right_open()
                .width(Length::Shrink)
                .align_y(Vertical::Center),
        ]
        .align_y(Alignment::Center),
    )
    .padding([4, 8])
    .style(button::text)
    .on_press(Message::Noop)
    .width(Length::Fill)
}

fn label_message_button_opt(label: &'_ str, message: Option<Message>) -> Button<'_, Message> {
    if let Some(message) = message {
        label_message_button(label, message)
    } else {
        button(text(label).align_y(Vertical::Center))
            .padding([4, 8])
            .style(button::primary)
    }
}

fn label_message_button_fill_opt(label: &'_ str, message: Option<Message>) -> Button<'_, Message> {
    label_message_button_opt(label, message).width(Length::Fill)
}

fn material_icon_message_button(icon_id: &'_ str, message: Message) -> Button<'_, Message> {
    button(material_icon(icon_id))
        //.padding([4, 8])
        .style(button::secondary)
        .on_press(message)
        .width(Length::Shrink)
}

fn material_icon_sized_message_button(
    icon_id: &'_ str,
    size: impl Into<Pixels>,
    message: Message,
) -> Button<'_, Message> {
    button(material_icon_sized(icon_id, size))
        .style(button::secondary)
        .on_press(message)
        .width(Length::Shrink)
}

fn labeled_message_checkbox(
    label: &'_ str,
    checked: bool,
    message: fn(bool) -> Message,
) -> checkbox::Checkbox<'_, Message> {
    checkbox(checked)
        .label(label)
        .on_toggle(message)
        .width(Length::Fill)
    //.style(checkbox::secondary)
}

fn labeled_message_toggler(
    label: &'_ str,
    checked: bool,
    message: fn(bool) -> Message,
) -> toggler::Toggler<'_, Message> {
    toggler(checked)
        .label(label)
        .on_toggle(message)
        .width(Length::Fill)
}

#[allow(dead_code)]
fn labeled_message_radio<T: Copy + Eq>(
    label: &'_ str,
    value: T,
    selection: T,
    message: fn(T) -> Message,
) -> radio::Radio<'_, Message> {
    radio(label, value, Some(selection), message).width(Length::Fill)
    //.style(checkbox::secondary)
}

#[allow(dead_code)]
fn labeled_message_checkbox_opt(
    label: &'_ str,
    checked: bool,
    message: Option<fn(bool) -> Message>,
) -> checkbox::Checkbox<'_, Message> {
    if let Some(message) = message {
        labeled_message_checkbox(label, checked, message)
    } else {
        checkbox(checked).label(label).width(Length::Fill)
        //.style(checkbox::secondary)
    }
}

#[allow(dead_code)]
fn labeled_dynamic_text_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    message: fn(String) -> Message,
    submit_message: Option<Message>,
) -> Column<'a, Message> {
    let mut input = DynamicTextInput::<Message>::new(placeholder, value)
        .width(Length::Fill)
        .on_change(message);

    if let Some(submit_message) = submit_message {
        input = input.on_submit(submit_message);
    }

    col!(text(label).width(Length::Fill), input,).width(Length::Fill)
}
