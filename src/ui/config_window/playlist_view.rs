//! Playlist panel: toolbar with search, table, inline-editable rows,
//! quick-add row and summary footer. Includes the empty/first-run state.

use crate::dataloading::dataprovider::{ItemChange, ItemSource};
use crate::ui::config_window::dj_panel::SidebarMessage;
use crate::ui::config_window::search::data_matches_search_query;
use crate::ui::config_window::{ConfigWindow, EditField, edit_input_id};
use crate::ui::theme::RowLook;
use crate::ui::theme::{self, sz};
use crate::ui::widgets::{buttons, heading, material_symbol, material_symbol_sized};
use crate::ui::with_tooltip;
use crate::{DanceInterpreter, DialogMessage, Message, QuickAddMessage};
use iced::advanced::text::{Shaping, Wrapping};
use iced::widget::{
    Column, Space, button, column as col, container, mouse_area, row, scrollable, text, text_input,
};
use iced::{Alignment, Color, Element, Length, Theme};
use std::sync::LazyLock;

pub static PLAYLIST_SCROLLABLE_ID: LazyLock<iced::widget::Id> =
    LazyLock::new(iced::widget::Id::unique);

// Column metrics (design px).
const COL_NUMBER: f32 = 28.0;
const COL_DANCE: f32 = 150.0;
const COL_STATUS: f32 = 78.0;
const COL_ACTIONS: f32 = 78.0;
const ROW_HEIGHT: f32 = 68.0;

pub fn build<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    if dance_interpreter.data_provider.playlist().is_empty() {
        return build_empty_playlist_view(dance_interpreter);
    }

    let panel = col![
        build_toolbar(config_window, dance_interpreter),
        build_table_head(),
        build_rows(config_window, dance_interpreter),
        build_quick_add(config_window),
        build_footer(dance_interpreter),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::card)
        .clip(true)
        .into()
}

// ---------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------

fn build_toolbar<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    let count = dance_interpreter.data_provider.playlist().len();

    let title = row![
        material_symbol_sized("format_list_bulleted", false, 15.0)
            .style(theme::text_style(|tk| tk.muted)),
        heading(
            "Evening playlist",
            sz::PANEL_TITLE,
            iced::font::Weight::Semibold
        )
        .style(theme::text_style(|tk| tk.text)),
        text(format!("{} songs", count))
            .size(sz::TINY)
            .style(theme::text_style(|tk| tk.muted)),
    ]
    .spacing(9)
    .align_y(Alignment::Center);

    let mut tools = row![].spacing(10).align_y(Alignment::Center);

    if config_window.search_visible {
        tools = tools.push(build_search(config_window));
    }

    tools = tools.push(with_tooltip(
        buttons::ghost_icon("upload_file", false, sz::ICON, Message::AddAudioFiles),
        "Add audio files",
    ));

    tools = tools.push(with_tooltip(
        buttons::ghost_icon(
            "undo",
            false,
            sz::ICON,
            Message::Dialog(DialogMessage::UndoDelete),
        ),
        "Undo delete",
    ));

    container(
        row![
            title,
            container(tools).width(Length::Fill).align_x(Alignment::End)
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center),
    )
    .padding([14, 17])
    .width(Length::Fill)
    .into()
}

fn build_search<'a>(config_window: &'a ConfigWindow) -> Element<'a, Message> {
    let input = text_input("Find a song, artist, dance…", &config_window.search_query)
        .size(sz::TINY)
        .width(Length::Fixed(170.0))
        .id(config_window.search_input_id.clone())
        .on_input(|v| Message::Search(crate::SearchMessage::Changed(v)))
        .on_submit(Message::Noop)
        .style(theme::input_boxed)
        .padding([5, 8]);

    let close = buttons::ghost_icon(
        "close",
        false,
        14.0,
        Message::Search(crate::SearchMessage::Toggle),
    );

    container(
        row![
            material_symbol_sized("search", false, 15.0).style(theme::text_style(|tk| tk.muted)),
            input,
            close,
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([0, 8])
    .style(|t: &Theme| {
        let tk = theme::Tokens::of(t);
        container::Style::default()
            .background(tk.panel_deep)
            .border(iced::Border {
                color: tk.border,
                width: 1.0,
                radius: 4.0.into(),
            })
    })
    .height(Length::Fixed(30.0))
    .align_y(Alignment::Center)
    .into()
}

// ---------------------------------------------------------------------------
// Table
// ---------------------------------------------------------------------------

fn build_table_head() -> Element<'static, Message> {
    let head_cell = |label: &'static str, width: Length| -> Element<'static, Message> {
        container(
            text(label)
                .size(sz::MICRO)
                .font(theme::body_font(iced::font::Weight::Semibold))
                .style(theme::text_style(|tk| tk.muted_head)),
        )
        .width(width)
        .into()
    };

    // The 3px spacer mirrors the row accent bar; its Fill height also
    // inflates the line so align_y centers the head cells in the band.
    let head = row![
        Space::new().width(Length::Fixed(3.0)).height(Length::Fill),
        head_cell("#", Length::Fixed(COL_NUMBER)),
        head_cell("SONG / ARTIST", Length::Fill),
        head_cell("DANCE", Length::Fixed(COL_DANCE)),
        head_cell("STATUS", Length::Fixed(COL_STATUS)),
        Space::new().width(Length::Fixed(COL_ACTIONS)),
    ]
    .spacing(7)
    .padding([0, 10])
    .height(Length::Fixed(33.0))
    .align_y(Alignment::Center);

    container(head).style(theme::table_head).into()
}

fn build_rows<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    let mut rows: Column<'_, Message> = col![];

    for (i, item) in dance_interpreter
        .data_provider
        .playlist()
        .iter()
        .enumerate()
    {
        let is_match = data_matches_search_query(&config_window.search_query, (&item.song).into());
        rows = rows.push(build_song_row(
            dance_interpreter,
            &item.song,
            i,
            is_match,
            config_window.editing,
            &config_window.edit_draft,
        ));
    }

    scrollable(rows)
        .width(Length::Fill)
        .height(Length::Fill)
        .id(PLAYLIST_SCROLLABLE_ID.clone())
        .style(theme::scrollable_style)
        .into()
}

fn build_song_row<'a>(
    dance_interpreter: &'a DanceInterpreter,
    song: &'a crate::dataloading::songinfo::SongInfo,
    idx: usize,
    is_match: bool,
    editing: Option<(usize, EditField)>,
    edit_draft: &'a str,
) -> Element<'a, Message> {
    let (is_current, is_next, is_traktor, is_played) =
        dance_interpreter.data_provider.get_play_state(idx);

    let dance_color = dance_interpreter.data_provider.get_dance_color(&song.dance);

    // Number / state icon cell.
    let number: Element<'a, Message> = if is_traktor && !is_current {
        material_symbol_sized("agriculture", false, 15.0)
            .style(theme::text_style(|tk| tk.led_partial))
            .into()
    } else if is_current {
        material_symbol_sized("graphic_eq", false, sz::ICON)
            .style(theme::text_style(|tk| tk.accent_soft))
            .into()
    } else {
        text(format!("{:02}", idx + 1))
            .size(sz::TINY)
            .style(move |t: &Theme| iced::widget::text::Style {
                color: Some(if is_played {
                    theme::Tokens::of(t).faint
                } else {
                    theme::Tokens::of(t).muted
                }),
            })
            .into()
    };

    // Song title + artist: plain text that enters inline editing on
    // double-click (a single click is deliberately ignored, like the old
    // dynamic text input). Placeholders keep empty rows clickable.
    let editing_field = editing.filter(|(i, _)| *i == idx).map(|(_, field)| field);

    let title_field: Element<'a, Message> = if editing_field == Some(EditField::Title) {
        text_input("Title", edit_draft)
            .id(edit_input_id(idx, EditField::Title))
            .size(sz::ROW)
            .on_input(Message::EditDraftChanged)
            .on_submit(Message::EndSongEdit)
            .style(theme::input_plain)
            .padding(0)
            .into()
    } else {
        mouse_area(editable_text(
            &song.title,
            "Title",
            sz::ROW,
            |t: &Theme| theme::Tokens::of(t).text_soft,
            |t: &Theme| theme::Tokens::of(t).faint,
        ))
        .on_double_click(Message::BeginSongEdit(idx, EditField::Title))
        .into()
    };

    let artist_field: Element<'a, Message> = if editing_field == Some(EditField::Artist) {
        text_input("Artist", edit_draft)
            .id(edit_input_id(idx, EditField::Artist))
            .size(sz::BASE)
            .on_input(Message::EditDraftChanged)
            .on_submit(Message::EndSongEdit)
            .style(theme::input_plain_dim)
            .padding(0)
            .into()
    } else {
        mouse_area(editable_text(
            &song.artist,
            "Artist",
            sz::BASE,
            |t: &Theme| theme::Tokens::of(t).muted,
            |t: &Theme| theme::Tokens::of(t).faint,
        ))
        .on_double_click(Message::BeginSongEdit(idx, EditField::Artist))
        .into()
    };

    let song_cell = col![title_field, artist_field]
        .spacing(4)
        .clip(true)
        .width(Length::Fill);

    let chip = build_dance_cell(
        &song.dance,
        edit_draft,
        idx,
        dance_color,
        editing_field.is_some(),
    );

    let status: Element<'a, Message> = if is_current {
        row![
            crate::ui::widgets::dot(5.0, |t| theme::Tokens::of(t).accent_soft),
            text("On air")
                .size(sz::SMALL)
                .style(theme::text_style(|tk| tk.accent_soft)),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into()
    } else if is_next {
        row![
            material_symbol_sized("skip_next", false, 13.0),
            text("Up next")
                .size(sz::SMALL)
                .style(theme::text_style(|tk| tk.text_soft)),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .into()
    } else if is_traktor {
        text("DJ live")
            .size(sz::SMALL)
            .style(theme::text_style(|tk| tk.led_partial))
            .into()
    } else if is_played {
        row![
            material_symbol_sized("check", false, 13.0),
            text("Played")
                .size(sz::SMALL)
                .style(theme::text_style(|tk| tk.faint)),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .into()
    } else {
        text("Ready")
            .size(sz::SMALL)
            .style(theme::text_style(|tk| tk.faint))
            .into()
    };

    let actions = row![
        with_tooltip(
            buttons::ghost_icon(
                "play_arrow",
                false,
                14.0,
                Message::ItemChanged(ItemChange::PlaylistAbsolute(idx))
            ),
            "Show now"
        ),
        with_tooltip(
            buttons::ghost_icon(
                "skip_next",
                false,
                14.0,
                Message::SetNextItem(ItemSource::Playlist(idx))
            ),
            "Queue as next"
        ),
        with_tooltip(
            buttons::ghost_icon(
                "delete",
                false,
                14.0,
                Message::Dialog(DialogMessage::RequestDelete(ItemSource::Playlist(idx)))
            ),
            "Delete song"
        ),
    ]
    .spacing(2)
    .align_y(Alignment::Center);

    let content = row![
        container(number).width(Length::Fixed(COL_NUMBER)),
        song_cell,
        container(chip).width(Length::Fixed(COL_DANCE)),
        container(status).width(Length::Fixed(COL_STATUS)),
        container(actions).width(Length::Fixed(COL_ACTIONS)),
    ]
    .spacing(7)
    .align_y(Alignment::Center)
    .padding([0, 10])
    .width(Length::Fill);

    // The accent bar is rendered on every row at a constant 3px width (only
    // colored for the current song) so all columns of all rows — and the
    // table head, which reserves the same 3px spacer — line up exactly.
    let bar = container(Space::new())
        .width(Length::Fixed(3.0))
        .height(Length::Fill)
        .style(move |t: &Theme| {
            container::Style::default().background(if is_current {
                theme::Tokens::of(t).accent_bar
            } else {
                Color::TRANSPARENT
            })
        });

    let look = RowLook {
        zebra: idx % 2 == 1,
        current: is_current,
        matched: is_match && !is_current,
    };

    // The inner row needs an explicit fixed height: iced's layout marks
    // children of Shrink-height rows as "compressed", which collapses the
    // accent bar's Fill height to 0 and leaves the content stuck to the top.
    // With a fixed height the bar expands to the full stripe, inflating the
    // line height that align_y then centers the content block in.
    container(
        row![bar, content]
            .width(Length::Fill)
            .height(Length::Fixed(ROW_HEIGHT))
            .align_y(Alignment::Center),
    )
    .height(Length::Fixed(ROW_HEIGHT))
    .width(Length::Fill)
    .style(move |t: &Theme| theme::row(t, look))
    .into()
}

// Dance cell: compact chip with an inline editable dance name. Submitting
// matches the dance against the known dances (assigning its color). Like the
// song fields, the name is plain text until double-clicked.
fn build_dance_cell<'a>(
    dance: &'a str,
    edit_draft: &'a str,
    idx: usize,
    color: Option<Color>,
    editing: bool,
) -> Element<'a, Message> {
    let dance_field: Element<'a, Message> = if editing {
        text_input("Unassigned", edit_draft)
            .id(edit_input_id(idx, EditField::Dance))
            .size(sz::BASE)
            .on_input(Message::EditDraftChanged)
            .on_submit(Message::EndSongEdit)
            .style(move |t: &Theme, status: iced::widget::text_input::Status| {
                let mut style = theme::input_plain(t, status);
                style.value = match color {
                    Some(color) if !edit_draft.is_empty() => theme::readable(color, t),
                    _ => theme::Tokens::of(t).muted,
                };
                style
            })
            .padding(0)
            .into()
    } else {
        mouse_area(editable_text(
            dance,
            "Unassigned",
            sz::BASE,
            move |t: &Theme| {
                color
                    .map(|c| theme::readable(c, t))
                    .unwrap_or_else(|| theme::Tokens::of(t).muted)
            },
            |t: &Theme| theme::Tokens::of(t).faint,
        ))
        .on_double_click(Message::BeginSongEdit(idx, EditField::Dance))
        .into()
    };

    let chip = container(
        row![
            crate::ui::widgets::dot(7.0, move |t| {
                color
                    .map(|c| theme::readable(c, t))
                    .unwrap_or_else(|| theme::Tokens::of(t).muted)
            }),
            dance_field,
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .padding([7, 11])
    .width(Length::Fill)
    .clip(true)
    .style(theme::chip());

    chip.into()
}

/// A plain-text stand-in for a row's inline text input (shown while the field
/// is not being edited). Renders the placeholder word in the placeholder
/// color when the value is empty, so blank rows stay clickable.
fn editable_text<'a>(
    value: &'a str,
    placeholder: &'static str,
    size: f32,
    value_color: impl Fn(&Theme) -> Color + 'a,
    placeholder_color: impl Fn(&Theme) -> Color + 'a,
) -> Element<'a, Message> {
    if value.is_empty() {
        text(placeholder)
            .size(size)
            .shaping(Shaping::Advanced)
            .wrapping(Wrapping::None)
            .style(move |t: &Theme| iced::widget::text::Style {
                color: Some(placeholder_color(t)),
            })
            .into()
    } else {
        text(value)
            .size(size)
            .shaping(Shaping::Advanced)
            .wrapping(Wrapping::None)
            .style(move |t: &Theme| iced::widget::text::Style {
                color: Some(value_color(t)),
            })
            .into()
    }
}

// ---------------------------------------------------------------------------
// Quick add + footer
// ---------------------------------------------------------------------------

fn build_quick_add<'a>(config_window: &'a ConfigWindow) -> Element<'a, Message> {
    let submit = Message::QuickAdd(QuickAddMessage::Submit);

    let content = row![
        material_symbol_sized("add", false, sz::ICON).style(theme::text_style(|tk| tk.muted)),
        text_input("Add a song title…", &config_window.quick_title)
            .size(sz::ROW)
            .on_input(|v| Message::QuickAdd(QuickAddMessage::UpdateTitle(v)))
            .on_submit(submit.clone())
            .style(theme::input_boxed)
            .padding([5, 8]),
        text_input("Artist", &config_window.quick_artist)
            .size(sz::ROW)
            .width(Length::Fixed(150.0))
            .on_input(|v| Message::QuickAdd(QuickAddMessage::UpdateArtist(v)))
            .on_submit(submit.clone())
            .style(theme::input_boxed)
            .padding([5, 8]),
        text_input("Dance", &config_window.quick_dance)
            .size(sz::ROW)
            .width(Length::Fixed(130.0))
            .on_input(|v| Message::QuickAdd(QuickAddMessage::UpdateDance(v)))
            .on_submit(submit.clone())
            .style(theme::input_boxed)
            .padding([5, 8]),
        buttons::primary_small("Add song", submit),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    container(content)
        .padding([12, 14])
        .width(Length::Fill)
        .style(|t: &Theme| {
            let tk = theme::Tokens::of(t);
            container::Style::default().border(iced::Border {
                color: tk.border_soft,
                width: 1.0,
                radius: 0.0.into(),
            })
        })
        .into()
}

fn build_footer(dance_interpreter: &DanceInterpreter) -> Element<'static, Message> {
    let played = dance_interpreter
        .data_provider
        .playlist()
        .iter()
        .filter(|item| item.played)
        .count();
    let remaining = dance_interpreter.data_provider.playlist().len() - played;

    container(
        row![
            text(format!("{} played · {} remaining", played, remaining))
                .size(sz::TINY)
                .style(theme::text_style(|tk| tk.muted)),
            container(
                text("Double-click a field to edit")
                    .size(sz::TINY)
                    .style(theme::text_style(|tk| tk.faint))
            )
            .width(Length::Fill)
            .align_x(Alignment::End),
        ]
        .width(Length::Fill),
    )
    .padding([10, 17])
    .into()
}

// ---------------------------------------------------------------------------
// Empty state
// ---------------------------------------------------------------------------

fn build_empty_playlist_view(dance_interpreter: &DanceInterpreter) -> Element<'static, Message> {
    let traktor_available = dance_interpreter
        .data_provider
        .traktor_provider
        .get_song_info()
        .is_some();

    let open_card = empty_state_card(
        "folder_open",
        "Open playlist file",
        "Load an .m3u from disk",
        Message::OpenPlaylist,
    );

    let traktor_card = if traktor_available {
        empty_state_card(
            "add",
            "Add Traktor song",
            "Take the song playing on the DJ deck",
            Message::AddTraktorSong,
        )
    } else {
        empty_state_card(
            "link",
            "Use DJ integration",
            "Connect Traktor or another DJ source",
            Message::Sidebar(SidebarMessage::Toggle),
        )
    };

    let create_card = empty_state_card(
        "note_add",
        "Create playlist",
        "Start with a blank entry",
        Message::AddBlankEntry,
    );

    let content = col![
        heading("No playlist loaded.", 22.0, iced::font::Weight::Semibold)
            .style(theme::text_style(|tk| tk.text)),
        text("Drop an .m3u or audio file anywhere in this window, or start below.")
            .size(sz::SMALL)
            .style(theme::text_style(|tk| tk.muted)),
        // The three cards share the available width equally, capped at
        // 3 × 200 + spacing.
        container(
            row![open_card, traktor_card, create_card]
                .spacing(16)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .max_width(632.0),
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .width(Length::Fill);

    container(
        col![
            Space::new().height(Length::Fill),
            content,
            Space::new().height(Length::Fill)
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::card)
    .clip(true)
    .into()
}

fn empty_state_card(
    icon: &'static str,
    title: &'static str,
    subtitle: &'static str,
    message: Message,
) -> Element<'static, Message> {
    let card = button(
        container(
            col![
                material_symbol(icon, false)
                    .size(30.0)
                    .style(theme::text_style(|tk| tk.accent_soft)),
                text(title)
                    .size(sz::BASE)
                    .font(theme::body_font(iced::font::Weight::Medium))
                    .style(theme::text_style(|tk| tk.text)),
                text(subtitle)
                    .size(sz::TINY)
                    .style(theme::text_style(|tk| tk.muted)),
            ]
            .spacing(8)
            .align_x(Alignment::Center)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::Center),
    )
    .padding(20)
    .width(Length::FillPortion(1))
    .height(Length::Fixed(140.0))
    .style(|t: &Theme, s: button::Status| {
        let tk = theme::Tokens::of(t);
        let hovered = matches!(s, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(iced::Background::Color(if hovered {
                tk.match_bg
            } else {
                tk.panel_deep
            })),
            text_color: tk.text,
            border: iced::Border {
                color: tk.border,
                width: 1.0,
                radius: 7.0.into(),
            },
            ..button::Style::default()
        }
    })
    .on_press(message);

    card.into()
}
