use crate::ui::widgets::canvas::{IconBase, draw_icon_base, stroke_icon};
use iced::widget::canvas::{self, Frame, Path};
use iced::{Point, Radians, Theme};
use std::f32::consts::PI;

pub fn draw(theme: &Theme, frame: &mut Frame, enabled: bool) {
    let base @ IconBase { center, arc_r, .. } = draw_icon_base(theme, frame, enabled);

    // Power arc
    // Gap of 70° centered at the top (-π/2 in screen coords).
    // Arc drawn from right edge of gap → clockwise around → left edge of gap.
    let gap_half = 35.0_f32.to_radians(); // half the gap angle
    let top = -PI / 2.0; // top of circle (-90°)
    let start = top + gap_half; // right side of gap  ≈ -55°
    let end = top - gap_half + 2.0 * PI; // left side of gap   ≈ 235° (going all the way around)

    let arc = Path::new(|b| {
        b.arc(canvas::path::Arc {
            center,
            radius: arc_r,
            start_angle: Radians(start),
            end_angle: Radians(end),
        });
    });
    stroke_icon(frame, &arc, &base, 1.0);

    // Power line (vertical, through the gap)
    // Runs from ~20% inside the radius up to the circumference.
    let line = Path::new(|b| {
        b.move_to(Point::new(center.x, center.y - arc_r * 0.18));
        b.line_to(Point::new(center.x, center.y - arc_r));
    });
    stroke_icon(frame, &line, &base, 1.0);
}
