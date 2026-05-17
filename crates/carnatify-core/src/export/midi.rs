use crate::note::Note;
use midly::num::{u15, u24, u28, u4, u7};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind};
use std::path::Path;

const PPQ: u16 = 480;
const CHANNEL: u4 = u4::new(0);
const VELOCITY: u7 = u7::new(100);

pub fn write_midi(path: &Path, notes: &[Note], tempo: u32) -> Result<(), Box<dyn std::error::Error>> {
    let tempo = tempo.max(1);
    let mut track = Track::new();

    track.push(TrackEvent {
        delta: u28::from(0),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::from(
            60_000_000u32 / tempo,
        ))),
    });
    track.push(TrackEvent {
        delta: u28::from(0),
        kind: TrackEventKind::Meta(MetaMessage::TimeSignature(4, 2, 24, 8)),
    });
    track.push(TrackEvent {
        delta: u28::from(0),
        kind: TrackEventKind::Meta(MetaMessage::KeySignature(0, false)),
    });
    track.push(TrackEvent {
        delta: u28::from(0),
        kind: TrackEventKind::Midi {
            channel: CHANNEL,
            message: MidiMessage::ProgramChange {
                program: u7::new(0),
            },
        },
    });

    let mut elapsed_ticks = 0u32;
    let mut time_secs = 0f32;

    for note in notes {
        let start_tick = secs_to_ticks(time_secs, tempo);
        let duration_ticks = secs_to_ticks(note.duration, tempo).max(1);

        let delta_on = start_tick.saturating_sub(elapsed_ticks);
        elapsed_ticks += delta_on;
        track.push(TrackEvent {
            delta: u28::from(delta_on),
            kind: TrackEventKind::Midi {
                channel: CHANNEL,
                message: MidiMessage::NoteOn {
                    key: u7::new(note.midi_note_number()),
                    vel: VELOCITY,
                },
            },
        });

        elapsed_ticks += duration_ticks;
        track.push(TrackEvent {
            delta: u28::from(duration_ticks),
            kind: TrackEventKind::Midi {
                channel: CHANNEL,
                message: MidiMessage::NoteOff {
                    key: u7::new(note.midi_note_number()),
                    vel: VELOCITY,
                },
            },
        });

        time_secs += note.duration;
    }

    track.push(TrackEvent {
        delta: u28::from(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    let smf = Smf {
        header: Header::new(Format::SingleTrack, Timing::Metrical(u15::from(PPQ))),
        tracks: vec![track],
    };

    smf.save(path)?;
    Ok(())
}

fn secs_to_ticks(duration_secs: f32, tempo: u32) -> u32 {
    let ticks = duration_secs * f32::from(PPQ) * tempo as f32 / 60.0;
    ticks.round().max(1.0) as u32
}
