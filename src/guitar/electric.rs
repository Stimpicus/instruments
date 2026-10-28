//! A solid-body electric guitar: Les Paul-style, with humbucking pickups.
//!
//! The strings are the same modal strings as the acoustic. What changes:
//! - **The body.** A solid body barely moves at the bridge, so the strings lose
//!   almost nothing into it and sustain far longer. Two narrow selective losses
//!   remain: the neck resonance (about 220 Hz) and the bridge's (about 2 kHz)
//!   (Zollner 2014, ch. 7.7).
//! - **The output.** A magnetic pickup senses the string's velocity over its pole
//!   pieces (Horton & Moore 2009), so each mode is heard in proportion to its shape
//!   at the poles: a comb set by the pickup's position. A humbucker has two rows of
//!   poles about 19 mm apart; their sum notches the band where the spacing is half a
//!   wavelength (Zollner ch. 5.4). The coil, its capacitance, the pots and the cable
//!   make a resonant low-pass (ch. 5.9).
//!
//! Numbers, sources and which ones are hypotheses: `docs/electric-guitar.md`.

use std::f64::consts::{PI, TAU};

use super::body::{Body, Dense, Mode};
use super::{Guitar, StringSpec, midi_hz};

/// A magnetic pickup as the string and the amp see it.
#[derive(Clone, Debug)]
pub struct Pickup {
    /// Pole rows: (distance from the bridge saddle in m, relative sensitivity).
    pub rows: Vec<(f64, f64)>,
    /// Width (standard deviation) of a pole's magnetic window along the string, m.
    pub aperture: f64,
    /// Sensitivity to horizontal string motion relative to vertical.
    pub horizontal: f64,
    /// Volts per m/s of string velocity at full weight.
    pub volts_per_mps: f64,
    /// Per-string sensitivity (low E first): how much steel each string carries past the magnet,
    /// and how high each pole sits. Set so the open strings sound equally loud (see `les_paul`).
    pub string_gain: [f64; 6],
    /// Coil inductance (H) and resistance (ohm); capacitance of coil plus cable (F);
    /// the load of the volume and tone pots and the amp input in parallel (ohm).
    pub l: f64,
    pub r: f64,
    pub c: f64,
    pub load: f64,
}

/// Which pickup the selector switch picks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Selector {
    Neck,
    Bridge,
    /// Both in parallel (the middle position).
    Both,
}

impl Selector {
    pub fn parse(s: &str) -> Option<Selector> {
        Some(match s {
            "neck" => Selector::Neck,
            "bridge" => Selector::Bridge,
            "both" => Selector::Both,
            _ => return None,
        })
    }
}

impl Pickup {
    /// How strongly the pickup hears a mode with wavenumber `kn` (= n pi / vibrating length):
    /// the mode shape at each pole row, smoothed by the pole's window.
    pub fn weight(&self, kn: f64) -> f64 {
        let window = (-(kn * self.aperture).powi(2) / 2.0).exp();
        self.volts_per_mps * window * self.rows.iter().map(|&(x, g)| g * (kn * x).sin()).sum::<f64>()
    }

    /// Resonance frequency (Hz) and Q of the coil's circuit.
    pub fn resonance(&self) -> (f64, f64) {
        let w0 = 1.0 / (self.l * self.c).sqrt();
        (w0 / TAU, 1.0 / (w0 * (self.l / self.load + self.r * self.c)))
    }

    /// The coil's circuit: H(s) = 1 / ((1 + R/Rl) + s (L/Rl + R C) + s^2 L C), by the
    /// bilinear transform prewarped at the resonance.
    pub fn circuit(&self, x: &[f64], rate: f64) -> Vec<f64> {
        let (a0, a1, a2) = (1.0 + self.r / self.load, self.l / self.load + self.r * self.c, self.l * self.c);
        let w0 = 1.0 / a2.sqrt();
        let k = w0 / (w0 / (2.0 * rate)).tan();
        let mut f = Biquad::from_analog([1.0, 0.0, 0.0], [a0, a1, a2], k);
        x.iter().map(|&v| f.run(v)).collect()
    }
}

/// A second-order section, direct form II transposed.
#[derive(Clone, Debug, Default)]
pub struct Biquad {
    b: [f64; 3],
    a: [f64; 2],
    z: [f64; 2],
}

impl Biquad {
    /// From analog (b0 + b1 s + b2 s^2) / (a0 + a1 s + a2 s^2), with s = k (1 - z^-1) / (1 + z^-1).
    pub fn from_analog(b: [f64; 3], a: [f64; 3], k: f64) -> Biquad {
        let d = a[0] + a[1] * k + a[2] * k * k;
        Biquad {
            b: [(b[0] + b[1] * k + b[2] * k * k) / d, (2.0 * b[0] - 2.0 * b[2] * k * k) / d, (b[0] - b[1] * k + b[2] * k * k) / d],
            a: [(2.0 * a[0] - 2.0 * a[2] * k * k) / d, (a[0] - a[1] * k + a[2] * k * k) / d],
            z: [0.0; 2],
        }
    }

    pub fn run(&mut self, x: f64) -> f64 {
        let y = self.b[0] * x + self.z[0];
        self.z[0] = self.b[1] * x - self.a[0] * y + self.z[1];
        self.z[1] = self.b[2] * x - self.a[1] * y;
        y
    }
}

/// Fret positions and scale of a Les Paul: 24.75 in.
const SCALE: f64 = 24.75 * 0.0254;

/// A PAF-style humbucker centred `centre` m from the bridge: two pole rows 19 mm apart,
/// the slug row (towards the bridge here) 2.5 dB hotter than the screw row.
pub fn humbucker(centre: f64) -> Pickup {
    let slug = 10f64.powf(2.5 / 20.0);
    Pickup {
        rows: vec![(centre - 0.0095, slug / (1.0 + slug) * 2.0), (centre + 0.0095, 1.0 / (1.0 + slug) * 2.0)],
        aperture: 2.5e-3,
        horizontal: 0.15,
        volts_per_mps: 0.04,
        string_gain: [1.0; 6],
        l: 4.5,
        r: 8000.0,
        c: 800e-12,
        load: 200e3,
    }
}

/// The pickup(s) a selector position gives. Both = the two in parallel: about their average.
pub fn pickup(sel: Selector) -> Pickup {
    // Centres from one measurement of a '59 Les Paul (hypothesis): neck ~151 mm, bridge ~51 mm from the saddles.
    let (neck, bridge) = (humbucker(0.151), humbucker(0.051));
    match sel {
        Selector::Neck => neck,
        Selector::Bridge => bridge,
        Selector::Both => {
            let mut p = bridge;
            p.rows = neck.rows.iter().chain(&p.rows).map(|&(x, g)| (x, g * 0.5)).collect();
            p.l = 2.25;
            p.r = 4000.0;
            p
        }
    }
}

/// Balance the strings at the pickup, as a set-up guitar is: equal pickup level for each open string
/// picked the same way. Velocity sensing alone made the open high E 12 dB louder than the low E (a
/// thin plain string carries far less steel past the magnet, and poles are set to even it out), and
/// the founder heard high notes as piercing (2026-10-27). Equal open strings is a hypothesis.
fn balance(g: &mut Guitar) {
    let rate = g.rate as f64;
    let level = |g: &Guitar, s: usize| {
        let x = g.render_for(&super::Pluck::new(s, 0, 0.6), 0.5);
        let seg = &x[(0.05 * rate) as usize..];
        (seg.iter().map(|v| v * v).sum::<f64>() / seg.len() as f64).sqrt()
    };
    let levels: Vec<f64> = (0..6).map(|s| level(g, s)).collect();
    let mean = (levels.iter().map(|l| l.ln()).sum::<f64>() / 6.0).exp();
    let pu = g.pickup.as_mut().expect("balance needs a pickup");
    for (k, l) in pu.string_gain.iter_mut().zip(&levels) {
        *k = mean / l;
    }
}

/// The solid body: tiny mobility, with the neck and bridge resonances as narrow losses.
fn solid_body() -> Body {
    // (Hz, Q, peak mobility m/(N s)). Frequencies: Zollner 7.7 (Les Paul Classic). Q and level: hypotheses.
    let named: Vec<Mode> = [(220.0, 20.0, 0.002), (2000.0, 10.0, 0.0005)]
        .iter()
        .map(|&(f, q, peak)| Mode { f, q, m: q / (peak * TAU * f), rad: 1.0 })
        .collect();
    Body::new(&named, &Dense { from_hz: 300.0, to_hz: 20_000.0, spacing: 200.0, q: 20.0, mobility: 1e-5, mass_corner: 3000.0, seed: 11 })
}

/// A Les Paul with Slash's string gauges (Ernie Ball 11-48, nickel wound) and humbuckers.
pub fn les_paul(rate: u32, sel: Selector) -> Guitar {
    let inch = 0.0254;
    let area = |d_in: f64| PI * (d_in * inch).powi(2) / 4.0;
    // Tensions: D'Addario's 10-46 chart (25.5 in) scaled by gauge^2 for the 11-48 set and to the 24.75 in scale
    // (derived, a hypothesis until the unit weights are read). Wound strings' stiffness: their cores (hypothesis).
    // Decay: Zollner measured the low E's fundamental falling 30 dB in 25 s and its 1 kHz partial in 7 s
    // (fresh 10-46 strings); a Les Paul's high E fundamental about 4.4 s. Strings between: interpolated
    // on log frequency (hypothesis). The air term eta_A is set to give each fundamental its time.
    let t30 = |open: f64| 25.0 * (midi_hz(open) / midi_hz(40.0)).powf((4.4f64 / 25.0).ln() / 4f64.ln());
    let s = |open: f64, lbf: f64, stiff_in: f64, eta_b: f64| {
        let w0 = TAU * midi_hz(open);
        let eta_f = 5e-5;
        let eta = 2.0 * (30.0 / (20.0 * std::f64::consts::LOG10_E) / t30(open)) / w0;
        StringSpec { open, tension: lbf * 4.448, stiff_d: stiff_in * inch, steel_area: area(stiff_in), eta_f, eta_a: (eta - eta_f) * w0, eta_b }
    };
    let strings = [
        s(40.0, 17.3, 0.018, 4.4e-3),
        s(45.0, 20.0, 0.016, 0.4e-2),
        s(50.0, 20.1, 0.0135, 1.1e-2),
        s(55.0, 17.5, 0.018, 1.0e-2),
        s(59.0, 16.8, 0.014, 1.1e-2),
        s(64.0, 18.5, 0.011, 0.7e-2),
    ];
    let mut g = Guitar::new(rate, strings, SCALE, solid_body());
    g.pickup = Some(pickup(sel));
    balance(&mut g);
    // A solid body's strings ring far longer; render enough of them.
    g.max_secs = 12.0;
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fft::{C, fft};
    use crate::guitar::Pluck;

    /// Peak spectral magnitude within 1.5% of `hz` in x[a..b] (Hann window, zero-padded FFT).
    fn level(x: &[f64], rate: f64, hz: f64, a: f64, b: f64) -> f64 {
        let s = &x[(a * rate) as usize..((b * rate) as usize).min(x.len())];
        let n = (s.len() * 2).next_power_of_two();
        let mut buf = vec![C::default(); n];
        for (i, &v) in s.iter().enumerate() {
            buf[i] = C::new(v * (PI * i as f64 / s.len() as f64).sin().powi(2), 0.0);
        }
        fft(&mut buf, false);
        let bin = |f: f64| (f * n as f64 / rate) as usize;
        (bin(hz * 0.985)..=bin(hz * 1.015)).map(|k| buf[k].abs()).fold(0.0, f64::max)
    }

    fn single_coil_at(x: f64) -> Pickup {
        let mut p = humbucker(x);
        p.rows = vec![(x, 1.0)];
        p
    }

    #[test]
    fn a_pickup_at_a_quarter_of_the_string_misses_the_fourth_harmonic() {
        let rate = 44_100.0;
        let mut g = les_paul(44_100, Selector::Bridge);
        let f0 = midi_hz(45.0);
        let ratio = |g: &Guitar| {
            let x = g.render_for(&Pluck::new(1, 0, 0.6), 1.0);
            level(&x, rate, 4.0 * f0, 0.1, 0.6) / level(&x, rate, 3.0 * f0, 0.1, 0.6)
        };
        g.pickup = Some(single_coil_at(SCALE / 4.0));
        let at_node = ratio(&g);
        // Control: the same pickup a little further along hears the fourth harmonic.
        g.pickup = Some(single_coil_at(SCALE / 5.3));
        let off_node = ratio(&g);
        let db = 20.0 * (at_node / off_node).log10();
        assert!(db < -20.0, "4th harmonic at its node is {db:.1} dB against off the node");
    }

    #[test]
    fn the_humbucker_notches_where_its_poles_are_half_a_wavelength_apart() {
        let rate = 44_100.0;
        let mut g = les_paul(44_100, Selector::Bridge);
        // A single coil at the humbucker's centre has no notch; the humbucker does.
        let spread = |g: &Guitar| {
            let x = g.render_for(&Pluck::new(0, 0, 0.6), 0.8);
            let (f0, b) = (midi_hz(40.0), g.inharmonicity(0, 0));
            let freq = |n: f64| n * f0 * ((1.0 + b * n * n) / (1.0 + b)).sqrt();
            // Partials near the notch (where the poles are half a wavelength apart: n = L / d)
            // against partials well below it.
            let notch_n = SCALE / 0.019;
            let sum = |centre: f64| (1..60).map(|n| n as f64).filter(|n| (n / centre - 1.0).abs() < 0.08).map(|n| level(&x, rate, freq(n), 0.05, 0.4)).sum::<f64>();
            sum(notch_n) / sum(0.45 * notch_n)
        };
        let hum = spread(&g);
        g.pickup = Some(single_coil_at(0.051));
        let single = spread(&g);
        let db = 20.0 * (hum / single).log10();
        assert!(db < -10.0, "humbucker against single coil near c/2d: {db:.1} dB");
    }

    #[test]
    fn a_solid_body_sustains_as_measured_and_far_longer_than_the_acoustic() {
        let rate = 44_100.0;
        let mut g = les_paul(44_100, Selector::Bridge);
        g.max_secs = 30.0;
        let x = g.render_for(&Pluck::new(0, 0, 0.6), 30.0);
        let f0 = midi_hz(40.0);
        let fall = |x: &[f64], hz: f64, t: f64| 20.0 * (level(x, rate, hz, 0.5, 1.5) / level(x, rate, hz, t, t + 1.0)).log10();
        // Zollner: 30 dB in 25 s at the fundamental, in 7 s at the 1 kHz partial. Within 30%.
        let per_s = fall(&x, f0, 20.5) / 20.0;
        let t30 = 30.0 / per_s;
        assert!((17.5..32.5).contains(&t30), "fundamental T30 {t30:.1} s");
        let k = (1000.0 / f0).round();
        let t30_k = 30.0 / (fall(&x, k * f0 * (1.0 + g.inharmonicity(0, 0) * k * k).sqrt(), 5.5) / 5.0);
        assert!((4.9..9.1).contains(&t30_k), "1 kHz partial T30 {t30_k:.1} s");
        // Control: the acoustic's same note loses its fundamental many times faster.
        let a = super::super::steel_string(44_100).render_for(&Pluck::new(0, 0, 0.6), 6.0);
        let acoustic_per_s = 20.0 * (level(&a, rate, f0, 0.5, 1.5) / level(&a, rate, f0, 4.5, 5.5)).log10() / 4.0;
        assert!(acoustic_per_s > 3.0 * per_s, "acoustic {acoustic_per_s:.2} dB/s, electric {per_s:.2} dB/s");
    }

    #[test]
    fn the_neck_pickup_is_darker_than_the_bridge() {
        let bright = |sel: Selector| {
            let x = les_paul(44_100, sel).render_for(&Pluck::new(2, 2, 0.6), 1.0);
            let s = &x[2205..22_050];
            s.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum::<f64>() / s.iter().map(|v| v * v).sum::<f64>()
        };
        let (neck, bridge) = (bright(Selector::Neck), bright(Selector::Bridge));
        assert!(neck < bridge * 0.7, "neck {neck:.4}, bridge {bridge:.4}");
    }

    #[test]
    fn the_open_strings_are_balanced_at_the_pickup() {
        let g = les_paul(44_100, Selector::Bridge);
        let rms = |s: usize| {
            let x = g.render_for(&Pluck::new(s, 0, 0.6), 0.5);
            20.0 * (x[2205..].iter().map(|v| v * v).sum::<f64>() / (x.len() - 2205) as f64).sqrt().log10()
        };
        let l: Vec<f64> = (0..6).map(rms).collect();
        let spread = l.iter().cloned().fold(f64::MIN, f64::max) - l.iter().cloned().fold(f64::MAX, f64::min);
        assert!(spread < 1.0, "open strings span {spread:.1} dB: {l:?}");
        // Control: without the balance, velocity sensing makes the high E much louder than the low E.
        let mut raw = g.clone();
        raw.pickup.as_mut().unwrap().string_gain = [1.0; 6];
        let raw_rms = |s: usize| {
            let x = raw.render_for(&Pluck::new(s, 0, 0.6), 0.5);
            20.0 * (x[2205..].iter().map(|v| v * v).sum::<f64>() / (x.len() - 2205) as f64).sqrt().log10()
        };
        assert!(raw_rms(5) - raw_rms(0) > 6.0, "unbalanced high E over low E: {:.1} dB", raw_rms(5) - raw_rms(0));
    }

    #[test]
    fn a_hard_picked_high_note_starts_only_slightly_sharp() {
        // Tension modulation: one measured steel-string note glided 12.5 cents (Hodgkinson 2010).
        // Plucking a short fretted string as far as an open one made B fret 17 start 32 cents sharp
        // at 0.15 s and slide for seconds, heard as a "wah". Measured at 0.15 s against the target.
        let rate = 44_100.0;
        let g = les_paul(44_100, Selector::Bridge);
        let x = g.render_for(&Pluck::new(4, 17, 0.8), 0.4);
        let target = midi_hz(59.0 + 17.0);
        let (a, n) = ((0.15 * rate) as usize, (0.08 * rate) as usize);
        let cents = (-80..=80)
            .map(|c| c as f64)
            .max_by(|&c1, &c2| {
                let m = |c: f64| {
                    let f = target * 2f64.powf(c / 1200.0);
                    let (re, im) = (0..n).fold((0.0, 0.0), |(re, im), i| {
                        let w = (PI * i as f64 / n as f64).sin().powi(2) * x[a + i];
                        let ph = TAU * f * i as f64 / rate;
                        (re + w * ph.cos(), im + w * ph.sin())
                    });
                    re * re + im * im
                };
                m(c1).total_cmp(&m(c2))
            })
            .unwrap();
        assert!(cents < 20.0, "B fret 17, hard pick, 0.15 s: {cents:+.0} cents");
    }

    #[test]
    fn the_coil_resonance_is_where_the_components_put_it() {
        let p = humbucker(0.05);
        let (f0, q) = p.resonance();
        // L 4.5 H, C 800 pF, R 8 k, load 200 k: about 2.65 kHz, Q about 2 (the research brief's worked example).
        assert!((f0 - 2653.0).abs() < 20.0 && (1.8..2.4).contains(&q), "f0 {f0:.0} Hz, Q {q:.2}");
        // And the digital filter peaks there.
        let rate = 44_100.0;
        let gain = |hz: f64| {
            let x: Vec<f64> = (0..44_100).map(|i| (TAU * hz * i as f64 / rate).sin()).collect();
            let y = p.circuit(&x, rate);
            (y[22_050..].iter().map(|v| v * v).sum::<f64>() / x[22_050..].iter().map(|v| v * v).sum::<f64>()).sqrt()
        };
        let (low, peak, high) = (gain(200.0), gain(2600.0), gain(8000.0));
        assert!(peak > 1.6 * low && high < 0.3 * low, "200 Hz {low:.2}, 2.6 kHz {peak:.2}, 8 kHz {high:.2}");
    }
}
