use crate::note::MIDDLE_C_HZ;
use crate::stream::CarnaticNoteStream;
use rodio::source::{SineWave, Source};
use rodio::Player;
use std::time::Duration;

pub fn play_audio(stream: &mut CarnaticNoteStream) {
    let mut handle = rodio::DeviceSinkBuilder::open_default_sink()
        .expect("open default audio stream");
    handle.log_on_drop(false);
    let player = Player::connect_new(&handle.mixer());

    for note in stream.by_ref() {
        let source = SineWave::new(note.frequency_hz())
            .take_duration(Duration::from_secs_f32(note.duration_secs()))
            .amplify(1.0);
        player.append(source);
    }

    player.append(
        SineWave::new(MIDDLE_C_HZ)
            .take_duration(Duration::from_secs_f32(1.0))
            .amplify(0.0),
    );
    player.sleep_until_end();
}
