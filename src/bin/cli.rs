use std::time::Duration;
use transcriber::tasks;

#[tokio::main]
async fn main() {
    // temp/f99f41fa-3c39-474b-93be-ff225ace2801/input.mp3

    let out = tasks::get_duration(
        "f99f41fa-3c39-474b-93be-ff225ace2801".to_string(),
        "input.mp3".to_string(),
    )
    .await;

    let parts = out.split("=");

    let parsed = parts.last().unwrap().trim().parse::<f32>().unwrap();

    let duration = Duration::from_secs_f64(parsed.into());

    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = (duration.as_secs() / 60) / 60;
    println!("{}:{}:{}", hours, minutes, seconds);
}
