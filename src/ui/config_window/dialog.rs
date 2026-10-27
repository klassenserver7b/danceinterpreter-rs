//! Modal dialogs: delete confirmation, static merge, settings, keyboard
//! help, the "add static" form and the flatpak playlist chooser.

use std::path::{Path, PathBuf};

use crate::dataloading::dataprovider::ItemSource;
use crate::ui::config_window::ConfigWindow;
use crate::ui::song_window::Backdrop;
use crate::ui::theme::{self, pad4, sz};
use crate::ui::widgets::{buttons, eyebrow, heading, icon_button_tooltip, kbd, push_right};
use crate::{
    DanceInterpreter, DialogMessage, Message, SongWindowMessage, StaticsMessage, TraktorMessage,
};
use iced::widget::{
    Button, Column, column as col, container, mouse_area, opaque, pick_list, row, scrollable,
    space::Space, stack, text, text_input, toggler,
};
use iced::{Alignment, Element, Length, Theme};
use iced_aw::number_input;

/// Max width of modal cards (design px).
const DIALOG_MAX_WIDTH: f32 = 440.0;
/// Vertical spacing between title, body and actions in a modal card.
const DIALOG_SPACING: f32 = 18.0;
/// Padding around a modal card's content (design px).
const DIALOG_PAD: f32 = 25.0;

#[derive(Debug, Clone)]
pub enum DialogState {
    Delete(ItemSource),
    MergeStatic {
        old_name: String,
        new_name: String,
    },
    Settings,
    Help,
    AddStatic,
    /// Pick one of the playlists discovered inside the folder granted
    /// through the flatpak portal (state lives on `ConfigWindow`).
    PlaylistChooser,
}

pub fn build_dialog<'a>(
    state: &'a DialogState,
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> Element<'a, Message> {
    let close = Message::Dialog(DialogMessage::Close);

    let (title, body, actions) = match state {
        DialogState::Delete(source) => {
            let target_name = match source {
                ItemSource::Playlist(i) => dance_interpreter
                    .data_provider
                    .playlist()
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

            let is_static = matches!(source, ItemSource::Static(_));
            let kind = if is_static { "static" } else { "song" };

            (
                format!("Delete {}?", kind),
                col![text(format!(
                    "Remove \u{201C}{}\u{201D} from the {}? You can undo with Ctrl+Z.",
                    target_name,
                    if is_static { "statics" } else { "playlist" }
                ))
                .size(sz::BASE)
                .style(theme::text_style(|tk| tk.muted))],
                dialog_actions([
                    buttons::secondary("Cancel", Message::Dialog(DialogMessage::Close)),
                    buttons::danger(
                        format!("Delete {}", kind),
                        Message::Dialog(DialogMessage::ConfirmDelete),
                    ),
                ]),
            )
        }
        DialogState::MergeStatic { old_name, new_name } => (
            "Merge statics?".to_string(),
            col![text(format!(
                "A static named \u{201C}{}\u{201D} already exists.\nMerge \u{201C}{}\u{201D} into it? All playlist songs will be re-tagged.",
                new_name, old_name
            ))
            .size(sz::BASE)
            .style(theme::text_style(|tk| tk.muted))],
            dialog_actions([
                buttons::secondary("Cancel", Message::Dialog(DialogMessage::Close)),
                buttons::primary(
                    "Merge",
                    Message::Statics(StaticsMessage::ConfirmMerge(
                        old_name.clone(),
                        new_name.clone(),
                    )),
                ),
            ]),
        ),
        DialogState::Settings => build_settings(config_window, dance_interpreter),
        DialogState::Help => build_help(),
        DialogState::AddStatic => build_add_static(config_window),
        DialogState::PlaylistChooser => build_playlist_chooser(config_window),
    };

    // Cap tall bodies at ¾ of the window height (scrolls only when needed;
    // the Shrink height keeps short bodies compact). The inner right padding
    // keeps the scrollbar off the rows' toggles and controls.
    let body: Element<'a, Message> = match state {
        DialogState::Settings | DialogState::Help => container(
            scrollable(container(body).padding(pad4(0.0, 12.0, 0.0, 0.0)))
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new().width(6.0).scroller_width(6.0),
                ))
                .style(theme::scrollable_style),
        )
        .width(Length::Fill)
        .max_height((config_window.size.height * 0.75).max(240.0))
        .into(),
        _ => body.into(),
    };

    let card = container(
        col![
            row![
                heading(title, sz::H2, iced::font::Weight::Medium)
                    .style(theme::text_style(|tk| tk.text)),
                Space::new().width(Length::Fill),
                icon_button_tooltip("close", 18.0, close, "Close"),
            ]
            .align_y(Alignment::Center),
            body,
            actions,
        ]
        .spacing(DIALOG_SPACING)
        .width(Length::Fill),
    )
    .padding(DIALOG_PAD)
    .width(Length::Fill)
    .max_width(DIALOG_MAX_WIDTH)
    .style(theme::modal_card);

    let backdrop = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::backdrop),
    )
    .on_press(Message::Dialog(DialogMessage::Close));

    let centered = container(opaque(card))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

    opaque(stack![backdrop, centered])
}

/// Modal action row: slightly larger buttons, aligned to the right edge
/// (the confirming / destructive action sits bottom-right).
fn dialog_actions<'a>(
    actions: impl IntoIterator<Item = Button<'a, Message>>,
) -> Element<'a, Message> {
    let items: Vec<Element<'a, Message>> = actions
        .into_iter()
        .map(|btn| btn.padding([10, 18]).into())
        .collect();

    push_right(row(items).spacing(10)).into()
}

/// A labeled settings row.
fn setting_row<'a>(label: &'a str, control: Element<'a, Message>) -> Element<'a, Message> {
    row![
        text(label)
            .size(sz::BASE)
            .style(theme::text_style(|tk| tk.text_soft))
            .width(Length::Fill),
        control,
    ]
    .align_y(Alignment::Center)
    .spacing(20)
    .padding([10, 0])
    .width(Length::Fill)
    .into()
}
fn section_label(label: &'static str) -> Element<'static, Message> {
    container(eyebrow(label).style(theme::text_style(|tk| tk.muted)))
        .padding(pad4(16.0, 0.0, 2.0, 0.0))
        .into()
}

/// Settings pick list entry: wraps an iced theme with a friendly label.
/// The builtin "Dark" theme carries our sage-green design, so it is
/// labeled "Green".
#[derive(Debug, Clone, PartialEq)]
struct ThemeOption(Theme);

impl std::fmt::Display for ThemeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self.0 {
            Theme::Dark => "Green",
            Theme::Light => "Light",
            Theme::Dracula => "Dracula",
            Theme::Nord => "Nord",
            Theme::SolarizedDark => "Solarized Dark",
            Theme::SolarizedLight => "Solarized Light",
            Theme::GruvboxDark => "Gruvbox Dark",
            Theme::GruvboxLight => "Gruvbox Light",
            Theme::CatppuccinLatte => "Catppuccin Latte",
            Theme::CatppuccinFrappe => "Catppuccin Frappé",
            Theme::CatppuccinMacchiato => "Catppuccin Macchiato",
            Theme::CatppuccinMocha => "Catppuccin Mocha",
            Theme::TokyoNight => "Tokyo Night",
            Theme::TokyoNightStorm => "Tokyo Night Storm",
            Theme::TokyoNightLight => "Tokyo Night Light",
            Theme::KanagawaWave => "Kanagawa Wave",
            Theme::KanagawaDragon => "Kanagawa Dragon",
            Theme::KanagawaLotus => "Kanagawa Lotus",
            Theme::Moonfly => "Moonfly",
            Theme::Nightfly => "Nightfly",
            Theme::Oxocarbon => "Oxocarbon",
            Theme::Ferra => "Ferra",
            Theme::Custom(_) => "Custom",
        })
    }
}

/// Theme options in pick-list order: the default first, then the green
/// design theme, light, and the remaining iced built-ins.
const THEME_OPTIONS: &[ThemeOption] = &[
    ThemeOption(Theme::Dracula),
    ThemeOption(Theme::Dark),
    ThemeOption(Theme::Light),
    ThemeOption(Theme::Nord),
    ThemeOption(Theme::SolarizedDark),
    ThemeOption(Theme::SolarizedLight),
    ThemeOption(Theme::GruvboxDark),
    ThemeOption(Theme::GruvboxLight),
    ThemeOption(Theme::CatppuccinLatte),
    ThemeOption(Theme::CatppuccinFrappe),
    ThemeOption(Theme::CatppuccinMacchiato),
    ThemeOption(Theme::CatppuccinMocha),
    ThemeOption(Theme::TokyoNight),
    ThemeOption(Theme::TokyoNightStorm),
    ThemeOption(Theme::TokyoNightLight),
    ThemeOption(Theme::KanagawaWave),
    ThemeOption(Theme::KanagawaDragon),
    ThemeOption(Theme::KanagawaLotus),
    ThemeOption(Theme::Moonfly),
    ThemeOption(Theme::Nightfly),
    ThemeOption(Theme::Oxocarbon),
    ThemeOption(Theme::Ferra),
];

fn build_settings<'a>(
    config_window: &'a ConfigWindow,
    dance_interpreter: &'a DanceInterpreter,
) -> (String, Column<'a, Message, Theme>, Element<'a, Message>) {
    let theme_pick = pick_list(
        THEME_OPTIONS,
        THEME_OPTIONS
            .iter()
            .find(|option| option.0 == config_window.theme)
            .cloned(),
        |option| Message::SetConfigTheme(option.0),
    )
    .width(Length::Fixed(160.0))
    .text_size(sz::BASE)
    .padding(theme::INPUT_PAD)
    .style(theme::pick_list_style)
    .menu_style(theme::pick_list_menu_style);

    let scale_input = number_input(&dance_interpreter.song_window.scale, 0.5..=3.0, |val| {
        Message::SongWindow(SongWindowMessage::ChangeScale(val))
    })
    .step(0.1)
    .width(Length::Fixed(120.0))
    .style(|t: &Theme, _status| {
        let tk = theme::Tokens::of(t);
        iced_aw::style::number_input::Style {
            button_background: Some(iced::Background::Color(tk.btn2_bg)),
            icon_color: tk.btn2_text,
        }
    });

    let backdrop_pick = pick_list(
        Backdrop::ALL,
        Some(dance_interpreter.song_window.backdrop),
        |backdrop| Message::SongWindow(SongWindowMessage::SetBackdrop(backdrop)),
    )
    .width(Length::Fixed(160.0))
    .text_size(sz::BASE)
    .padding(theme::INPUT_PAD)
    .style(theme::pick_list_style)
    .menu_style(theme::pick_list_menu_style);

    let switch = |checked: bool, msg: fn(bool) -> Message| {
        toggler(checked)
            .on_toggle(msg)
            .style(theme::toggler_style)
            .width(Length::Shrink)
            .into()
    };

    let body = col![
        setting_row("Theme", theme_pick.into()),
        setting_row(
            "Autoscroll to current song",
            switch(config_window.enable_autoscroll, Message::EnableAutoscroll),
        ),
        setting_row(
            "Debug logging (DJ integration)",
            switch(
                dance_interpreter
                    .data_provider
                    .traktor_provider
                    .debug_logging,
                |b| { Message::Traktor(TraktorMessage::EnableDebugLogging(b)) }
            ),
        ),
        section_label("AUDIENCE WINDOW"),
        setting_row("Text scale", scale_input.into()),
        setting_row("Backdrop", backdrop_pick.into()),
        container(
            text("The backdrop fills the audience window behind the song info.")
                .size(sz::TINY)
                .style(theme::text_style(|tk| tk.faint)),
        )
        .padding(pad4(0.0, 0.0, 8.0, 0.0)),
        setting_row(
            "Show thumbnails",
            switch(dance_interpreter.song_window.enable_image, |v| {
                Message::SongWindow(SongWindowMessage::EnableImage(v))
            },),
        ),
        setting_row(
            "Show next dance",
            switch(dance_interpreter.song_window.enable_next_dance, |v| {
                Message::SongWindow(SongWindowMessage::EnableNextDance(v))
            },),
        ),
        setting_row(
            "Open audience window",
            buttons::secondary("Focus window", Message::FocusSongWindow).into(),
        ),
        version_line(),
    ]
    .spacing(0);

    ("Settings".to_string(), body, col![].into())
}

fn build_help() -> (
    String,
    Column<'static, Message, Theme>,
    Element<'static, Message>,
) {
    let shortcuts: &[(&str, &str)] = &[
        ("Next song", "Space / \u{2192}"),
        ("Previous song", "\u{2190}"),
        ("Blank display", "End"),
        ("Song window fullscreen", "F11"),
        ("Find a song", "Ctrl+F"),
        ("Add blank entry", "Ctrl+N"),
        ("Undo delete", "Ctrl+Z"),
        ("Reload statics", "F5"),
        ("Scroll playlist", "PgUp / PgDn"),
        ("Audience text scale", "Ctrl + / \u{2212}"),
        ("Navigation sidebar", "Ctrl+B"),
        ("Info column", "Ctrl+J"),
        ("This help", "?"),
    ];

    let mut body = col![].spacing(4);

    for (action, keys) in shortcuts {
        body = body.push(
            container(
                row![
                    text(*action)
                        .size(sz::BASE)
                        .style(theme::text_style(|tk| tk.text_soft))
                        .width(Length::Fill),
                    kbd(*keys, theme::kbd_style),
                ]
                .align_y(Alignment::Center),
            )
            .padding([7, 0])
            .width(Length::Fill),
        );
    }

    body = body.push(version_line());

    ("Keyboard shortcuts".to_string(), body, col![].into())
}

fn version_line() -> Element<'static, Message> {
    container(
        text(concat!("Danceinterpreter v", env!("CARGO_PKG_VERSION")))
            .size(sz::MICRO)
            .style(theme::text_style(|tk| tk.faint)),
    )
    .padding(pad4(14.0, 0.0, 0.0, 0.0))
    .into()
}

fn build_add_static<'a>(
    config_window: &'a ConfigWindow,
) -> (String, Column<'a, Message, Theme>, Element<'a, Message>) {
    let submit = Message::Statics(StaticsMessage::AddNamed(
        config_window.add_static_name.clone(),
    ));

    let name_input = text_input("Static name", &config_window.add_static_name)
        .size(sz::BASE)
        .on_input(|v| Message::Dialog(DialogMessage::ChangeAddStaticName(v)))
        .on_submit(submit.clone())
        .style(theme::input_boxed)
        .padding([7, 9]);

    let body = col![
        text("A static is shown alone on the audience display \u{2014} a dance label or an announcement like \u{201C}Break\u{201D}.")
            .size(sz::BASE)
            .style(theme::text_style(|tk| tk.muted)),
        name_input,
    ]
    .spacing(12);

    let actions = dialog_actions([
        buttons::secondary("Cancel", Message::Dialog(DialogMessage::Close)),
        buttons::primary("Add static", submit),
    ]);

    ("Add static".to_string(), body, actions)
}

/// Playlist chooser entry: keeps the full path for loading and shows a
/// short label (the location relative to the granted folder) in the list.
#[derive(Debug, Clone, PartialEq)]
struct PlaylistOption {
    path: PathBuf,
    label: String,
}

impl PlaylistOption {
    fn new(path: &Path, granted_folder: &Path) -> Self {
        let label = path
            .strip_prefix(granted_folder)
            .ok()
            .and_then(|relative| relative.to_str())
            .map(str::to_string)
            .filter(|relative| !relative.is_empty())
            .unwrap_or_else(|| {
                path.file_name().map_or_else(
                    || path.display().to_string(),
                    |name| name.to_string_lossy().into_owned(),
                )
            });

        Self {
            path: path.to_path_buf(),
            label,
        }
    }
}

impl std::fmt::Display for PlaylistOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

fn build_playlist_chooser<'a>(
    config_window: &ConfigWindow,
) -> (String, Column<'a, Message, Theme>, Element<'a, Message>) {
    let options: Vec<PlaylistOption> = config_window
        .playlist_candidates
        .iter()
        .map(|path| PlaylistOption::new(path, &config_window.playlist_granted_folder))
        .collect();

    let selected = config_window
        .playlist_selected
        .clone()
        .or_else(|| config_window.playlist_candidates.first().cloned())
        .map(|path| PlaylistOption::new(&path, &config_window.playlist_granted_folder));

    let pick = pick_list(options, selected, |option| {
        Message::Dialog(DialogMessage::SelectPlaylist(option.path))
    })
    .width(Length::Fill)
    .text_size(sz::BASE)
    .padding(theme::INPUT_PAD)
    .style(theme::pick_list_style)
    .menu_style(theme::pick_list_menu_style);

    let body = col![
        text("The sandbox access covers everything inside the chosen folder. Which playlist should be loaded?")
            .size(sz::BASE)
            .style(theme::text_style(|tk| tk.muted)),
        pick,
    ]
    .spacing(12);

    let actions = dialog_actions([
        buttons::secondary("Cancel", Message::Dialog(DialogMessage::Close)),
        buttons::primary(
            "Load playlist",
            Message::Dialog(DialogMessage::ConfirmPlaylist),
        ),
    ]);

    ("Choose playlist".to_string(), body, actions)
}
