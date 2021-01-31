use ggez::graphics::{Color, DrawParam, Drawable, Text};
use glam::vec2;
use libosu::prelude::HitObject;

use crate::BeatmapData;

pub fn draw_circle(
    ctx: &mut ggez::Context,
    map_data: &BeatmapData,
    current_ms: i32,
    object: &HitObject,
    combo_color: Color,
    combo_index: u8,
) {
    let hitcircle =
        ggez::graphics::Image::new(ctx, "/Skins/Varvalian 2019-06-25/hitcircle.png").unwrap();
    hitcircle
        .draw(
            ctx,
            DrawParam::new()
                .dest(vec2(object.pos.x as f32, object.pos.y as f32))
                .offset(vec2(0.5, 0.5))
                .scale(
                    vec2(map_data.cs_osupixels * 2.0, map_data.cs_osupixels * 2.0)
                        / vec2(hitcircle.dimensions().w, hitcircle.dimensions().h),
                ),
        )
        .unwrap();

    let hitcircleoverlay =
        ggez::graphics::Image::new(ctx, "/Skins/Varvalian 2019-06-25/hitcircleoverlay.png")
            .unwrap();
    hitcircleoverlay
        .draw(
            ctx,
            DrawParam::new()
                .dest(vec2(object.pos.x as f32, object.pos.y as f32))
                .offset(vec2(0.5, 0.5))
                .scale(
                    vec2(map_data.cs_osupixels * 2.0, map_data.cs_osupixels * 2.0)
                        / vec2(
                            hitcircleoverlay.dimensions().w,
                            hitcircleoverlay.dimensions().h,
                        ),
                )
                .color(combo_color),
        )
        .unwrap();

    let approach_circle_size = (object.start_time.0 - current_ms) as f32 / map_data.ar_ms as f32;

    let radius = map_data.cs_osupixels * (1.0 + approach_circle_size);

    let approachcircle =
        ggez::graphics::Image::new(ctx, "/Skins/Varvalian 2019-06-25/approachcircle.png").unwrap();
    approachcircle
        .draw(
            ctx,
            DrawParam::new()
                .dest(vec2(object.pos.x as f32, object.pos.y as f32))
                .offset(vec2(0.5, 0.5))
                .scale(
                    vec2(radius * 2.0, radius * 2.0)
                        / vec2(approachcircle.dimensions().w, approachcircle.dimensions().h),
                )
                .color(combo_color),
        )
        .unwrap();

    let combo_number = if combo_index < 10 {
        ggez::graphics::Image::new(
            ctx,
            format!("/Skins/Varvalian 2019-06-25/combo-{}.png", combo_index),
        )
        .unwrap()
    } else {
        ggez::graphics::Image::new(ctx, "/Skins/Varvalian 2019-06-25/ranking-B.png").unwrap()
    };
    combo_number
        .draw(
            ctx,
            DrawParam::new()
                .dest(vec2(object.pos.x as f32, object.pos.y as f32))
                .offset(vec2(0.5, 0.5))
                .scale(
                    vec2(map_data.cs_osupixels, map_data.cs_osupixels)
                        / vec2(combo_number.dimensions().w, combo_number.dimensions().h),
                ),
        )
        .unwrap();
}
