use std::format;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
// use tokio::io::{AsyncBufReadExt, BufReader};
use std::{thread, time};
use tokio::sync::broadcast::Sender;

pub async fn get_duration(uuid: String, file: String) -> String {
    let child = Command::new("ffprobe")
        .arg("-i")
        .arg(&format!(
            "temp/{}/{}",
            "f99f41fa-3c39-474b-93be-ff225ace2801", "input.mp3"
        ))
        .arg("-show_format")
        .arg("-v")
        .arg("quiet")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| println!("{}", err))
        .unwrap();

    let duration = Command::new("grep")
        .arg("duration")
        .stdin(Stdio::from(child.stdout.unwrap()))
        .stdout(Stdio::piped())
        .output()
        .map_err(|err| println!("{}", err))
        .unwrap();

    String::from_utf8_lossy(&duration.stdout).to_string()
}

pub fn convert_to_wav(uuid: String, file: String, channel: Sender<String>) {
    _ = channel.send("Preparing Task".to_string());

    // FFMPEG processes it too quickly, temporary delay for "processing" time
    thread::sleep(time::Duration::from_secs(1));

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
            if matches!(reader.read_line(&mut line), Ok(0) | Err(_)) {
                break;
            }
            _ = channel.send(line.clone());
            // sleep(Duration::from_millis(1)).await;
        }
    }
}
