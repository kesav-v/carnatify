use carnatify_core::{CarnaticNoteStream, Pitch, PlaybackOptions};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct JsNote {
    pub offset: i8,
    pub duration: f32,
    pub frequency_hz: f32,
    pub duration_secs: f32,
}

#[wasm_bindgen]
pub struct CarnaticNoteStreamWasm {
    inner: CarnaticNoteStream,
}

#[wasm_bindgen]
impl CarnaticNoteStreamWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(notes: String, pitch: String, tempo: u32) -> Result<CarnaticNoteStreamWasm, JsValue> {
        Self::from_notes_with_options(&notes, &pitch, tempo)
    }

    #[wasm_bindgen(js_name = from_notes)]
    pub fn from_notes(notes: &str) -> Result<CarnaticNoteStreamWasm, JsValue> {
        Self::from_notes_with_options(notes, "C", 90)
    }

    #[wasm_bindgen(js_name = from_notes_with_options)]
    pub fn from_notes_with_options(
        notes: &str,
        pitch: &str,
        tempo: u32,
    ) -> Result<CarnaticNoteStreamWasm, JsValue> {
        let pitch = Pitch::from_name(pitch)
            .ok_or_else(|| JsValue::from_str(&format!("unknown pitch: {pitch}")))?;
        Ok(Self {
            inner: CarnaticNoteStream::from_notes_with_options(
                notes,
                PlaybackOptions { pitch, tempo },
            ),
        })
    }

    pub fn next_note(&mut self) -> Option<JsNote> {
        self.inner.next().map(|n| JsNote {
            offset: n.offset,
            duration: n.duration,
            frequency_hz: n.frequency_hz(),
            duration_secs: n.duration_secs(),
        })
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }
}
