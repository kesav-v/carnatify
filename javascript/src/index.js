import init, { CarnaticNoteStreamWasm } from "../pkg/carnatify_wasm.js";

let initPromise = null;

export async function initCarnatify() {
  if (!initPromise) {
    initPromise = init();
  }
  return initPromise;
}

export class CarnaticNoteStream {
  static async from_notes(notes, pitch = "C", tempo = 90) {
    await initCarnatify();
    return new CarnaticNoteStream(notes, pitch, tempo);
  }

  constructor(notes, pitch = "C", tempo = 90) {
    this._inner = CarnaticNoteStreamWasm.from_notes_with_options(
      notes,
      pitch,
      tempo,
    );
  }

  next_note() {
    return this._inner.next_note();
  }

  collect_notes() {
    const notes = [];
    for (let n = this.next_note(); n != null; n = this.next_note()) {
      notes.push(n);
    }
    this.reset();
    return notes;
  }

  reset() {
    this._inner.reset();
  }
}
