use ggez::{
    conf::{WindowMode, WindowSetup},
};
use helper::{ar_to_ms, cs_to_osupixels};
use libosu::{
    beatmap::Beatmap,
    db::OsuDB,
    replay::Replay,
};
use std::{io::BufReader, path::Path};

mod encoder;
mod graphics;
mod helper;
mod player;
pub struct BeatmapData {
    pub beatmap: Beatmap,
    pub ar_ms: i32,
    pub cs_osupixels: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let replay = Replay::parse(&mut BufReader::new(std::fs::File::open("replay.osr")?)).unwrap();

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

        let beatmap = libosu::beatmap::Beatmap::parse(&mut BufReader::new(std::fs::File::open(
            beatmap_file,
        )?))?;

        BeatmapData {
            ar_ms: ar_to_ms(beatmap.difficulty.approach_rate),
            cs_osupixels: cs_to_osupixels(beatmap.difficulty.circle_size),
            beatmap,
        }
    };

    let (mut ctx, event_loop) = ggez::ContextBuilder::new("osr2mp4-rs", "nobbele")
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

    let player = player::Player::new(&mut ctx, replay, map_data, 30);

    ggez::event::run(ctx, event_loop, player)
}
