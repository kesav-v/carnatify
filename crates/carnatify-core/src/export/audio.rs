use crate::note::Note;
use hound::{SampleFormat, WavSpec, WavWriter};
use std::f32::consts::TAU;
use std::path::Path;
use std::process::Command;

const SAMPLE_RATE: u32 = 44_100;
const AMPLITUDE: f32 = 0.3;

pub fn write_wav(path: &Path, notes: &[Note]) -> Result<(), Box<dyn std::error::Error>> {
    let samples = synthesize(notes);
    write_wav_samples(path, &samples)?;
    Ok(())
}

pub fn write_mp3(path: &Path, notes: &[Note]) -> Result<(), Box<dyn std::error::Error>> {
    let wav_path = path.with_extension("wav.tmp");
    write_wav(&wav_path, notes)?;
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            wav_path.to_string_lossy().as_ref(),
            path.to_string_lossy().as_ref(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    let _ = std::fs::remove_file(&wav_path);

    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(
            "mp3 export requires ffmpeg on PATH (e.g. brew install ffmpeg)".into(),
        ),
    }
}

fn synthesize(notes: &[Note]) -> Vec<f32> {
    let total_samples = notes
        .iter()
        .map(|n| (n.duration * SAMPLE_RATE as f32).ceil() as usize)
        .sum();
    let mut buffer = vec![0.0f32; total_samples];
    let mut offset = 0usize;

    for note in notes {
        let freq = note.frequency_hz();
        let n_samples = (note.duration * SAMPLE_RATE as f32).ceil() as usize;
        for i in 0..n_samples {
            if offset + i >= buffer.len() {
                break;
            }
            let t = i as f32 / SAMPLE_RATE as f32;
            buffer[offset + i] += (TAU * freq * t).sin() * AMPLITUDE;
        }
        offset += n_samples;
    }

    buffer
}

fn write_wav_samples(path: &Path, samples: &[f32]) -> Result<(), hound::Error> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec)?;
    for &sample in samples {
        let clipped = sample.clamp(-1.0, 1.0);
        let amplitude = (clipped * i16::MAX as f32) as i16;
        writer.write_sample(amplitude)?;
    }
    writer.finalize()?;
    Ok(())
}
