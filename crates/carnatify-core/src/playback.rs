/// Playback settings matching `server/notes_parser.py` (`--pitch`, `--tempo`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaybackOptions {
    pub pitch: Pitch,
    /// Beats per minute (MIDI tempo).
    pub tempo: u32,
}

impl Default for PlaybackOptions {
    fn default() -> Self {
        Self {
            pitch: Pitch::C,
            tempo: 90,
        }
    }
}

/// Chromatic pitch names (`note_utils.shifts` in Python).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pitch {
    #[default]
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

impl Pitch {
    /// Semitone offset from C (same as Python `shifts[pitch]`).
    pub fn semitone_shift(self) -> i8 {
        match self {
            Pitch::C => 0,
            Pitch::Cs => 1,
            Pitch::D => 2,
            Pitch::Ds => 3,
            Pitch::E => 4,
            Pitch::F => 5,
            Pitch::Fs => 6,
            Pitch::G => 7,
            Pitch::Gs => 8,
            Pitch::A => 9,
            Pitch::As => 10,
            Pitch::B => 11,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "C" => Some(Pitch::C),
            "C#" => Some(Pitch::Cs),
            "D" => Some(Pitch::D),
            "D#" => Some(Pitch::Ds),
            "E" => Some(Pitch::E),
            "F" => Some(Pitch::F),
            "F#" => Some(Pitch::Fs),
            "G" => Some(Pitch::G),
            "G#" => Some(Pitch::Gs),
            "A" => Some(Pitch::A),
            "A#" => Some(Pitch::As),
            "B" => Some(Pitch::B),
            _ => None,
        }
    }

    pub const NAMES: &'static [&'static str] =
        &["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
}
