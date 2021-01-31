use std::{iter::Peekable, str::FromStr};

use ggez::{
    audio::SoundSource,
    event::{quit, EventHandler},
    graphics::{Canvas, Color, DrawMode, DrawParam, Drawable, FillOptions, Image, Rect},
    mint, Context,
};
use libosu::{prelude::*, replay::ReplayAction};

use crate::{
    encoder::Encoder,
    graphics::{circle::draw_circle, slider::draw_slider, spinner::draw_spinner},
    BeatmapData,
};

pub struct Player {
    current_ms: i32,
    current_action_ms: i32,
    current_action: ReplayAction,
    replay_data_iter: Peekable<std::vec::IntoIter<ReplayAction>>,
    combo_index: usize,
    was_prev_first_new_combo: bool,

    fps: i32,
    paused: bool,

    encoder: Option<Encoder>,
    //canvas: Canvas,
    replay: Replay,
    background: Image,
    music: ggez::audio::Source,
    map_data: BeatmapData,
}

impl Player {
    pub fn new(ctx: &mut Context, replay: Replay, map_data: BeatmapData, fps: i32) -> Self {
        let mut iter = replay
            .parse_action_data()
            .expect("Unable to parse replay")
            .frames
            .into_iter()
            .peekable();
        Self {
            current_ms: 6000,
            current_action_ms: 0,
            current_action: iter.next().expect("Replay is empty"),
            replay_data_iter: iter,
            combo_index: 0,
            was_prev_first_new_combo: false,

            fps,
            paused: false,

            encoder: None,
            //canvas: ggez::graphics::Canvas::with_window_size(ctx).unwrap(),
            replay,
            background: Image::from_bytes(
                ctx,
                &std::fs::read(
                    map_data
                        .beatmap
                        .events
                        .iter()
                        .find_map(|item| match item {
                            Event::Background(e) => Some(&e.filename),
                            _ => None,
                        })
                        .map(|s| map_data.folder.join(s))
                        .expect("No background image"),
                )
                .expect("Couldn't find beatmap background image"),
            )
            .expect("Couldn't load beatmap background image"),
            music: {
                let mut source = ggez::audio::Source::from_data(
                    ctx,
                    ggez::audio::SoundData::from_bytes(
                        &std::fs::read(map_data.folder.join(&map_data.beatmap.audio_filename))
                            .expect("Couldn't find beatmap music file"),
                    ),
                )
                .expect("Couldn't load beatmap music file");
                source.set_volume(0.1);
                source.set_query_interval(std::time::Duration::from_millis(1000 / 60));
                source.play(ctx).expect("Could not start music");
                source
            },
            map_data,
        }
    }
}

impl EventHandler for Player {
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        if let Some(encoder) = self.encoder.take() {
            encoder.finish();
        }
        false // false means quit
    }

    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        if !self.paused {
            self.current_ms = self.music.elapsed().as_millis() as i32;
        }

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: ggez::event::KeyCode,
        _keymods: ggez::event::KeyMods,
        _repeat: bool,
    ) {
        if keycode == ggez::event::KeyCode::Space {
            self.paused = !self.paused;
            if self.music.paused() {
                self.music.resume();
            } else {
                self.music.pause();
            }
        }
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        println!("{}ms -> {:?}", self.current_ms, self.current_action);

        //ggez::graphics::set_canvas(ctx, Some(&self.canvas));
        ggez::graphics::clear(
            ctx,
            Color {
                r: 0.3,
                g: 0.4,
                b: 0.5,
                a: 1.0,
            },
        );

        self.background.draw(
            ctx,
            DrawParam::new().scale(
                glam::vec2(
                    ggez::graphics::drawable_size(ctx).0,
                    ggez::graphics::drawable_size(ctx).1,
                ) / glam::vec2(
                    self.background.dimensions().w,
                    self.background.dimensions().h,
                ),
            ),
        )?;

        let mut active_object_iter = {
            let current_ms = self.current_ms;
            let ar_ms = self.map_data.ar_ms;
            let beatmap = &self.map_data.beatmap;
            let iter = self
                .map_data
                .beatmap
                .hit_objects
                .iter()
                .filter(move |&obj| {
                    current_ms >= obj.start_time.0 - ar_ms
                        && match &obj.kind {
                            HitObjectKind::Circle => current_ms < obj.start_time.0,
                            HitObjectKind::Slider(..) => {
                                let duration = beatmap
                                    .get_slider_duration(obj)
                                    .expect("Expected slider duration");
                                current_ms < obj.start_time.0 + duration as i32
                            }
                            HitObjectKind::Spinner(SpinnerInfo { end_time }) => {
                                current_ms < end_time.0
                            }
                        }
                })
                .peekable();
            iter
        };

        if let Some(first_new_combo) = active_object_iter.peek().map(|o| o.new_combo) {
            if first_new_combo && !self.was_prev_first_new_combo {
                self.combo_index = (self.combo_index + 1) % self.map_data.beatmap.colors.len();
            }
            self.was_prev_first_new_combo = first_new_combo;
        }

        let mut active_combo_index = self.combo_index;
        for (i, object) in active_object_iter.enumerate() {
            if object.new_combo && i != 0 {
                active_combo_index = (active_combo_index + 1) % self.map_data.beatmap.colors.len()
            }
            let color = self.map_data.beatmap.colors[active_combo_index];
            let color = Color::from_rgb(color.red, color.green, color.blue);
            match &object.kind {
                HitObjectKind::Circle => {
                    draw_circle(ctx, &self.map_data, self.current_ms, object, color)
                }
                HitObjectKind::Slider(info) => {
                    draw_slider(ctx, &self.map_data, self.current_ms, object, info, color)
                }
                HitObjectKind::Spinner(..) => {
                    draw_spinner(ctx, &self.map_data, self.current_ms, &object)
                }
            }
        }

        ggez::graphics::Text::new(format!(
            "{} playing {} - {} [{}]",
            self.replay.player_username,
            self.map_data.beatmap.artist_unicode,
            self.map_data.beatmap.title_unicode,
            self.map_data.beatmap.difficulty_name
        ))
        .draw(ctx, DrawParam::new().dest(mint::Point2 { x: 0.0, y: 16.0 }))
        .unwrap();

        ggez::graphics::Mesh::new_rectangle(
            ctx,
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
            ctx,
            DrawParam::new().dest(mint::Point2 {
                x: self.current_action.x,
                y: self.current_action.y,
            }),
        )
        .unwrap();

        ggez::graphics::Mesh::new_rectangle(
            ctx,
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
            ctx,
            DrawParam::new().dest(mint::Point2 {
                x: match self.current_action.buttons {
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

        ggez::graphics::present(ctx).unwrap();

        /*if let Some(encoder) = &mut self.encoder {
            encoder
                .encode(&self.canvas.image().to_rgba8(ctx).unwrap());
        }*/

        loop {
            if let Some(next) = self.replay_data_iter.peek() {
                if self.current_ms >= self.current_action_ms as i32 + next.time as i32 {
                    // Guaranteed to work since we successfully peeked, same as `next`
                    self.current_action = self.replay_data_iter.next().unwrap();
                    self.current_action_ms += self.current_action.time as i32;
                    // Break if this is the last one as it's just garbage data
                    if self.replay_data_iter.peek().is_none() {
                        quit(ctx);
                        break;
                    }
                } else {
                    break;
                }
            } else {
                quit(ctx);
                break;
            }
        }

        Ok(())
    }
}
