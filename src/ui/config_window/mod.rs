//! The operator-facing configuration window, redesigned after
//! `docs/src/App.tsx`: left navigation sidebar, page header, "now playing"
//! hero card, playlist/statics panels, a right rail (up-next, DJ integration,
//! tips), the quick-show bar, toasts and modal dialogs.

pub mod dialog;
pub mod nav_sidebar;
pub mod now_card;
pub mod page_header;
pub mod playlist_view;
pub mod right_column;
pub mod show_bar;
pub mod sidebar;
pub mod statics_view;
pub mod toast;
pub mod top_bar;

pub use dialog::DialogState;
pub use playlist_view::PLAYLIST_SCROLLABLE_ID;

use crate::ui::config_window::sidebar::Sidebar;
use crate::ui::theme;
use crate::{DanceInterpreter, Message, Window};
use iced::widget::{column as col, container, row, stack};
use iced::{Color, Element, Length, Size, Theme, window};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Which main view is shown in the primary column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Playlist,
    Statics,
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

pub struct ConfigWindow {
    pub id: window::Id,
    pub closed: bool,
    pub size: Size,
    pub enable_autoscroll: bool,
    pub sidebar: Sidebar,
    pub view: View,
    pub theme: Theme,
    pub follow_system_theme: bool,

    pub search_visible: bool,
    pub search_query: String,
    pub search_input_id: iced::widget::Id,
    pub active_dialog: Option<DialogState>,
    pub color_picker_open: Option<String>,
    pub color_picker_old_color: Option<Color>,

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
            theme: Theme::Dark,
            follow_system_theme: true,

            search_visible: false,
            search_query: String::new(),
            search_input_id: iced::widget::Id::unique(),
            active_dialog: None,
            color_picker_open: None,
            color_picker_old_color: None,

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
    pub fn view<'a>(&'a self, dance_interpreter: &'a DanceInterpreter) -> Element<'a, Message> {
        // Primary content column: page heading, hero card and the active panel.
        let primary = container(
            col![
                page_header::build(self, dance_interpreter),
                now_card::build(self, dance_interpreter),
                match self.view {
                    View::Playlist => playlist_view::build(self, dance_interpreter),
                    View::Statics => statics_view::build(self, dance_interpreter),
                },
            ]
            .spacing(22),
        )
        .padding(crate::ui::theme::pad4(18.0, 31.0, 22.0, 31.0))
        .width(Length::Fill)
        .height(Length::Fill);

        // Slide-in DJ integration panel (animated width, pushes content).
        let dj_panel = self
            .sidebar
            .build(dance_interpreter)
            .width(self.sidebar.state.interpolate(
                0.0,
                (self.size.width / 5.0).min(400.0),
                Instant::now(),
            ));

        let content = row![primary, dj_panel, right_column::build(self, dance_interpreter)]
            .width(Length::Fill)
            .height(Length::Fill);

        let main = row![
            nav_sidebar::build(self, dance_interpreter),
            col![
                top_bar::bar_with_divider(top_bar::build(self, dance_interpreter)),
                content,
                show_bar::build(self, dance_interpreter),
            ]
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill);

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
}

/// Shared helper: a "source tag" chip like `PLAYLIST` / `QUEUED`.
pub fn source_tag<'a>(text: &'a str) -> Element<'a, Message> {
    container(
        crate::ui::widgets::text_micro(text)
            .style(|t: &Theme| iced::widget::text::Style {
                color: Some(theme::Tokens::of(t).tag_text),
            }),
    )
    .padding([2, 5])
    .style(|t: &Theme| {
        let tk = theme::Tokens::of(t);
        container::Style::default().border(iced::Border {
            color: tk.tag_border,
            width: 1.0,
            radius: 3.0.into(),
        })
    })
    .into()
}

pub mod search {
    use crate::dataloading::displayable_data::DisplayableData;
    use fuzzy_matcher::FuzzyMatcher;
    use fuzzy_matcher::skim::SkimMatcherV2 as Matcher;

    pub fn data_matches_search_query(search_query: &str, data: DisplayableData) -> bool {
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
