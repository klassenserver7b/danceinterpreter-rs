//! The audience-facing song window, redesigned after the approved
//! `docs/src/audience.css`: a huge dance headline, the song block with cover
//! art and an "UP NEXT" hint in the bottom-right corner.
//!
//! The window always stays dark for maximum contrast on the projector. When
//! the display is blank (`ItemSource::Blank`), the window turns fully black.

use crate::Window;
use crate::{DanceInterpreter, Message};
use iced::advanced::text::Shaping;
use iced::alignment::{Horizontal, Vertical};
use iced::font::Weight;
use iced::widget::space::horizontal;
use iced::widget::text::LineHeight;
use iced::widget::{Text, column as col, container, image, row, stack};
use iced::{ContentFit, Element, Gradient, Length, Theme, window};

/// Audience text colors from `docs/src/audience.css`.
mod colors {
    use iced::color;
    /// Body text / dance headline / song title.
    pub const TEXT: iced::Color = color!(0xE3, 0xED, 0xDA);
    /// Artist line.
    pub const ARTIST: iced::Color = color!(0xA9, 0xB6, 0x9F);
    /// "UP NEXT" label.
    pub const MUTED: iced::Color = color!(0x9A, 0xA9, 0x91);
    /// Next dance name.
    pub const NEXT: iced::Color = color!(0xC5, 0xD0, 0xBC);
    /// Cover art hairline.
    pub const COVER_BORDER: iced::Color = color!(0x73, 0x74, 0x5B);
}

/// Background gradient: `radial-gradient(#1b2119, #0c100d 80%)` approximated
/// as a top-to-bottom linear gradient (iced has no radial gradients).
fn background_style(_theme: &Theme) -> container::Style {
    container::Style::default().background(Gradient::Linear(
        iced::gradient::Linear::new(std::f32::consts::PI)
            .add_stop(0.0, iced::color!(0x1B, 0x21, 0x19))
            .add_stop(1.0, iced::color!(0x0C, 0x10, 0x0D)),
    ))
}

pub struct SongWindow {
    pub id: window::Id,
    pub closed: bool,
    pub size: iced::Size,

    pub enable_image: bool,
    pub enable_next_dance: bool,
    pub scale: f32,
}

impl Window for SongWindow {
    fn new(id: window::Id) -> Self {
        Self {
            id,
            closed: false,
            size: iced::Size::default(),

            enable_image: true,
            enable_next_dance: true,
            scale: 1.0,
        }
    }

    fn on_resize(&mut self, size: iced::Size) {
        self.size = size;
    }

    fn on_close(&mut self) {
        self.closed = true;
    }

    fn is_closed(&self) -> bool {
        self.closed
    }
}

impl SongWindow {
    pub fn view<'a>(&self, state: &'a DanceInterpreter) -> Element<'a, Message> {
        let Some(displayable_data) = state.data_provider.get_current_displayable_data() else {
            // Blank display: pure black, nothing else.
            return container(horizontal())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Style::default().background(iced::Color::BLACK))
                .into();
        };

        let h = self.size.height;
        let w = self.size.width;
        let scale = self.scale;

        // Type scale (docs/src/audience.css, slightly bumped for legibility).
        let dance_size = h / 8.0 * scale;
        let title_size = h / 20.0 * scale;
        let artist_size = h / 28.0 * scale;
        let next_label_size = h / 34.0 * scale;
        let next_dance_size = h / 24.0 * scale;

        // Proportional paddings / gaps.
        let pad_top = h * 0.08;
        let pad_side = w * 0.07;
        let pad_bottom = h * 0.16;
        let song_gap = h / 33.3; // 3vh
        let song_margin = h / 16.7; // 6vh
        let cover_size = h / 9.1; // 11vh
        let next_pad_bottom = h * 0.06;

        // Huge dance headline.
        let dance_text = Text::new(displayable_data.headline)
            .size(dance_size)
            .font(crate::ui::theme::head_font(Weight::Medium))
            .line_height(LineHeight::Relative(1.15))
            .color(colors::TEXT)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .shaping(Shaping::Advanced);

        // Song block: cover art beside title/artist. Statics have no sublines.
        let has_song_info = !displayable_data.subline_upper.is_empty()
            || !displayable_data.subline_lower.is_empty();

        let mut center = col![dance_text]
            .spacing(song_margin)
            .align_x(Horizontal::Center);

        if has_song_info {
            let text_block = col![
                Text::new(displayable_data.subline_upper)
                    .size(title_size)
                    .font(crate::ui::theme::head_font(Weight::Medium))
                    .color(colors::TEXT)
                    .shaping(Shaping::Advanced),
                Text::new(displayable_data.subline_lower)
                    .size(artist_size)
                    .font(crate::ui::theme::body_font(Weight::Normal))
                    .color(colors::ARTIST)
                    .shaping(Shaping::Advanced),
            ]
            .spacing(h / 60.0)
            .align_x(Horizontal::Left);

            let song_block = if self.enable_image {
                if let Some(image_handle) = displayable_data.subline_image.as_ref() {
                    let cover = container(
                        image(image_handle)
                            .content_fit(ContentFit::Cover)
                            .width(Length::Fill)
                            .height(Length::Fill),
                    )
                    .width(Length::Fixed(cover_size))
                    .height(Length::Fixed(cover_size))
                    .clip(true)
                    .style(|_| {
                        container::Style::default().border(iced::Border {
                            color: colors::COVER_BORDER,
                            width: 1.0,
                            radius: 3.0.into(),
                        })
                    });

                    row![cover, text_block]
                        .spacing(song_gap)
                        .align_y(Vertical::Center)
                } else {
                    row![text_block]
                }
            } else {
                row![text_block]
            };

            center = center.push(song_block);
        }

        let center_overlay = container(center)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(crate::ui::theme::pad4(
                pad_top, pad_side, pad_bottom, pad_side,
            ))
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center);

        let mut layers = stack![center_overlay];

        // "UP NEXT" hint in the bottom-right corner.
        if self.enable_next_dance
            && let Some(next_displayable_data) = state.data_provider.get_next_displayable_data()
        {
            let next_block = col![
                Text::new("UP NEXT")
                    .size(next_label_size)
                    .font(crate::ui::theme::head_font(Weight::Semibold))
                    .color(colors::MUTED),
                Text::new(next_displayable_data.headline)
                    .size(next_dance_size)
                    .font(crate::ui::theme::head_font(Weight::Medium))
                    .color(colors::NEXT)
                    .line_height(LineHeight::Relative(1.3))
                    .shaping(Shaping::Advanced),
            ]
            .spacing(h / 60.0)
            .align_x(Horizontal::Right);

            layers = layers.push(
                container(row![horizontal(), next_block].width(Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(crate::ui::theme::pad4(0.0, pad_side, next_pad_bottom, 0.0))
                    .align_y(Vertical::Bottom),
            );
        }

        container(layers)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(background_style)
            .into()
    }
}
