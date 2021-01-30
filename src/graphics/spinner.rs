use ggez::{
    graphics::{drawable_size, Color, DrawMode, DrawParam, Drawable},
    mint,
};
use libosu::prelude::HitObject;

use crate::BeatmapData;

pub fn draw_spinner(
    ctx: &mut ggez::Context,
    _map_data: &BeatmapData,
    _current_ms: i32,
    _object: &HitObject,
) {
    ggez::graphics::Mesh::new_circle(
        ctx,
        DrawMode::stroke(1.0),
        mint::Point2 { x: 0.0, y: 0.0 },
        10.0,
        1.0,
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        },
    )
    .unwrap()
    .draw(
        ctx,
        DrawParam::new().dest(mint::Point2 {
            x: drawable_size(ctx).0 / 2.0,
            y: drawable_size(ctx).1 / 2.0,
        }),
    )
    .unwrap();
}
