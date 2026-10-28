# Electric guitar + Marshall amp: research brief for the modal synth

All sources were checked on 2026-10-27. Source keys like [Z5.2] point to the source list in §8.

Grades used in this brief:
- **ADOPTED** means one authoritative primary or two independent credible sources agree.
- **HYPOTHESIS** means anything weaker.
- **DERIVED** means my own arithmetic from adopted inputs. It is only as good as those inputs.

How I read the PDFs: WebFetch could not read them, so I ran the locally installed `pdftotext` on the PDFs WebFetch had saved. No downloaded code was run.

## Bottom line first

- **The biggest gap is the amplifier circuit values.** I could not read a Marshall factory schematic for the 2203 or the 1959, because they circulate as images. Every preamp resistor and capacitor value below comes from hobbyist analysis, forum posts or the CAPS plugin source. Treat all of them as HYPOTHESIS except the tone-stack topology and the 12AX7 datasheet constants.
- **Slash's Appetite amp is not a JCM800 2203 anyway.** The best account says it was a *modified 1959-family Super Lead* rented from S.I.R. Sources conflict on which unit it was (#39 or #36) and on what the modifications were. Slash himself says he doesn't know. Any "JCM800" model is a stand-in.
- **Sweet Child specifics are mostly undocumented.** I found no primary source for "neck pickup on the intro", "wah on the solo", or for any delay or reverb on the record. Those claims come from tutorial sites and are HYPOTHESIS; use your ears on the record. Izzy Stradlin's rig is undocumented in everything I could read.
- **The parts I'd trust are from Zollner's measurements and the Yeh & Smith paper.** These cover the string decay data, humbucker geometry (19 mm pole spacing), the comb notch, the aperture, the coil R/L/C of a real 1968 Gibson humbucker, the speaker specs, the tone-stack transfer function and the 12AX7 constants.
- **Usable CC BY 4.0 reference data exists.** The best is the *Marshall JVM 410H dataset*: DI input plus the amp output, recorded through a reactive load with no cab. The only CC-licensed cab IR I found is one CC BY file of unknown cabinets. I found no CC0 or CC-BY Marshall 4x12 IR.

---

## 1. Solid-body string behaviour

### 1.1 Decay (T30 = time for a partial to fall 30 dB; T60 ≈ 2·T30 if the decay is exponential, DERIVED)

| Quantity | Value | Conditions | Grade / source |
|---|---|---|---|
| E2 fundamental (82.4 Hz) T30 | **25 s** | US Stratocaster, factory-fresh 10-46, open string, "best case" | ADOPTED (measured, primary) [Z7.7] |
| E2 4th partial (330 Hz) T30 | ≈25 s ("nearly the same time") | same | ADOPTED [Z7.7] |
| E2 12th partial (1 kHz) T30 | **7 s** | same | ADOPTED [Z7.7] |
| Theoretical ceiling from air radiation plus internal loss | E2 (.046") ≈ **80 s**; E4 (.009") ≈ **6.8 s** | Stokes air-damping model. "Seldom if ever reached" in measurement | ADOPTED as a ceiling [Z7.7] |
| Frequency law (air term) | T30 ∝ 1/√f; heavier strings give longer sustain | model | ADOPTED [Z7.7] |
| Les Paul Historic E4 fundamental | ≈ **4.4 s** T30 (detuning one semitone would "halve" it to 2.2 s); can exceed **7 s** depending on how the guitar is supported | measured conductance at the nut plus a calculation | DERIVED from [Z7.7] |
| Les Paul selective absorbers | neck resonance **≈220 Hz**; bridge resonance **≈2 kHz** | Les Paul Classic | ADOPTED [Z7.7] |
| Used strings | decay time **−50 to −80 %** versus fresh, worst in the treble; very old string T30 falls ∝ f⁻³ | Les Paul Classic A string; old E2 on a stone table | ADOPTED [Z7.7] |
| Bearing-loss scale | 1 % energy lost per reflection at each end → **8.7 dB/s** at 100 Hz, 17.4 dB/s at 200 Hz | worked example | ADOPTED [Z7.7] |
| Where losses come from on a solid body | Mostly radiation plus internal loss (and winding friction on wound strings). Bridge, neck and body add only "small, frequency-selective" dips. The neck of a solid body is relatively flexible; the bridge is "relatively immobile" (Fleischer, cited by Zollner) | | ADOPTED [Z7.7] |
| Acoustic-guitar comparison numbers | **not obtained** | Fleischer & Zwicker (Acustica 1998) is paywalled and unread | GAP |

**Model implication.** Your acoustic model's bridge/body coupling term becomes small and very selective on a solid body. Set the per-mode loss as:

1/τ_n = σ_air(f_n) + σ_int(f_n) + Σ_k dip_k(f_n)

- σ_air ∝ √f.
- σ_int grows with f and is larger for wound strings.
- Add narrow extra-loss notches at about 220 Hz (neck) and about 2 kHz (bridge) for a Les Paul.
- Start from T30 ≈ 25 s at the low E fundamental and about 7 s at 1 kHz, then shorten by 50–80 % for "not fresh" strings. That last step is your call.

Also model **two polarizations**. They beat, and the vertical (fretboard-normal) one dominates the pickup output (§2.1).

### 1.2 Scale, tuning, strings

| Item | Value | Grade / source |
|---|---|---|
| Les Paul scale | 24.75 in = **628.65 mm** | HYPOTHESIS-strength: Wikipedia only; a Gibson spec page was not read [WLP]. Zollner's humbucker tests rescale to a 63 cm scale [Z5.10] |
| E♭ tuning fundamentals (A4 = 440 Hz) | 77.78, 103.83, 138.59, 185.00, 233.08, 311.13 Hz | arithmetic |
| D'Addario EXL110 10-46 tensions (25.5 in, E standard) | E .010 16.2 · B .013 15.4 · G .017 16.6 · D .026 18.4 · A .036 19.0 · E .046 16.9 lbf | ADOPTED. Two retailer reproductions of the D'Addario chart agree; a third differs on A/E (19.5/17.5). D'Addario's own PDF now redirects to a calculator [DAD] |
| Same set on a Les Paul in E♭ | × (24.75/25.5)²·2^(−2/12) = **×0.839** → 13.6, 12.9, 13.9, 15.4, 15.9, 14.2 lbf | DERIVED. Tension formula T = UW·(2Lf)²/386.4 (lb, in, Hz) is from D'Addario [DAD] |
| Slash signature set (Ernie Ball, 2020) | **.011 .014 .018 .028 .038 .048** | ADOPTED. Ernie Ball shop page plus retailer [EB] |
| Strings Slash used *on Appetite (1987)* | **undocumented** | GAP |
| 11-48 tensions | not obtained (unit weights unread) | GAP |

---

## 2. Magnetic pickups

### 2.1 What the pickup senses

- **It senses string velocity.** The flux through the coil depends on string position; Faraday's law turns its time derivative into a voltage. ADOPTED: Horton & Moore 2009 [HM] and Guadagnin et al. 2017 JAES (the latter from its abstract via search; full text unread).
- **Vertical (fretboard-normal) motion dominates.** Horizontal motion contributes a frequency-doubled component. The flux-versus-distance curve is nonlinear and adds harmonics [HM].
- **Sensitivity falls with distance.** At about 3 mm string-to-pole spacing, output drops roughly **3–4 dB per mm** [Z5.4.4].
- Paiva, Pakarinen & Välimäki 2012 (JAES 60(10)) models position, aperture, multiple pickups, the RLC resonance and flux nonlinearity. **It is paywalled; I read only the abstract** [PPV].

### 2.2 Position (comb) effect: modal form

In a modal synth this is exact and cheap. Each mode n with shape sin(nπx/L) contributes to the pickup output in proportion to its velocity times the aperture-weighted mode shape at the pickup:

- **Single point:** w_n = sin(nπ x_p / L), where x_p is measured from the bridge.
- **Humbucker, two pole rows a distance d apart:**
  w_n = sin(nπ x₁/L) + g·sin(nπ x₂/L), with x₂ = x₁ + d.
  For equal coils this equals 2 sin(nπ x_c/L)·cos(nπ d/2L). The cosine is the humbucker notch, and it falls where f_n ≈ c/(2d) (DERIVED). That matches Zollner's "half-wavelength between poles" explanation [Z5.4.4].
- **Stiffness pushes the notch up.** Use the stiff-string mode shapes and frequencies your model already has. Zollner reports that dispersion raises the notch a lot: when the pole spacing shrinks to 0.4×, the notch rises 4–5× instead of the 2.5× a non-dispersive string would give [Z5.10].

| Item | Value | Grade |
|---|---|---|
| Gibson humbucker pole spacing | **18–19 mm** (Zollner uses 19 mm in his model). Screw head ⌀5 mm, slug ⌀4.8 mm | ADOPTED [Z5.4.4][Z5.2][Z5.10] |
| Other spacings | Fender humbucker 20 mm; Gibson mini-humbucker 13 mm; single-coil-size humbuckers 6–9 mm; blade types 7.5 mm | ADOPTED [Z5.10] |
| E2 humbucker notch, measured/modelled | **≈3 kHz**. Together with the RLC low-pass it gives a "Chebyshev-like" treble cut | ADOPTED [Z5.10] |
| E♭2 notch, non-dispersive estimate | c = 2Lf₀ = 97.8 m/s → c/(2·0.019) ≈ **2.57 kHz** (the real value is higher because of dispersion) | DERIVED |
| E♭4 notch | c = 391 m/s → ≈10.3 kHz, above the pickup resonance, so irrelevant | DERIVED |
| Coil sensitivity ratio (g) | The slug coil is more sensitive than the screw coil by **2–3 dB** at the main maxima, so g ≈ 0.7–0.8 (DERIVED). The coupled side-lobe of a single coil sits 14–20 dB down | ADOPTED [Z5.4.4] |
| Notch depth | Complete if g = 1; 15 dB versus 25 dB depth makes "no significant difference" to listening | ADOPTED [Z5.4.4] |

**Pickup positions on a Les Paul.** I found no Gibson drawing. One forum measurement of a '59 Burst puts the neck pickup centre 18-13/16 in (477.8 mm) from the nut and the bridge pickup centre 22-3/4 in (577.9 mm) from the nut. On a 628.65 mm scale that is **neck ≈ 151 mm and bridge ≈ 51 mm from the saddle line**. This is HYPOTHESIS: n = 1 guitar, forum source, and saddle compensation is ignored [SD-fs]. A TDPRI thread with more measurements sits behind a TollBit paywall and was not read.

What this implies (DERIVED):
- The neck pickup sits at x/L ≈ 0.24, so harmonics 4 and 8 are strongly suppressed (|sin| ≈ 0.12 at n = 4).
- The bridge pickup at x/L ≈ 0.081 has its first null near n ≈ 12.
- Each pickup's two pole rows sit about ±9.5 mm from its centre. Which way the screw coil faces in Slash's guitar is undocumented (HYPOTHESIS).

### 2.3 Aperture (magnetic window)

- **The window is set by the pole piece, not by the coil size** (measured with a rotating-crank test bench). Its shape is roughly Gaussian [Z5.4.4].
- **It is a gap low-pass**, roughly a sin(x)/x response or a Gaussian. For single coils it has "little influence", because the electrical resonance already limits the band to about 5 kHz [Z5.4.4].
- **Implementation:** convolve the weight w(x) with a Gaussian of a few mm width centred on each pole row. Equivalently, multiply w_n by exp(−(k_n σ)²/2), where k_n = nπ/L and σ is a few mm. The window width in mm is only shown in figures (not extracted), so **σ ≈ 2–3 mm is HYPOTHESIS**.

### 2.4 Electrical model

Pickup equivalent circuit: source → R (copper) in series with L → C_pickup across the output, then the load C_cable ∥ R_vol ∥ R_tone ∥ R_amp. With C = C_pickup + C_cable and R_l = the parallel load:

H(s) = 1 / [ (1 + R/R_l) + s(L/R_l + R·C) + s²·L·C ]
f₀ = 1/(2π√(LC)),  Q ≈ 1 / (ω₀ (L/R_l + R·C))   (standard circuit algebra)

Zollner adds eddy-current loss resistors in parallel with L for the metal parts (cover, baseplate, screws). These lower Q [Z5.9].

| Item | Value | Grade / source |
|---|---|---|
| 1968 Gibson ES-335 bridge humbucker, **bobbin plus wire only** | screw coil 1.13 H / 4420 Ω / 47 pF; slug coil 0.98 H / 3928 Ω / 47 pF (the 11 % / 13 % mismatch is probably uneven turns) | ADOPTED (measured) [Z5.9] |
| Same guitar, neck humbucker coils | 0.96 H / 3660 Ω / 47 pF and 0.95 H / 3693 Ω / 40 pF | ADOPTED [Z5.9] |
| Screw coil alone, no metal | 1.125 H, 4420 Ω, 43 pF. Resonance **21 kHz** unloaded; **7.7 kHz** at 330 pF; **5.5 kHz** at 700 pF; **4.6 kHz** at 1030 pF | ADOPTED [Z5.9] |
| Metal parts | The screws *raise* L; the screws and baseplate *lower* Q; a cover lowers Q further; the magnet adds only 2–3 % to L | ADOPTED [Z5.9][Z5.2] |
| Coupling between the two coils | k ≈ **0.2**, so L_total = L₁ + L₂ + 2k√(L₁L₂). Measured series impedance is about 20 % above the plain sum | ADOPTED [Z5.9] |
| Assembled PAF-type total L | **≈4–5 H**. Evidence: a forum measurement of a Duncan '59 at 4.94 H; Lemme's 21-pickup average of 4.1 H / 12 kΩ / 130 pF (via a secondary site); Lemme's own forum example uses 5 H | HYPOTHESIS (no primary measurement of an assembled PAF was read) |
| Slash APH-2 (Alnico II Pro Slash) | DCR neck **7.92 kΩ**, bridge **8.53 kΩ**; "resonant peak" 6.4 / 6.0 kHz (load unknown) | HYPOTHESIS: retailer copies of the Duncan spec; another retailer says 8.9 kΩ. The Duncan spec sheet itself was unread [SDU] |
| What Slash's guitar actually had | Seymour Duncan **Alnico II Pro (APH-1)** "zebra" humbuckers in a Kris Derrig '59 replica, used on all GN'R albums | ADOPTED: Seymour Duncan blog 2016 plus Guitar.com 2021 [SDB][GC] |
| Typical loaded PAF resonance | 2.0–2.5 kHz | HYPOTHESIS (one Premier Guitar column, via search) |
| Pots / caps | Pots are typically 250 k or 500 k (±20 %). Tone cap is typically 20–50 nF; at full tone-down it pulls the resonance **below 1 kHz**. 1970s–80s Gibsons with 100 k tone / 300 k volume pots lose about **5 dB** of treble versus 500 k. Turning volume down slightly removes the resonance peak | ADOPTED [Z9.1][Z5.10] |
| Cable / amp | 330–1030 pF cable is the range Zollner tests; "long-ish" = 700 pF; tube amp input 1 MΩ | ADOPTED [Z5.10][Z9.1] |

**Worked starting point (DERIVED, HYPOTHESIS inputs).** Take L = 4.5 H, R = 8 kΩ, C = 100 pF + 700 pF, and R_l = 500k ∥ 500k ∥ 1M = 200 kΩ.
- That gives **f₀ ≈ 2.65 kHz and Q ≈ 2.1**, a peak of about +6 dB. This matches the 2–2.5 kHz figure.
- With the tone fully down (22 nF), f₀ ≈ 0.5 kHz.
- Fit L, C and Q to the record by ear. Zollner shows that the cable alone moves f₀ by about 1.5×.

---

## 3. Amplifier

### 3.1 What Slash actually used on Appetite (sources conflict, shown, not resolved)

| Claim | Source | Grade |
|---|---|---|
| He chose a **Marshall 1959T Super Lead Tremolo, "Stock #39"**, from S.I.R. LA. It was modified by Tim Caswell, had EL34s and an added master volume. #39 was out on tour with George Lynch, so S.I.R. secretly supplied **#36** (a Super Lead without tremolo), which Frank Levi modded with caps from donor amps to match. "Slash never seemed to notice." The whereabouts of the amp are unknown | Guitar.com, J. Beckner, 2021-11-09 [GC] | single secondary → HYPOTHESIS |
| "believed to have used a rented 100w Superlead that had been modified"; Slash: "It had been modded to some extent, but I don't know how or by whom" | MusicRadar (undated) [MR] | agrees with [GC] that it was a modified rented Super Lead, so **"modified rented 100 W Super Lead" is ADOPTED**; unit number and mods are not |
| Marshall AFD100 (2011): the "#34" mode = the **modified JCM800 2203** Slash used **since the late '80s**, i.e. post-Appetite; the "AFD" mode was voiced against the Appetite masters ("Night Train") | Marshall press release via Premier Guitar 2011-01-12 [PG-AFD] | ADOPTED for the post-Appetite rig only |

So the target circuit is the **1959 Super Lead family with an added master volume and unknown gain mods**. The 2203 is the closest documented circuit (it descends from series-cascaded, modded Plexis [RR]), but it is a stand-in.

### 3.2 Preamp: 2203 / 1959 values (all HYPOTHESIS unless marked; Marshall schematic primary not read)

| Node | Value | Source |
|---|---|---|
| Input | High input 1 MΩ, Low 68 k (Plexi section) | [RR] |
| V1 first stage (2203) | cathode **2.7 k ∥ 0.68 µF** | [RR] + GroupDIY forum ("2k7∥680nF"), so two weak agreeing sources |
| Cold-clipper stage | cathode **10 k, unbypassed** (Soldano used 39 k) | [RR] + GroupDIY + music-electronics-forum. Stage labels conflict (V1A versus V1B) |
| Next stage | cathode **820 Ω** (often ∥ 0.68 µF, which some describe as a mid-boost network) | [RR], forum |
| Mixer / attenuator | **470 k ∥ 470 pF** treble peaker. The 470 k into a 500 k gain pot "dumps 48 %" | [RR] |
| Bright cap on the volume pot | Robinette: Marshall used **0.001 µF**; family variants 100 / 390 / 470 pF | [RR] (conflicting variants) |
| Plate resistors | **100 k** typical (no primary read) | forum only |
| Cathode follower → tone stack | DC-coupled cathode follower (V2B). Yeh & Smith: the stack sees about a 1 k source and a 1 M load | [RR][YS] |
| 1959 (Plexi) | No master volume and no cold clipper; two parallel channels with two gain stages each. Bright channel cathode 2.7 k ∥ 0.68 µF with a 0.0022 µF coupling cap; normal channel 820 Ω ∥ 330 µF with a 0.022 µF coupling cap; 470 k mixers with a 470 pF bright cap | [RR] |

**12AX7 constants. ADOPTED:** the GE datasheet and the Svetlana datasheet agree. At 250 V and −2 V: **μ = 100, r_p = 62.5 kΩ, g_m = 1.6 mA/V, I_a = 1.2 mA** [12AX7]. EHX lists r_p 54.1 k at 300 V; Sovtek gives μ 85–110.

**Small-signal stage gain (DERIVED, ignoring the load).** A = μR_a / (R_a + r_p + (μ+1)R_k,unbypassed). With R_a = 100 k:
- Fully bypassed cathode: **A ≈ 61 (35.8 dB)**.
- 10 k unbypassed (cold clipper): **A ≈ 8.5 (18.6 dB)**, but it is biased near cut-off, so its large-signal behaviour is strongly asymmetric.
- The 2.7 k ∥ 0.68 µF cathode gives a low-frequency shelf. The zero is at 1/(2πR_kC_k) ≈ **87 Hz**; the pole is near **≈230 Hz** once the cathode's own resistance is included.
- The 820 Ω ∥ 0.68 µF zero is at about 285 Hz.
- Next-stage loading (470 k / 1 M) lowers each gain.

### 3.3 Tone stack (topology ADOPTED; Marshall values conflict)

The Marshall tone stack is the Fender 5F6-A Bassman stack with different values. This is ADOPTED from three sources: Yeh's CCRMA page says "most tone stacks (Fender, Marshall, Vox) are the same circuit… formulas… specific to the Bassman/Marshall circuit"; Zollner calls the JTM-45 stack "a direct copy" of the 5F6-A; and Miklánek et al. 2023 calls it the "FMV" topology [YSpage][Z10.3][MWVS].

| Part (Yeh & Smith naming) | Bassman 5F6-A | JTM-45 | JTM-50 change | 2203 per CAPS source | "81 2000 Lead" per CAPS |
|---|---|---|---|---|---|
| R1 treble pot | 250 k | = Bassman | — | **220 k** | 250 k |
| R2 bass pot (log) | 1 M | = | — | 1 M | 1 M |
| R3 mid pot | 25 k | = | — | **22 k** | 25 k |
| R4 slope | 56 k | = | **33 k** | 33 k | 56 k |
| C1 treble cap | 250 pF | = | **500 pF** | **470 pF** | 500 pF |
| C2, C3 | 20 nF | = | — | 22 nF | 22 nF |
| Source | [YS] (primary) | [Z10.3] | [Z10.3] | [CAPS] code comment | [CAPS] |

R4 = 33 k with C2 = C3 = 22 nF for the 2203 is supported by CAPS, plus a rig-talk post ("traditional 33k/.022/.022"), plus Zollner's JTM-50 R4. Treat it as a weak ADOPT. **C1 is 470 pF versus 500 pF, R1 is 220 k versus 250 k, and R3 is 22 k versus 25 k: unresolved conflicts.** Robinette gives a slope resistor range of 33–56 k. Miklánek et al. note that real component values deviate from the schematic and fitted them by optimization [MWVS].

**Transfer function, verbatim from Yeh & Smith 2006 [YS].** t = treble, m = mid, l = bass, each in [0, 1]; l is swept logarithmically.

```
H(s) = (b1 s + b2 s^2 + b3 s^3) / (a0 + a1 s + a2 s^2 + a3 s^3)
b1 = t C1R1 + m C3R3 + l(C1R2 + C2R2) + (C1R3 + C2R3)
b2 = t(C1C2R1R4 + C1C3R1R4) - m^2(C1C3R3^2 + C2C3R3^2) + m(C1C3R1R3 + C1C3R3^2 + C2C3R3^2)
     + l(C1C2R1R2 + C1C2R2R4 + C1C3R2R4) + lm(C1C3R2R3 + C2C3R2R3) + (C1C2R1R3 + C1C2R3R4 + C1C3R3R4)
b3 = lm(C1C2C3R1R2R3 + C1C2C3R2R3R4) - m^2(C1C2C3R1R3^2 + C1C2C3R3^2R4)
     + m(C1C2C3R1R3^2 + C1C2C3R3^2R4) + t C1C2C3R1R3R4 - tm C1C2C3R1R3R4 + tl C1C2C3R1R2R4
a0 = 1
a1 = (C1R1 + C1R3 + C2R3 + C2R4 + C3R4) + m C3R3 + l(C1R2 + C2R2)
a2 = m(C1C3R1R3 - C2C3R3R4 + C1C3R3^2 + C2C3R3^2) + lm(C1C3R2R3 + C2C3R2R3)
     - m^2(C1C3R3^2 + C2C3R3^2) + l(C1C2R2R4 + C1C2R1R2 + C1C3R2R4 + C2C3R2R4)
     + (C1C2R1R4 + C1C3R1R4 + C1C2R3R4 + C1C2R1R3 + C1C3R3R4 + C2C3R3R4)
a3 = lm(C1C2C3R1R2R3 + C1C2C3R2R3R4) - m^2(C1C2C3R1R3^2 + C1C2C3R3^2R4)
     + m(C1C2C3R3^2R4 + C1C2C3R1R3^2 - C1C2C3R1R3R4) + l C1C2C3R1R2R4 + C1C2C3R1R3R4
Bilinear, c = 2/T:
B0=-b1c-b2c^2-b3c^3  B1=-b1c+b2c^2+3b3c^3  B2=b1c+b2c^2-3b3c^3  B3=b1c-b2c^2+b3c^3
A0=-a0-a1c-a2c^2-a3c^3  A1=-3a0-a1c+a2c^2+3a3c^3  A2=-3a0+a1c+a2c^2-3a3c^3  A3=-a0+a1c-a2c^2+a3c^3
```

Facts from the paper:
- The three poles are all real, so the stack cannot resonate.
- There is a zero at DC, and a zero pair that can become complex, which produces the **mid notch**.
- The **treble control moves only the zeros**; bass and mid move the poles.
- The 1 k source and 1 M load do not change the response.
- At 44.1 kHz the error is under 0.5 dB for high-pass-like settings. For low-pass settings it reaches 3 dB at 13 kHz (6 kHz at fs = 20 kHz).
- Their sanity check: "a typical guitar speaker is from 100 Hz to 6000 Hz" [YS].

Zollner warns that the stack is not where an amp's character lives. The combination of stages does that, especially Marshall's presence-in-NFB and the 4x12 [Z10.3].

### 3.4 Phase inverter, power amp, NFB, presence

| Item | Value | Grade / source |
|---|---|---|
| Phase inverter | Long-tailed pair with a shared 470 Ω bias resistor; 2203 adds a 4.7 k NFB tail resistor in parallel with the presence pot. The LTP gives about ¼ the gain of two normal triode stages | HYPOTHESIS [RR] |
| Power stage | 100 W: **4 × EL34 push-pull, class AB** (the 50 W has 2). US export amps used 6550. Screen resistors 1 k / 5 W; bias-feed 56 k (47 k in US); grid leaks 220 k (150 k in US); solid-state rectifier | EL34s on the Appetite amp: [GC]. Rest: [RR] → HYPOTHESIS |
| NFB history | JTM-45: 27 k from the 16 Ω tap into a 5 k presence pot (strong NFB, ≈3× the Bassman's). Later Marshalls raise the feedback resistor 27 k → 47 k → **100 k** and move the tap 16 → 8 → **4 Ω**, so NFB decreases | ADOPTED [Z10.5.12]; 100 k / 4 Ω also in [RR] |
| Presence | Early circuits: 5 k pot; later: 22 k pot ∥ 4.7 k; presence cap 0.1 µF (some 0.68 µF). Presence cuts NFB at high frequency, so it acts as a treble shelf. It "loses its effect" once the power stage is overdriven | presence behaviour ADOPTED [Z10.5.12]; values HYPOTHESIS [RR] |
| Output transformer primary impedance (100 W, 4×EL34) | **not obtained** | GAP |
| Push-pull behaviour | Class AB gives crossover distortion and mostly cancels even harmonics. The output transformer adds distortion and **hysteresis**. Sag and blocking (grid current charging coupling caps → bias shift → momentary cut-off) are dynamic effects that static waveshapers cannot produce | ADOPTED [PY] |

### 3.5 DSP modelling approaches

| Approach | What it is | Source |
|---|---|---|
| Static waveshaping per stage | Memoryless curve per stage, e.g. the Line 6 patent curves: symmetric f(x) = (\|2x\| − x²)·sign(x), plus a piecewise asymmetric curve (the patent text has typos; the review gives corrected forms). **Oversample** to suppress aliasing: Line 6 TubeTone ran **8× at 249.6 kHz** | [PY] |
| Interleaved static shapers + filters | Measure each stage's static curve and filters; ignore loading between stages (Möller, Gromowski & Zölzer 2002) | [PY] |
| Dynamic / bias-shift models | Karjalainen et al. 2006: a V_gk→V_p table, a grid-current table, a bias-feedback loop, and a Miller low-pass. Pritchard 1991: polarity-dependent high-pass before an asymmetric shaper, then a second shaper with sag | [PY] |
| Koren triode | E₁ = (E_P/k_P)·ln(1 + exp(k_P(1/μ + E_G/√(k_VB + E_P²)))); I_P = (E₁^X / k_G1)(1 + sgn E₁). **12AX7: μ 100, X 1.4, k_G1 1060, k_P 600, k_VB 300, R_GI 2 k (grid current when E_G > 0), C_CG 2.3 pF, C_PG 2.4 pF, C_CP 0.9 pF** | ADOPTED, primary [KOR] (Glass Audio 1996; web version updated 2003-03-20) |
| Circuit solvers | WDF, DK-method state space with Newton iteration. A DAFx-16 WDF study of the 5F6-A family (including the JTM-45) says models without **grid current** cannot reproduce an overdriven preamp | DAFx-16 claim seen only in a search snippet → HYPOTHESIS |
| Grey-box | RNN for the nonlinear stages plus a white-box DK-method tone stack. It beat a black-box RNN with little data, on a Marshall JVM 410H | [MWVS] |
| Black-box neural | LSTM / WaveNet "snapshots". In a MUSHRA test (n = 24 trained listeners, Peavey 6505) the models were statistically indistinguishable from the real amp in realism | [CDS] |

**What matters most for perceived realism (HYPOTHESIS ranking, with evidence):**
1. **Speaker cab / mic response.** The cab is the last and strongest linear filter, with a 75–85 Hz resonance, a ~5 kHz cliff and resonant mid peaks [Z11.3][CEL]. Every listening study I read convolves with a cab IR before listening [CDS].
2. **Aliasing control.** Unsuppressed aliasing gives the "noisy, dissonant" sound of cheap emulations [PY].
3. **Edge-of-breakup dynamics** (memory, bias shift, grid current). In Cassidy & De Sena, high-gain excerpts hid the model differences, while the **edge-of-breakup excerpt exposed 5 of 6 models** [CDS]. Slash's tone is mid-gain, so this matters.
4. **Gain staging and the asymmetric cold-clipper stage**, rather than the exact tone-stack values. This is Zollner's view [Z10.3].

Pakarinen & Yeh: objective evaluation is "extremely difficult… the best way to rate… is by listening"; "careful tuning… can make a tremendous improvement" [PY].

---

## 4. Speaker cabinet and mic

| Item | Value | Grade / source |
|---|---|---|
| Celestion **G12M Greenback** | Fs **75 Hz**, range **75–5000 Hz**, 98 dB, 25 W | ADOPTED (Celestion sheet) [CEL] |
| Celestion **G12T-75** | Fs **85 Hz**, range **80–5000 Hz**, 97 dB, 75 W ("probably the world's top selling 4x12") | ADOPTED [CEL] |
| Which speakers were in Slash's Appetite cab | **unknown** (1980s Marshall 4x12s commonly had G12T-75; not sourced) | GAP / HYPOTHESIS |
| General 12" guitar speaker | Simple theory: power flat above resonance, falling 1/f² above about 600 Hz, with beaming compensating on axis. Measured on-axis: "decent level" to **5 kHz, then a nosedive**; cone break-up ripples from **1.5 kHz**; break-up modes "as low as 1 kHz" | ADOPTED [Z11.3] |
| Celestion character | A pronounced **mid-dip in the 600 Hz – 4 kHz** region; the sound is shaped by the Q of resonance peaks | ADOPTED [Z11.3] |
| Unit-to-unit spread | Two new Celestion Blues differ by about **±3 dB**; two original G12-80s by about **±5 dB** | ADOPTED [Z11.3] |
| Cabinet | The 4x12 is "bass-heavy"; cavity resonances depend on geometry; wood type "does not play any role" | ADOPTED [Z10.3][Z11.3] |
| Shure SM57 | 40–15,000 Hz, dynamic, cardioid, "carefully contoured presence rise". **The size and frequency of the presence peak are only in the graph image; not extracted** | ADOPTED range [SM57]; peak = GAP |
| Is a measured IR standard practice? | Research practice: measure the cab IR with an exponential sine sweep (Marshall 1960AV, V30s, Royer R-121, 20 mm off the dust cap) and convolve it as an LTI system [CDS]. Pakarinen & Yeh mention a simple low-pass as the early product approach [PY]. Measured-IR use in commercial products is widely known but **not sourced here** | partly ADOPTED |

**Practical approach.** Use a measured IR if a licensed one exists (see §6: none for a Marshall 4x12). Otherwise build a synthetic cab:
- a 2nd-order high-pass resonance at 75–85 Hz with Q ≈ 1–2;
- a few peaking EQs for break-up resonances between 1 and 4 kHz, with a mid-dip;
- a steep low-pass of 4th order or higher at about 5 kHz;
- an SM57 presence bump.

The Q values, the bump and the order are HYPOTHESIS: tune them by ear against the record.

---

## 5. "Sweet Child O' Mine": documented tone facts

| Claim | Source | Grade |
|---|---|---|
| Guitar: Derrig '59 replica with Duncan Alnico II Pros; amp: modified rented Super Lead | §2.4, §3.1 | ADOPTED / partly |
| The solo intro took "at least eight" takes | Guitar World "Dear Guitar Hero" (seen via search; page truncated on fetch) | HYPOTHESIS (not read directly) |
| Live (Illusion era): intro "distortion" from a Les Paul with the **rhythm (neck) pickup** full up into a 100 W Marshall | Slash, 2001 Snakepit fan Q&A, reposted on a fan archive | HYPOTHESIS (secondary repost; live, not the record) |
| Studio intro on the neck pickup with tone rolled back; solo moves to the bridge pickup with wah in the harmonic-minor section; wah "cocked" at about 60–70 % | Tutorial and gear sites (Gibson app lesson, songsfrommemory, guitarchalk); they **conflict** on bridge-pickup use and on wah sweep versus fixed | HYPOTHESIS |
| Wah pedal model / behaviour | Generic Cry Baby/VOX-type: an inductor-based (≈0.5 H) Miller-tuned band-pass; sweep about **0.44–2.3 kHz**, **Q ≈ 3.3–15** (measured VOX) | ADOPTED for the device [Z10.8.4]; its use on the record is HYPOTHESIS |
| Delay / reverb on the record | **nothing found** | GAP |
| Izzy Stradlin's guitar and amp on the track | **nothing found** (Guitar World Slash/Izzy feature would not load) | GAP |
| Production | Mike Clink producer; recorded 1987 in Los Angeles; band basics tracked live | Wikipedia + search snippets → HYPOTHESIS |

---

## 6. Freely licensed reference material (not downloaded)

| Resource | What | License (as shown on page) | Use |
|---|---|---|---|
| **Marshall JVM 410H Dataset**, Miklánek, Wright, Välimäki, Schimmel, Zenodo 2023-05-25, doi:10.5281/zenodo.7970723, https://zenodo.org/record/7970723 | 6 min of guitar and bass DI, plus amp output from the **OD1 channel** recorded at the speaker output into a **Two Notes Torpedo Captor 8 reactive load**, at 44.1 kHz. Includes tone-stack-setting sweeps. No cab | **CC BY 4.0** | **Best fit.** Validate the amp model (preamp + stack + power amp) against a real Marshall; add your own cab |
| **Guitar-TECHS**, Pedroza Villalobos et al., Zenodo 2025-04-06, https://zenodo.org/records/14963133 ; site https://guitar-techs.github.io | DI plus mic'd-amp signals, synchronized. Amps: Orange CR-60 (SM57), Yamaha YB-15, Orange CR-12. Not Marshall | **CC BY 4.0** (Zenodo and site) | Check the DI → mic'd amp chain. **Caveat:** the Zenodo page also says only the ego/exo capture audio is included, so confirm the DI and amp-mic files are in the archive |
| **EGFxSet**, Pedroza, Meza, Roman, Zenodo 2022-09-02, doi:10.5281/zenodo.7044411 | Clean Stratocaster tones, 5 pickup selections, every fret, plus 12 pedal effects. No amp | **CC BY 4.0** | Check pickup-position comb and RLC (single-coil, not humbucker) |
| EGDB rendered with BIAS FX2, Peng & Wu, Zenodo 2024, doi:10.5281/zenodo.12674910 | EGDB DI rendered through a commercial **amp simulator** | CC BY 4.0 (the original EGDB license is unstated) | Weak: the target is a simulator, not a real amp |
| Freesound "Bristol.wav", user mansardian, 2022-08-28, https://freesound.org/people/mansardian/sounds/648392/ | Guitar-cab IR: 2 unnamed cabinets, blend of R121, SM57, i5 and C414; 48 kHz / 24-bit | **CC BY 4.0** | The only CC-licensed cab IR found. Not a known Marshall 4x12 |
| Cassidy & De Sena 2023 paper (DAFx23) | Describes a measured Marshall 1960AV / V30 IR. The *paper* is CC BY 4.0; the data are said to be on the Institute of Sound Recording GitHub, **license unverified** | — | Verify before use |
| **Not acceptable** | ToneTwist AFx Mesa Mark V (**CC BY-NC 4.0**); TONE3000 IRs/captures (custom "T3K" license, no redistribution); Erik Ekholm "Metal Cabs" (license unstated); EGDB original (license unstated); IDMT-SMT (NC-ND) | | |

**Gap:** no CC0 or CC-BY measured **Marshall 4x12 / Celestion** IR found.

---

## 7. Model structure (put together)

```
pluck → modal string bank (two polarizations per mode; T60 per §1.1; E♭ tuning, 628.65 mm)
  → pickup: y(t) = Σ_n v_n(t)·W_n,  W_n = A_n·[sin(nπx1/L) + g·sin(nπx2/L)]
         A_n = Gaussian aperture factor; x1 ≈ 51 mm (bridge) or ≈ 151 mm (neck), x2 = x1 ± 19 mm; g ≈ 0.75
         (optional: mild static nonlinearity for flux-vs-distance)
  → pickup RLC + pots + cable: 2nd-order H(s) per §2.4 (f0 ≈ 2–3 kHz, Q ≈ 2 at full volume/tone)
  → [wah: band-pass 0.44–2.3 kHz, Q 3–15, if used]
  → preamp, oversampled ≥8×: stage1 (gain ≈ 35 dB, 87–230 Hz cathode shelf) → bright/470k∥470pF network
       → cold-clipper stage (gain ≈ 18 dB small-signal, strongly asymmetric, near cut-off)
       → stage 3 (820 Ω) → cathode follower (soft limiting on grid current)
       each stage: coupling-cap HPF + asymmetric waveshaper (or Koren triode solved per stage)
       + grid-current/bias-shift memory for edge-of-breakup realism
  → tone stack: Yeh–Smith 3rd-order IIR with Marshall values (R4 33 k, C2 = C3 22 nF; C1 470–500 pF; R1 220–250 k; R3 22–25 k)
  → master volume → LTP phase inverter → EL34 push-pull class AB (symmetric soft clip + crossover)
       with NFB loop (100 k from 4 Ω) and presence shelf; optional sag
  → cab: measured IR, or synthetic per §4 (75–85 Hz resonance, mid-dip, ~5 kHz cliff) + SM57 presence
```

---

## 8. Sources (all checked 2026-10-27)

- **[YS]** D. T. Yeh, J. O. Smith, "Discretization of the '59 Fender Bassman Tone Stack", Proc. DAFx-06, Montreal, 2006, pp. 1–6. https://ccrma.stanford.edu/~dtyeh/papers/yeh06_dafx.pdf. Read in full.
- **[YSpage]** D. T. Yeh, CCRMA tone stack page (undated). https://ccrma.stanford.edu/~dtyeh/tonestack/
- **[PY]** J. Pakarinen, D. T. Yeh, "A Review of Digital Techniques for Modeling Vacuum-Tube Guitar Amplifiers", Computer Music Journal 33(2):85–100, 2009, doi:10.1162/comj.2009.33.2.85. The official version is paywalled (MUSE); the brief was written from a third-party copy whose authorization is unclear, so it is not linked here.
- **[KOR]** N. L. Koren, "Improved vacuum tube models for SPICE simulations", Glass Audio 8(5), 1996; web version updated 2003-03-20. https://www.normankoren.com/Audio/Tubemodspice_article.html
- **[MWVS]** Š. Miklánek, A. Wright, V. Välimäki, J. Schimmel, "Neural Grey-Box Guitar Amplifier Modelling with Limited Data", DAFx23, pp. 151–158, 2023 (CC BY 4.0). https://www.dafx.de/paper-archive/2023/DAFx23_paper_52.pdf
- **[CDS]** W. J. Cassidy, E. De Sena, "Perceptual Evaluation and Genre-Specific Training of Deep Neural Network Models of a High-Gain Guitar Amplifier", DAFx23, 2023. https://www.dafx.de/paper-archive/2023/DAFx23_paper_40.pdf
- **[Z…]** M. Zollner, *Physics of the Electric Guitar* (English translation of *Physik der Elektrogitarre*, 2014), GITEC chapter PDFs, https://gitec-forum-eng.de/the-book/. The online versions omit some figures, so many numbers exist only as plots.
  - [Z1.6] poteg-1-6-string-decay.pdf
  - [Z7.6.3] poteg-7-6-3-decay-time-t-30.pdf
  - [Z7.7] poteg-7-7-absorption-of-string-oscillations.pdf
  - [Z5.2] poteg-5-2-humbucking-pus.pdf
  - [Z5.4.4] poteg-5-4-4-aperture.pdf
  - [Z5.9] poteg-5-9-equivalent-circuits.pdf
  - [Z5.10] poteg-5-10-transfer-behavior.pdf
  - [Z9.1] poteg-9-1-potentiometers.pdf
  - [Z10.3] poteg-10-03-tone-controls.pdf
  - [Z10.5.12] poteg-10-05-12-vox-marshall-fender.pdf
  - [Z10.8.4] poteg-10-08-04-wah.pdf
  - [Z11.3] poteg-11-3-frequency-response.pdf
  - **Not read:** 5.15 database and 10.10.7 Vox/Fender/Marshall comparison (both over the 10 MB fetch limit); the German print edition.
- **[HM]** N. G. Horton, T. R. Moore, "Modeling the magnetic pickup of an electric guitar", Am. J. Phys. 77(2):144–150, 2009. https://users.manchester.edu/Facstaff/GWClark/PHYS301/AJP%20Articles/AJP%20Electric%20Guitar%20pickup.pdf (course-hosted copy)
- **[PPV]** R. C. D. Paiva, J. Pakarinen, V. Välimäki, "Acoustics and Modeling of Pickups", JAES 60(10):768–782, 2012. **Paywalled; abstract only.** The Aalto thesis copy returned 403.
- Guadagnin, Lihoreau, Lotton, Brasseur, JAES 2017 (velocity sensor), https://www.aes.org/e-lib/browse.cfm?elib=19201. **Abstract via search only.**
- **[12AX7]** GE 12AX7 data sheet, https://w140.com/tekwiki/images/5/5e/GE_12AX7_data_sheet.pdf, and Svetlana 12AX7 sheet, https://img.audiomania.ru/data/svetlana_12ax7.pdf. Values were taken from search extracts of both; the PDFs were not opened directly.
- **[CAPS]** T. Goetze, CAPS ToneStack.cc preset table (LMMS 0.4.10 Debian source). https://sources.debian.org/src/lmms/0.4.10-2.3/plugins/ladspa_effect/caps/ToneStack.cc. Code comments, a secondary source.
- **[RR]** R. Robinette, "How the Marshall JCM800 Works" (undated, hobbyist). https://robrobinette.com/How_the_Marshall_JCM800_Works.htm
- Forums (HYPOTHESIS-level):
  - GroupDIY, "Marshall 2203 (JCM800) oscillation with mods", https://groupdiy.com/threads/marshall-2203-jcm800-oscillation-with-mods.82893/
  - rig-talk, "JCM800 brown eye mod?", https://www.rig-talk.com/forum/threads/jcm800-brown-eye-mod.230790/
  - **[SD-fs]** Seymour Duncan forum, "Bridge pickup pole spacing for Gibson Les Paul", https://forum.seymourduncan.com/threads/bridge-pickup-pole-spacing-for-gibson-les-paul.314567/ (source of the '59 pickup positions, via search extract)
  - TDPRI Les Paul pickup locations thread: **TollBit paywall, unread**
- **[CEL]** Celestion G12T-75 and G12M Greenback spec sheets. https://celestion.com/productpdf.php?id=898 and ?id=899
- **[SM57]** Shure SM57 spec sheet (copy). https://collegeofsanmateo.edu/theatre/docs/sm57_specsheet.pdf. Values via search extract; the response graph was not read.
- **[GC]** J. Beckner, "Five myths about Slash's gear that people still believe", Guitar.com, 2021-11-09. https://guitar.com/features/opinion-analysis/slash-guitar-gear-myths/
- **[MR]** MusicRadar, "The story of Slash's signature Marshall amps…" (undated). https://musicradar.com/news/the-story-of-slashs-signature-marshall-amps-and-how-they-capture-his-classic-tones
- **[PG-AFD]** Premier Guitar, Marshall AFD100 launch (press release), 2011-01-12. https://www.premierguitar.com/marshall-launches-slash-signature-limited-edition-afd100-head
- **[SDB]** Seymour Duncan blog, "Inside The Alnico II Pro Slash Signature Set", 2016-07-18. https://www.seymourduncan.com/blog/latest-updates/inside-the-alnico-ii-pro-slash-signature-set
- **[SDU]** APH-2 specs via retailers (Reverb, Bajaao, Soundhouse; conflicting Chicago Music Exchange). The Duncan spec page was not read.
- **[EB]** Ernie Ball Slash signature set. https://shop.ernieball.com/slash-signature-string-set.html (via search extract), plus zZounds.
- **[DAD]** EXL110 tensions via retailer reproductions (Yamaha Music London, player.rs) and D'Addario "All about string tension". https://www.daddario.com/en-au/blogs/guitar/all-about-string-tension
- **[WLP]** Wikipedia, "Gibson Les Paul" (scale length).
- Lemme: buildyourguitar.com/resources/lemme/ failed (**expired TLS certificate**). Lemme's numbers came secondhand: https://ozvalveamps.org/pickups.html and a Harmony Central forum post (5 H with 500 / 1000 pF gives 3183 / 2251 Hz).
- **Blocked or truncated, not read:**
  - Guitar World: "Dear Guitar Hero: Slash"; "The story of… Sweet Child O' Mine"; the Slash/Izzy Appetite feature; the Mike Clink feature (all returned only a header)
  - The MusicRadar Sweet Child song-story body
  - The Lemme book; Zollner print-only figures
  - Fleischer & Zwicker 1998
