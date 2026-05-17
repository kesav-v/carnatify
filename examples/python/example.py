#!/usr/bin/env python3
"""Parse Carnatic notation and print each note."""

from carnatify import CarnaticNoteStream

NOTATION = "S,R,G,M,P,D,N,S"
PITCH = "C"
TEMPO = 90

stream = CarnaticNoteStream.from_notes(NOTATION, pitch=PITCH, tempo=TEMPO)

for note in stream.collect_notes():
    print(f"offset={note.offset}  {note.frequency_hz:.1f} Hz  {note.duration_secs:.3f}s")
