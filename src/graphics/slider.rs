use ggez::graphics::{Color, DrawMode, DrawParam, Drawable};
use libosu::{prelude::{HitObject, SliderInfo}};

use crate::BeatmapData;

use super::circle::draw_circle;

pub fn draw_slider(
    ctx: &mut ggez::Context,
    map_data: &BeatmapData,
    current_ms: i32,
    object: &HitObject,
    slider: &SliderInfo,
) {
    draw_circle(ctx, map_data, current_ms, object);

    let points = libosu::spline::Spline::from_control(
        slider.kind, 
        &slider.control_points, 
        Some(slider.pixel_length)
    ).spline_points;

    let end_point = points[points.len() - 1];
    let second_to_last_end_point = points[points.len() - 2];
    let end_direction = (end_point.y - second_to_last_end_point.y)
        .atan2(end_point.x - second_to_last_end_point.x)
        + std::f64::consts::FRAC_PI_2;
    let end_offset = libosu::math::Point {
        x: end_direction.cos() * map_data.cs_osupixels as f64,
        y: end_direction.sin() * map_data.cs_osupixels as f64,
    };

    ggez::graphics::MeshBuilder::new()
        .line(
            &points
                .windows(2)
                .map(|p| {
                    let direction =
                        (p[1].y - p[0].y).atan2(p[1].x - p[0].x) + std::f64::consts::FRAC_PI_2;
                    let offset = libosu::math::Point {
                        x: direction.cos() * map_data.cs_osupixels as f64,
                        y: direction.sin() * map_data.cs_osupixels as f64,
                    };
                    p[0] + offset
                })
                .chain(std::iter::once(end_point + end_offset))
                .map(|p| {
                    glam::vec2(p.x as f32, p.y as f32)
                })
                .collect::<Vec<_>>(),
            1.0,
            Color {
                r: 1.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
        )
        .unwrap()
        .line(
            &points
                .windows(2)
                .map(|p| {
                    let direction =
                        (p[1].y - p[0].y).atan2(p[1].x - p[0].x) + std::f64::consts::FRAC_PI_2;
                    let offset = libosu::math::Point {
                        x: direction.cos() * map_data.cs_osupixels as f64,
                        y: direction.sin() * map_data.cs_osupixels as f64,
                    };
                    p[0] - offset
                })
                .chain(std::iter::once(end_point - end_offset))
                .map(|p| {
                    glam::vec2(p.x as f32, p.y as f32)
                })
                .collect::<Vec<_>>(),
            1.0,
            Color {
                r: 1.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
        )
        .unwrap()
        .circle(
            DrawMode::stroke(1.0),
            glam::vec2(end_point.x as f32, end_point.y as f32),
            map_data.cs_osupixels,
            1.0,
            Color {
                r: 0.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
        )
        .unwrap()
        .build(ctx)
        .unwrap()
        .draw(ctx, DrawParam::new())
        .unwrap();
}