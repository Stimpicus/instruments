# Electric guitar and Marshall amp

A Les Paul-style solid body with humbuckers (`src/guitar/electric.rs`), played through a Marshall-style valve amp and 4x12 cabinet (`src/guitar/amp.rs`). It was built on 2026-10-27 so Sweet Child O' Mine can be heard on the instrument it was recorded on, from the same Guitar Pro file as the acoustic.

Sources and their grades are in [electric-guitar-research.md](electric-guitar-research.md); the keys below (Z7.7 and so on) point into its source list.

**Grades:**
- **ADOPTED:** one authoritative primary, or two independent sources that agree.
- **DERIVED:** arithmetic from adopted inputs.
- **HYPOTHESIS:** anything weaker, including every value set by judgement. Tune these by ear.

## What is modelled

```
pluck → modal strings (same model as the acoustic, solid-body losses)
      → humbucker: each mode weighted by its shape at the two pole rows, velocity sensed
      → coil + pots + cable: resonant low-pass
      → Marshall preamp at 4x sample rate: V1A → gain pot → V1B cold clipper → V2A
          (each stage Koren's 12AX7 on its load line; coupling caps with grid current = bias shift)
      → tone stack (Yeh & Smith, Marshall values) → presence → master → power amp soft clip with sag
      → back to 44.1 kHz → synthetic 4x12 cabinet with a close mic
```

Each guitar part goes through its own amp; the amps' outputs are mixed. Distortion therefore mixes notes within a part, never across parts.

## Numbers

| Part | Value | Grade |
|---|---|---|
| Scale | 24.75 in (628.65 mm) | HYPOTHESIS strength (Wikipedia) |
| Strings | Ernie Ball Slash set, 11-48 | ADOPTED for his signature set; what he used in 1987 is undocumented |
| Tensions | D'Addario 10-46 chart × gauge² × (24.75/25.5)²: 77–89 N | DERIVED; a hypothesis until the 11-48 unit weights are read |
| Low E decay | fundamental T30 25 s, 1 kHz partial T30 7 s (fresh strings) | ADOPTED (Zollner 7.7). The test measures 24.4 s and 7.7 s, but it checks that the model reaches the targets set from those same numbers. It is not an independent validation. |
| High E decay | fundamental T30 4.4 s | DERIVED (Zollner, Les Paul) |
| Strings in between | interpolated on log frequency | HYPOTHESIS |
| Used strings | not applied: the model's strings are fresh (used ones decay 50–80% faster) | your call |
| Body | neck resonance 220 Hz, bridge 2 kHz as narrow extra losses; Q and level | frequencies ADOPTED (Zollner 7.7); Q and level HYPOTHESIS |
| Pickup sensing | string velocity; horizontal motion at 0.15 of vertical | velocity ADOPTED (Horton & Moore 2009); 0.15 HYPOTHESIS |
| Pole rows | 19 mm apart; slug row 2.5 dB hotter, towards the bridge | spacing ADOPTED (Zollner 5.4); orientation HYPOTHESIS |
| Pole window | Gaussian, σ = 2.5 mm | shape ADOPTED, width HYPOTHESIS |
| Positions | neck 151 mm, bridge 51 mm from the saddles | HYPOTHESIS (one forum measurement of a '59) |
| Coil circuit | L 4.5 H, R 8 kΩ, C 800 pF (coil plus cable), load 200 kΩ → 2.65 kHz, Q 2.1 | HYPOTHESIS inputs, DERIVED resonance |
| Output level | 0.04 V per m/s of string velocity: Slash's part peaks at 1.0 V, rms 0.11 V | HYPOTHESIS (set for a plausible humbucker level) |
| String balance | each string's sensitivity set so the open strings give equal pickup level | HYPOTHESIS. Velocity sensing alone made the open high E 12 dB louder than the low E, and the top of the neck 21 dB louder; the founder heard high notes (F5, F#5, E4) as piercing. A real guitar evens this out with less steel in thin strings and with pole heights. After the amp the high E still sits about 8 dB over the low E in rms (the amp and cab trim bass). Within a string, level still rises about 8 dB over 17 frets. Measure with `cargo run --release --example note_levels`. |
| 12AX7 | Koren: μ 100, X 1.4, kG1 1060, kP 600, kVB 300, grid R 2 kΩ | ADOPTED (Koren) |
| Preamp | B+ 300 V; plates 100 k; V1A 2.7 k bypassed; V1B 10 k unbypassed (cold clipper); V2A 820 Ω bypassed; coupling 22 nF; 470 k ∥ 470 pF into the gain pot | HYPOTHESIS (2203 forum and plugin values; no factory schematic read) |
| Tone stack | Yeh & Smith transfer function; R1 220 k, R2 1 M, R3 22 k, R4 33 k, C1 470 pF, C2 = C3 22 nF | topology ADOPTED; values weak, three of them in conflict |
| Power amp | symmetric tanh soft clip; presence as a treble shelf; 20% sag over 50 ms | HYPOTHESIS. This is not a circuit model of the phase inverter, the EL34s or the output transformer. |
| Cabinet | high-pass 80 Hz Q 1.2; +2 dB at 120 Hz; −3 dB at 800 Hz; +4 dB at 2.5 kHz; 4th-order low-pass at 5 kHz | shape follows Celestion data (75–85 Hz resonance, 5 kHz limit) and Zollner (mid dip); every value HYPOTHESIS. No freely licensed measured Marshall 4x12 response exists. |
| Oversampling | 4x, 128-tap windowed-sinc resampling | measured: aliasing −41.9 dB with it, −21.6 dB without (3.1 kHz sine, gain 9) |

**Default panel:** gain 6, bass 5, mid 7, treble 6, presence 5, master 5 (HYPOTHESIS: a mid-gain crunch). The workbench knobs read 0–10 like the amp's.

How the amp responds (a 330 Hz sine, distortion against level):

| Gain | 10 mV | 30 mV | 100 mV | 300 mV | 1 V |
|---|---|---|---|---|---|
| 3 | 0% | 0% | 1% | 5% | 33% |
| 6 | 1% | 2% | 10% | 44% | 67% |
| 9 | 2% | 15% | 55% | 56% | 98% |

Measured with `cargo run --release --example amp_curve`. Soft playing cleans up and digging in breaks up, which is what the edge-of-breakup research says listeners notice most.

## What Sweet Child doesn't document

Which pickup Slash used on the intro, whether there is wah on the solo, any delay or reverb on the record, and Izzy's whole rig are all undocumented (see research §5). The pickup selector and the amp's knobs are there for your ear.

## Not modelled yet

- the cathode follower's clipping, and the phase inverter, power valves and output transformer as circuits;
- sympathetic ringing of the open strings (a solid body passes little energy to them);
- pick and slide noise heard through the pickup;
- stiffness raising the humbucker notch (Zollner: a lot). The model's mode shapes are ideal sines, so the notch sits where the pole spacing alone puts it.
- wah, delay, reverb;
- the single-note "real vs model" blind trials compare against acoustic GuitarSet notes. Run them with the instrument set to acoustic.

## Tests

`cargo test --release --lib electric amp`:
- the open strings are balanced at the pickup (against an unbalanced control);
- a hard-picked high note starts less than 20 cents sharp (it was +29 to +37 before plucks became force-based; fails on that code);
- a pickup at a quarter of the string misses the 4th harmonic (−101 dB against a control position);
- the humbucker notches where its poles are half a wavelength apart (−15.8 dB against a single coil);
- sustain as measured, at least 3x longer than the acoustic;
- the neck pickup is darker than the bridge;
- the coil resonance sits where the components put it;
- the tone stack has its known shape, and its digital version is within 0.5 dB of the analog;
- a triode is linear when quiet and clips unevenly when driven, against a symmetric-clipper control;
- soft playing is clean and hard playing breaks up (fails on the grid bug of lesson 24);
- oversampling keeps aliases down;
- a loud note leaves the preamp briefly blocked.
