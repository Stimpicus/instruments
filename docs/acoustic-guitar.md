# Acoustic guitar (steel string): design, numbers, verification

The model renders plucked notes of a 6-string steel-string flat-top in standard tuning (E2 A2 D3 G3 B3 E4) offline, and packs them as a multisample instrument. For Strudel or any other sampler, `guitar set <dir> acoustic` writes WAVs plus an `acoustic.instrument.json` region map; a Rust program can build the set in memory.

The research behind the numbers, with full citations and access notes, is in [acoustic-guitar-research.md](acoustic-guitar-research.md) (checked 2026-10-27).

## How it works

1. **Strings: modal, two polarizations.** Partial n of a string has frequency n·f0·√(1+B·n²). That is the stiff-string law, with B computed from tension, length and an effective stiffness diameter.
   - Each partial decays at its own rate. The internal loss uses Woodhouse's fitted model: η = (η_F + η_A/ω + η_B·B·n²)/(1 + B·n²), per string, for new strings.
   - The bridge adds a loss σ = 2·f0·Z0·Re Y(f) and a detuning −2·f0·Z0·Im Y(f). Both come from the body's mobility at that partial's frequency, so notes near strong body modes die fast and are pulled in pitch, as on a real guitar.
   - The vertical polarization (into the top) couples fully. The horizontal one couples at 0.1 of that, and sits a little lower in pitch.
2. **Pluck.** The string is released from a triangle at a point 13 cm from the saddle. That produces the pluck-position notches: every 5th harmonic on the open low E.
   - The contact width is a Gaussian, exp(−(nπw/L)²).
   - Velocity sets both the amplitude and the width: harder is louder and brighter.
   - A short noise burst adds the pick scrape.
3. **Tension modulation.** The pluck stretches the string, so the note starts sharp and settles as it decays.
4. **Body: modal.** Three measured low modes (the "triad") sit above a dense, seeded statistical mode set. Its bridge-velocity impulse response is rendered once, and every note's bridge force is FFT-convolved with it.
5. **Sympathetic strings.** The bridge's acceleration drives the five open strings. Their forces go back into the body, so a note rings the other strings that share its partials.
6. **Radiation.** The sound is the bridge velocity through a breathing-sphere filter (iω/ωc)/(1+iω/ωc) with ωc = 2π·250 Hz. Pressure follows acceleration below the corner and velocity above it.
7. **Set.** Every semitone E2 to C6 is played on the string a player in open position would use, at three dynamics.
   - The render is parallel and deterministic (about 1.5 s on 24 threads).
   - The whole set shares one gain, so the loudness differences stay physical.
   - Samples are capped at 4 s, with the last 0.4 s faded out.

8. **Legato.** Hammer-ons and pull-offs are their own excitations (`Tool::Hammer`, `Tool::Pull`): the string isn't plucked again.
   - A hammer-on carries on the string's existing vibration at the new fret. It is a wide, dark excitation at about 80% of the ringing note's strength, plus a faint fingertip tap.
   - A pull-off is a fingertip flick.
   - Both shapes are hypotheses for the ear.
   - Measured on the Lost for Words intro: every hammer-on raises its new pitch by 25-34 dB at the moment it lands (10/10). 112 of 115 written notes stand at least 6 dB above both semitone neighbours right after their onset, against 0/115 for the same test shifted a semitone.

9. **Bends, slides and vibrato** (`Moves` on a pluck, field 9 of a performance event).
   - All three scale every partial of the ringing string together.
   - A bend is a smooth pitch curve.
   - A slide steps a semitone at each fret it crosses, because the frets stop the string. Each crossing costs 3% of the amplitude (hypothesis). On wound strings, the finger rubs the windings and buzzes at slide speed / 0.5 mm (hypothesis).
   - Vibrato is an upward-only wobble, as on a steel string, easing in over 0.25 s.
   - Partials pushed above the band fall silent rather than alias.
   - Measured: a test lick lands 9/9 pitches within a quarter semitone (slide, bend, release, slide-in, hammer, pull-off), and its vibrato stays at or above the held pitch.
   - Tab notation is in `tools/arrange.py`; an example is in `docs/examples/technique-demo.json`.

## Numbers and where they come from

ADOPTED meets the research bar. DERIVED is arithmetic on adopted inputs. FITTED was matched to real recordings (GuitarSet, below). HYPOTHESIS is my choice, for the founder's ear to judge.

| Quantity | Value | Status |
|---|---|---|
| Strings | D'Addario EJ16 (.012–.053), tensions 103.1, 102.9, 132.6, 132.1, 127.7, 110.1 N (E4…E2) at a 25.4" scale | ADOPTED (manufacturer data), DERIVED for 25.4" |
| Scale length | 25.4" (645 mm) | HYPOTHESIS (typical dreadnought, not checked against a spec sheet) |
| Steel E | 200 GPa | HYPOTHESIS (190–210); checks against Woodhouse's measured EI within ~10% |
| Inharmonicity B | plain strings from gauge (1.95e-5 E4, 6.2e-5 B3); wound strings from effective diameters 0.0142/0.0145/0.0156/0.0186" back-solved from Woodhouse's measured B (low E 1.1e-4) | ADOPTED (low E: two independent measurements), the rest single-source |
| String losses η_F, η_A, η_B | Woodhouse 2012 Table I, new Martin 80/20 strings | ADOPTED (single primary, different brand) |
| Body triad | 99 Hz Q 37, 182 Hz Q 23, 350 Hz Q 40 (means of six steel-string guitars) | ADOPTED (Carcagno 2018) |
| Triad peak mobility | 0.03, 0.03, 0.02 m/(N·s) | ADOPTED (≈ −30 dB peaks), with individual peak heights a HYPOTHESIS |
| Dense body above 420 Hz | mean Re Y 0.0015 m/(N·s), bridge-mass roll-off above 8 kHz, mode spacing 35 Hz, Q 35 | FITTED (level, corner), HYPOTHESIS (spacing, Q) |
| 350 Hz mode radiation efficiency | 0.3 of the bridge's motion (the other modes 1) | HYPOTHESIS, from Carcagno's description of the mode as a transverse bending mode of the top. Set after the founder heard F4 (on that mode) as harsh. It cut F4 from +15 to +3.5 dB against C4 in the Lost for Words intro. |
| Pluck deflection on fretted notes | pluck_amplitude is the open string's; a fretted note is deflected by the compliance ratio d(len-d)/len against the open string's, i.e. a pluck is a force | Physics (a string's stiffness at the pluck point); the open-string amplitude stays a HYPOTHESIS. Added 2026-10-27 after the founder heard sustained high notes on the electric as a "wah": with fixed deflection, B fret 17 picked hard started 32 cents sharp and glided down for 2 s (tension modulation). Now 12-14 cents, against one measured steel-string glide of 12.5 cents (Hodgkinson 2010, low E, mf). Open strings unchanged; high-fret notes are a little quieter on both guitars. |
| Damping at release | extra loss 2.2 + 0.0004 w per second (w in rad/s) on the string and on the strings it set ringing. Every note is held for its written length and then gets the full tail, always (the founder's rule), including when its string is picked again and when it is held a long time | HYPOTHESIS. First a uniform 40 ms fade, then 22 + 0.004 w after the founder heard chord changes cut "too harshly"; then 10x longer at the founder's request (2026-10-27): 60 dB down in about 2.8 s at 100 Hz, 1.5 s at 1 kHz, 0.7 s at 3 kHz. Notes keep their written lengths. Tests: 0.3 s after the release it is more than 4 dB under the undamped note but less than 25 dB down; more than 40 dB down by 3 s; darker than the undamped note; a re-picked string's old note keeps its tail; a note held past `max_secs` and the last note of a song both fade out. Each of the last three fails on the code it replaced. |
| Radiation corner | 250 Hz breathing sphere | Woodhouse's ear-validated choice: HYPOTHESIS grade |
| Pluck point | 13 cm from the saddle | ADOPTED as typical (Traube & Smith test plucks: 12–14 cm) |
| Pick width | 2.5 mm soft → 1.0 mm hard | FITTED (high bands); the velocity slope is a HYPOTHESIS |
| Finger width | 9 mm soft → 6 mm hard | Woodhouse used 7.5 mm (by ear); the slope is a HYPOTHESIS |
| Pluck amplitude | 0.3 mm + 2.7 mm·v^1.5 | HYPOTHESIS; gives ~13 dB RMS between the soft and hard layers |
| Horizontal coupling, split | 0.1 of vertical; 5e-4 fractional split (vertical higher) | HYPOTHESIS (Woodhouse measured "several Hz" splits on nylon; no steel data) |
| Pick angle | 0.6 rad from the top's plane, ±0.15 rad per seed | HYPOTHESIS |
| Pick noise | 4 ms burst, 2% of peak force | HYPOTHESIS |

## Verification (2026-10-27)

Everything below is reproducible with the tools in `tools/` (`python -I`), against the code at the commit that adds this file.

- **Unit tests (`cargo test --release`, 9 tests):**
  - Pitch: open A, D, high E and G fret 7 are within 2 cents, with a negative control showing the check fails when the target is 20 cents off.
  - Inharmonicity matches the published low-E and high-E B within 10%.
  - Renders are deterministic and finite, and the seed varies them.
  - Harder plucks are louder and brighter.
  - Every key and velocity maps to exactly one sample, and the set is normalized to its loudest sample.
  - FFT convolution matches direct convolution (with a negative control).
- **The analyzer recovers the model's physics** (`tools/analyze.py`):
  - B measured from the render: 1.07e-4 on the low E (input 1.1e-4) and 1.91e-5 on the high E (input 1.95e-5).
  - Pluck-position notches fall at harmonics 5, 10, 15 and 20.
- **Pitch of the whole set:** 135 samples, median of 0.5 s windows from 0.8 to 2.3 s.
  - Median error 0.60 cents; worst +7.1 cents.
  - The worst is G2 (low E, fret 3), which sits on the 99 Hz body mode. Its vertical polarization is pulled 9 cents flat and dies in 1.5 s; the horizontal one carries the tail. Tuning is weighted to what is heard over time, so the tail ends a few cents sharp. A real guitar does the same thing at a body resonance, though perhaps less.
  - A single-window measurement can be fooled by the beating between polarizations. One window on F3 read +10 cents while the true pitch was +0.5 to +1.5.
- **Against a real guitar.**
  - **Reference:** GuitarSet v1.1.0 (CC BY 4.0) mic recordings. 366 notes were isolated by annotation (no other onset from 0.3 s before to 1 s after); 136 were used, after dropping notes annotated shorter than 0.9 s (player-damped) and capping at 25 per string.
  - **Method:** model at velocity 0.55 with a pick, same string and fret, `tools/compare.py`.
  - **Decay** (dB/s, 0.1–0.9 s), model vs real per string: −13.9/−13.5, −13.6/−14.1, −14.1/−16.3, −12.3/−10.0, −13.2/−12.1, −17.6/−16.7.
  - **Spectral shape:** within about ±8 dB per octave band from 250 Hz to 8 kHz on strings 1–4 and 6. The B string is up to 10 dB darker at 1 kHz and 8 kHz.
  - **Not comparable:** the 125 Hz band under the treble strings. The real recordings carry ringing from earlier notes in the music, so that band was not fitted.
  - **Limits:** this is one guitar of unknown model, one microphone and one room. Matching it says "in the family of real acoustic guitars", not "a dreadnought".
- **In a game engine's sampler:** the set plays as a sampled instrument, fingerpicked and strummed, without errors (checked in our own engine), and in Strudel through `.sfz`.
- **By ear, 2026-10-27:** the founder compared the Lost for Words acoustic intro (tab arrangement, fingerstyle with hammer-ons, dry, after the 350 Hz radiation fix and the thumb weight) with the Spotify release and called it "near identical". With the synthetic room on, it was further from the record, so the room is off by default in the workbench.
- **Earlier:** before that pass, the founder heard the hot F4 as out of tune and harsh, and the note weights as reversed. Both were fixed (see "Known gaps"). The founder's ear decides quality; the measurements only catch errors.

## Known gaps and next knobs

- **Below about 300 Hz** string and body modes really "veer" (Woodhouse). The model treats coupling as a perturbation, which is good above that and approximate below it.
- **Hot and dead notes:** 2026-10-27, the founder heard the hot F4 as out of tune and harsh. Its pitch measured +0.6 cents (in tune), but it peaked 8 dB above its neighbours. Per-mode radiation efficiency now turns the 350 Hz bending mode down to 0.3 (a workbench slider). The original finding follows.
- **Hot and dead notes are strong.**
  - In the mid layer, the RMS over the first 0.5 s ranges from −16 to −32 dB across the neck, and neighbouring notes differ by up to 9 dB.
  - The loudest are notes whose fundamental sits on a body mode: F3/F#3 on 182 Hz, F4 on 350 Hz.
  - Real guitars do this, but every body mode here radiates through the same breathing-sphere filter. The 350 Hz mode is a transverse bending mode of the top (Carcagno), which should radiate less efficiently, so F4 is probably exaggerated.
  - Per-mode radiation efficiency is the next modelling step, if the ear agrees.
- **No longitudinal "phantom" partials, no fret buzz, no finger squeak or release noise, no player damping.** Notes ring until the engine's release.
- **Mono and dry:** no room, no stereo microphone image.
- **Memory:** the standard set is 135 samples, about 95 MB as f32. An engine that converts samples to stereo frames roughly doubles that in RAM. Fewer layers, every second semitone, or a shorter cap would cut it, at a quality cost.
- **Knobs for tuning by ear:**
  - `Pluck` distance and angle
  - pick and finger width (`render`)
  - η_B per string (new vs worn strings: the biggest "dead string" cue)
  - `h_mobility` and `split` (beating and two-stage decay)
  - body triad frequencies and Q (a shift of about 1% is audible)
  - the dense mobility level
  - `radiation_hz`
