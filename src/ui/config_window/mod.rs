//! The operator-facing configuration window, redesigned after
//! `docs/src/App.tsx`: left navigation sidebar, "now playing" hero card,
//! playlist/statics panels, a right rail (up-next, DJ integration, tips), the
//! quick-show bar, toasts and modal dialogs.

pub mod dialog;
pub mod dj_panel;
pub mod nav_sidebar;
pub mod now_card;
pub mod playlist_view;
pub mod right_column;
pub mod show_bar;
pub mod statics_view;
pub mod toast;
pub mod top_bar;

pub use dialog::DialogState;
pub use playlist_view::PLAYLIST_SCROLLABLE_ID;

/// LED dot color for the DJ connection state (right rail status, DJ panel).
pub fn led_color(theme: &Theme, state: ConnectionState) -> Color {
    let t = theme::Tokens::of(theme);
    match state {
        ConnectionState::Connected => t.led_connected,
        ConnectionState::Traktor | ConnectionState::CoverLoader => t.led_partial,
        ConnectionState::Disconnected => t.led_off,
    }
}

use crate::traktor_api::ConnectionState;
use crate::ui::config_window::dj_panel::Sidebar;
use crate::ui::theme::{self, pad4};
use crate::ui::widgets::text_micro;
use crate::{DanceInterpreter, Message, Window};
use iced::widget::{column as col, container, mouse_area, row, stack};
use iced::{Alignment, Animation, Color, Element, Length, Size, Theme, animation, window};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Which main view is shown in the primary column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Playlist,
    Statics,
}

/// Which field of a playlist row is being inline-edited (double-click to
/// enter, Enter/Escape or editing another field to leave).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
    Title,
    Artist,
    Dance,
}

/// A song's editable field values as they were before its inline editing
/// session began. Escape restores them, rolling the whole session back —
/// including drafts that were auto-committed when switching fields.
#[derive(Debug, Clone)]
pub struct EditSnapshot {
    pub title: String,
    pub artist: String,
    pub dance: String,
}

/// Widget id of the inline edit input for a playlist row field, so the
/// `focus` task can target the input that only exists while editing.
pub fn edit_input_id(idx: usize, field: EditField) -> iced::widget::Id {
    let field = match field {
        EditField::Title => "title",
        EditField::Artist => "artist",
        EditField::Dance => "dance",
    };

    iced::widget::Id::from(format!("song-{idx}-{field}"))
}

/// A short-lived notification bubble above the quick-show bar.
#[derive(Debug, Clone)]
pub struct Toast {
    pub text: String,
    pub deadline: Instant,
}

impl Toast {
    pub const LIFETIME: Duration = Duration::from_millis(3500);

    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            deadline: Instant::now() + Self::LIFETIME,
        }
    }

    pub fn expired(&self, now: Instant) -> bool {
        now >= self.deadline
    }
}

/// Slide/expand duration shared by the nav sidebar, info rail and DJ panel.
pub const SLIDE_DURATION: Duration = Duration::from_millis(140);

pub struct ConfigWindow {
    pub id: window::Id,
    pub closed: bool,
    pub size: Size,
    pub enable_autoscroll: bool,
    pub sidebar: Sidebar,
    pub view: View,
    pub theme: Theme,

    /// Collapsible columns: left navigation sidebar and right info rail.
    pub nav_anim: Animation<bool>,
    pub rail_anim: Animation<bool>,

    pub search_visible: bool,
    pub search_query: String,
    pub search_input_id: iced::widget::Id,
    pub active_dialog: Option<DialogState>,
    pub color_picker_open: Option<String>,
    pub color_picker_old_color: Option<Color>,

    /// The playlist row field currently open for inline editing, if any.
    pub editing: Option<(usize, EditField)>,

    /// Buffer for the inline editor: keystrokes land here and are only
    /// committed to the song on Enter or click-away (Escape rolls back).
    pub edit_draft: String,

    /// The edited song's values before the current editing session began;
    /// Escape restores them. Cleared when the session ends (Enter,
    /// click-away or the playlist mutating under the editor).
    pub edit_snapshot: Option<(usize, EditSnapshot)>,

    pub quick_title: String,
    pub quick_artist: String,
    pub quick_dance: String,

    pub add_static_name: String,
    pub toast: Option<Toast>,
    pub playlist_file: Option<PathBuf>,
}

impl Window for ConfigWindow {
    fn new(id: window::Id) -> Self {
        Self {
            id,
            closed: false,
            size: Size::default(),

            enable_autoscroll: true,
            sidebar: Sidebar::new(),
            view: View::Playlist,
            theme: Theme::Dracula,

            nav_anim: Animation::new(true)
                .duration(SLIDE_DURATION)
                .easing(animation::Easing::EaseInOut),
            rail_anim: Animation::new(true)
                .duration(SLIDE_DURATION)
                .easing(animation::Easing::EaseInOut),

            search_visible: false,
            search_query: String::new(),
            search_input_id: iced::widget::Id::unique(),
            active_dialog: None,
            color_picker_open: None,
            color_picker_old_color: None,

            editing: None,
            edit_draft: String::new(),
            edit_snapshot: None,

            quick_title: String::new(),
            quick_artist: String::new(),
            quick_dance: String::new(),

            add_static_name: String::new(),
            toast: None,
            playlist_file: None,
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
    /// Expanded width of a window-proportional side panel: the window width
    /// divided by `divisor`, clamped between a readable floor and the panel's
    /// design width (all in px).
    fn expanded_width(window_width: f32, divisor: f32, min: f32, max: f32) -> f32 {
        (window_width / divisor).clamp(min, max)
    }

    pub fn view<'a>(&'a self, dance_interpreter: &'a DanceInterpreter) -> Element<'a, Message> {
        let now = Instant::now();

        // Primary content column: hero card and the active panel.
        let primary = container(
            col![
                now_card::build(dance_interpreter),
                match self.view {
                    View::Playlist => playlist_view::build(self, dance_interpreter),
                    View::Statics => statics_view::build(self, dance_interpreter),
                },
            ]
            .spacing(22),
        )
        .padding(pad4(18.0, theme::PAGE_GUTTER, 22.0, theme::PAGE_GUTTER))
        .width(Length::Fill)
        .height(Length::Fill);

        // Slide-in DJ integration panel (animated width, pushes content).
        let dj_panel = self.sidebar.build(
            dance_interpreter,
            self.sidebar.state.interpolate(
                0.0,
                Self::expanded_width(self.size.width, 5.0, 0.0, dj_panel::PANEL_WIDTH),
                now,
            ),
        );

        // Collapsible columns: navigation sidebar (left) and info rail
        // (right). Their expanded width follows the window (like the DJ
        // panel), capped at the design width with a readable floor.
        let nav = nav_sidebar::build(
            self,
            dance_interpreter,
            self.nav_anim.interpolate(
                0.0,
                Self::expanded_width(self.size.width, 6.0, 180.0, nav_sidebar::SIDEBAR_WIDTH),
                now,
            ),
        );

        let rail = right_column::build(
            dance_interpreter,
            self.rail_anim.interpolate(
                0.0,
                Self::expanded_width(self.size.width, 5.0, 210.0, right_column::RAIL_WIDTH),
                now,
            ),
        );

        let content = row![primary, dj_panel, rail]
            .spacing(10)
            .width(Length::Fill)
            .height(Length::Fill);

        let main = row![
            nav,
            col![
                top_bar::bar_with_divider(top_bar::build(self)),
                content,
                show_bar::build(dance_interpreter),
            ]
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        // While a playlist field is open for editing, a click that no widget
        // claims commits the draft (treated exactly like pressing Enter), so
        // re-editing always takes a double click. The wrapper is a polite
        // observer: the editor, buttons and other inputs capture their clicks
        // before they ever reach it — it only fires for dead areas.
        let main: Element<'a, Message> = if self.editing.is_some() {
            mouse_area(main).on_press(Message::EndSongEdit).into()
        } else {
            main.into()
        };

        // Overlays: toast bubble, then modal dialog on top.
        let mut overlays = stack![main];

        if let Some(toast_element) = toast::build(&self.toast) {
            overlays = overlays.push(toast_element);
        }

        if let Some(state) = &self.active_dialog {
            let dialog = dialog::build_dialog(state, self, dance_interpreter);
            overlays = overlays.push(dialog);
        }

        overlays.into()
    }

    /// Whether the left navigation sidebar is (being) shown.
    pub fn nav_open(&self) -> bool {
        self.nav_anim.value()
    }

    /// Whether the right info rail is (being) shown.
    pub fn rail_open(&self) -> bool {
        self.rail_anim.value()
    }

    pub fn toggle_nav_sidebar(&mut self) {
        self.nav_anim.go_mut(!self.nav_open(), Instant::now());
    }

    pub fn toggle_info_rail(&mut self) {
        self.rail_anim.go_mut(!self.rail_open(), Instant::now());
    }
}

/// Shared helper: a "source tag" chip like `PLAYLIST` / `QUEUED`.
pub fn source_tag<'a>(text: &'a str) -> Element<'a, Message> {
    container(text_micro(text).style(theme::text_style(|tk| tk.tag_text)))
        .padding([2, 5])
        .style(|t: &Theme| {
            let tk = theme::Tokens::of(t);
            container::Style::default().border(iced::Border {
                color: tk.tag_border,
                width: 1.0,
                radius: theme::r::XS.into(),
            })
        })
        .into()
}

/// [`source_tag`] pushed to the right edge of the row it lives in.
pub fn source_tag_right<'a>(text: &'a str) -> Element<'a, Message> {
    container(source_tag(text))
        .width(Length::Fill)
        .align_x(Alignment::End)
        .into()
}

pub mod search {
    use crate::dataloading::displayable_data::DisplayableData;
    use fuzzy_matcher::FuzzyMatcher;
    use fuzzy_matcher::skim::SkimMatcherV2 as Matcher;
    use std::sync::LazyLock;

    /// The skim matcher is expensive to construct (builds lookup tables), so
    /// one shared instance serves every keystroke of every search field.
    static MATCHER: LazyLock<Matcher> = LazyLock::new(Matcher::default);

    pub fn data_matches_search_query(search_query: &str, data: DisplayableData) -> bool {
        let matcher = &*MATCHER;

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
