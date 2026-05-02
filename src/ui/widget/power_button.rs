use iced::widget::canvas;
use iced::widget::canvas::{Frame, LineCap, Path, Stroke};
use iced::{Point, Radians, Theme};
use std::f32::consts::PI;

pub fn draw(theme: &Theme, frame: &mut Frame, enabled: bool) {
    let size = frame.size();
    let cx = size.width / 2.0;
    let cy = size.height / 2.0;
    let center = Point::new(cx, cy);

    // Scale everything relative to the smaller dimension
    let dim = size.width.min(size.height);
    let bg_r = dim * 0.46; // background circle radius
    let arc_r = dim * 0.30; // power arc radius
    let stroke_w = dim * 0.07;

    // Colors
    let (icon_color, bg_color) = if enabled {
        (
            theme.extended_palette().primary.base.color,
            theme.extended_palette().background.weaker.color,
        )
    } else {
        (
            theme.extended_palette().secondary.base.color,
            theme.extended_palette().background.weaker.color,
        )
    };

    let bg_path = Path::new(|b| b.circle(center, bg_r));
    frame.fill(
        &bg_path,
        canvas::Fill {
            style: canvas::Style::Solid(bg_color),
            ..canvas::Fill::default()
        },
    );

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

    frame.stroke(
        &arc,
        Stroke::default()
            .with_color(icon_color)
            .with_width(stroke_w)
            .with_line_cap(LineCap::Butt),
    );

    // Power line (vertical, through the gap)
    // Runs from ~20% inside the radius up to the circumference.
    let line = Path::new(|b| {
        b.move_to(Point::new(cx, cy - arc_r * 0.18));
        b.line_to(Point::new(cx, cy - arc_r));
    });

    frame.stroke(
        &line,
        Stroke::default()
            .with_color(icon_color)
            .with_width(stroke_w)
            .with_line_cap(LineCap::Butt),
    );
}
