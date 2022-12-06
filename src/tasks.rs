use std::format;
/**
* Functions that are to be chained together as an audio work space
* system.
*
* convert to wav file
* ffmpeg -y -i night-run-125181.mp3 -acodec pcm_u8 -ar 22050 song.wav
* denoise and remove background noise
* ffmpeg -i song.wav -af lowpass=3000,highpass200,afftdn=nf=-25 optmised.wav
* split to segments
* ffmpeg -i optmised.wav -f segment -segment_time 30 -c copy out%03d.wav
*
* ffmpeg -i sample.mp3 -acodec pcm_u8 -ar 22050 sample.wav
   ffmpeg -i sample.wav -af lowpass=3000,highpass=200,afftdn=nf=-25 optmised.wav
   ffmpeg -i optmised.wav -f segment -segment_time 10 -c copy out%03d.wav
*
*
*/
use std::io::{BufRead, BufReader, Error};
use std::process::{Command, Stdio};

struct Audio {
    pub file: String,
}

fn convert_to_wav(input: Audio) -> Result<(), Error> {
    let mut cmd = Command::new("ffmpeg")
        .args([
            "-i",
            format!("{}", input.file),
            "-ss",
            "00:00:00",
            "-to",
            "00:02:00",
            "output.wav",
            "-progress",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdout = cmd.stdout.as_mut().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();

        for line in stdout_lines {
            println!("Read: {:?}", line);
        }
    }

    cmd.wait().unwrap();
    Ok(())
}
