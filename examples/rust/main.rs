use carnatify_core::export::{write_json, write_midi, write_mp3, write_wav};
use carnatify_core::{CarnaticNoteStream, Pitch, PlaybackOptions};
use clap::{Parser, ValueEnum};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

/// Play Carnatic notation from the command line.
#[derive(Parser, Debug)]
#[command(name = "carnatify", version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "")]
    input_file: String,

    #[arg(default_value = "")]
    notes: String,

    #[arg(short, long, default_value = "")]
    output_file: String,

    #[arg(short, long, value_enum, default_value_t = CliPitch::C)]
    pitch: CliPitch,

    #[arg(short, long, default_value_t = 90)]
    tempo: u32,
}

#[derive(Clone, Debug, ValueEnum)]
#[value(rename_all = "UPPER")]
enum CliPitch {
    #[value(name = "C")]
    C,
    #[value(name = "C#")]
    Cs,
    #[value(name = "D")]
    D,
    #[value(name = "D#")]
    Ds,
    #[value(name = "E")]
    E,
    #[value(name = "F")]
    F,
    #[value(name = "F#")]
    Fs,
    #[value(name = "G")]
    G,
    #[value(name = "G#")]
    Gs,
    #[value(name = "A")]
    A,
    #[value(name = "A#")]
    As,
    #[value(name = "B")]
    B,
}

impl From<CliPitch> for Pitch {
    fn from(p: CliPitch) -> Self {
        match p {
            CliPitch::C => Pitch::C,
            CliPitch::Cs => Pitch::Cs,
            CliPitch::D => Pitch::D,
            CliPitch::Ds => Pitch::Ds,
            CliPitch::E => Pitch::E,
            CliPitch::F => Pitch::F,
            CliPitch::Fs => Pitch::Fs,
            CliPitch::G => Pitch::G,
            CliPitch::Gs => Pitch::Gs,
            CliPitch::A => Pitch::A,
            CliPitch::As => Pitch::As,
            CliPitch::B => Pitch::B,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Midi,
    Wav,
    Mp3,
    Json,
}

fn output_format(path: &str) -> Result<OutputFormat, String> {
    match Path::new(path)
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("mid") => Ok(OutputFormat::Midi),
        Some("wav") => Ok(OutputFormat::Wav),
        Some("mp3") => Ok(OutputFormat::Mp3),
        Some("json") => Ok(OutputFormat::Json),
        Some(ext) => Err(format!(
            "unsupported output extension '.{ext}' (use .mid, .wav, .mp3, or .json)"
        )),
        None => Err("output file must have an extension (.mid, .wav, .mp3, or .json)".into()),
    }
}

fn main() {
    let args = Args::parse();

    assert!(
        !args.notes.is_empty() || !args.input_file.is_empty(),
        "provide notes as an argument or via --input-file"
    );

    let notes = if !args.notes.is_empty() {
        args.notes
    } else {
        fs::read_to_string(&args.input_file).expect("could not read input file")
    };

    let options = PlaybackOptions {
        pitch: args.pitch.into(),
        tempo: args.tempo,
    };

    let mut stream = CarnaticNoteStream::from_notes_with_options(&notes, options);

    if args.output_file.is_empty() {
        stream.play_audio();
        return;
    }

    let format = output_format(&args.output_file).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    let collected = stream.collect_notes();
    let path = Path::new(&args.output_file);

    let result = match format {
        OutputFormat::Midi => write_midi(path, &collected, args.tempo),
        OutputFormat::Wav => write_wav(path, &collected),
        OutputFormat::Mp3 => write_mp3(path, &collected),
        OutputFormat::Json => write_json(path, &collected),
    };

    if let Err(e) = result {
        eprintln!("failed to write {}: {e}", args.output_file);
        std::process::exit(1);
    }
}
