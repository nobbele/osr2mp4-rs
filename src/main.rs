use encoder::Encoder;
use ggez::{
    conf::{WindowMode, WindowSetup},
    graphics::{Color, DrawMode, DrawParam, Drawable, FillOptions, Rect},
    mint,
};
use graphics::{circle::draw_circle, slider::{draw_slider}, spinner::draw_spinner};
use helper::{ar_to_ms, cs_to_osupixels};
use libosu::{beatmap::Beatmap, db::OsuDB, prelude::{HitObjectKind, SpinnerInfo}, replay::{Buttons, Replay}};
use std::{io::BufReader, path::Path, println};

mod encoder;
mod graphics;
mod helper;

pub struct Game {
    pub cursor_pos: (f32, f32),
}

pub struct BeatmapData {
    pub beatmap: Beatmap,
    pub ar_ms: i32,
    pub cs_osupixels: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let before = std::time::Instant::now();
    let replay =
        Replay::parse(&mut BufReader::new(std::fs::File::open("replay.osr")?)).unwrap();

    let map_data = {
        let osudb = OsuDB::parse(BufReader::new(
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

        let beatmap = libosu::beatmap::Beatmap::parse(&mut BufReader::new(std::fs::File::open(beatmap_file)?))?;

        BeatmapData {
            ar_ms: ar_to_ms(beatmap.difficulty.approach_rate),
            cs_osupixels: cs_to_osupixels(beatmap.difficulty.circle_size),
            beatmap,
        }
    };

    let mut replay_data_iter = replay.parse_action_data()?.frames.into_iter().peekable();

    let mut encoder = Encoder::new(640, 480, 30);

    let (mut ctx, _) = ggez::ContextBuilder::new("osr2mp4-rs", "nobbele")
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
        .add_zipfile_bytes(include_bytes!("../resources.zip").to_vec())
        .build()
        .unwrap();

    ggez::filesystem::print_all(&mut ctx);

    let canvas = ggez::graphics::Canvas::with_window_size(&mut ctx).unwrap();

    let mut game = Game {
        cursor_pos: (0.0, 0.0),
    };

    let mut current_ms = 6000;
    let mut current_action = replay_data_iter.next().unwrap();
    let mut current_action_ms = 0;
    'outer: loop {
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
            current_ms >= obj.start_time.as_seconds().as_millis().0 - map_data.ar_ms
                && match &obj.kind {
                    HitObjectKind::Circle => current_ms < obj.start_time.as_seconds().as_millis().0,
                    HitObjectKind::Slider(..) => {
                        let duration = map_data.beatmap.get_slider_duration(obj)
                            .expect("Expected slider duration");
                        current_ms < obj.start_time.as_seconds().as_millis().0 + duration as i32
                    }
                    HitObjectKind::Spinner(SpinnerInfo {
                        end_time
                    }) => {
                        current_ms < end_time.as_seconds().as_millis().0
                    }
                }
        });

        for object in active_object_iter {
            match &object.kind {
                HitObjectKind::Circle => {
                    draw_circle(&mut ctx, &map_data, current_ms, object)
                }
                HitObjectKind::Slider(info) => draw_slider(
                    &mut ctx,
                    &map_data,
                    current_ms,
                    object,
                    info,
                ),
                HitObjectKind::Spinner(..) => {
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
                    Buttons::K1 => 0.0,
                    Buttons::K2 => 10.0,
                    Buttons::M1 => 20.0,
                    Buttons::M2 => 30.0,
                    Buttons::SMOKE => 40.0,
                    _ => -10.0,
                },
                y: 0.0,
            }),
        )
        .unwrap();

        ggez::graphics::present(&mut ctx).unwrap();

        encoder.encode(&canvas.image().to_rgba8(&mut ctx).unwrap());

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

        current_ms += 1000 / encoder.framerate as i32
    }

    encoder.finish();

    println!("Finished in {} seconds", before.elapsed().as_secs_f32());
    Ok(())
}
