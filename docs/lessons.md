# Lessons learned (2026-10-27)

What building and playing the acoustic guitar taught us that should carry over to other instruments and other songs. Each lesson says where it came from, so it can be re-checked.

## Source data: what works best

Ranked by what we actually got out of each, best first. Formats we have not tried are marked untested.

| Source | What it gives | What it lacks | Our experience |
|---|---|---|---|
| **Guitar Pro 7/8 file** (.gp: a zip holding XML) | Everything in the next row, for every track at once, plus the tuning, a tempo map and section names, with no reading errors and no reading time | Nothing we needed. Older .gp3–5 and .gpx files and MusicXML are untested | **Best, tested 2026-10-27.** Sweet Child O' Mine: 181 bars, 3 guitar parts, 5,170 notes, imported in under a second (`tools/gpimport.py`). The pitch check passes 98–99% of the two rhythm parts' notes, with a shifted control of 0. On Slash's lead it passes 91% once pre-bends are counted; the rest are bends moving right after the pick and low notes under other strings. |
| **Guitar Pro 3/4/5 file** (.gp3 .gp4 .gp5: older binary formats, read with PyGuitarPro) | The same musical content: rhythm, ties, techniques, tuning, markers, repeats | Transcribers sometimes encode effects oddly (see lesson 21) | **Works, tested 2026-10-27** on Hotel California (Unplugged), a .gp3: 130 bars, 3 guitar parts including a capo-2 12-string, 3,148 notes. The pitch check passes 97%, 99.5% and 99% per part, with a shifted control of 0. |
| **Engraved notation + tab** (the PDF of a UG official tab) | Pitches **and** rhythm, ties, rests, staccato, tempo and tempo changes, swing marking, grace notes, bends (to the quarter tone), trills, exact chord voicings | Comes as images: ingesting it means reading by eye, about 100 bars in a few hours, with small judgment calls (grace-note timing, an ambiguous beam group) | **Best so far.** Layla from it played "great" to the founder. 1,492 of 1,526 notes checked. |
| **Plain-text tab** (the six-line ASCII format) | Pitches, string and fret, techniques (h p / \ ~ b x) | **Rhythm** (spacing is typed by hand and inconsistent), note ends (no rests), ties, tempo, voicings where it only names chords | Riffs on a regular grid came out well. Solo timing was guessed and audibly wrong. Imported automatically by `tools/tabimport.py`. |
| **Chord chart** | The progression | Everything else: voicings, rhythm, riffs, lead, bar lengths | Only generic accompaniment. The founder rightly heard it as "rhythm guitar only". |
| **Memory** | Whatever is remembered | Reliability | Fine for in-house tests (founder's call), but label every bar that comes from it. |
| **MIDI** (not tried) | Exact rhythm and pitch | Which string and fret, and the techniques: everything that makes a guitar a guitar | **Untested.** We would have to guess fingering and techniques. Weaker than tab for this instrument, maybe fine for keyboard-like ones. |

**Recommendation:** for each new piece, get a Guitar Pro file first (Songsterr and Ultimate Guitar Pro export them), then an engraved PDF, then a text tab. A text tab is enough for riffs on a regular grid; anything with lead lines or rhythmic subtlety needs notation.

**Guitar Pro file facts** (from alphaTab's open-source reader, checked 2026-10-27; they matched the file):
- bend values are 25 per quarter tone, so 50 per semitone and 100 = a full tone;
- bend offsets are a percentage of the note;
- slide flags are bits: 1 shift, 2 legato, 4 out down, 8 out up, 16 in from below, 32 in from above;
- a hammer-on or pull-off is marked on its destination note;
- string numbers must be checked against each note's own MIDI pitch, not assumed (here 0 = the lowest string).

**Guitar Pro 3–5 facts** (PyGuitarPro 0.11, read in its source):
- string 1 is the highest;
- durations are 960 ticks per quarter;
- bend values divide the file's raw value by 25, so they count quarter tones (semitones = value / 2, maximum 12 = 3 tones);
- the hammer-on/pull-off flag sits on the note it starts *from* (unlike GP7/8, which marks the destination).

## Universal lessons

### What a performance needs
1. **Rhythm matters as much as pitch.** The same notes at guessed timings sounded wrong; with the engraved rhythm they sounded right. Any source without note values needs a second source for timing, or the ear.
2. **Note ends matter as much as note starts.** Rests, staccato and ties decide what rings. The renderer models damping explicitly: the next pluck on that string, a finger lift, or a rest (`X`). A source that only gives onsets loses half the articulation.
3. **Techniques are first-class, not decorations.** Hammer-ons, pull-offs, slides, bends, vibrato, trills and dead notes are each their own excitation or pitch movement in the model. Approximating them with plain plucks was audibly wrong.
4. **Swing and tempo marks are data.** ♫ = ♩♪ (a full triplet shuffle) and tempo changes (92 → 80 → 97 in Layla) are part of the score, not interpretation.

### What the player adds
5. **Hand behaviour depends on style.** Lifting fingers at a chord change suits chordal picking, and it cut lead lines and riffs short when it was applied everywhere. Such rules have to be opt-in per passage (`shape_damping`).
6. **Roles have weights.** In fingerstyle the thumb plays the bass heavier than the fingers play the treble (`thumb_weight`). Without that, the founder heard the note weights as "reversed".
7. **Several players need separate hands.** Two guitars on one timeline damped each other's strings until each part got its own state (`parts`).

### Tuning the instrument model
8. **Fix the physics, not the symptom.** A note the founder heard as out of tune was in tune but 8 dB too loud. A body mode radiated as if it breathed when it is a bending mode. Per-mode radiation efficiency fixed it at the cause.
9. **Compare against what the listener hears.** The dry render matched the record "near identical". Our synthetic room moved it away, because the record already carries its own production. Keep rooms optional.
10. **The founder's ear is the arbiter.** The meters catch errors (a wrong pitch, a broken check); they don't judge quality. Tune by listening notes, and record each verdict in the instrument's doc.

### Reading machine sources
18. **Honour the source's semantics, not just its notes.** Guitar Pro plays each note for its written length, while our arranger lets strings ring. The importer therefore gives every note its duration and turns ties into continuations, not new picks.
19. **Tuning is part of the song.** Sweet Child is tuned a semitone down. The model retunes the same strings at lower tension, and the workbench relabels its fretboard to match.
20. **A pitch check must include what the source asks for.** A note that starts bent (a pre-bend) sounds above its fret. Counting the starting bend moved Slash's lead from 87% to 91%. Before that, its shifted control "passed" 48 times, which is how we noticed.
21. **Sources encode effects in their own ways; interpret narrowly and say so.** One transcriber wrote a rolled 12-string chord as five 64th-note beats, each repeating the earlier notes as new picks. The importer treats a repeat straight after a 64th-or-shorter note as a continuation (109 cases in Hotel California). It is labelled as interpretation in the code.
22. **A capo can hide in the tuning.** Older files store "capo 2" as every string raised two semitones. A uniform raise is converted back to a capo, the same physics, so the song keeps one tuning.
23. **Async UIs need stale-answer guards.** A slower answer about the previously selected song overwrote the current one's parts. The browser test caught it; answers for a song that is no longer selected are now ignored.

### Checking our own work
11. **Every check needs a control that must fail.** The chord and note checks are each run against a shifted or wrong input. One first draft passed the wrong pitches almost as often as the right ones (an attack is broadband) and was rewritten.
12. **Fit the check to the texture.** A chord detector misread sparse fingerpicking, because one melody note dominated. A note-level check, the written pitch against its semitone neighbours over the time the note rings, works for every texture. Its window must be long enough to separate semitones at low pitches: about 95 ms failed below C3.
13. **A checker that can't evaluate must fail, not crash or skip.** Audio shorter than the chart now counts as misses.
14. **Validate transcriptions mechanically.** `tools/bars.py` rejects notes outside their bar, duplicate bars and unreadable lines. The arranger warns when a slide or release has nothing ringing. A clean run of both is the first proof that a transcription is self-consistent.

### Working practice
15. **Transcriptions disagree.** Two tabs of the same song differed in notes (Layla bar 13). Pick one source as authoritative per song and say which.
16. **Keep the pipeline layered and the layers generic:**

    source (tab, PDF, chart) → transcription text (`bars.py`) → song JSON (arranger) → timed plucks (events CSV) → renderer.

    A new instrument needs a new renderer and its own techniques. The layers above stay the same shape.
17. **Private material stays out of git.** Arrangements of other people's songs live in the git-ignored `listening/covers/`, and references live in `reference/`. Only the tools and our own examples are committed.

### Modelling circuits
24. **Test the whole chain at the level it is played, not only its parts.** The first amp model let every grid conduct from 0 V instead of from its cathode's bias voltage. Each valve stage passed its own test, but the whole amp half-wave clipped even a 10 mV signal (56% distortion at any level). A sweep of distortion against input level showed it at once. The amp now has a test that soft playing stays clean, and that test fails on the old bug.
25. **A test fixture can hide a missing-folder bug.** The song player's Play/Pause lived on the recordings row, which disables itself when `listening/renders` is empty. The browser test had a fixture recording, so it passed, while the founder's real setup had no renders folder and a dead Pause button. Test the empty case too.
26. **Never test on the user's live port.** A test server on the founder's port showed them fixture songs, and their Rescan couldn't find their own files. Tests now run on 8766.

### Reading machine sources (continued)
27. **Labels lie; instrument numbers don't.** Guitar Pro 3-5 store every track as a "guitar" (voice, piano, synth too). The General MIDI program (24-31 = guitars) picks the real ones.
28. **Songs can mix tunings.** Wish You Were Here has two guitars in standard and two in D G D G B D. A track that is another's tuning raised evenly takes a capo; any other keeps its own tuning, and the workbench renders such songs part by part on separately tuned guitars.
29. **When a file can't prove a convention, use a documented one and say so.** Guitar Pro 6 (.gpx) notes carry no pitch to check string order against. The importer uses GPIF's documented order (alphaTab's parser: string 0 is the lowest) and reports that it did; a drop-tuned power-chord check agreed (18 same-fret triads on the low strings, 0 on the high).
30. **A "wah" can be a pitch glide.** The founder heard wah on held notes where the model has no wah. Measuring pitch over time found hard-picked high notes starting 37 cents sharp and sliding down for two seconds, because every note was plucked with the same deflection however short the fretted string. Measure the thing that moves (pitch, level, band levels over time) before guessing at effects; the first suspect (polarization beating) measured too small to explain it.
31. **Name text crosses process boundaries in a code page.** On Windows, Python writes piped output in the ANSI code page unless told otherwise, so "Guitare électrique" reached the workbench as byte 0xE9 and matched no part. The workbench now runs Python with `-X utf8` (which still works under `-I`, unlike PYTHONIOENCODING).
32. **Every format has its own way of saying "let it ring".** The GP3-5 path read let-ring; the GP7/8 path didn't, so Stairway's 47 marked notes (136 with repeats) stopped at their written lengths: abrupt silence after held notes. When a feature is added for one format, check the others.
33. **Size buffers for what the model adds after the score ends.** The mix ended 1 s after the last release, but release tails run 4 s, so a tail could be cut at -33 dB. A scan for sudden drops found it only after it was made to count the silence after the file ends (it first passed the known-bad case).
34. **Don't add rules on top of the founder's.** The founder's rule was: every note is held for its written length and then gets the tail, always. I added "a re-picked string cuts the old tail" for physical reasons, and a length cap still skipped the tail on long notes. Both broke the rule and were heard as notes cutting to silence. Raise a physics argument; don't encode it over an instruction.
35. **Check what an "it ends quietly" test measures.** The renderer pads 50 ms of silence after the last sound, and two tests measured that padding, so they passed whatever happened. Each now measures the 50 ms before the sound ends, and fails on the bug it guards.
