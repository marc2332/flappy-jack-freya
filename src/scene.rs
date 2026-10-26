use freya::{
    engine::prelude::{Canvas, ClipOp, Paint, PaintStyle, PathBuilder, SkRect},
    prelude::*,
};

use crate::game::{self, Game};

pub const NIGHT: Color = Color::from_rgb(8, 7, 15);
pub const ORANGE: Color = Color::from_rgb(246, 153, 67);
pub const CREAM: Color = Color::from_rgb(246, 220, 164);
pub const MUTED: Color = Color::from_rgb(163, 148, 177);

fn paint(color: Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    paint
}

fn polygon(canvas: &Canvas, points: &[(f32, f32)], color: Color) {
    let Some((first, rest)) = points.split_first() else {
        return;
    };
    let mut path = PathBuilder::new();
    path.move_to(*first);
    for point in rest {
        path.line_to(*point);
    }
    path.close();
    canvas.draw_path(&path.detach(), &paint(color));
}

pub fn backdrop(canvas: &Canvas, width: f32, height: f32, seconds: f32) {
    canvas.draw_rect(SkRect::from_wh(width, height), &paint(NIGHT));
    for index in 0..55 {
        let horizontal = ((index * 137 + 31) % 997) as f32 / 997.0;
        let vertical = ((index * 71 + 19) % 401) as f32 / 601.0;
        let alpha = (60.0 + 35.0 * (seconds * 0.7 + index as f32).sin()) as u8;
        canvas.draw_circle(
            (horizontal * width, vertical * height),
            if index % 4 == 0 { 1.3 } else { 0.7 },
            &paint(Color::from_argb(alpha, 196, 181, 217)),
        );
    }
    let moon_x = width * 0.84;
    let moon_y = height * 0.18;
    let radius = (height * 0.065).min(42.0);
    for ring in (1..=5).rev() {
        canvas.draw_circle(
            (moon_x, moon_y),
            radius + ring as f32 * 10.0,
            &paint(Color::from_argb(4, 214, 200, 242)),
        );
    }
    canvas.draw_circle(
        (moon_x, moon_y),
        radius,
        &paint(Color::from_rgb(194, 184, 211)),
    );
    canvas.draw_circle(
        (moon_x + radius * 0.38, moon_y - radius * 0.18),
        radius * 0.88,
        &paint(NIGHT),
    );
    for index in 0..3 {
        let horizontal = width * (0.12 + index as f32 * 0.35);
        let vertical =
            height * (0.43 + index as f32 * 0.09) + (seconds + index as f32 * 2.0).sin() * 9.0;
        ghost(canvas, horizontal, vertical, 0.7 + index as f32 * 0.1);
    }
}

fn ghost(canvas: &Canvas, horizontal: f32, vertical: f32, scale: f32) {
    let saved = canvas.save();
    canvas.translate((horizontal, vertical));
    canvas.scale((scale, scale));
    let mut path = PathBuilder::new();
    path.move_to((-13.0, 16.0));
    path.line_to((-13.0, -4.0));
    path.cubic_to((-13.0, -24.0), (13.0, -24.0), (13.0, -4.0));
    path.line_to((13.0, 16.0));
    path.line_to((6.0, 11.0));
    path.line_to((0.0, 17.0));
    path.line_to((-6.0, 11.0));
    path.close();
    canvas.draw_path(&path.detach(), &paint(Color::from_argb(27, 180, 161, 207)));
    canvas.draw_oval(SkRect::from_xywh(-6.0, -6.0, 3.0, 6.0), &paint(NIGHT));
    canvas.draw_oval(SkRect::from_xywh(3.0, -6.0, 3.0, 6.0), &paint(NIGHT));
    canvas.restore_to_count(saved);
}

pub fn flight(game: Game, seconds: f32) -> RenderCallback {
    RenderCallback::new(move |context| {
        let RenderTarget::Background { canvas, .. } = &context.target else {
            return;
        };
        let width = context.size.width;
        let height = context.size.height;
        backdrop(canvas, width, height, seconds);
        let scale = (width / 1000.0).min(height / 480.0);
        if scale <= 0.0 {
            return;
        }
        let saved = canvas.save();
        canvas.translate((
            (width - 1000.0 * scale) / 2.0,
            (height - 480.0 * scale) / 2.0,
        ));
        canvas.scale((scale, scale));
        canvas.clip_rect(SkRect::from_wh(1000.0, 480.0), ClipOp::Intersect, true);
        cemetery(canvas, game.time);
        for gate in &game.gates {
            gate_piece(
                canvas,
                gate.x * 10.0,
                0.0,
                (gate.gap_y - gate.gap_half) * 20.0,
                false,
                gate.passed,
            );
            let bottom = (gate.gap_y + gate.gap_half) * 20.0;
            gate_piece(
                canvas,
                gate.x * 10.0,
                bottom,
                480.0 - bottom,
                true,
                gate.passed,
            );
        }
        for candy in &game.candies {
            let horizontal = candy.x * 10.0;
            let vertical = candy.y * 20.0;
            canvas.draw_circle(
                (horizontal, vertical),
                16.0,
                &paint(Color::from_argb(22, 255, 205, 100)),
            );
            polygon(
                canvas,
                &[
                    (horizontal - 12.0, vertical - 5.0),
                    (horizontal - 20.0, vertical - 10.0),
                    (horizontal - 20.0, vertical + 10.0),
                    (horizontal - 12.0, vertical + 5.0),
                ],
                ORANGE,
            );
            polygon(
                canvas,
                &[
                    (horizontal + 12.0, vertical - 5.0),
                    (horizontal + 20.0, vertical - 10.0),
                    (horizontal + 20.0, vertical + 10.0),
                    (horizontal + 12.0, vertical + 5.0),
                ],
                ORANGE,
            );
            polygon(
                canvas,
                &[
                    (horizontal, vertical - 12.0),
                    (horizontal + 12.0, vertical),
                    (horizontal, vertical + 12.0),
                    (horizontal - 12.0, vertical),
                ],
                CREAM,
            );
        }
        if game.alive {
            little_jack(canvas, &game);
        } else {
            for index in 0..20 {
                let angle = index as f32 * 2.4;
                let time = game.death_time;
                let horizontal = game::PLAYER_X * 10.0
                    + angle.cos() * time * (100.0 + (index % 3) as f32 * 40.0);
                let vertical = game.y * 20.0 + angle.sin() * time * 180.0 + 360.0 * time * time;
                canvas.draw_oval(
                    SkRect::from_xywh(horizontal, vertical, 4.0, 7.0),
                    &paint(if index % 2 == 0 { CREAM } else { ORANGE }),
                );
            }
        }
        canvas.draw_rect(
            SkRect::from_xywh(0.0, 477.0, 1000.0, 3.0),
            &paint(Color::from_rgb(80, 58, 91)),
        );
        canvas.restore_to_count(saved);
    })
}

fn cemetery(canvas: &Canvas, seconds: f32) {
    for layer in 0..2 {
        let speed = 9.0 + layer as f32 * 10.0;
        let color = if layer == 0 {
            Color::from_rgb(21, 17, 32)
        } else {
            Color::from_rgb(32, 24, 43)
        };
        for index in -1..15 {
            let horizontal = index as f32 * 90.0 - (seconds * speed).rem_euclid(90.0);
            let base = 480.0;
            let height = 26.0 + ((index + 20) * 17 % 35) as f32 + layer as f32 * 8.0;
            canvas.draw_round_rect(
                SkRect::from_xywh(horizontal, base - height, 26.0, height + 10.0),
                12.0,
                12.0,
                &paint(color),
            );
            canvas.draw_rect(
                SkRect::from_xywh(horizontal + 46.0, base - height - 20.0, 6.0, height + 20.0),
                &paint(color),
            );
            canvas.draw_rect(
                SkRect::from_xywh(horizontal + 35.0, base - height - 6.0, 28.0, 5.0),
                &paint(color),
            );
        }
    }
}

fn gate_piece(
    canvas: &Canvas,
    horizontal: f32,
    top: f32,
    height: f32,
    points_up: bool,
    passed: bool,
) {
    let width = game::GATE_WIDTH * 10.0;
    let color = if passed {
        Color::from_rgb(67, 73, 75)
    } else {
        Color::from_rgb(111, 91, 137)
    };
    let dark = Color::from_rgb(44, 34, 58);
    canvas.draw_rect(
        SkRect::from_xywh(horizontal, top, width, height),
        &paint(Color::from_argb(100, 32, 24, 43)),
    );
    for offset in [0.0, 15.0, 30.0, 45.0] {
        canvas.draw_rect(
            SkRect::from_xywh(horizontal + offset, top, 5.0, height),
            &paint(color),
        );
        let edge = if points_up { top } else { top + height };
        let inner = if points_up { edge + 11.0 } else { edge - 11.0 };
        polygon(
            canvas,
            &[
                (horizontal + offset - 3.0, inner),
                (horizontal + offset + 2.5, edge),
                (horizontal + offset + 8.0, inner),
            ],
            color,
        );
    }
    let mut vertical = top + 22.0;
    while vertical < top + height - 12.0 {
        canvas.draw_rect(
            SkRect::from_xywh(horizontal, vertical, width, 7.0),
            &paint(dark),
        );
        canvas.draw_rect(
            SkRect::from_xywh(horizontal, vertical, width, 2.0),
            &paint(color),
        );
        vertical += 50.0;
    }
}

fn little_jack(canvas: &Canvas, game: &Game) {
    let saved = canvas.save();
    canvas.translate((game::PLAYER_X * 10.0, game.y * 20.0));
    canvas.rotate((game.velocity * 1.3).clamp(-16.0, 23.0), None);
    canvas.draw_oval(
        SkRect::from_xywh(-19.0, -15.0, 38.0, 30.0),
        &paint(Color::from_rgb(180, 67, 18)),
    );
    for (horizontal, color) in [
        (-10.0, Color::from_rgb(223, 99, 23)),
        (-5.0, Color::from_rgb(246, 129, 29)),
        (2.0, Color::from_rgb(227, 100, 20)),
    ] {
        canvas.draw_oval(
            SkRect::from_xywh(horizontal - 7.0, -15.0, 14.0, 30.0),
            &paint(color),
        );
    }
    polygon(
        canvas,
        &[(-3.0, -14.0), (-2.0, -23.0), (4.0, -25.0), (3.0, -14.0)],
        Color::from_rgb(102, 123, 48),
    );
    let worried = game.danger();
    let eye_top = -6.0 + worried * 3.0;
    polygon(
        canvas,
        &[(-12.0, eye_top), (-3.0, -4.0), (-6.0, 2.0)],
        CREAM,
    );
    polygon(canvas, &[(3.0, -4.0), (12.0, eye_top), (7.0, 2.0)], CREAM);
    polygon(
        canvas,
        &[
            (-10.0, 5.0),
            (-3.0, 8.0),
            (0.0, 5.0),
            (3.0, 8.0),
            (11.0, 4.0),
            (6.0, 11.0),
            (-5.0, 11.0),
        ],
        CREAM,
    );
    let wing = if game.flap_time < 0.12 { -10.0 } else { 5.0 };
    polygon(
        canvas,
        &[
            (-17.0, -2.0),
            (-35.0, wing - 4.0),
            (-29.0, wing + 7.0),
            (-18.0, 8.0),
        ],
        Color::from_rgb(109, 89, 135),
    );
    let mut outline = paint(Color::from_rgb(120, 43, 22));
    outline.set_style(PaintStyle::Stroke);
    outline.set_stroke_width(1.0);
    canvas.draw_oval(SkRect::from_xywh(-19.0, -15.0, 38.0, 30.0), &outline);
    canvas.restore_to_count(saved);
}
