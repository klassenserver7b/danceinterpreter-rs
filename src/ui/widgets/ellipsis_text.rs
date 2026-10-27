//! Single-line text that truncates with a trailing ellipsis (…) when it does
//! not fit the available width.
//!
//! iced's built-in [`text`](iced::widget::text) cannot truncate: a string
//! wider than its bounds either wraps or overflows, and the app worked
//! around that with hard `clip(true)` containers — which cuts the text off
//! mid-word without any indication. This widget instead shrinks the *string*
//! itself: it measures candidates with the renderer's own paragraph machinery
//! (the same shaping engine that renders them) and displays the largest
//! prefix that still fits together with a trailing `…`, so the cut is always
//! visible and deliberate.

use iced::advanced::layout::{self};
use iced::advanced::renderer;
use iced::advanced::text::paragraph::{self, Paragraph};
use iced::advanced::text::{self, LineHeight, Shaping, Wrapping};
use iced::advanced::widget::{self, Widget};
use iced::advanced::Layout;
use iced::alignment;
use iced::mouse;
use iced::widget::text::Style;
use iced::{Element, Length, Pixels, Rectangle, Size};

/// Appended to the displayed string when the text had to be truncated.
const ELLIPSIS: &str = "…";

/// Single-line text that truncates with an ellipsis (…) when it does not fit
/// into the available width.
///
/// Mirrors the API of [`iced::widget::Text`] for the options that matter
/// here: [`EllipsisText::size`], [`EllipsisText::font`],
/// [`EllipsisText::line_height`] and [`EllipsisText::style`].
pub struct EllipsisText<'a, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: text::Renderer,
{
    content: text::Fragment<'a>,
    width: Length,
    size: Option<Pixels>,
    line_height: LineHeight,
    font: Option<Renderer::Font>,
    shaping: Shaping,
    style: Box<dyn Fn(&Theme) -> Style + 'a>,
}

impl<'a, Theme, Renderer> EllipsisText<'a, Theme, Renderer>
where
    Renderer: text::Renderer,
{
    /// Creates a new [`EllipsisText`] with the given contents.
    pub fn new(content: impl text::IntoFragment<'a>) -> Self {
        Self {
            content: content.into_fragment(),
            width: Length::Fill,
            size: None,
            line_height: LineHeight::default(),
            font: None,
            shaping: Shaping::default(),
            style: Box::new(|_| Style { color: None }),
        }
    }

    /// Sets the width of the [`EllipsisText`] boundaries.
    ///
    /// `Length::Shrink` (node reports the rendered width) is the right choice
    /// inside a [`row`](iced::widget::row): the segments truncate against the
    /// space remaining after their predecessors, so earlier segments take
    /// priority. `Length::Fill` (the default) is right for a full-width line
    /// of its own.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the size of the [`EllipsisText`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the [`LineHeight`] of the [`EllipsisText`].
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the font of the [`EllipsisText`].
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the [`Shaping`] strategy of the [`EllipsisText`].
    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.shaping = shaping;
        self
    }

    /// Sets the style of the [`EllipsisText`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self {
        self.style = Box::new(style);
        self
    }

    /// The string to display: the content itself when it fits, otherwise the
    /// largest character prefix that fits together with a trailing
    /// [`ELLIPSIS`] (empty when even the ellipsis alone does not fit).
    fn display(&self, bounds: Size, size: Pixels, font: Renderer::Font) -> String {
        let max_width = bounds.width;
        let content: &str = &self.content;

        // Measures the rendered width of a candidate string. `Wrapping::None`
        // keeps it a single line whose bounds are exactly what the widget
        // will later draw.
        let measured_width = |candidate: &str| {
            Renderer::Paragraph::with_text(text::Text {
                content: candidate,
                bounds,
                size,
                line_height: self.line_height,
                font,
                align_x: text::Alignment::Default,
                align_y: alignment::Vertical::Top,
                shaping: self.shaping,
                wrapping: Wrapping::None,
            })
            .min_bounds()
            .width
        };

        if measured_width(content) <= max_width {
            return content.to_string();
        }

        if measured_width(ELLIPSIS) > max_width {
            // Degenerate: not even the ellipsis fits; drop the text entirely.
            return String::new();
        }

        // Largest character prefix that still fits together with the
        // ellipsis. Binary search: the empty prefix always fits (established
        // above), the full content never does.
        let total = content.chars().count();
        let (mut lo, mut hi) = (0usize, total);
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;

            if measured_width(&content.chars().take(mid).collect::<String>()) <= max_width {
                lo = mid;
            } else {
                hi = mid;
            }
        }

        let mut display: String = content.chars().take(lo).collect();
        // Drop whitespace left over at the cut (`foo ` → `foo`); it can only
        // shrink the string, so it still fits.
        let trimmed = display.trim_end().len();
        display.truncate(trimmed);
        display.push_str(ELLIPSIS);
        display
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for EllipsisText<'_, Theme, Renderer>
where
    Renderer: text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<paragraph::Plain<Renderer::Paragraph>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(paragraph::Plain::<Renderer::Paragraph>::default())
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let paragraph = tree
            .state
            .downcast_mut::<paragraph::Plain<Renderer::Paragraph>>();

        layout::sized(limits, self.width, Length::Shrink, |limits| {
            let bounds = limits.max();
            let size = self.size.unwrap_or_else(|| renderer.default_size());
            let font = self.font.unwrap_or_else(|| renderer.default_font());

            let display = self.display(bounds, size, font);

            paragraph.update(text::Text {
                content: display.as_str(),
                bounds,
                size,
                line_height: self.line_height,
                font,
                align_x: text::Alignment::Default,
                align_y: alignment::Vertical::Top,
                shaping: self.shaping,
                wrapping: Wrapping::None,
            });

            paragraph.min_bounds()
        })
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let paragraph = tree
            .state
            .downcast_ref::<paragraph::Plain<Renderer::Paragraph>>();
        let bounds = layout.bounds();
        if !viewport.intersects(&bounds) {
            return;
        }

        let paragraph = paragraph.raw();
        let anchor = bounds.anchor(
            paragraph.min_bounds(),
            paragraph.align_x(),
            paragraph.align_y(),
        );

        renderer.fill_paragraph(
            paragraph,
            anchor,
            (self.style)(theme).color.unwrap_or(defaults.text_color),
            *viewport,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<EllipsisText<'a, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(ellipsis_text: EllipsisText<'a, Theme, Renderer>) -> Self {
        Element::new(ellipsis_text)
    }
}
