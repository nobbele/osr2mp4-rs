use encoder::Encoder;
use ggez::{
    conf::{WindowMode, WindowSetup},
    graphics::{drawable_size, Color, DrawMode, DrawParam, Drawable, FillOptions, Rect},
    mint,
};
use libosu::HitObject;
use std::{io::BufReader, path::Path, println};

mod curves;
mod encoder;

const DRAW_TO_VIDEO: bool = false;

pub struct Game {
    pub cursor_pos: (f32, f32),
}

fn ar_to_ms(ar: f32) -> i32 {
    let base = if ar >= 5.0 {
        450.0 + (10.0 - ar) * 150.0
    } else {
        1200.0 + (5.0 - ar) * 120.0
    };
    base as i32
}

#[test]
fn test_ar_to_ms() {
    assert_eq!(ar_to_ms(11.0), 300);
    assert_eq!(ar_to_ms(10.0), 450);
    assert_eq!(ar_to_ms(9.3), 555);
    assert_eq!(ar_to_ms(9.0), 600);
    assert_eq!(ar_to_ms(5.0), 1200);
    assert_eq!(ar_to_ms(4.0), 1320);
    assert_eq!(ar_to_ms(0.0), 1800);
}

fn cs_to_osupixels(cs: f32) -> f32 {
    54.4 - 4.48 * cs
}

#[test]
fn test_cs_to_osupixels() {
    assert_eq!(cs_to_osupixels(7.0), 23.04);
    assert_eq!(cs_to_osupixels(6.0), 27.52);
    assert_eq!(cs_to_osupixels(4.0), 36.480003); // floating point precision lol
}

pub struct BeatmapData {
    pub beatmap: libosu::Beatmap,
    pub ar_ms: i32,
    pub cs_osupixels: f32,
}

fn draw_circle(
    ctx: &mut ggez::Context,
    map_data: &BeatmapData,
    current_ms: i32,
    object: &HitObject,
) {
    ggez::graphics::Mesh::new_circle(
        ctx,
        DrawMode::fill(),
        mint::Point2 { x: 0.0, y: 0.0 },
        map_data.cs_osupixels,
        1.0,
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
    )
    .unwrap()
    .draw(
        ctx,
        DrawParam::new().dest(mint::Point2 {
            x: object.pos.0 as f32,
            y: object.pos.1 as f32,
        }),
    )
    .unwrap();

    let approach_circle_size =
        (object.start_time.as_milliseconds() - current_ms) as f32 / map_data.ar_ms as f32;

    ggez::graphics::Mesh::new_circle(
        ctx,
        DrawMode::stroke(1.0),
        mint::Point2 { x: 0.0, y: 0.0 },
        map_data.cs_osupixels * (1.0 + approach_circle_size),
        1.0,
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    )
    .unwrap()
    .draw(
        ctx,
        DrawParam::new().dest(mint::Point2 {
            x: object.pos.0 as f32,
            y: object.pos.1 as f32,
        }),
    )
    .unwrap();
}

fn draw_spinner(
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

#[derive(Debug)]
pub struct SliderData<'a> {
    /// The algorithm used to calculate the spline.
    kind: &'a libosu::SliderSplineKind,
    /// The control points that make up the body of the slider.
    control: &'a Vec<libosu::Point<i32>>,
    /// The number of times this slider should repeat.
    repeats: &'a u32,
    /// How long this slider is in pixels.
    pixel_length: &'a f64,
    /// The number of milliseconds long that this slider lasts.
    duration: &'a u32,
}

fn draw_slider(
    ctx: &mut ggez::Context,
    map_data: &BeatmapData,
    current_ms: i32,
    object: &HitObject,
    slider: SliderData,
) {
    draw_circle(ctx, map_data, current_ms, object);
    let mut points = Vec::with_capacity(slider.control.len() + 1);
    points.push(glam::vec2(object.pos.0 as f32, object.pos.1 as f32));
    points.extend(
        slider
            .control
            .iter()
            .map(|p| glam::vec2(p.0 as f32, p.1 as f32)),
    );

    if let libosu::SliderSplineKind::Bezier = slider.kind {
        points = curves::get_bezier(points);
    };

    let end_point = points[points.len() - 1];
    let second_to_last_end_point = points[points.len() - 2];
    let end_direction = (end_point.y - second_to_last_end_point.y)
        .atan2(end_point.x - second_to_last_end_point.x)
        + std::f32::consts::FRAC_PI_2;
    let end_offset = glam::vec2(
        end_direction.cos() * map_data.cs_osupixels,
        end_direction.sin() * map_data.cs_osupixels,
    );

    ggez::graphics::MeshBuilder::new()
        .line(
            &points
                .windows(2)
                .map(|p| {
                    let direction =
                        (p[1].y - p[0].y).atan2(p[1].x - p[0].x) + std::f32::consts::FRAC_PI_2;
                    let offset = glam::vec2(
                        direction.cos() * map_data.cs_osupixels,
                        direction.sin() * map_data.cs_osupixels,
                    );
                    p[0] + offset
                })
                .chain(std::iter::once(end_point + end_offset))
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
                        (p[1].y - p[0].y).atan2(p[1].x - p[0].x) + std::f32::consts::FRAC_PI_2;
                    let offset = glam::vec2(
                        direction.cos() * map_data.cs_osupixels,
                        direction.sin() * map_data.cs_osupixels,
                    );
                    p[0] - offset
                })
                .chain(std::iter::once(end_point - end_offset))
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
            DrawMode::fill(),
            end_point,
            map_data.cs_osupixels,
            1.0,
            Color {
                r: 0.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
        )
        .build(ctx)
        .unwrap()
        .draw(ctx, DrawParam::new())
        .unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let before = std::time::Instant::now();
    let replay =
        libosu::Replay::parse(BufReader::new(std::fs::File::open("replay2.osr").unwrap())).unwrap();

    let map_data = {
        let osudb = libosu::OsuDB::parse(BufReader::new(
            std::fs::File::open("C:\\Program Files\\osu!\\osu!.db").unwrap(),
        ))
        .unwrap();

        let beatmap = osudb
            .beatmaps
            .into_iter()
            .find(|beatmap| beatmap.hash == replay.beatmap_hash)
            .expect("Couldn't find replay beatmap in local beatmaps");

        let folder = Path::new("D:\\osu\\Songs").join(beatmap.folder_name);
        let beatmap_file = folder.join(beatmap.beatmap_file_name);

        let beatmap = libosu::Beatmap::from_osz(std::fs::read_to_string(beatmap_file)?)?;

        BeatmapData {
            ar_ms: ar_to_ms(beatmap.difficulty.approach_rate),
            cs_osupixels: cs_to_osupixels(beatmap.difficulty.circle_size),
            beatmap,
        }
    };

    let mut replay_data_iter = replay.actions.into_iter().peekable();

    let mut encoder = if DRAW_TO_VIDEO {
        Some(Encoder::new(640, 480, 30))
    } else {
        None
    };

    let (mut ctx, mut event_loop) = ggez::ContextBuilder::new("osr2mp4-rs", "nobbele")
        .window_mode(WindowMode {
            width: 640.0,
            height: 480.0,
            resizable: false,
            ..WindowMode::default()
        })
        .window_setup(WindowSetup {
            title: "osr2mp4-rs".to_owned(),
            ..WindowSetup::default()
        })
        .build()
        .unwrap();

    let canvas = ggez::graphics::Canvas::with_window_size(&mut ctx).unwrap();

    let mut game = Game {
        cursor_pos: (0.0, 0.0),
    };

    let mut running = true;

    let mut current_ms = 6000;
    let mut current_action = replay_data_iter.next().unwrap();
    let mut current_action_ms = 0;
    'outer: while running {
        println!(
            "({}/{}). {:?}",
            current_ms, current_action_ms, current_action
        );

        game.cursor_pos = (current_action.x, current_action.y);

        ggez::graphics::set_canvas(&mut ctx, Some(&canvas));
        ggez::graphics::clear(
            &mut ctx,
            Color {
                r: 0.3,
                g: 0.4,
                b: 0.5,
                a: 1.0,
            },
        );

        let active_object_iter = map_data.beatmap.hit_objects.iter().filter(|&obj| {
            current_ms >= obj.start_time.as_milliseconds() - map_data.ar_ms
                && match &obj.kind {
                    libosu::HitObjectKind::Circle => current_ms < obj.start_time.as_milliseconds(),
                    libosu::HitObjectKind::Slider {
                        duration, repeats, ..
                    } => {
                        current_ms < obj.start_time.as_milliseconds() + (duration * repeats) as i32
                    }
                    libosu::HitObjectKind::Spinner { end_time } => {
                        current_ms < end_time.as_milliseconds()
                    }
                }
        });

        for object in active_object_iter {
            match &object.kind {
                libosu::HitObjectKind::Circle => {
                    draw_circle(&mut ctx, &map_data, current_ms, object)
                }
                libosu::HitObjectKind::Slider {
                    kind,
                    control,
                    repeats,
                    pixel_length,
                    duration,
                } => draw_slider(
                    &mut ctx,
                    &map_data,
                    current_ms,
                    object,
                    SliderData {
                        kind,
                        control,
                        repeats,
                        pixel_length,
                        duration,
                    },
                ),
                libosu::HitObjectKind::Spinner { .. } => {
                    draw_spinner(&mut ctx, &map_data, current_ms, &object)
                }
            }
        }

        ggez::graphics::Text::new(format!(
            "{} playing {} - {} [{}]",
            replay.player_username,
            map_data.beatmap.artist_unicode,
            map_data.beatmap.title_unicode,
            map_data.beatmap.difficulty_name
        ))
        .draw(
            &mut ctx,
            DrawParam::new().dest(mint::Point2 { x: 0.0, y: 16.0 }),
        )
        .unwrap();

        ggez::graphics::Mesh::new_rectangle(
            &mut ctx,
            DrawMode::Fill(FillOptions::DEFAULT),
            Rect {
                x: 0.0,
                y: 0.0,
                w: 10.0,
                h: 10.0,
            },
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
        )
        .unwrap()
        .draw(
            &mut ctx,
            DrawParam::new().dest(mint::Point2 {
                x: game.cursor_pos.0,
                y: game.cursor_pos.1,
            }),
        )
        .unwrap();

        ggez::graphics::Mesh::new_rectangle(
            &mut ctx,
            DrawMode::Fill(FillOptions::DEFAULT),
            Rect {
                x: 0.0,
                y: 0.0,
                w: 10.0,
                h: 10.0,
            },
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        )
        .unwrap()
        .draw(
            &mut ctx,
            DrawParam::new().dest(mint::Point2 {
                x: match current_action.buttons {
                    n if n & (1 << 2) > 0 => 0.0,
                    n if n & (1 << 3) > 0 => 10.0,
                    _ => -10.0,
                },
                y: 0.0,
            }),
        )
        .unwrap();

        if !DRAW_TO_VIDEO {
            ggez::graphics::set_canvas(&mut ctx, None);
            ggez::graphics::draw(&mut ctx, &canvas, DrawParam::new()).unwrap();
            event_loop.poll_events(|e| match e {
                ggez::event::winit_event::Event::WindowEvent { event, .. } => match event {
                    ggez::event::winit_event::WindowEvent::CloseRequested => {
                        running = false;
                    }
                    _ => {}
                },
                _ => {}
            });
        }
        ggez::graphics::present(&mut ctx).unwrap();

        if let Some(encoder) = &mut encoder {
            encoder.encode(&canvas.image().to_rgba8(&mut ctx).unwrap());
        }

        loop {
            if let Some(next) = replay_data_iter.peek() {
                if current_ms >= current_action_ms + next.time as i32 {
                    current_action_ms += next.time as i32;
                    // Guaranteed to work since we successfully peeked
                    current_action = replay_data_iter.next().unwrap();
                    // Break if this is the last one as it's just garbage data
                    if replay_data_iter.peek().is_none() {
                        break 'outer;
                    }
                } else {
                    break;
                }
            } else {
                break 'outer;
            }
        }

        if let Some(encoder) = &encoder {
            current_ms += 1000 / encoder.framerate as i32;
        } else {
            current_ms += 1000 / 60;
        }
    }

    if let Some(encoder) = encoder {
        encoder.finish();
    }

    println!("Finished in {} seconds", before.elapsed().as_secs_f32());
    Ok(())
}
