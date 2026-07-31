use crate::dataloading::dataprovider::{ItemChange, ItemSource, SongDataEdit};
use crate::dataloading::songinfo::SongInfo;
use crate::ui::config_window::ConfigWindow;
use crate::ui::config_window::item_row::{
    build_item_row_container_styled, empty_folder_button_style,
};
use crate::ui::config_window::search_bar::data_matches_search_query;
use crate::ui::config_window::sidebar::SidebarMessage;
use crate::ui::widget::dynamic_text_input::DynamicTextInput;
use crate::ui::widgets::buttons::material_symbol_message_button;
use crate::ui::widgets::{material_symbol, material_symbol_sized};
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, DialogMessage, DummySongMessage, Message};
use iced::alignment::Vertical;
use iced::widget::{
    Column, Row, Scrollable, Space, button, column as col, container, row, scrollable, text,
};
use iced::{Alignment, Element, Length};
use std::sync::LazyLock;

pub static PLAYLIST_SCROLLABLE_ID: LazyLock<iced::widget::Id> =
    LazyLock::new(iced::widget::Id::unique);

pub fn build<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Column<'a, Message> {
    if dance_interpreter.data_provider.playlist().is_empty() {
        return build_empty_playlist_view(dance_interpreter);
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

    for (i, item) in dance_interpreter
        .data_provider
        .playlist()
        .iter()
        .enumerate()
    {
        let song = &item.song;
        let is_match = data_matches_search_query(&config_window.search_query, song.into());
        let song_row = build_song_row(dance_interpreter, song, i);
        let accent_color = dance_interpreter.data_provider.get_dance_color(&song.dance);
        let sr_container = build_item_row_container_styled(song_row, is_match, i, accent_color);
        playlist_column = playlist_column.push(sr_container);
    }

    let dummy_row = row![
        text!("*").width(Length::Fixed(24.0)),
        iced::widget::text_input("Title...", &config_window.dummy_song_title)
            .width(Length::Fill)
            .on_input(|v| Message::DummySong(DummySongMessage::UpdateTitle(v)))
            .on_submit(Message::DummySong(DummySongMessage::Submit)),
        iced::widget::text_input("Artist...", &config_window.dummy_song_artist)
            .width(Length::Fill)
            .on_input(|v| Message::DummySong(DummySongMessage::UpdateArtist(v)))
            .on_submit(Message::DummySong(DummySongMessage::Submit)),
        iced::widget::text_input("Dance...", &config_window.dummy_song_dance)
            .width(Length::Fill)
            .on_input(|v| Message::DummySong(DummySongMessage::UpdateDance(v)))
            .on_submit(Message::DummySong(DummySongMessage::Submit)),
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

pub fn build_empty_playlist_view(dance_interpreter: &DanceInterpreter) -> Column<'_, Message> {
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

pub fn build_song_row<'a>(
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
                    Message::Dialog(DialogMessage::RequestDelete(ItemSource::Playlist(idx)))
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
