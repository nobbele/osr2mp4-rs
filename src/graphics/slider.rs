use ggez::graphics::{Color, DrawMode, DrawParam, Drawable};
use libosu::prelude::{HitObject, SliderInfo};

use crate::BeatmapData;

use super::circle::draw_circle;

pub fn draw_slider(
    ctx: &mut ggez::Context,
    map_data: &BeatmapData,
    current_ms: i32,
    object: &HitObject,
    slider: &SliderInfo,
    combo_color: Color,
) {
    let mut points = Vec::with_capacity(slider.control_points.len() + 1);
    points.push(object.pos);
    points.extend(slider.control_points.iter());
    let points =
        libosu::spline::Spline::from_control(slider.kind, &points, Some(slider.pixel_length))
            .spline_points;

    let start_point = points[0];
    let end_point = points[points.len() - 1];
    let second_to_last_end_point = points[points.len() - 2];
    let end_direction = (end_point.y - second_to_last_end_point.y)
        .atan2(end_point.x - second_to_last_end_point.x)
        + std::f64::consts::FRAC_PI_2;
    let end_offset = libosu::math::Point {
        x: end_direction.cos() * map_data.cs_osupixels as f64,
        y: end_direction.sin() * map_data.cs_osupixels as f64,
    };

    let body_points = points
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
        .map(|p| glam::vec2(p.x as f32, p.y as f32))
        .chain(
            points
                .windows(2)
                .map(|p| {
                    let direction = (p[1].y - p[0].y).atan2(p[1].x - p[0].x)
                        + std::f64::consts::FRAC_PI_2;
                    let offset = libosu::math::Point {
                        x: direction.cos() * map_data.cs_osupixels as f64,
                        y: direction.sin() * map_data.cs_osupixels as f64,
                    };
                    p[0] - offset
                })
                .chain(std::iter::once(end_point - end_offset))
                .map(|p| glam::vec2(p.x as f32, p.y as f32))
                .rev(),
        )
        .collect::<Vec<_>>();

    let fill_color = Color {
        r: 3.0 / 255.0,
        g: 3.0 / 255.0,
        b: 12.0 / 255.0,
        a: (3.0 + 3.0 + 12.0) / 255.0,
    };
    let stroke_color = Color {
        r: 120.0 / 255.0,
        g: 120.0 / 255.0,
        b: 120.0 / 255.0,
        a: 1.0,
    };

    ggez::graphics::MeshBuilder::new()
        .circle(
            DrawMode::fill(),
            glam::vec2(start_point.x as f32, start_point.y as f32),
            map_data.cs_osupixels,
            1.0,
            fill_color,
        )
        .unwrap()
        .circle(
            DrawMode::stroke(5.0),
            glam::vec2(start_point.x as f32, start_point.y as f32),
            map_data.cs_osupixels,
            1.0,
            stroke_color,
        )
        .unwrap()
        .polyline(
            DrawMode::fill(),
            &body_points,
            fill_color,
        )
        .unwrap()
        .polyline(
            DrawMode::stroke(5.0),
            &body_points,
            stroke_color,
        )
        .unwrap()
        .circle(
            DrawMode::fill(),
            glam::vec2(end_point.x as f32, end_point.y as f32),
            map_data.cs_osupixels,
            1.0,
            fill_color,
        )
        .unwrap()
        .circle(
            DrawMode::stroke(5.0),
            glam::vec2(end_point.x as f32, end_point.y as f32),
            map_data.cs_osupixels,
            1.0,
            stroke_color,
        )
        .unwrap()
        .build(ctx)
        .unwrap()
        .draw(ctx, DrawParam::new())
        .unwrap();

    draw_circle(ctx, map_data, current_ms, object, combo_color);
}
