//! Canvas-drawn toggle for the DJ panel (power, restart).

use crate::Message;
use crate::ui::theme::Tokens;
use iced::advanced::graphics::core::event::Event;
use iced::mouse::Cursor;
use iced::widget::canvas::{Cache, Frame, Geometry, Path};
use iced::widget::{Action, Canvas, canvas};
use iced::{Element, Length, Point, Rectangle, Renderer, Theme, mouse, window};
use std::rc::Rc;
use std::time::Instant;

type DrawFunction<'a> = Rc<dyn Fn(&Theme, &mut Frame, bool) + 'a>;
type ToggleFunction<'a> = Rc<dyn Fn(bool) -> Message + 'a>;

const ANIM_SECS: f32 = 0.28;

/// A canvas-drawn toggle with an optional ripple press animation.
#[derive(Clone)]
pub struct CanvasToggle<'a> {
    is_checked: bool,
    on_toggle: Option<ToggleFunction<'a>>,
    on_draw: Option<DrawFunction<'a>>,
    cache: &'a Cache,
}

impl<'a> CanvasToggle<'a> {
    /// Canvas size (square) in px.
    const DEFAULT_SIZE: f32 = 75.0;

    /// Creates a toggle showing `is_checked`, drawn through `cache`.
    pub fn new(is_checked: bool, cache: &'a Cache) -> Self {
        Self {
            is_checked,
            on_toggle: None,
            on_draw: None,
            cache,
        }
    }

    /// Sets the message builder invoked on press (`true` = checked).
    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(bool) -> Message,
    {
        self.on_toggle = Some(Rc::new(f));
        self
    }

    /// Sets the draw function: `(theme, frame, is_checked)`.
    pub fn on_draw<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(&Theme, &mut Frame, bool) + 'a,
    {
        self.on_draw = Some(Rc::new(f));
        self
    }
}

impl<'a> From<CanvasToggle<'a>> for Canvas<CanvasToggle<'a>, Message, Theme, Renderer> {
    fn from(value: CanvasToggle<'a>) -> Self {
        Canvas::new(value)
            .width(Length::Fixed(CanvasToggle::DEFAULT_SIZE))
            .height(Length::Fixed(CanvasToggle::DEFAULT_SIZE))
    }
}

impl<'a> From<CanvasToggle<'a>> for Element<'a, Message, Theme, Renderer> {
    fn from(value: CanvasToggle<'a>) -> Self {
        <CanvasToggle<'a> as Into<Canvas<CanvasToggle, Message>>>::into(value).into()
    }
}

impl<'a> canvas::Program<Message> for CanvasToggle<'a> {
    // State now holds the instant of the last click for the ripple animation.
    type State = Option<Instant>;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<Action<Message>> {
        if let Some(started) = *state {
            if started.elapsed().as_secs_f32() < ANIM_SECS {
                // Mouse events and the redraw-requested window event both
                // creates loop, until animation is finished
                if matches!(
                    event,
                    Event::Mouse(_) | Event::Window(window::Event::RedrawRequested(_))
                ) {
                    return Some(Action::request_redraw());
                }
            } else {
                *state = None;
            }
        }

        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && cursor.is_over(bounds)
            && let Some(on_toggle) = &self.on_toggle
        {
            *state = Some(Instant::now());
            return Some(Action::publish(on_toggle(!self.is_checked)));
        }

        None
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let Some(on_draw) = &self.on_draw else {
            return Vec::new();
        };

        // When there is no active animation we can use the shared cache so
        // the canvas is only redrawn when the parent invalidates it.
        if state.is_none() {
            let geo = self.cache.draw(renderer, bounds.size(), |frame| {
                on_draw(theme, frame, self.is_checked);
            });
            return vec![geo];
        }

        // --- animated frame: draw base + ripple overlay on a fresh Frame ---
        let mut frame = Frame::new(renderer, bounds.size());
        on_draw(theme, &mut frame, self.is_checked);

        if let Some(started) = state {
            let progress = (started.elapsed().as_secs_f32() / ANIM_SECS).clamp(0.0, 1.0);

            let size = frame.size();
            let cx = size.width / 2.0;
            let cy = size.height / 2.0;
            let dim = size.width.min(size.height);

            // Ripple expands from 0 → bg radius and fades from 0.45 → 0
            let max_r = dim * super::BG_RADIUS;
            let r = max_r * progress;
            let alpha = (1.0 - progress) * 0.45;

            let mut ripple_color = Tokens::of(theme).accent_soft;
            ripple_color.a = alpha;

            let ripple = Path::new(|b| b.circle(Point::new(cx, cy), r));
            frame.fill(
                &ripple,
                canvas::Fill {
                    style: canvas::Style::Solid(ripple_color),
                    rule: canvas::fill::Rule::NonZero,
                },
            );
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}
