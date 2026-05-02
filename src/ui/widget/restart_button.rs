use iced::widget::canvas;
use iced::widget::canvas::{Frame, LineCap, Path, Stroke};
use iced::{Point, Radians, Theme};
use std::f32::consts::PI;

pub fn draw(theme: &Theme, frame: &mut Frame, enabled: bool) {
    let size = frame.size();
    let cx = size.width / 2.0;
    let cy = size.height / 2.0;
    let center = Point::new(cx, cy);
    let dim = size.width.min(size.height);
    let bg_r = dim * 0.46;
    let arc_r = dim * 0.30;
    let stroke_w = dim * 0.07;
    let arrow_size = dim * 0.13;

    // secondary color when enabled; muted background-strong when disabled
    let (icon_color, bg_color) = if enabled {
        (
            theme.extended_palette().secondary.base.color,
            theme.extended_palette().background.weaker.color,
        )
    } else {
        (
            theme.extended_palette().background.strongest.color,
            theme.extended_palette().background.weaker.color,
        )
    };

    // Background circle
    let bg_path = Path::new(|b| b.circle(center, bg_r));
    frame.fill(
        &bg_path,
        canvas::Fill {
            style: canvas::Style::Solid(bg_color),
            ..canvas::Fill::default()
        },
    );

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
    frame.stroke(
        &arc,
        Stroke::default()
            .with_color(icon_color)
            .with_width(stroke_w)
            .with_line_cap(LineCap::Butt),
    );

    let end_x = cx - gap * 16.0;
    let end_y = cy - arc_r;

    // Two wings ±35° from the back direction
    let (s, c) = 45.0_f32.to_radians().sin_cos();
    let w1 = (-c, -s); // rotate +35°
    let w2 = (-c, s); // rotate -35°

    let tip = Point::new(end_x, end_y);
    let p1 = Point::new(end_x - w1.0 * arrow_size, end_y + w1.1 * arrow_size);
    let p2 = Point::new(end_x - w2.0 * arrow_size, end_y + w2.1 * arrow_size);

    let arrowhead = Path::new(|b| {
        b.move_to(p1);
        b.line_to(tip);
        b.line_to(p2);
    });
    frame.stroke(
        &arrowhead,
        Stroke::default()
            .with_color(icon_color)
            .with_width(stroke_w * 0.75)
            .with_line_cap(LineCap::Butt),
    );
}
