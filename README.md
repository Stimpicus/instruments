# instruments

Physically modelled instruments, rendered offline into multisamples that a sampler can play. Rust, no dependencies, no audio device.

| Instrument | Name | Docs |
|---|---|---|
| Steel-string acoustic guitar (6 strings, standard tuning) | `acoustic` | [docs/acoustic-guitar.md](docs/acoustic-guitar.md) |
| Electric guitar (Les Paul-style, humbuckers) through a Marshall-style amp and 4x12 | workbench and `guitar perform` only | [docs/electric-guitar.md](docs/electric-guitar.md) |

Lessons that carry over to other instruments and songs, and which source data works best: [docs/lessons.md](docs/lessons.md).

## Use

- **As a library:** depend on this crate (it has no dependencies of its own) and call `guitar::set::render_set` to build the multisample set in memory: about 1.5 s on 24 threads.
- **In a sampler** (Strudel or any engine that reads region maps): `cargo run --release --bin guitar -- set <dir> acoustic` writes `<dir>/acoustic.instrument.json` plus `<dir>/acoustic/*.wav`: key and velocity ranges, root keys, velocity curves and release per region.
- **One note:** `guitar note <string 0-5> <fret> <velocity 0-1> out.wav [pick|finger] [seed]`. Set `GUITAR_GAIN=<k>` for a fixed gain, so dynamics can be compared.
- **From a Guitar Pro file (best source):** `python -I tools/gpimport.py song.gp song.json` (also `.gpx` from Guitar Pro 6, and `.gp3`/`.gp4`/`.gp5`, which need `pip install pyguitarpro`; `.gtp` from Guitar Pro 1/2 is not supported) turns each guitar track into a part, with the exact rhythm, tempo map, sections, tuning and techniques. `--list` shows the tracks; `--tracks 2,3` picks some.
- **From a transcription:** `python -I tools/bars.py song.txt song.json`, one bar per line in tab notation (for engraved PDFs read by eye).
- **From a tab:** `python -I tools/tabimport.py tab.txt` reads a plain-text tab in the usual six-line format into arranger sections, one per bar, grouped per block (riff, solo...). It understands h p / \ ~ b x. A song file then strings them into a form, with `parts` for several guitars at once. Tabs have no note values, so timing comes from the tab's spacing: three characters per eighth note where the bar fits that grid, otherwise spread evenly over the bar.
- **A performance:** `python -I tools/arrange.py events song.json plucks.csv`, then `guitar perform plucks.csv "<name>"` writes `<name> (dry).wav` and `<name> (room).wav`. Add `0.5 electric:neck` (or `:bridge`, `:both`) to play it on the electric through the amp at its default settings. Only needed when you want files: the workbench renders songs on demand.
- **Diagnostics:** `guitar losses <string> <fret>` prints the per-partial loss breakdown.

## Workbench (listening and tuning)

Double-click `workbench.cmd`, or run `cargo run --release --bin workbench`, then open http://127.0.0.1:8765. It binds to this machine only.

- **Guitar Pro files:** every `.gp`, `.gpx`, `.gp3`, `.gp4` or `.gp5` file in `reference/tracks/` (or `--gp-dir <folder>`) is listed under "Guitar Pro files". It is imported and rendered in memory, with its guitar tracks as parts; drop new files in and press Rescan.
- **Songs:** song descriptions in `listening/covers/*.json` (format in `tools/arrange.py`) are arranged and played straight from the model with the current knob settings, in memory, in a few seconds per song. Each note sits on the string and fret a player would use and rings until that string is played again or the fretting finger moves. A room is optional. While a song plays, a scrolling tab roll shows each string's notes (fret numbers, how long each rings, the current chord) and the fretboard lights the ringing notes.
- **Song player:** the last rendered song, with Play/Pause (Stop or Escape also pause; Play resumes), a waveform, a playhead and click-to-seek. Notes saved while listening record the song and the time.
- **Acoustic or electric:** the instrument selector switches everything you play, notes and songs, to the Les Paul through the Marshall, with a pickup selector and the amp's six knobs (0–10). With several parts, each goes through its own amp.
- **Play:** a fretboard, open chords with strums, and progressions (strum patterns, arpeggio, Travis picking) at any BPM. A new note on a string damps the one ringing there.
- **Shape the sound:** sliders for every model knob, re-rendered live. Store two settings as A and B and compare them, openly or blind.
- **See it:** a waveform and spectrogram of the last sound, plus pitch, B, peak and render time.
- **Real vs model:** blind trials against the GuitarSet notes in `reference/guitarset-notes`, and a browser to play any real note next to the model.
- **Notes for Claude:** saved with the settings to `listening/log.jsonl`. Every blind answer goes there too, so the next tuning pass starts from what was heard.

`reference/` and `listening/` are not committed. Rebuild the references with `tools/guitarset_notes.py` from GuitarSet's annotation and audio_mono-mic archives.

## Measuring

`tools/` holds Python (numpy, scipy). Run it with `python -I`.

- `analyze.py <wav> <f0>`: pitch, inharmonicity, per-partial T60, centroid, glide, spectrogram PNG.
- `guitarset_notes.py`: isolates single notes from GuitarSet (CC BY 4.0) for reference.
- `compare.py`: compares the model to those notes, band by band and string by string.

Reference recordings are not in the repo. Download GuitarSet v1.1.0 from https://zenodo.org/records/3371780.

## License
GPL-3.0 (see `LICENSE`). You may use, change and share this code, but anything you distribute that is built
on it must be released under the GPL too. The reference data the tools can download (GuitarSet, the Marshall
JVM410H dataset) are not part of this repository and keep their own licenses (both CC BY 4.0).
