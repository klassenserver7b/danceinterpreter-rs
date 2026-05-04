use crate::Message;
use iced::advanced::graphics::core::event::Event;
use iced::widget::canvas::{Cache, Frame, Geometry};
use iced::widget::{Action, Canvas, canvas};
use iced::{Element, Length, Rectangle, Renderer, Theme, mouse};
use std::rc::Rc;

type DrawFunction<'a> = Rc<dyn Fn(&Theme, &mut Frame, bool) + 'a>;
type ToggleFunction<'a> = Rc<dyn Fn(bool) -> Message + 'a>;

#[derive(Clone)]
pub struct CanvasToggle<'a> {
    is_checked: bool,
    on_toggle: Option<ToggleFunction<'a>>,
    on_draw: Option<DrawFunction<'a>>,
    cache: &'a Cache,
    width: Length,
    height: Length,
}

impl<'a> CanvasToggle<'a> {
    const DEFAULT_SIZE: f32 = 75.0;

    pub fn new(is_checked: bool, cache: &'a Cache) -> Self {
        Self {
            is_checked,
            on_toggle: None,
            on_draw: None,
            cache,
            width: Length::Fixed(Self::DEFAULT_SIZE),
            height: Length::Fixed(Self::DEFAULT_SIZE),
        }
    }

    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(bool) -> Message,
    {
        self.on_toggle = Some(Rc::new(f));
        self
    }

    pub fn on_draw<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(&Theme, &mut Frame, bool) + 'a,
    {
        self.on_draw = Some(Rc::new(f));
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Length::Fixed(width);
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        self.height = Length::Fixed(height);
        self
    }
}

impl<'a> From<CanvasToggle<'a>> for Canvas<CanvasToggle<'a>, Message, Theme, Renderer> {
    fn from(value: CanvasToggle<'a>) -> Self {
        let w = value.width;
        let h = value.height;
        Canvas::new(value).width(w).height(h)
    }
}

impl<'a> From<CanvasToggle<'a>> for Element<'a, Message, Theme, Renderer> {
    fn from(value: CanvasToggle<'a>) -> Self {
        <CanvasToggle<'a> as Into<Canvas<CanvasToggle, Message>>>::into(value).into()
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
