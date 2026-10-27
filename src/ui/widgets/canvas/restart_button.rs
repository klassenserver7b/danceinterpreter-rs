use crate::ui::widgets::canvas::{IconBase, draw_icon_base, stroke_icon};
use iced::mouse::Cursor;
use iced::widget::canvas::{self, Frame, Path};
use iced::{Point, Radians, Rectangle, Theme};
use std::f32::consts::PI;

pub fn draw(theme: &Theme, frame: &mut Frame, _bounds: Rectangle, _cursor: Cursor, enabled: bool) {
    let base @ IconBase {
        cx,
        cy,
        center,
        dim,
        arc_r,
        ..
    } = draw_icon_base(theme, frame, enabled);

    // Restart arc
    // 300° clockwise arc, 60° gap left from the top.
    let gap = 30.0_f32.to_radians();
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
    let end_x = cx - gap * 16.0;
    let end_y = cy - arc_r;

    // Two wings ±45° from the back direction
    let (s, c) = 45.0_f32.to_radians().sin_cos();
    let w1 = (-c, -s); // rotate +45°
    let w2 = (-c, s); // rotate -45°
    let arrow_size = dim * 0.13;

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
