use async_process::Command;
use std::format;
use std::io::{BufRead, BufReader, Error};
use std::process::Stdio;
use tokio::sync::broadcast::Sender;

pub async fn convert_to_wav(
    uuid: String,
    file: String,
    channel: Sender<String>,
) -> Result<(), Error> {
    let mut cmd = Command::new("ffmpeg")
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
        .arg("progress.log.txt")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdout = cmd.stdout.as_mut().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();

        for line in stdout_lines {
            println!("Read: {:?}", line);
            let _ = channel.send(line.unwrap());
        }
    }

    cmd.wait().unwrap();
    Ok(())
}
