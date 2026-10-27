use crate::Message;
use crate::ui::theme;
use iced::advanced::text::Shaping;
use iced::widget::{Text, container, text, toggler};
use iced::{Alignment, Color, Element, Font, Length, Pixels, Renderer, Theme};

pub mod buttons;
pub mod canvas;
pub mod color_swatch;
pub mod parity;
pub mod suggestion_text_input;

/// Accent bar width of table rows (the table head reserves a matching
/// spacer so all columns line up).
pub const ACCENT_BAR_W: f32 = 3.0;

/// Uniform stripe height of playlist and statics rows.
pub const ROW_HEIGHT: f32 = 68.0;

/// Vertical/horizontal padding of panel toolbars (playlist, statics).
pub const PANEL_HEADER_PAD: [f32; 2] = [14.0, 17.0];

/// Small labeled toggler (switch) styled with the design tokens.
pub fn labeled_message_toggler(
    label: &'_ str,
    checked: bool,
    message: fn(bool) -> Message,
) -> toggler::Toggler<'_, Message> {
    toggler(checked)
        .label(label)
        .on_toggle(message)
        .width(Length::Fill)
        .text_size(theme::sz::SMALL)
        .style(theme::toggler_style)
}

pub fn material_symbol_colored(
    id: &'_ str,
    filled: bool,
    color: impl Into<Color>,
) -> Text<'_, Theme, Renderer> {
    material_symbol(id, filled).color(color)
}

pub fn material_symbol_sized(
    id: &'_ str,
    filled: bool,
    size: impl Into<Pixels>,
) -> Text<'_, Theme, Renderer> {
    material_symbol(id, filled).size(size)
}

pub fn material_symbol(id: &'_ str, filled: bool) -> Text<'_, Theme, Renderer> {
    let font_name = if filled {
        theme::FONT_ICON_FILLED
    } else {
        theme::FONT_ICON
    };

    Text::new(id)
        .font(Font::with_name(font_name))
        .shaping(Shaping::Advanced)
        .width(Length::Shrink)
}

/// Keyboard hint pill, e.g. `Space`, `Ctrl+F`.
pub fn kbd<'a>(
    label: impl text::IntoFragment<'a>,
) -> iced::widget::Container<'a, Message, Theme, Renderer> {
    iced::widget::container(
        text(label)
            .size(theme::sz::MICRO)
            .style(theme::text_style(|tk| tk.kbd_text)),
    )
    .padding([1, 5])
    .style(theme::kbd)
}

/// Body text at the micro size (tags, eyebrows, hints).
pub fn text_micro<'a>(label: impl text::IntoFragment<'a>) -> Text<'a, Theme, Renderer> {
    text(label).size(theme::sz::MICRO)
}

/// Small filled circle (status dots, dance color dots). `color` resolves the
/// fill from the active theme.
pub fn dot(size: f32, color: impl Fn(&Theme) -> Color + 'static) -> Element<'static, Message> {
    container(iced::widget::Space::new())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |t: &Theme| {
            iced::widget::container::Style::default()
                .background(color(t))
                .border(iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: (size / 2.0).into(),
                })
        })
        .into()
}

/// Manrope heading text.
pub fn heading<'a>(
    label: impl text::IntoFragment<'a>,
    size: impl Into<Pixels>,
    weight: iced::font::Weight,
) -> Text<'a, Theme, Renderer> {
    text(label).size(size).font(theme::head_font(weight))
}

/// Standard panel header: muted icon, semibold panel title, optional muted
/// count (`subtitle`), right-aligned `tools`, [`PANEL_HEADER_PAD`] padding.
pub fn panel_header<'a>(
    icon: &'static str,
    title: impl text::IntoFragment<'a>,
    subtitle: impl text::IntoFragment<'a>,
    tools: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        iced::widget::row![
            material_symbol_sized(icon, false, 15.0).style(theme::text_style(|tk| tk.muted)),
            heading(title, theme::sz::PANEL_TITLE, iced::font::Weight::Semibold)
                .style(theme::text_style(|tk| tk.text)),
            text(subtitle)
                .size(theme::sz::TINY)
                .style(theme::text_style(|tk| tk.muted)),
            container(tools).width(Length::Fill).align_x(Alignment::End),
        ]
        .spacing(9)
        .align_y(Alignment::Center),
    )
    .padding(PANEL_HEADER_PAD)
    .width(Length::Fill)
    .into()
}

/// Standard table row shell: a fixed-height stripe ([`ROW_HEIGHT`]) with the
/// hairline row border and a constant-width accent bar on the left edge.
///
/// The inner row needs the explicit fixed height: iced's layout marks
/// children of Shrink-height rows as "compressed", which collapses the
/// accent bar's Fill height to 0 and leaves the content stuck to the top.
/// With a fixed height the bar expands to the full stripe, inflating the
/// line height that align_y then centers the content block in.
///
/// `current_bar` resolves the bar color while [`theme::RowLook::current`] is
/// set (the bar stays transparent otherwise), so every row's columns — and
/// the table head's matching spacer — line up exactly.
pub fn row_shell<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    look: theme::RowLook,
    current_bar: impl Fn(&Theme) -> Color + 'a,
) -> Element<'a, Message> {
    let bar = container(iced::widget::Space::new())
        .width(Length::Fixed(ACCENT_BAR_W))
        .height(Length::Fill)
        .style(move |t: &Theme| {
            iced::widget::container::Style::default().background(if look.current {
                current_bar(t)
            } else {
                Color::TRANSPARENT
            })
        });

    container(
        iced::widget::row![bar, content.into()]
            .width(Length::Fill)
            .height(Length::Fixed(ROW_HEIGHT))
            .align_y(Alignment::Center),
    )
    .height(Length::Fixed(ROW_HEIGHT))
    .width(Length::Fill)
    .style(move |t: &Theme| theme::row(t, look))
    .into()
}
