//! Canvas-drawn controls (power, restart, ripple toggle).
//!
//! These widgets paint their glyphs with `iced::widget::canvas` instead of
//! icon fonts so they can animate (ripple press feedback) beyond what a
//! static glyph offers.

pub mod canvas_toggle;
pub mod power_button;
pub mod restart_button;

use crate::ui::theme;
use iced::widget::canvas::{self, Frame, Path};
use iced::{Color, Point, Theme};

/// Background circle radius, as a fraction of the canvas' smaller dimension.
pub const BG_RADIUS: f32 = 0.46;
/// Icon arc radius, as a fraction of the canvas' smaller dimension.
pub const ARC_RADIUS: f32 = 0.30;

/// Geometry and colors the icon on top of the shared base needs.
pub struct IconBase {
    /// Center x.
    pub cx: f32,
    /// Center y.
    pub cy: f32,
    /// Center point.
    pub center: Point,
    /// Smaller canvas dimension (for scaling other icon parts).
    pub dim: f32,
    /// Icon arc radius.
    pub arc_r: f32,
    /// Icon stroke width.
    pub stroke_w: f32,
    /// Icon color (accent while enabled, muted otherwise).
    pub icon_color: Color,
}

/// Fills the shared background circle and returns the geometry and colors
/// for the icon drawn on top of it.
pub fn draw_icon_base(theme: &Theme, frame: &mut Frame, enabled: bool) -> IconBase {
    let size = frame.size();
    let dim = size.width.min(size.height);
    let cx = size.width / 2.0;
    let cy = size.height / 2.0;
    let center = Point::new(cx, cy);
    let arc_r = dim * ARC_RADIUS;
    let stroke_w = dim * 0.07;

    let tk = theme::Tokens::of(theme);
    let icon_color = if enabled { tk.accent_soft } else { tk.muted };

    let bg = Path::new(|b| b.circle(center, dim * BG_RADIUS));
    frame.fill(
        &bg,
        canvas::Fill {
            style: canvas::Style::Solid(tk.panel_deep),
            ..canvas::Fill::default()
        },
    );

    IconBase {
        cx,
        cy,
        center,
        dim,
        arc_r,
        stroke_w,
        icon_color,
    }
}

/// Strokes a shape with the icon color and width, butt-capped. `width_scale`
/// scales the shared icon stroke width (e.g. 0.75 for the restart arrowhead).
pub fn stroke_icon(frame: &mut Frame, path: &Path, base: &IconBase, width_scale: f32) {
    frame.stroke(
        path,
        canvas::Stroke::default()
            .with_color(base.icon_color)
            .with_width(base.stroke_w * width_scale)
            .with_line_cap(canvas::LineCap::Butt),
    );
}
