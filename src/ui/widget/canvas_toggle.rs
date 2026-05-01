use crate::Message;
use iced::advanced::graphics::core::event::Event;
use iced::widget::canvas::{Frame, Geometry};
use iced::widget::{Action, Canvas, canvas};
use iced::{Element, Rectangle, Renderer, Theme, mouse};

type DrawFunction<'a> = Box<dyn Fn(&Theme, &mut Frame, bool) + 'a>;

pub struct CanvasToggle<'a> {
    is_checked: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    on_draw: Option<DrawFunction<'a>>,
    cache: &'a canvas::Cache,
}

impl<'a> CanvasToggle<'a> {
    pub fn new(is_checked: bool, cache: &'a canvas::Cache) -> Self {
        Self {
            is_checked,
            on_toggle: None,
            on_draw: None,
            cache,
        }
    }

    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(bool) -> Message,
    {
        self.on_toggle = Some(Box::new(f));
        self
    }

    pub fn on_draw<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(&Theme, &mut Frame, bool) + 'a,
    {
        self.on_draw = Some(Box::new(f));
        self
    }
}

impl<'a> From<CanvasToggle<'a>> for Canvas<CanvasToggle<'a>, Message, Theme, Renderer> {
    fn from(value: CanvasToggle<'a>) -> Self {
        Canvas::new(value)
    }
}

impl<'a> From<CanvasToggle<'a>> for Element<'a, Message, Theme, Renderer> {
    fn from(value: CanvasToggle<'a>) -> Self {
        Canvas::new(value).into()
    }
}

impl<'a> canvas::Program<Message> for CanvasToggle<'a> {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && cursor.is_over(bounds)
            && let Some(on_toggle) = &self.on_toggle
        {
            Some(Action::publish(on_toggle(!self.is_checked)))
        } else {
            None
        }
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        if let Some(on_draw) = &self.on_draw {
            let geo = self.cache.draw(renderer, bounds.size(), |frame| {
                on_draw(theme, frame, self.is_checked);
            });
            vec![geo]
        } else {
            Vec::new()
        }
    }

    fn mouse_interaction(
        &self,
        _state: &(),
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}
