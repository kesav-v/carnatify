use crate::note::Note;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Serialize)]
struct NotesOutput<'a> {
    notes: &'a [NoteJson],
}

#[derive(Serialize)]
struct NoteJson {
    offset: i8,
    duration: f32,
}

impl From<Note> for NoteJson {
    fn from(note: Note) -> Self {
        Self {
            offset: note.offset,
            duration: note.duration,
        }
    }
}

pub fn write_json(path: &Path, notes: &[Note]) -> Result<(), Box<dyn std::error::Error>> {
    let payload: Vec<NoteJson> = notes.iter().copied().map(NoteJson::from).collect();
    let output = NotesOutput { notes: &payload };
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    output.serialize(&mut ser)?;
    let json = String::from_utf8(buf).expect("json must be utf-8");
    let mut file = File::create(path)?;
    writeln!(file, "{json}")?;
    Ok(())
}
