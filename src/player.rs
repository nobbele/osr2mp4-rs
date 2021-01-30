use std::iter::Peekable;

use ggez::{
    event::{quit, EventHandler},
    graphics::{Canvas, Color, DrawMode, DrawParam, Drawable, FillOptions, Rect},
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
    
    fps: i32,

    encoder: Option<Encoder>,
    canvas: Canvas,
    replay: Replay,
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

            fps,

            encoder: None,
            canvas: ggez::graphics::Canvas::with_window_size(ctx).unwrap(),
            replay,
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
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        println!("{}ms -> {:?}", self.current_ms, self.current_action);

        ggez::graphics::set_canvas(ctx, Some(&self.canvas));
        ggez::graphics::clear(
            ctx,
            Color {
                r: 0.3,
                g: 0.4,
                b: 0.5,
                a: 1.0,
            },
        );

        let active_object_iter = self.map_data.beatmap.hit_objects.iter().filter(|&obj| {
            self.current_ms >= obj.start_time.0 - self.map_data.ar_ms
                && match &obj.kind {
                    HitObjectKind::Circle => self.current_ms < obj.start_time.0,
                    HitObjectKind::Slider(..) => {
                        let duration = self
                            .map_data
                            .beatmap
                            .get_slider_duration(obj)
                            .expect("Expected slider duration");
                        self.current_ms < obj.start_time.0 + duration as i32
                    }
                    HitObjectKind::Spinner(SpinnerInfo { end_time }) => {
                        self.current_ms < end_time.0
                    }
                }
        });

        for object in active_object_iter {
            match &object.kind {
                HitObjectKind::Circle => {
                    draw_circle(ctx, &self.map_data, self.current_ms, object)
                }
                HitObjectKind::Slider(info) => {
                    draw_slider(ctx, &self.map_data, self.current_ms, object, info)
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
        .draw(
            ctx,
            DrawParam::new().dest(mint::Point2 { x: 0.0, y: 16.0 }),
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

        if let Some(encoder) = &mut self.encoder {
            encoder
                .encode(&self.canvas.image().to_rgba8(ctx).unwrap());
        }

        println!("calculating stuff");

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

        self.current_ms += 1000 / self.fps;

        Ok(())
    }
}
