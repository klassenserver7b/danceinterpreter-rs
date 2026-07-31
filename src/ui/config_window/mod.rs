pub mod bottombar;
pub mod dialog;
pub mod item_row;
pub mod playlist_view;
pub mod search_bar;
pub mod sidebar;
pub mod statics_view;
pub mod top_bar;

pub use dialog::DialogState;
pub use playlist_view::PLAYLIST_SCROLLABLE_ID;

use crate::ui::config_window::sidebar::Sidebar;
use crate::{DanceInterpreter, Message, Window};
use iced::widget::{column as col, row, stack};
use iced::{Color, Element, Size, Theme, window};
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
    pub active_dialog: Option<DialogState>,
    pub color_picker_open: Option<String>,
    pub color_picker_old_color: Option<Color>,

    pub dummy_song_title: String,
    pub dummy_song_artist: String,
    pub dummy_song_dance: String,
}

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
            let search_bar = search_bar::build_search_bar(&self.search_query);
            main_column = main_column.push(search_bar);
        }

        let content_view = if self.is_statics_view {
            statics_view::build(self, dance_interpreter)
        } else {
            playlist_view::build(self, dance_interpreter)
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
            let dialog = dialog::build_dialog(state, dance_interpreter);
            stack![main_view, dialog].into()
        } else {
            main_view.into()
        }
    }
}
