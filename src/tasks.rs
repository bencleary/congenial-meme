use std::format;
use async_process::{Command, Stdio};
use futures_lite::{io::BufReader, prelude::*};
use tokio::sync::broadcast::Sender;
use tokio::time::{sleep, Duration};

pub async fn convert_to_wav(
    uuid: String,
    file: String,
    channel: Sender<String>,
) {

    channel.send("Update 1".to_string());

    sleep(Duration::from_secs(5)).await;

    channel.send("waited".to_string());

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
        .stdout(Stdio::piped())
        .spawn().unwrap();


    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();

    while let Some(line) = lines.next().await {
        channel.send(line.unwrap());
    }
}
