/// Middle C frequency (Hz), matching `note_utils.freqs[0]` in Python.
pub const MIDDLE_C_HZ: f32 = 261.63;

/// MIDI note number for middle C (Python `MIDDLE_C`).
pub const MIDDLE_C_MIDI: u8 = 60;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Note {
    /// Semitone offset from middle C (carnatic offset + pitch shift).
    pub offset: i8,
    /// Duration in seconds (tempo already applied).
    pub duration: f32,
}

impl Note {
    pub fn frequency_hz(&self) -> f32 {
        MIDDLE_C_HZ * 2f32.powf(f32::from(self.offset) / 12.0)
    }

    pub fn duration_secs(&self) -> f32 {
        self.duration
    }

    pub fn midi_note_number(&self) -> u8 {
        (i16::from(MIDDLE_C_MIDI) + i16::from(self.offset))
            .clamp(0, 127) as u8
    }
}
