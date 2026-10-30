#[cfg(unix)]
pub fn play_audio_file(file_path: &str, volume: f32) {
    if volume <= 0.0 {
        return;
    }
    let uri = if file_path.starts_with('/') {
        format!("file://{file_path}")
    } else {
        format!("file:///{file_path}")
    };

    let child = std::process::Command::new("gst-launch-1.0")
        .args([
            "playbin",
            &format!("uri={uri}"),
            &format!("volume={volume:.2}"),
        ])
        .spawn();

    if let Ok(mut child) = child {
        std::thread::spawn(move || {
            let _ = child.wait();
        });
    }
}

#[cfg(windows)]
pub fn play_audio_file(file_path: &str, volume: f32) {
    if volume <= 0.0 {
        return;
    }
    let path = file_path.to_owned();
    std::thread::spawn(move || {
        let Ok(mut device) = rodio::DeviceSinkBuilder::open_default_sink() else {
            return;
        };
        device.log_on_drop(false);
        let Some(source) = std::fs::File::open(&path)
            .ok()
            .and_then(|file| rodio::Decoder::try_from(file).ok())
        else {
            return;
        };
        let player = rodio::Player::connect_new(device.mixer());
        player.set_volume(volume);
        player.append(source);
        // The device must stay open until the cry ends.
        player.sleep_until_end();
    });
}
