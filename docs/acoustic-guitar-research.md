# Steel-string acoustic guitar: physical-model research brief

All sources were checked on 2026-10-27. Source keys like [S1] are listed in §7, with URL and access status.

**Labels.** ADOPTED = meets the bar: one primary, or two independent sources that agree. DERIVED = my own arithmetic on adopted inputs; the formula is given so you can redo it. HYPOTHESIS = below the bar. UNREAD = a primary I could not open (paywall, 403, or bot wall).

## Bottom line first

- **No dreadnought-specific body data was obtained.** The best steel-string body numbers come from 6 Fylde "Falstaff" guitars [S6], a large-bodied model that is not a dreadnought. Fletcher & Rossing's Martin D-28 table is UNREAD. The one secondary copy of that table I saw has misaligned columns.
- **No steel-string measurement of vertical vs horizontal decay rates or polarization split was found.** The only measured polarization data I read [S8] is for a nylon-strung flamenco guitar.
- **D'Addario does not publish core diameters for wound strings.** B for the wound strings must be measured. The literature says this explicitly [S5].
- **No CC0 steel-string per-string, per-fret, decay-length note set exists in what I found.** The cleanest per-string, per-fret, three-dynamic set (Univ. of Iowa) is a nylon classical guitar.
- **One source has an internal inconsistency.** Woodhouse 2012 [S5] gives steel-string tensions and masses that imply a 0.70 m scale. Its B values are consistent with about 0.645 m. Details are in §1.3.

---

## 1. Strings

### 1.1 D'Addario EJ16 (.012–.053, phosphor bronze, light)

Unit weights come from D'Addario's own String Tension Pro dataset [S1b], fetched directly from D'Addario's data endpoint. Tensions come from the EJ16 product page [S1a]. Both are ADOPTED (manufacturer primary).

| Str | Note | f0 (Hz) | Item | Gauge (in) | UW (lb/in) [S1b] | μ (kg/m) | T, page (lb) [S1a] | T @25.5" (N) | T @25.4" (N, DERIVED) | Z0=√(Tμ) @25.4" (kg/s) |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | E4 | 329.63 | PL012 | .012 | 3.19161e-5 | 0.000570 | 23.4 | 103.8 | 103.1 | 0.242 |
| 2 | B3 | 246.94 | PL016 | .016 | 5.67397e-5 | 0.001013 | 23.3 | 103.6 | 102.9 | 0.323 |
| 3 | G3 | 196.00 | PB024 | .024 | 1.161226e-4 | 0.002074 | 30.1 | 133.6 | 132.6 | 0.524 |
| 4 | D3 | 146.83 | PB032 | .032 | 2.06068e-4 | 0.003680 | 29.9 | 133.0 | 132.1 | 0.697 |
| 5 | A2 | 110.00 | PB042 | .042 | 3.549583e-4 | 0.006339 | 28.9 | 128.6 | 127.7 | 0.900 |
| 6 | E2 | 82.41 | PB053 | .053 | 5.45324e-4 | 0.009738 | 24.9 | 110.9 | 110.1 | 1.035 |

- **D'Addario's formula:** T(lb) = UW·(2·L·f)²/386.4, with L in inches. This was quoted via search snippet of D'Addario's tension article [S1c]; the article page itself returned 429.
- **The page tensions are for a 25.5" scale.** Recomputing from UW reproduces every page value within 0.1 lb at 25.5". At 25.4" the results are 0.2–0.3 lb low. This check discriminates between the two scales. Total is 160.5 lb.
- **For a 25.4" (645.2 mm) scale,** multiply by (25.4/25.5)² = 0.9922. The 25.4" dreadnought scale is your figure and I did not verify it against a Martin or Taylor spec sheet.
- **Conflict: an older D'Addario chart gives heavier wound strings.** The old PDF (tension_chart_13934) now redirects to the Shopify store, so it is UNREAD directly. As extracted by the open-source `stringcalc` project [S2], it lists:
  - PB024 1.1682e-4 (30.2 lb)
  - PB032 2.1018e-4 (30.5 lb)
  - PB042 3.6722e-4 (29.9 lb)
  - PB053 5.6962e-4 (26.0 lb)
  - Plain strings are the same as above.
  - Total is 163.2 lb, which matches a UMGF forum copy.
  - The current dataset gives PB053 4.3% lighter. Both are D'Addario figures from different eras, and I have not resolved which is right.
- **Construction:** hexagonal high-carbon steel core with phosphor-bronze wrap [S1a]. **Core diameters are not published** in the product page or dataset. The dataset fields are gauge, mass per unit length, material, and construction only.

### 1.2 Steel Young's modulus: HYPOTHESIS, range 190–210 GPa

- MakeItFrom lists ASTM A228 music wire at 190 GPa [S3]. That is a property database, not the standard.
- The commonly quoted 207 GPa / 30 Mpsi had no primary located. ASTM A228 itself and Fletcher & Rossing's table are UNREAD.
- **Indirect check (DERIVED from [S5]):** Woodhouse's measured EI for plain strings gives these values, assuming a solid round steel wire of the measured μ and ρ = 7850 kg/m³:
  - B3 string: E ≈ 204 GPa
  - E4 string: E ≈ 257 GPa, an outlier that suggests measurement scatter
- Use E ≈ 200 GPa and treat ±5% as the design uncertainty.

### 1.3 Inharmonicity B, with f_n = n·f0·√(1 + B·n²)

The definition and the formula B = π²·EI/(T·L²) = π³·E·d⁴/(64·T·L²) are from [S5], eqs. 1–3. For a fretted note at the same tension, B ∝ 1/L²: the 12th fret has 4× the open-string B.

| Str | Computed, EJ16 plain @25.4", E=190/200/210 GPa (DERIVED) | Measured, Martin 80/20 **new** [S5] | Measured, same strings **used** [S5] | Other measurement |
|---|---|---|---|---|
| E4 | 1.85 / 1.95 / 2.05 e-5 | 2.1e-5 | 2.0e-5 | — |
| B3 | 5.86 / 6.17 / 6.48 e-5 | 5.8e-5 | 6.8e-5 | — |
| G3 (wound) | not predictable | 3.0e-5 | 4.4e-5 | — |
| D3 | not predictable | 3.2e-5 | 5.3e-5 | — |
| A2 | not predictable | 4.5e-5 | 6.6e-5 | — |
| E2 | not predictable | 10.6e-5 | 11.4e-5 | **1.1–1.14e-4**, varying in time after a mezzo-forte pluck [S11] |

- **Plain strings: ADOPTED.** The independent computation and Woodhouse's measurement agree within about 10%.
- **Low E ≈ 1.1e-4: ADOPTED.** It rests on two independent measurements:
  - Woodhouse, Martin 80/20 strings
  - Hodgkinson et al., an Ovation piezo sample from a commercial library, with unknown strings
- **Other wound strings: single source** (Woodhouse; ADOPTED as one peer-reviewed primary). These are Martin 80/20 strings, not EJ16.
  - "For overwound strings … the effective value of EI or B cannot be reliably predicted, and must be measured" [S5].
  - Aging raised B by 0–60% on the wound strings [S5].
- **Effective core diameter for EJ16: HYPOTHESIS.** If you want one for your model, back-solving Woodhouse's new-string B at EJ16 tensions (E = 200 GPa, L = 645 mm) gives:
  - G 0.0142"
  - D 0.0145"
  - A 0.0156"
  - E 0.0186"

  This is an *effective* stiffness diameter that lumps in the wrap's contribution. It is not a physical core measurement.
- **The [S5] inconsistency, disclosed.** Woodhouse's tensions (121, 135, 182, 185, 169, 147 N) and masses (0.57, 1.13, 2.42, 4.37, 7.13, 11.02 g/m) give L = √(T/(4f²μ)) = 0.700 m on all six strings. That is longer than any steel-string guitar. His EI and B pairs are self-consistent only at L ≈ 0.645 m. So:
  - Treat his B and EI as the reliable quantities.
  - Do not use his T or μ for EJ16 work.

---

## 2. Decay

### 2.1 Intrinsic string damping model (Valette form)

Woodhouse fits this to the "floor" of measured mode damping across all frets [S5 eq. 4; S8 eq. 8]:

η_n = [T(η_F + η_A/ω_n) + EI·η_B·(nπ/L)²] / [T + EI·(nπ/L)²] = (η_F + η_A/ω_n + η_B·B·n²) / (1 + B·n²)

- η_F is the "friction" term, η_A the "air" term (units 1/s), and η_B the "bending" (viscoelastic) term.
- Equation caveat: PDF text extraction dropped the Greek symbols. The structure is unambiguous, and the second form is my algebraic rewrite using B. Check against the PDF before coding.
- Amplitude decay rate is σ_n = η_n·ω_n/2, and T60 = 2·ln(1000)/(η_n·ω_n).

**Measured coefficients, steel strings (Martin 80/20) [S5 Table I]. ADOPTED (single peer-reviewed primary; brand differs from EJ16).**

| Str | η_F new (e-5) | η_F used | η_B new (e-2) | η_B used | η_A new (1/s) | η_A used |
|---|---|---|---|---|---|---|
| E4 | 7 | 6 | 0.7 | 0.7 | 2.5 | 1.5 |
| B3 | 6 | 7 | 1.1 | 1.1 | 1.0 | 1.0 |
| G3 | 11 | 17 | 1.8 | 6.8 | 1.2 | 0.6 |
| D3 | 9 | 33 | 1.1 | 4.1 | 1.3 | 0.7 |
| A2 | 11 | 17 | 0.4 | 2.9 | 1.2 | 0.6 |
| E2 | 5 | 10 | 0.3 | 3.8 | 1.7 | 1.4 |

For comparison, nylon (D'Addario Pro Arté hard) has η_F of 2–40 e-5 and η_B of about 2e-2. Nylon is more damped, mainly on the trebles [S5].

**String-only T60, open strings, new strings.** DERIVED from the table above, using Woodhouse's B. This assumes rigid supports, so these are upper bounds: real notes decay faster because of bridge and body losses.

| Str | n=1 | n=2 | n=4 | n=8 | n=16 | n=32 |
|---|---|---|---|---|---|---|
| E4 | 5.2 s | 4.9 | 4.5 | 3.6 (2.6 kHz) | 2.3 (5.3 kHz) | 0.8 (10.7 kHz) |
| B3 | 12.6 | 11.6 | 9.6 | 6.2 | 2.1 (4.0 kHz) | 0.4 (8.1 kHz) |
| G3 | 10.3 | 9.4 | 7.7 | 5.3 | 2.3 (3.1 kHz) | 0.5 (6.4 kHz) |
| D3 | 10.0 | 9.4 | 8.4 | 6.5 | 3.5 (2.4 kHz) | 1.0 (4.8 kHz) |
| A2 | 10.8 | 10.2 | 9.1 | 7.4 | 4.8 (1.8 kHz) | 1.8 (3.6 kHz) |
| E2 | 8.0 | 7.9 | 7.6 | 7.0 | 5.1 (1.3 kHz) | 1.9 (2.8 kHz) |

- **Used wound strings:** high partials collapse. For example, the E2 string at n=16 drops to 1.2 s, and at n=32 to 0.2 s.
- **Perception [S5]:** η_B is "the most important parameter for the perception of the degradation of sound in well-used strings." This is your "new vs dead strings" knob.

### 2.2 Bridge/body losses

- **What dominates:** "In most cases the decay rate is dominated by losses into the guitar body, rather than by internal damping in the string" [S5]. Body losses are why string modes near body resonances decay fast and the measured decay-vs-frequency cloud is scattered [S5, S8].
- **Usable formula (DERIVED, standard transmission-line result; not quoted from a source I read):** for a string of impedance Z0 terminated by bridge admittance Y, the added decay rate is σ_bridge ≈ 2·f0·Z0·Re{Y(ω_n)}. Use Z0 from the §1.1 table.
- **Worked example:** low E (Z0 = 1.04 kg/s) at a body peak with |Y| ≈ 0.03 m/s/N (−30 dB; see §3) gives σ ≈ 5.1 s⁻¹, so T60 ≈ 1.35 s. Off-peak, Re{Y} is 1–2 orders lower, and string losses take over again.
- **Woodhouse's synthesis approach [S7, S8]:** couple string and body through measured Y, with 1/Y_coupled = 1/Y_body + 1/Y_string ([S5] eq. 5), rather than adding decay rates.

### 2.3 Polarizations and double decay

- **Theory [S7]:** Woodhouse 2004a §3.4 (a classical guitar test case, but the argument is general):
  - At low body modal overlap, only the string polarization parallel to a body mode's motion couples. The perpendicular one is "unaffected by body coupling … and therefore unable to radiate sound."
  - The result is two decay factors but "little trace of the 'double decay'" in radiated sound.
  - Coupled string/body modes show "veering" only below about 300 Hz for his test guitar. Above that, the string and body modes keep similar frequencies but different Q. So use a synthesis that carries string damping fully; undamped-modes-then-Rayleigh "gives poor results."
- **Measurement [S8]:** a 650 mm flamenco guitar with nylon strings, plucked by wire-break 20 mm from the bridge at 0°, 45° and 90°.
  - Body coupling predicts a polarization split of at most 0.1 Hz.
  - The measured split was "several Hz" at higher partials, seen "for all notes analysed."
  - The normal-to-top polarization is the **higher** frequency of each pair.
  - Woodhouse tentatively attributes the split to slight rolling on the fret and/or bridge, which gives different effective lengths.
  - No "convincing example of 'double exponential decay'" was found.
- **Steel-string values: GAP.** I found no measured split or vertical vs horizontal decay rates for steel strings.
  - Brauchler et al. 2020 [S9] (steel? not confirmed) attribute polarization coupling to torsion of the string on the saddle. Full text returned 403; the abstract is all I read.
  - A search snippet quoted "≈0.2 Hz at the first eigenfrequency," which may come from [S9]. Unverified, so HYPOTHESIS.
- **Perception: UNREAD.** Järveläinen & Karjalainen 2002 (ISMA) tested the perception of beating and two-stage decay in dual-polarization models. No numbers obtained.

---

## 3. Body

### 3.1 Steel-string flat-top low modes: 6 Fylde "Falstaff" guitars [S6]

- **Setup:** large-bodied, not a dreadnought. Sitka tops from one flitch, Cameroon ebony bridges, Elixir 80/20 light strings.
- **Method:** laser vibrometer plus impulse hammer at the saddle, between the 5th and 6th strings, normal to the top, with the strings damped.
- **Status:** ADOPTED (peer-reviewed JASA).

| Back wood | F1 (Hz) | Q1 | F2 (Hz) | Q2 | F3 (Hz) | Q3 |
|---|---|---|---|---|---|---|
| Brazilian rosewood | 97 | 34 | 177 | 18 | 336 | 36 |
| Indian rosewood | 101 | 38 | 188 | 26 | 368 | 58 |
| Maple | 100 | 35 | 187 | 25 | 363 | 53 |
| Mahogany | 99 | 33 | 184 | 25 | 351 | 35 |
| Sapele | 96 | 39 | 175 | 17 | 335 | 44 |
| Walnut | 98 | 42 | 182 | 28 | 347 | 24 |
| **Range** | **96–101** | **33–42** | **175–188** | **17–28** | **335–368** | **24–58** |

- **Mode identity [S6]:**
  - F1 and F2 are two members of the low "coupled triad," i.e. the A0/T(1,1)₁ and T(1,1)₂ family, both strongly influenced by the back.
  - F3 is "a transverse bending mode of the top plate," with very weak back influence.
- **Bridge admittance |Y| [S6, Fig. 1]:**
  - Plotted on a −70 to −30 dB re 1 m/s/N axis, so the peaks reach about −30 dB ≈ 0.03 m/s/N.
  - Three strong peaks at low frequency, then "a relatively smooth and lower-level trace at higher frequencies."
  - Exact curve values were not extracted (figure only). A Yamaha FG-403MS steel-string reference was modal-fitted up to 1200 Hz.
- **Classical/flamenco comparison [S5]** (15 guitars measured; four plotted are classical/flamenco):
  - Helmholtz-type peak at 80–100 Hz, a deep antiresonance, and strong peaks at 150–250 Hz
  - A dip around 400–500 Hz
  - Inter-guitar spread: σ ≈ 5.5–6% in mode frequency and ≈33% in Q
- **Damped-strings bias [S8]:** damped strings lower the measured Q. On a classical guitar, compensation raised Q from 20 to 25 at 91 Hz and from 33 to 47 at 205 Hz.
- **Dreadnought / Martin D-28 (Fletcher & Rossing; Rossing, Popp & Polstein 1983 SMAC): UNREAD.**
  - A search snippet attributed top-plate modes "163 326 390 431 643 733 756 Hz" to a D-28: 7 values under 8 mode labels.
  - A second, unattributed D-28 row "165 257 337 369 480 509 678 693 Hz" also surfaced.
  - The two conflict and neither is verified. Do not use them without the book.

### 3.2 Radiation vs frequency

- **Measured radiation efficiency for steel-string guitars: GAP.** Lai & Burgess 1990 (JASA) is UNREAD.
- **What [S5] used:** bridge acceleration filtered by a breathing-sphere radiator, R(ω) = (iω/ωc)/(1 + iω/ωc), with ωc/2π = 250 Hz (a ≈ 0.2 m). It "proved to give satisfactory-sounding results."
  - That is a design choice validated by ear, not a measurement. Treat it as a reasonable HYPOTHESIS default: a first-order high-pass below about 250 Hz, flat above.

### 3.3 Implementation sizes (secondary, via [S13])

- **FIR body filter:** Karjalainen et al. 1999 found orders below about 1000 taps unsatisfactory for the acoustic guitar at 44.1 kHz.
- **Hybrid:** factoring the lowest resonances out as 2nd-order resonators cut this to about 500 taps.
- **Phase:** minimum-phase body equalization "destroys the reverberant character."
- **Warped filters:** Karjalainen & Smith 1996 cut the order but have stability issues on conversion.
- **Status:** HYPOTHESIS. All of the above is relayed through Bank's thesis; the primaries are UNREAD.

### 3.4 Perceptual thresholds for the body [S5]

From 3AFC tests on synthesized guitar sounds, classical datum guitar. ADOPTED.

- **Shift of all body mode frequencies:** the best listeners hear about 1% or less. The best single result was 0.32%, about 6 cents.
- **Shifting one band:** the threshold was about 1.9% for the 150–250 Hz band and about 1.5% for the 500–1000 Hz band (best-four averages).
- **Q scaling:** about 20% is needed to hear a change.
- **Real-world consequence:** [S6] found that back-wood differences produced small modal shifts. Players' discrimination was poor, and only "mediocre" for the most divergent pair.

---

## 4. Excitation

- **Woodhouse's model [S5, S7]:**
  - Pluck = step force at the pluck point, with a chosen angle that sets the polarization mix.
  - Finger/plectrum width = frequency-domain Gaussian ∝ exp(−ω²/K²), with K = v/h, v = √(T/μ), and **h = 7.5 mm** "to obtain a reasonably realistic sound."
  - This is a tuned choice, not a measured contact width: HYPOTHESIS-grade as a physical value, but proven usable.
- **Pluck-position comb (standard, cf. [S10] §3):**
  - For an ideal triangular pluck at fraction β = d/L from the bridge, mode n's amplitude ∝ sin(nπβ)/n² (displacement). Modes with nβ an integer vanish.
  - At d = 13 cm on L = 645 mm, β ≈ 0.20, so harmonics 5, 10, 15, … are notched.
- **Pluck distances in the literature:** these are experimental settings, not population statistics.
  - Traube & Smith 2001 [S10]: test plucks at 12, 13 and 14 cm on a 1995 Collings and a 1953 Martin 000-18 (phosphor bronze, light and medium strings), with a 0.88 mm plastic pick. Their sweep covered 1–20 cm.
  - Mohamad et al. 2017: 30–170 mm on an electric guitar. Search snippet only.
  - Woodhouse lab plucks: 20 mm.
- **Qualitative ([S10], ADOPTED as one primary):**
  - Near the bridge the tone is "softer in volume, brighter and sharper."
  - The neutral hand position is "just behind the sound hole."
  - Low strings are usually plucked further from the bridge than high strings.
- **Typical player distance distribution: GAP.** Say 8–16 cm from the saddle as a HYPOTHESIS.
- **Plectrum contact duration and plectrum force spectrum: GAP.**
  - The relevant numbers I found are for the harpsichord (Perng, Smith & Rossing 2010; Le Carrou SMAC 2013).
  - The JOS model treats a plectrum as a spring that releases at a force threshold or separation [S12].
  - Evangelista & Eckerholm 2010 (IEEE TASLP, touch and collisions) is UNREAD.
  - Pavlidou & Richardson on finger-string interaction is UNREAD.
- **Dynamics vs brightness, measured: GAP.**
  - The Jaffe & Smith EKS includes a "dynamic-level lowpass filter" [S12]. That is a method, not data.
  - You can measure it yourself from the Iowa pp/mf/ff set (§6), which is nylon, so the trend transfers but the values may not.
- **Tension-modulation pitch glide (one data point, scope stated) [S11]:**
  - Low-E open string (the paper calls it "E3"), Ovation piezo sample, mezzo forte.
  - f0 fell from 83.4 Hz to 82.8 Hz, about 12.5 cents, with time constant 0.38 s.
  - B also varies in time, with time constant 0.23 s.
  - This is n = 1 note through a piezo pickup, so it is a baseline, not a trend.
  - The Tolonen, Välimäki & Karjalainen 2000 model (IEEE TSAP 8(3):300–310) is UNREAD. Glide perception thresholds (Järveläinen & Välimäki 2001 ICMC) are UNREAD.
- **Phantom / longitudinal partials:** frequency-doubled extra peaks were measured on guitar [S8]. Their amplitude is modulated by longitudinal modes (Woodhouse 2004b via [S13]).

---

## 5. Synthesis methods: state of the art

| Method | Captures | Misses | Evidence on realism |
|---|---|---|---|
| **Extended Karplus-Strong** (Jaffe & Smith 1983, CMJ 7(2):56–69) [S12] | Delay loop with:<br>• pick-direction LP<br>• pick-position comb<br>• string-damping filter<br>• stiffness allpass<br>• tuning allpass<br>• dynamic-level LP | Body, polarizations, coupling, nonlinearity | No listening tests read |
| **Digital waveguide** (Smith; survey [S14]) | Physically parameterized traveling waves. Losses and dispersion lumped into loop filters. Cheap. | Spatially distributed nonlinearity, unless extended | Survey only |
| **Commuted synthesis** (Smith 1993; Karjalainen, Välimäki & Jánosy 1993) [S12, S14] | Body impulse response folded into the excitation table (LTI assumption), so a full body costs nothing per voice | Body ↔ string feedback, so mode veering and decay changes near body resonances are lost. Excitation and body cannot be separated per pluck. | Laurson et al. 2001: "very high quality" classical guitar from a score, with the key credited to control detail [S12] |
| **One-pole loss filter** H(z) = g(1+a1)/(1+a1 z⁻¹) (Välimäki et al. 1996 JAES, UNREAD) | Overall plus frequency-dependent decay | Per-partial irregularity, such as body-induced fast decays | "Adequate for simulating the acoustic guitar" (per [S13]) |
| **High-order loss filters** (Bank & Välimäki 2003, IEEE SPL 10(1):18–20; abstract only) | Matches measured per-partial decay times; minimum-phase, with weighted error on decay times | Cost | Ear tolerance: overall decay time −25% to +40% inaudible (Järveläinen & Tolonen 2001, JAES 49(11):1049–1059, via [S13]; the 2000 AES conference abstract says 25–40%). **Loss filter accuracy matters less than you would think.** |
| **Dispersion allpass** (Rocchesso & Scalcon 1996; Rauhala & Välimäki 2006; Abel & Smith 2006; all UNREAD; [S13] summary) | Stretched partials (B) | Cost: order up to about 20 reported for the piano | Inharmonicity is perceived more easily in low tones even at lower B, because of beating (Järveläinen et al. 2001, via [S13]). For the guitar A string, actual B sits "close to the threshold" [S5]. Karjalainen & Järveläinen 2005 ("Is inharmonicity perceivable in the acoustic guitar?") is UNREAD. **Model B on the wound strings, especially E2; the trebles are optional.** |
| **Dual polarization + bridge coupling matrix** [S12; KVT 1998 UNREAD] | Beating and two-stage decay; sympathetic coupling via a 2×2 per-string (or 12×12 all-string) bridge transfer matrix. Eigenanalysis gives the coupled modes (Weinreich). | Steel-string calibration data (GAP, §2.3) | [S7]: radiated double decay is weak at low frequency. [S8]: the measured split comes from fret/saddle rolling, not body coupling, so the matrix alone will not create it. |
| **Modal / frequency-domain coupled string+body** (Woodhouse 2004) [S7, S8] | Measured Y(ω) at the bridge, full string damping model, both polarizations, pluck angle and width | Fret/saddle split, longitudinal phantoms. Offline cost (suits you: you render offline). | Best two of the methods tested: modal synthesis (first-order method) and frequency-domain synthesis, the latter faster. Underlies the JND studies [S5, S6]. |
| **FDTD** (Bilbao 2009 *Numerical Sound Synthesis*; Bilbao et al. collisions; Brauchler 2020 [S9]) | Nonlinearity, fret and finger collisions, torsion | Cost, stability design | UNREAD for the guitar |
| **Differentiable / DDSP fitting** of KS and waveguide parameters [S14, a 2026 draft survey] | Automatic parameter fitting to recordings | Physical interpretability if unconstrained | Survey-level only |

**Perceptual priorities, ranked by measured JNDs ([S5], ADOPTED):**

1. **Body mode frequencies,** where about 1% is audible. Get the 90–110 / 175–190 / 335–370 Hz triad right.
2. **Body Q,** where about 20% is audible.
3. **String damping parameters,** which need about a 3× change, except η_B, which drives the new vs old string character.
4. **Bending stiffness,** which is near threshold.

Overall decay-time tolerance is roughly −25% to +40% (via [S13]).

---

## 6. Reference recordings

Nothing was downloaded. License status was checked on 2026-10-27.

| Set | Instrument | Content | License (verified where) | Fit for you |
|---|---|---|---|---|
| **Univ. of Iowa MIS – Guitar** https://theremin.music.uiowa.edu/MISguitar.html | **Raimundo 118, nylon classical** (not steel) | Per string (sulE…sul_E), chromatic runs, **pp/mf/ff**. Anechoic chamber, 5 ft, Earthworks QTC40. 16/44.1 and 24/96, mono and stereo. Recorded 2011-12-11. | "may be downloaded and used for any projects, without restrictions" (MIS.html). **Not a CC license.** | Best methodology (anechoic, per string, three dynamics, long decays) but the wrong string type |
| **AG-PT-set** https://zenodo.org/records/10159492 | 7 steel-string acoustics, 6 players | 15 h 55 min. 12 techniques including "Pick Near Bridge" and "Pick Over the Soundhole." 32,592 labeled notes. 6.7 GB. | **CC BY 4.0** (Zenodo) | Good for pluck-position and brightness contrasts. Per-string/fret isolation not stated. Mic setup not stated. |
| **Acoustic Guitar Notes Dataset** (Kaggle, Alkooheji) https://www.kaggle.com/datasets/mohammedalkooheji/guitar-notes-dataset | Walden G551E (steel) plus Yamaha CM-40 (nylon) | About 1500 notes, D2–G#5, every note to fret 16, pick/finger/nail tagged. **2 s clips**, mono, 44.1 kHz. | **CC BY-SA 4.0** (Kaggle API) | Good for spectra and attack. **Too short for T60** on low strings. |
| **GuitarSet** https://zenodo.org/records/3371780 (v1.1.0) | Acoustic guitar, hexaphonic pickup plus mic | Excerpts with string/fret annotations, not isolated notes | **Conflict between versions:** v1.1.0 (3371780) = **CC BY 4.0**; v1.0.1 (1422265) = **CC BY-NC 4.0** | Per-string hex signals are useful for decay. Use v1.1.0. |
| **NSynth** https://magenta.tensorflow.org/datasets/nsynth | 13,343 "acoustic"-source guitar-family notes from commercial libraries (string type not stated) | **16 kHz**, 4 s, note-off at 3 s, 5 velocities | **CC BY 4.0** (Google, Magenta page) | Weak: an 8 kHz band limit, and the forced release truncates decays |
| **Freesound – Project16 "Electro acoustique Sherwood SH887"** https://freesound.org/people/Project16/packs/15407/ | Sherwood SH887 electro-acoustic (steel assumed, unconfirmed; pickup vs mic not stated) | "three octaves and five velocity levels." About 5 s FLAC, 44.1/16 mono. | **CC BY 3.0** (checked on one sound: #251184) | Multi-velocity, but short and of unknown capture method. Check each file's license. |
| **Longitudinal Dataset for Guitar String Ageing** (Wright, McKenzie, Hamilton, 2026) https://zenodo.org/records/19823590 | Not stated on the record page | 4.7 GB | **CC BY 4.0** | Potentially ideal for new vs used η_B. **Contents unverified** (API returned 403). |
| IDMT-SMT-Guitar https://zenodo.org/records/7544110 | 7 guitars, mostly electric/DI, one condenser-mic recording | Subset 2: 400 one-note files | **CC BY-NC-ND 4.0**: fails your bar | Excluded |
| Philharmonia samples https://philharmonia.co.uk/audio-sound-samples/ | Includes guitar (type unverified), MP3 | — | Own terms: free including commercial works, but not to be resold "as is." The CC BY-SA 3.0 claim comes from third parties only. | Excluded from CC-only; MP3 anyway |
| Freesound CC0 search | — | Search for "acoustic guitar single note steel" filtered to CC0: **0 results** | — | No CC0 steel per-note pack found |

---

## 7. Sources

All were checked 2026-10-27. "Read" means I read the full text unless noted otherwise.

- **[S1a]** D'Addario. EJ16 Phosphor Bronze Light 12-53, product page with string tension chart. https://www.daddario.com/products/ej16-phosphor-bronze-acoustic-guitar-strings-light-12-53 (read; the scale length is not stated on the page).
- **[S1b]** D'Addario String Tension Pro dataset (EJ16 rows). https://embed.cartfulsolutions.com/daddario-string-tension-finder/data/strings_dataset.json — this is the data file behind https://www.daddario.com/string-tension-pro, identified via [S2]'s scraper (read).
- **[S1c]** D'Addario. "All about string tension" (blog). https://www.daddario.com/en-gb/blogs/guitar/all-about-string-tension — **429**. Formula seen via search snippet only.
- **[S1d]** D'Addario. tension_chart_13934.pdf. https://www.daddario.com/globalassets/pdfs/accessories/tension_chart_13934.pdf — now redirects to the store; **UNREAD**. Values seen via [S2].
- **[S2]** Moon, Z. *stringcalc* (open source), data files daddario-tension.csv and daddario-stp.csv. https://github.com/zmoon/stringcalc (read; secondary).
- **[S3]** MakeItFrom. ASTM A228 Music Wire. https://makeitfrom.com/material-properties/ASTM-A228-SWP-A-K08500-Music-Wire (search snippet; database, not a primary).
- **[S5]** Woodhouse, J., Manuel, E.K.Y., Smith, L.A., Wheble, A.J.C., Fritz, C. (2012). "Perceptual thresholds for acoustical guitar models." *Acta Acust. united Acust.* 98(3):475–486. doi:10.3813/AAA.918531. CC BY 4.0 copy: https://hal.science/hal-01461755/document (read).
- **[S6]** Carcagno, S., Bucknall, R., Woodhouse, J., Fritz, C., Plack, C. (2018). "Effect of back wood choice on the perceived quality of steel-string acoustic guitars." *JASA* 144(6):3533–3547. doi:10.1121/1.5084735. Accepted manuscript: https://hal.science/hal-02025913/document (read).
- **[S7]** Woodhouse, J. (2004). "On the synthesis of guitar plucks." *Acta Acust. united Acust.* 90:928–944. https://euphonics.org/wp-content/uploads/2022/03/Guitar_I.pdf (read; equations partly garbled in extraction).
- **[S8]** Woodhouse, J. (2004). "Plucked guitar transients: comparison of measurements and synthesis." *Acta Acust. united Acust.* 90:945–965. https://euphonics.org/wp-content/uploads/2022/03/Guitar_II.pdf (read; the test guitar is nylon/flamenco).
- **[S9]** Brauchler, A. et al. (2020). "Examination of polarization coupling in a plucked musical instrument string via experiments and simulations." *Acta Acustica* 4(3). doi:10.1051/aacus/2020008 — **403**; abstract only, via search.
- **[S10]** Traube, C., Smith, J.O. (2001). "Extracting the fingering and the plucking points on a guitar string from a recording." IEEE WASPAA 2001. https://www.ee.columbia.edu/~dpwe/papers/TraubS01-guitar.pdf (read).
- **[S11]** Hodgkinson, M., Timoney, J., Lazzarini, V. (2010). "A model of partial tracks for tension-modulated steel-string guitar tones." DAFx-10. https://www.dafx.de/paper-archive/2010/DAFx10/HodgkinsonTimoneyLazzarini_DAFx10_P69.pdf (read).
- **[S12]** Smith, J.O. *Physical Audio Signal Processing* (online ed.; W3K 2010). The following pages were read:
  - Extended_Karplus_Strong_Algorithm
  - Coupled_Horizontal_Vertical_Waves
  - Commuted_Synthesis
  - Further_Reading_Commuted_Synthesis

  All at https://ccrma.stanford.edu/~jos/pasp/
- **[S13]** Bank, B. (2006). *Physics-based sound synthesis of string instruments including geometric nonlinearities*. PhD thesis, BME Budapest. https://home.mit.bme.hu/~bank/phd/phd.pdf (read selectively; secondary for the Aalto results).
- **[S14]** Tablas de Paula, P., Smith, J.O., Välimäki, V., Reiss, J.D. (2026). "Four decades of digital waveguides." arXiv:2604.12878 (**DRAFT**; skimmed).

**Unread primaries** (paywalled, or no open copy found): Fletcher & Rossing, *The Physics of Musical Instruments* (2nd ed. 1998); Rossing, Popp & Polstein 1983; Christensen & Vistisen 1980; Caldersmith 1978; Elejabarrieta et al.; Välimäki et al. 1996 JAES; Karjalainen, Välimäki & Tolonen 1998 CMJ; Bank & Välimäki 2003; Rauhala & Välimäki 2006; Abel & Smith 2006; Tolonen et al. 2000; Järveläinen et al. 2001 ARLO; Järveläinen & Tolonen 2001 JAES; Karjalainen & Järveläinen 2005; Lai & Burgess 1990; Evangelista & Eckerholm 2010; Mansour et al. 2014 (steel-string bridge admittances); Bilbao 2009.

I saw a PDF of Rossing & Fletcher's *The Science of String Instruments* hosted on an unofficial site (logosfoundation.org). I did not use it, because it looks like an unauthorized copy.
