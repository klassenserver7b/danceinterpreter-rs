use crate::DialogMessage;
use crate::Message;
use crate::dataloading::dataprovider::ItemSource;
use crate::ui::theme;
use crate::ui::with_tooltip;
use iced::advanced::text::Shaping;
use iced::widget::{Text, container, text, toggler};
use iced::{Alignment, Color, Element, Font, Length, Pixels, Renderer, Theme};

/// Button builders (primary, secondary, ghosts).
pub mod buttons;
/// Canvas-drawn animated controls.
pub mod canvas;
/// Small round color preview button.
pub mod color_swatch;
/// Text that truncates with a trailing ellipsis (…) when it does not fit.
pub mod ellipsis_text;
/// Vendored combo-box-style text input with a suggestion menu.
pub mod suggestion_text_input;

pub use ellipsis_text::EllipsisText;

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

/// Outlined (or filled) Material Symbols glyph at an explicit size.
pub fn material_symbol_sized(
    id: &'_ str,
    filled: bool,
    size: impl Into<Pixels>,
) -> Text<'_, Theme, Renderer> {
    material_symbol(id, filled).size(size)
}

/// Outlined (or filled) Material Symbols glyph at the default icon size.
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

/// Keyboard hint pill, e.g. `Space`, `Ctrl+F`. `style` picks the pill look —
/// usually [`crate::ui::theme::kbd_style`], or a tinted variant on colored surfaces.
pub fn kbd<'a>(
    label: impl text::IntoFragment<'a>,
    style: fn(&Theme) -> iced::widget::container::Style,
) -> iced::widget::Container<'a, Message, Theme, Renderer> {
    iced::widget::container(text(label).size(theme::sz::MICRO))
        .padding([1, 5])
        .style(style)
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

/// A 1px horizontal divider spanning the full width.
pub fn divider() -> Element<'static, Message> {
    container(iced::widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(|t: &Theme| {
            iced::widget::container::Style::default().background(theme::Tokens::of(t).border)
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
            icon_muted(icon, theme::sz::ICON_SM),
            heading(title, theme::sz::PANEL_TITLE, iced::font::Weight::Semibold)
                .style(theme::text_style(|tk| tk.text)),
            text(subtitle)
                .size(theme::sz::TINY)
                .style(theme::text_style(|tk| tk.muted)),
            push_right(tools),
        ]
        .spacing(9)
        .align_y(Alignment::Center),
    )
    .padding(PANEL_HEADER_PAD)
    .width(Length::Fill)
    .into()
}

/// Ghost icon button with a bottom tooltip — the standard toolbar icon.
pub fn icon_button_tooltip<'a>(
    icon: &'static str,
    size: impl Into<Pixels>,
    message: Message,
    tip: impl text::IntoFragment<'a>,
) -> Element<'a, Message> {
    with_tooltip(buttons::ghost_icon(icon, false, size, message), tip)
}

/// Ghost "delete" icon button for a playlist row or static, with tooltip.
pub fn delete_button<'a>(
    source: ItemSource,
    tip: impl text::IntoFragment<'a>,
) -> Element<'a, Message> {
    icon_button_tooltip(
        "delete",
        theme::sz::ICON_XS,
        Message::Dialog(DialogMessage::RequestDelete(source)),
        tip,
    )
}

/// Right-aligns `content` across the full available width.
pub fn push_right<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Container<'a, Message, Theme, Renderer> {
    container(content)
        .width(Length::Fill)
        .align_x(Alignment::End)
}

/// Full-size card panel: fills the view and clips overflowing content.
pub fn card_panel<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Container<'a, Message, Theme, Renderer> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::card)
        .clip(true)
}

/// Tiny semibold label in the muted-head color (table column heads, control
/// captions, dialog section labels).
pub fn eyebrow<'a>(label: impl text::IntoFragment<'a>) -> Text<'a, Theme, Renderer> {
    text(label)
        .size(theme::sz::MICRO)
        .font(theme::body_font(iced::font::Weight::Semibold))
        .style(theme::text_style(|tk| tk.muted_head))
}

/// Muted (low-emphasis) icon at an explicit size.
pub fn icon_muted(icon: &'static str, size: impl Into<Pixels>) -> Text<'static, Theme, Renderer> {
    material_symbol_sized(icon, false, size).style(theme::text_style(|tk| tk.muted))
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
