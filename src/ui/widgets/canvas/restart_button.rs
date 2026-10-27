use crate::ui::widgets::canvas::{ARROW_FRAC, IconBase, draw_icon_base, stroke_icon};
use iced::widget::canvas::{self, Frame, Path};
use iced::{Point, Radians, Theme};
use std::f32::consts::PI;

/// The restart arc's top gap, in degrees.
const GAP_DEG: f32 = 30.0;

pub fn draw(theme: &Theme, frame: &mut Frame, enabled: bool) {
    let base @ IconBase {
        center, dim, arc_r, ..
    } = draw_icon_base(theme, frame, enabled);

    // Restart arc
    // 300° clockwise arc, 60° gap left from the top.
    let gap = GAP_DEG.to_radians();
    let bot = PI / 2.0;
    let top = -bot;
    let arc_start = top - gap / 2.0;
    let arc_cut_start = bot - gap / 4.0;
    let arc_cut_end = bot + gap / 4.0;
    let arc_end = top - 1.25 * gap + 2.0 * PI; // 300°  (clockwise, long way round)

    let arc = Path::new(|b| {
        b.arc(canvas::path::Arc {
            center,
            radius: arc_r,
            start_angle: Radians(arc_start),
            end_angle: Radians(arc_cut_start),
        });
        b.arc(canvas::path::Arc {
            center,
            radius: arc_r,
            start_angle: Radians(arc_cut_end),
            end_angle: Radians(arc_end),
        });
    });
    stroke_icon(frame, &arc, &base, 1.0);

    // The 16.0 factor is arc/arrow geometry (rad→px nudge), not a font size.
    let end_x = center.x - gap * 16.0;
    let end_y = center.y - arc_r;

    // Two wings ±45° from the back direction
    let (s, c) = 45.0_f32.to_radians().sin_cos();
    let w1 = (-c, -s); // rotate +45°
    let w2 = (-c, s); // rotate -45°
    let arrow_size = dim * ARROW_FRAC;

    let tip = Point::new(end_x, end_y);
    let p1 = Point::new(end_x - w1.0 * arrow_size, end_y + w1.1 * arrow_size);
    let p2 = Point::new(end_x - w2.0 * arrow_size, end_y + w2.1 * arrow_size);

    let arrowhead = Path::new(|b| {
        b.move_to(p1);
        b.line_to(tip);
        b.line_to(p2);
    });
    stroke_icon(frame, &arrowhead, &base, 0.75);
}
