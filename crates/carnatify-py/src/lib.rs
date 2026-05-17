use carnatify_core::{CarnaticNoteStream, Note, Pitch, PlaybackOptions};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass(name = "Note")]
#[derive(Clone)]
struct PyNote {
    #[pyo3(get)]
    offset: i8,
    #[pyo3(get)]
    duration: f32,
}

impl From<Note> for PyNote {
    fn from(n: Note) -> Self {
        Self {
            offset: n.offset,
            duration: n.duration,
        }
    }
}

#[pymethods]
impl PyNote {
    #[getter]
    fn frequency_hz(&self) -> f32 {
        carnatify_core::MIDDLE_C_HZ * 2f32.powf(f32::from(self.offset) / 12.0)
    }

    #[getter]
    fn duration_secs(&self) -> f32 {
        self.duration
    }
}

#[pyclass(name = "CarnaticNoteStream")]
struct PyCarnaticNoteStream {
    inner: CarnaticNoteStream,
}

#[pymethods]
impl PyCarnaticNoteStream {
    #[staticmethod]
    #[pyo3(signature = (notes, pitch="C", tempo=90))]
    fn from_notes(notes: &str, pitch: &str, tempo: u32) -> PyResult<Self> {
        let pitch = parse_pitch(pitch)?;
        Ok(Self {
            inner: CarnaticNoteStream::from_notes_with_options(
                notes,
                PlaybackOptions { pitch, tempo },
            ),
        })
    }

    fn next_note(&mut self) -> Option<PyNote> {
        self.inner.next().map(PyNote::from)
    }

    fn collect_notes(&mut self) -> Vec<PyNote> {
        self.inner.collect_notes().into_iter().map(PyNote::from).collect()
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
}

fn parse_pitch(name: &str) -> PyResult<Pitch> {
    Pitch::from_name(name).ok_or_else(|| PyValueError::new_err(format!("unknown pitch: {name}")))
}

#[pymodule]
fn _carnatify_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyNote>()?;
    m.add_class::<PyCarnaticNoteStream>()?;
    Ok(())
}
