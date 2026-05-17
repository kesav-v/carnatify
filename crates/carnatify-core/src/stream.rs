use crate::note::Note;
use crate::playback::PlaybackOptions;

const OCTAVE: i8 = 12;

/// Semitone offset from middle C for each `BASE_NOTE` symbol (`/[srgmpdnSRGMPDN]/`).
fn carnatic_shift(base: u8) -> Option<i8> {
    match base {
        b'S' | b's' => Some(0),
        b'r' => Some(1),
        b'R' => Some(2),
        b'g' => Some(3),
        b'G' => Some(4),
        b'm' => Some(5),
        b'M' => Some(6),
        b'P' => Some(7),
        b'd' => Some(8),
        b'D' => Some(9),
        b'n' => Some(10),
        b'N' => Some(11),
        _ => None,
    }
}

fn is_base_note(ch: u8) -> bool {
    carnatic_shift(ch).is_some()
}

fn is_identifier_char(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || ch == b'_'
}

/// One item inside `swaram: (note | speedup | slowdown | KARVE | NEWLINE)+`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwaramToken {
    Note { spec: NoteSpec },
    SpeedUp,
    SlowDown,
}

/// Parsed `note: BASE_NOTE [ ( UPPER_OCTAVE | LOWER_OCTAVE )* ] [ KARVE* ]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NoteSpec {
    base: u8,
    octave_shift: i8,
    karve_count: u32,
}

/// A note ready to emit, with the speed level captured at parse time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedNote {
    spec: NoteSpec,
    /// Speed level active when this note was parsed (`NotesVisitor._speedup`).
    speedup: u8,
}

/// Parses Carnatic notation (`server/carnatic_notation_grammar.lark`) and yields [`Note`] events.
pub struct CarnaticNoteStream {
    input: Vec<u8>,
    pos: usize,
    speedup: u8,
    pending: Option<ParsedNote>,
    pitch_shift: i8,
    secs_per_beat_unit: f32,
}

impl CarnaticNoteStream {
    pub fn new(notes: impl Into<String>) -> Self {
        Self::with_options(notes, PlaybackOptions::default())
    }

    pub fn with_options(notes: impl Into<String>, options: PlaybackOptions) -> Self {
        let tempo = options.tempo.max(1);
        Self {
            input: notes.into().into_bytes(),
            pos: 0,
            speedup: 0,
            pending: None,
            pitch_shift: options.pitch.semitone_shift(),
            secs_per_beat_unit: 15.0 / tempo as f32,
        }
    }

    pub fn from_notes(notes: &str) -> Self {
        Self::new(notes)
    }

    pub fn from_notes_with_options(notes: &str, options: PlaybackOptions) -> Self {
        Self::with_options(notes, options)
    }

    pub fn reset(&mut self) {
        self.pos = 0;
        self.speedup = 0;
        self.pending = None;
    }

    pub fn collect_notes(&mut self) -> Vec<Note> {
        self.by_ref().collect()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn play_audio(&mut self) {
        crate::player::native::play_audio(self);
    }

    fn make_note(&self, parsed: ParsedNote) -> Note {
        let spec = parsed.spec;
        let carnatic_offset =
            carnatic_shift(spec.base).expect("parsed note has valid base")
                + spec.octave_shift * OCTAVE;
        let beat_duration = (1.0 + spec.karve_count as f32)
            / f32::from((1 << parsed.speedup) as i16);
        Note {
            offset: carnatic_offset + self.pitch_shift,
            duration: beat_duration * self.secs_per_beat_unit,
        }
    }

    fn emit_pending(&mut self) -> Option<Note> {
        self.pending.take().map(|parsed| self.make_note(parsed))
    }

    /// `expression: command` — advance past `#start …` / `#stop …`.
    fn skip_command(&mut self) {
        debug_assert_eq!(self.input.get(self.pos), Some(&b'#'));
        self.pos += 1;
        self.skip_whitespace();
        if self.consume_ascii(b"start") {
            self.skip_whitespace();
            if self.consume_ascii(b"beat") {
                self.skip_beat_command_body();
            } else if self.consume_ascii(b"chord") {
                self.skip_identifier();
                self.skip_chord_swaram();
            }
        } else if self.consume_ascii(b"stop") {
            self.skip_identifier();
        }
    }

    /// `beat: "beat" identifier (BEAT | NO_BEAT)+ [ NUMBER ]`
    fn skip_beat_command_body(&mut self) {
        self.skip_whitespace();
        self.skip_identifier();
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch == b'-' || ch == b'.' {
                self.pos += 1;
                continue;
            }
            if ch.is_ascii_digit() {
                while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
            }
            break;
        }
    }

    fn skip_identifier(&mut self) {
        self.skip_whitespace();
        while self.pos < self.input.len() && is_identifier_char(self.input[self.pos]) {
            self.pos += 1;
        }
    }

    /// Chord pattern notes on the same line as `#start chord …` (melody may follow on later lines).
    fn skip_chord_swaram(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos] != b'\n' {
            self.skip_swaram_token();
        }
    }

    fn skip_swaram_token(&mut self) {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return;
        }
        let ch = self.input[self.pos];
        if ch == b'(' || ch == b')' || ch == b',' {
            self.pos += 1;
            return;
        }
        if is_base_note(ch) {
            self.skip_note();
        }
    }

    /// `note: BASE_NOTE [ ( UPPER_OCTAVE | LOWER_OCTAVE )* ] [ KARVE* ]`
    fn parse_note(&mut self) -> Option<NoteSpec> {
        let base = *self.input.get(self.pos)?;
        if !is_base_note(base) {
            return None;
        }
        self.pos += 1;
        let mut octave_shift = 0i8;
        let mut karve_count = 0u32;
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                b'*' => {
                    octave_shift += 1;
                    self.pos += 1;
                }
                b'/' => {
                    octave_shift -= 1;
                    self.pos += 1;
                }
                b',' => {
                    karve_count += 1;
                    self.pos += 1;
                }
                _ => break,
            }
        }
        Some(NoteSpec {
            base,
            octave_shift,
            karve_count,
        })
    }

    fn skip_note(&mut self) {
        let _ = self.parse_note();
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn consume_ascii(&mut self, word: &[u8]) -> bool {
        self.skip_whitespace();
        if self.pos + word.len() > self.input.len() {
            return false;
        }
        if self.input[self.pos..self.pos + word.len()] != *word {
            return false;
        }
        self.pos += word.len();
        true
    }

    /// Next token inside the current swaram, or `None` at end of input.
    fn next_swaram_token(&mut self) -> Option<SwaramToken> {
        loop {
            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return None;
            }
            let ch = self.input[self.pos];
            if ch == b'#' {
                self.skip_command();
                continue;
            }
            if ch == b'\n' {
                self.pos += 1;
                continue;
            }
            if ch == b'(' {
                self.pos += 1;
                return Some(SwaramToken::SpeedUp);
            }
            if ch == b')' {
                self.pos += 1;
                return Some(SwaramToken::SlowDown);
            }
            if ch == b',' {
                // Standalone `KARVE` in swaram — no effect on melody (matches Python visitor).
                self.pos += 1;
                continue;
            }
            if let Some(spec) = self.parse_note() {
                return Some(SwaramToken::Note { spec });
            }
            // Ignore characters outside the grammar (spaces, `|`, etc.).
            self.pos += 1;
        }
    }
}

impl Iterator for CarnaticNoteStream {
    type Item = Note;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.next_swaram_token() {
                Some(SwaramToken::SpeedUp) => {
                    self.speedup = self.speedup.saturating_add(1);
                }
                Some(SwaramToken::SlowDown) => {
                    self.speedup = self.speedup.saturating_sub(1);
                }
                Some(SwaramToken::Note { spec }) => {
                    let previous = self.pending.replace(ParsedNote {
                        spec,
                        speedup: self.speedup,
                    });
                    if let Some(parsed) = previous {
                        return Some(self.make_note(parsed));
                    }
                }
                None => return self.emit_pending(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offsets(notes: &str) -> Vec<i8> {
        CarnaticNoteStream::from_notes(notes)
            .map(|n| n.offset)
            .collect()
    }

    fn durations(notes: &str) -> Vec<f32> {
        CarnaticNoteStream::from_notes(notes)
            .map(|n| n.duration)
            .collect()
    }

    #[test]
    fn parses_simple_sequence() {
        assert_eq!(offsets("SRG"), vec![0, 2, 4]);
    }

    #[test]
    fn note_modifiers_apply_to_that_note_only() {
        // Modifiers bind to the preceding BASE_NOTE only (`S*`, `R/`, `P`).
        assert_eq!(offsets("S*R/P"), vec![12, -10, 7]);
    }

    #[test]
    fn karve_extends_duration() {
        let d = durations("S,,R");
        assert_eq!(d.len(), 2);
        assert!((d[0] - 3.0 * d[1]).abs() < 1e-5);
    }

    #[test]
    fn speedup_and_slowdown_do_not_drop_notes() {
        assert_eq!(offsets("(SR)"), vec![0, 2]);
        assert_eq!(offsets("S(R)"), vec![0, 2]);
    }

    #[test]
    fn speedup_halves_beat_duration() {
        let d = durations("(SR)");
        assert_eq!(d.len(), 2);
        assert!((d[0] - d[1]).abs() < 1e-5);
        assert!((d[0] - 0.5 * 15.0 / 90.0).abs() < 1e-5);
    }

    #[test]
    fn skips_beat_command() {
        assert_eq!(offsets("#start beat tala --- 36\nSRG"), vec![0, 2, 4]);
    }

    #[test]
    fn skips_chord_command() {
        assert_eq!(
            offsets("#start chord arpeggio SRG\nP"),
            vec![7],
        );
    }

    #[test]
    fn ignores_formatting_characters() {
        assert_eq!(offsets("S | R | G"), vec![0, 2, 4]);
    }
}
