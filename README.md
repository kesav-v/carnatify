# carnatify

Pure Rust Carnatic notation parser, with Python and JavaScript bindings.

## Notation

`S, r, R, g, G, m, M, P, d, D, n, N` — use `*` / `/` for octave, `,` for duration, `( … )` for speed.

See [`grammar/carnatic_notation_grammar.lark`](grammar/carnatic_notation_grammar.lark).

## Layout

```
crates/carnatify-core/   library
crates/carnatify-wasm/   WASM bindings
crates/carnatify-py/     Python bindings (PyO3)
python/                  pip package (maturin)
javascript/              npm package (wasm-pack)
examples/rust/           CLI example
examples/python/
examples/javascript/
```

## Examples

**Rust** — play or export:

```bash
cargo run -p carnatify-example -- "SRGMP"
cargo run -p carnatify-example -- "SRG" -o out.mid
```

**Python:**

```bash
cd python && maturin develop --release
python ../examples/python/example.py
```

**JavaScript** — open `examples/javascript/index.html` after:

```bash
cd javascript && npm run build
```

## License

MIT
