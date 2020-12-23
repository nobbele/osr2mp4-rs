use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
};

pub struct Encoder {
    child: Child,
    child_in: ChildStdin,
    pub width: u16,
    pub height: u16,
    pub framerate: u16,
}

impl Encoder {
    pub fn new(width: u16, height: u16, framerate: u16) -> Self {
        let mut child = Command::new("ffmpeg")
            .args(&[
                "-vcodec",
                "rawvideo",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgba",
                "-s",
                &format!("{}x{}", width, height),
                "-framerate",
                &framerate.to_string(),
                "-i",
                "pipe:0",
                "-vf",
                "vflip",
                "-vcodec",
                "libx264",
                "-r",
                &framerate.to_string(),
                "./out.mp4",
                "-y",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .expect("Couldn't spawn ffmpeg");
        Self {
            child_in: child.stdin.take().expect("Couldn't write to ffmpeg"),
            child,
            width,
            height,
            framerate,
        }
    }

    pub fn encode(&mut self, data: &[u8]) {
        assert_eq!(data.len(), self.width as usize * self.height as usize * 4);
        self.child_in.write_all(&data).unwrap();
    }

    pub fn finish(mut self) -> PathBuf {
        self.child_in.flush().unwrap();
        std::mem::drop(self.child_in);
        self.child.wait().unwrap();
        Path::new("./out.mp4").to_path_buf()
    }
}
