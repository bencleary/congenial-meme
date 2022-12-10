use std::format;
use std::process::Stdio;
use tokio::sync::broadcast::Sender;
use tokio::time::{sleep, Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

pub async fn convert_to_wav(uuid: String, file: String, channel: Sender<String>) {
    _ = channel.send("Preparing Task".to_string());

    sleep(Duration::from_secs(5)).await;

    let mut child = Command::new("ffmpeg")
        .arg("-i")
        .arg(&format!("temp/{}/{}", uuid, file))
        .arg("-c")
        .arg("pcm_s16le")
        .arg("-ar")
        .arg("44100")
        .arg("-f")
        .arg("wav")
        .arg(&format!("temp/{}/{}", uuid, "output.wav"))
        .arg("-progress")
        .arg("pipe:1")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| println!("{}", err))
        .unwrap();

    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        loop {
            if matches!(reader.read_line(&mut line).await, Ok(0) | Err(_)) {
                break;
            }
            _ = channel.send(line.clone());
        }
    }
}
