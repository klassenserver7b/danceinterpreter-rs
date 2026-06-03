pub mod bottombar;
pub mod sidebar;
pub mod top_bar;

use crate::dataloading::dataprovider::song_data_provider::{
    RemovedSong, SongChange, SongDataEdit, SongDataSource,
};
use crate::dataloading::songinfo::SongInfo;
use crate::ui::config_window::sidebar::Sidebar;
use crate::ui::widget::dynamic_text_input::DynamicTextInput;
use crate::ui::{material_icon, material_icon_sized};
use crate::{DanceInterpreter, Message, Window};
use iced::alignment::Vertical;
use iced::widget::{
    Button, Column, Row, Scrollable, Space, button, checkbox, column as col, container, mouse_area,
    opaque, radio, row, scrollable, stack, text, toggler,
};
use iced::{Alignment, Color, Element, Length, Pixels, Renderer, Size, Theme, window};
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
    /// Set while a delete confirmation dialog is open.
    pub pending_delete: Option<SongDataSource>,
    /// Most recently deleted song, available for undo.
    pub last_deleted: Option<RemovedSong>,
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
            pending_delete: None,
            last_deleted: None,
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

        let content_view = if self.is_statics_view {
            self.build_statics_view(dance_interpreter)
        } else {
            self.build_playlist_view(dance_interpreter)
        };

        let side_bar = self
            .sidebar
            .build(dance_interpreter)
            .width(self.sidebar.state.interpolate(
                0.0,
                (self.size.width / 5.0).min(400.0),
                Instant::now(),
            ));
        let bottom_bar = bottombar::build(dance_interpreter);

        let base: Element<'a, Message> =
            col![row![col![top_bar, content_view], side_bar], bottom_bar]
                .spacing(5)
                .into();

        if let Some(source) = self.pending_delete.as_ref() {
            let title = delete_target_label(source, dance_interpreter);
            stack![base, self.build_delete_dialog(title)].into()
        } else {
            base
        }
    }

    fn build_delete_dialog<'a>(&'a self, target: String) -> Element<'a, Message> {
        let card = container(
            col![
                text("Delete song?").size(20),
                text(target),
                row![
                    label_message_button("Cancel", Message::CancelDelete).width(Length::Fill),
                    button(text("Delete").align_x(Alignment::Center))
                        .padding([4, 8])
                        .style(button::danger)
                        .on_press(Message::ConfirmDelete)
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
        .on_press(Message::CancelDelete);

        let centered_card = container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        // The backdrop captures clicks for dismissal; the card sits on top and
        // intercepts its own clicks so they don't fall through to the backdrop.
        opaque(stack![backdrop, opaque(centered_card)])
    }

    fn build_playlist_view(&'_ self, dance_interpreter: &DanceInterpreter) -> Column<'_, Message> {
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
                    material_icon_message_button(
                        "smart_display",
                        Message::SongChanged(SongChange::PlaylistAbsolute(i))
                    ),
                    material_icon_message_button(
                        "queue_play_next",
                        Message::SetNextSong(SongDataSource::Playlist(i))
                    ),
                    material_icon_message_button(
                        "delete",
                        Message::RequestDeleteSong(SongDataSource::Playlist(i))
                    ),
                ]
                .spacing(5)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            ]
            .align_y(Alignment::Center);

            let song_row = container(song_row)
                .style(move |t| {
                    let color = if i % 2 == 0 {
                        t.extended_palette().background.weakest.color
                    } else {
                        t.extended_palette().background.weaker.color
                    };

                    container::Style::default().background(color)
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

        let mut view: Column<'_, Message> = col!().spacing(5);

        if let Some(removed) = self.last_deleted.as_ref() {
            view = view.push(self.build_undo_bar(removed));
        }

        view.push(trow).push(playlist_scrollable)
    }

    fn build_undo_bar<'a>(&'a self, removed: &'a RemovedSong) -> Element<'a, Message> {
        let song = match removed {
            RemovedSong::Playlist { song, .. } => song,
            RemovedSong::Static { song, .. } => song,
        };

        let label = format!("Deleted '{}'", removed_song_label(song));

        container(
            row![
                text(label).width(Length::Fill).align_y(Vertical::Center),
                label_message_button_shrink("Undo", Message::UndoDelete),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(|t: &Theme| {
            container::Style::default().background(t.extended_palette().background.weak.color)
        })
        .into()
    }

    fn build_statics_view(&'_ self, _dance_interpreter: &DanceInterpreter) -> Column<'_, Message> {
        col![].width(Length::Fill).height(Length::Fill)
    }
}

/// Builds a human-readable label for the song targeted by a delete request,
/// used in the confirmation dialog.
fn delete_target_label(source: &SongDataSource, dance_interpreter: &DanceInterpreter) -> String {
    let song = match source {
        SongDataSource::Playlist(i) => dance_interpreter.data_provider.playlist_songs.get(*i),
        SongDataSource::Static(i) => dance_interpreter.data_provider.statics.get(*i),
        _ => None,
    };

    match song {
        Some(song) => removed_song_label(song),
        None => "this item".to_string(),
    }
}

/// Produces a short description of a song for dialog/undo text, preferring
/// title + artist, then dance, then a generic fallback.
fn removed_song_label(song: &SongInfo) -> String {
    let title = song.title.trim();
    let artist = song.artist.trim();
    let dance = song.dance.trim();

    if !title.is_empty() && !artist.is_empty() {
        format!("{} - {}", title, artist)
    } else if !title.is_empty() {
        title.to_string()
    } else if !dance.is_empty() {
        dance.to_string()
    } else {
        "Untitled song".to_string()
    }
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
