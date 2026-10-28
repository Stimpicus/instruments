//! A Marshall-style valve amplifier and 4x12 cabinet, after the JCM800 2203 (a stand-in:
//! Slash's Appetite amp was a modified, rented 100 W Super Lead whose mods are undocumented).
//!
//! - **Preamp:** three 12AX7 triode stages, each solved from Koren's triode equations on its
//!   load line (100 k plate resistor). The second is the "cold clipper" (10 k unbypassed
//!   cathode, biased near cut-off), which clips hard and unevenly. Between stages a coupling
//!   capacitor feeds the next grid; when the grid goes positive it draws current, charges the
//!   capacitor and shifts the bias: the stage "blocks" for a moment after a loud note, the
//!   memory that makes edge-of-breakup playing feel alive (Pakarinen & Yeh 2009).
//! - **Tone stack:** Yeh & Smith's (2006) exact transfer function of the Fender/Marshall
//!   network, with Marshall values.
//! - **Power amp:** EL34 push-pull as a symmetric soft clip, with the presence control as a
//!   treble lift and a little power-supply sag.
//! - **Cabinet:** a synthetic 4x12 with Celestion-like response (75-85 Hz resonance, a mid dip,
//!   a steep fall above 5 kHz) and a close microphone's presence. No freely licensed measured
//!   Marshall cab response was found; this one is a hypothesis to tune by ear.
//!
//! The nonlinear part runs at 4x the sample rate, so its harmonics don't fold back as noise.
//! Values and sources: `docs/electric-guitar.md`.

use std::f64::consts::{PI, TAU};

use super::electric::Biquad;

/// The front panel, each 0..1 (as the knobs read 0..10).
#[derive(Clone, Debug)]
pub struct Settings {
    pub gain: f64,
    pub bass: f64,
    pub mid: f64,
    pub treble: f64,
    pub presence: f64,
    pub master: f64,
}

impl Default for Settings {
    /// A mid-gain crunch, a starting point for Slash's sound (hypothesis).
    fn default() -> Settings {
        Settings { gain: 0.6, bass: 0.5, mid: 0.7, treble: 0.6, presence: 0.5, master: 0.5 }
    }
}

/// Every value inside the amp that is not a front-panel knob: component values, stage corners,
/// tapers and power-stage behaviour. `Design::default()` is the calibrated amp when one exists
/// (see `docs/electric-guitar.md`); `Design::guessed()` is the first version, built from forum and
/// plugin values. `set` changes one value by name (for calibration).
#[derive(Clone, Debug)]
pub struct Design {
    /// Preamp supply, V.
    pub b_plus: f64,
    /// Cathode resistors of the three preamp stages, ohm (stage 1 and 3 bypassed, stage 2 not).
    pub rk1: f64,
    pub rk2: f64,
    pub rk3: f64,
    /// Stage 1 cathode-capacitor shelf (zero, pole Hz); the bright "peaker" into the gain pot;
    /// stage 3 cathode shelf.
    pub shelf1_z: f64,
    pub shelf1_p: f64,
    pub peak_z: f64,
    pub peak_p: f64,
    pub shelf3_z: f64,
    pub shelf3_p: f64,
    /// Coupling into stages 2 and 3: source resistance, grid leak (ohm), capacitor (F).
    pub g2_rs: f64,
    pub g2_rg: f64,
    pub g2_c: f64,
    pub g3_rs: f64,
    pub g3_rg: f64,
    pub g3_c: f64,
    /// Gain pot law: (10^(a k) - 1) / (10^a - 1) for knob k in 0..1.
    pub gain_taper: f64,
    /// Attenuation between stage 2 and stage 3.
    pub pre_pad: f64,
    /// Tone stack (Yeh & Smith naming), ohm and F; bass pot law as for gain.
    pub r1: f64,
    pub r2: f64,
    pub r3: f64,
    pub r4: f64,
    pub c1: f64,
    pub c2: f64,
    pub c3: f64,
    pub bass_taper: f64,
    /// Presence: shelf corner (Hz) and how far the knob lifts it.
    pub presence_hz: f64,
    pub presence_span: f64,
    /// Tone-stack volts into the power stage at master 1, before the master pot's law.
    pub master_scale: f64,
    /// Power stage asymmetry (0 = symmetric push-pull) and sag (depth, time constant s).
    pub power_asym: f64,
    pub sag_depth: f64,
    pub sag_tau: f64,
    /// The speaker load seen by the output: a resonance (Hz, Q, dB) and a high-frequency shelf (Hz, dB).
    pub load_hz: f64,
    pub load_q: f64,
    pub load_db: f64,
    pub load_hf_hz: f64,
    pub load_hf_db: f64,
}

impl Design {
    /// The first, uncalibrated amp: 2203-style values from forums and plugin code (hypotheses).
    pub fn guessed() -> Design {
        Design {
            b_plus: 300.0,
            rk1: 2700.0,
            rk2: 10e3,
            rk3: 820.0,
            shelf1_z: 87.0,
            shelf1_p: 230.0,
            peak_z: 1.0 / (TAU * 470e3 * 470e-12),
            peak_p: 1.0 / (TAU * (470e3 * 1e6 / 1.47e6) * 470e-12),
            shelf3_z: 285.0,
            shelf3_p: 600.0,
            g2_rs: 100e3,
            g2_rg: 1e6,
            g2_c: 22e-9,
            g3_rs: 38e3,
            g3_rg: 470e3,
            g3_c: 22e-9,
            gain_taper: 2.0,
            pre_pad: 1.0,
            r1: 220e3,
            r2: 1e6,
            r3: 22e3,
            r4: 33e3,
            c1: 470e-12,
            c2: 22e-9,
            c3: 22e-9,
            bass_taper: 2.0,
            presence_hz: 2500.0,
            presence_span: 3.0,
            master_scale: 0.25,
            power_asym: 0.0,
            sag_depth: 0.2,
            sag_tau: 0.05,
            load_hz: 100.0,
            load_q: 1.0,
            load_db: 0.0,
            load_hf_hz: 3000.0,
            load_hf_db: 0.0,
        }
    }

    /// Change one value by its field name; false if there is no such field.
    pub fn set(&mut self, key: &str, v: f64) -> bool {
        let f = match key {
            "b_plus" => &mut self.b_plus,
            "rk1" => &mut self.rk1,
            "rk2" => &mut self.rk2,
            "rk3" => &mut self.rk3,
            "shelf1_z" => &mut self.shelf1_z,
            "shelf1_p" => &mut self.shelf1_p,
            "peak_z" => &mut self.peak_z,
            "peak_p" => &mut self.peak_p,
            "shelf3_z" => &mut self.shelf3_z,
            "shelf3_p" => &mut self.shelf3_p,
            "g2_rs" => &mut self.g2_rs,
            "g2_rg" => &mut self.g2_rg,
            "g2_c" => &mut self.g2_c,
            "g3_rs" => &mut self.g3_rs,
            "g3_rg" => &mut self.g3_rg,
            "g3_c" => &mut self.g3_c,
            "gain_taper" => &mut self.gain_taper,
            "pre_pad" => &mut self.pre_pad,
            "r1" => &mut self.r1,
            "r2" => &mut self.r2,
            "r3" => &mut self.r3,
            "r4" => &mut self.r4,
            "c1" => &mut self.c1,
            "c2" => &mut self.c2,
            "c3" => &mut self.c3,
            "bass_taper" => &mut self.bass_taper,
            "presence_hz" => &mut self.presence_hz,
            "presence_span" => &mut self.presence_span,
            "master_scale" => &mut self.master_scale,
            "power_asym" => &mut self.power_asym,
            "sag_depth" => &mut self.sag_depth,
            "sag_tau" => &mut self.sag_tau,
            "load_hz" => &mut self.load_hz,
            "load_q" => &mut self.load_q,
            "load_db" => &mut self.load_db,
            "load_hf_hz" => &mut self.load_hf_hz,
            "load_hf_db" => &mut self.load_hf_db,
            _ => return false,
        };
        *f = v;
        true
    }
}

impl Default for Design {
    fn default() -> Design {
        Design::guessed()
    }
}

/// A pot law: (10^(a k) - 1) / (10^a - 1); a = 2 is a typical "audio" taper, small a is near linear.
fn taper(k: f64, a: f64) -> f64 {
    let k = k.clamp(0.0, 1.0);
    if a.abs() < 1e-6 { k } else { (10f64.powf(a * k) - 1.0) / (10f64.powf(a) - 1.0) }
}

/// 12AX7 constants (Koren 1996/2003).
const MU: f64 = 100.0;
const EX: f64 = 1.4;
const KG1: f64 = 1060.0;
const KP: f64 = 600.0;
const KVB: f64 = 300.0;
/// Grid's resistance once it conducts, ohm (Koren's R_GI).
const RGI: f64 = 2000.0;

/// Plate current, A, for plate-cathode and grid-cathode voltages.
fn koren(vpk: f64, vgk: f64) -> f64 {
    if vpk <= 0.0 {
        return 0.0;
    }
    let e1 = vpk / KP * (1.0 + (KP * (1.0 / MU + vgk / (KVB + vpk * vpk).sqrt())).exp()).ln();
    if e1 <= 0.0 { 0.0 } else { 2.0 * e1.powf(EX) / KG1 }
}

/// One triode stage as a table: grid voltage in, change of plate voltage out.
struct Triode {
    /// Cathode voltage at rest: the grid conducts once it rises above this.
    vk_rest: f64,
    lo: f64,
    step: f64,
    table: Vec<f64>,
}

impl Triode {
    /// `rk` is the cathode resistor; `bypassed` means a capacitor holds the cathode at its
    /// resting voltage (full gain); otherwise the cathode follows the current (feedback).
    fn new(ra: f64, rk: f64, bypassed: bool, b_plus: f64) -> Triode {
        // Plate current for a grid voltage, by bisection on the load line.
        let solve = |vg: f64, vk_fixed: Option<f64>| {
            let (mut lo, mut hi) = (0.0, b_plus / ra);
            for _ in 0..60 {
                let ip = 0.5 * (lo + hi);
                let vk = vk_fixed.unwrap_or(ip * rk);
                let vp = b_plus - ip * ra;
                if ip > koren(vp - vk, vg - vk) { hi = ip } else { lo = ip }
            }
            0.5 * (lo + hi)
        };
        let ip0 = solve(0.0, None);
        let vk0 = bypassed.then_some(ip0 * rk);
        let rest = b_plus - ip0 * ra;
        let (lo, hi, n) = (-120.0, 10.0, 8192);
        let step = (hi - lo) / (n - 1) as f64;
        let table = (0..n).map(|i| b_plus - solve(lo + i as f64 * step, vk0) * ra - rest).collect();
        Triode { vk_rest: ip0 * rk, lo, step, table }
    }

    fn run(&self, vg: f64) -> f64 {
        let x = ((vg - self.lo) / self.step).clamp(0.0, (self.table.len() - 2) as f64);
        let i = x as usize;
        let f = x - i as f64;
        self.table[i] * (1.0 - f) + self.table[i + 1] * f
    }
}

/// A coupling capacitor into a grid with its leak resistor, fed from a source resistance.
/// Grid current (when the grid rises above the cathode) charges the capacitor: bias shift.
struct Grid {
    rs: f64,
    rg: f64,
    c: f64,
    /// The cathode's resting voltage: the grid conducts above it.
    vk: f64,
    vc: f64,
}

impl Grid {
    fn new(rs: f64, rg: f64, c: f64, into: &Triode) -> Grid {
        Grid { rs, rg, c, vk: into.vk_rest, vc: 0.0 }
    }

    fn run(&mut self, vin: f64, dt: f64) -> f64 {
        let v = vin - self.vc;
        // Divider from the source into the grid's leak; once the grid passes the cathode it also
        // conducts into it through R_GI: (v - vg) / rs = vg / rg + (vg - vk) / R_GI.
        let mut vg = v * self.rg / (self.rg + self.rs);
        if vg > self.vk {
            vg = (v / self.rs + self.vk / RGI) / (1.0 / self.rs + 1.0 / self.rg + 1.0 / RGI);
        }
        let i = (v - vg) / self.rs;
        self.vc += i * dt / self.c;
        vg
    }
}

/// Yeh & Smith 2006: the Fender/Marshall tone stack, H(s) = (b1 s + b2 s^2 + b3 s^3) / (1 + a1 s + a2 s^2 + a3 s^3).
/// t, m, l: treble, mid, bass (0..1; the bass pot is logarithmic).
pub fn tone_stack_analog(t: f64, m: f64, l: f64) -> ([f64; 4], [f64; 4]) {
    // Marshall values (2203 per the CAPS source; 470 pF vs 500 pF, 220 k vs 250 k and 22 k vs 25 k unresolved).
    tone_stack_of(&Design::guessed(), t, m, l)
}

/// The tone stack with a design's component values.
pub fn tone_stack_of(d: &Design, t: f64, m: f64, l: f64) -> ([f64; 4], [f64; 4]) {
    let (r1, r2, r3, r4) = (d.r1, d.r2, d.r3, d.r4);
    let (c1, c2, c3) = (d.c1, d.c2, d.c3);
    let b1 = t * c1 * r1 + m * c3 * r3 + l * (c1 * r2 + c2 * r2) + (c1 * r3 + c2 * r3);
    let b2 = t * (c1 * c2 * r1 * r4 + c1 * c3 * r1 * r4) - m * m * (c1 * c3 * r3 * r3 + c2 * c3 * r3 * r3)
        + m * (c1 * c3 * r1 * r3 + c1 * c3 * r3 * r3 + c2 * c3 * r3 * r3)
        + l * (c1 * c2 * r1 * r2 + c1 * c2 * r2 * r4 + c1 * c3 * r2 * r4)
        + l * m * (c1 * c3 * r2 * r3 + c2 * c3 * r2 * r3)
        + (c1 * c2 * r1 * r3 + c1 * c2 * r3 * r4 + c1 * c3 * r3 * r4);
    let ccc = c1 * c2 * c3;
    let b3 = l * m * (ccc * r1 * r2 * r3 + ccc * r2 * r3 * r4) - m * m * (ccc * r1 * r3 * r3 + ccc * r3 * r3 * r4)
        + m * (ccc * r1 * r3 * r3 + ccc * r3 * r3 * r4)
        + t * ccc * r1 * r3 * r4
        - t * m * ccc * r1 * r3 * r4
        + t * l * ccc * r1 * r2 * r4;
    let a1 = (c1 * r1 + c1 * r3 + c2 * r3 + c2 * r4 + c3 * r4) + m * c3 * r3 + l * (c1 * r2 + c2 * r2);
    let a2 = m * (c1 * c3 * r1 * r3 - c2 * c3 * r3 * r4 + c1 * c3 * r3 * r3 + c2 * c3 * r3 * r3)
        + l * m * (c1 * c3 * r2 * r3 + c2 * c3 * r2 * r3)
        - m * m * (c1 * c3 * r3 * r3 + c2 * c3 * r3 * r3)
        + l * (c1 * c2 * r2 * r4 + c1 * c2 * r1 * r2 + c1 * c3 * r2 * r4 + c2 * c3 * r2 * r4)
        + (c1 * c2 * r1 * r4 + c1 * c3 * r1 * r4 + c1 * c2 * r3 * r4 + c1 * c2 * r1 * r3 + c1 * c3 * r3 * r4 + c2 * c3 * r3 * r4);
    let a3 = l * m * (ccc * r1 * r2 * r3 + ccc * r2 * r3 * r4) - m * m * (ccc * r1 * r3 * r3 + ccc * r3 * r3 * r4)
        + m * (ccc * r3 * r3 * r4 + ccc * r1 * r3 * r3 - ccc * r1 * r3 * r4)
        + l * ccc * r1 * r2 * r4
        + ccc * r1 * r3 * r4;
    ([0.0, b1, b2, b3], [1.0, a1, a2, a3])
}

/// Response of an analog (b, a) in s at `hz`.
pub fn analog_response(b: &[f64; 4], a: &[f64; 4], hz: f64) -> f64 {
    let w = TAU * hz;
    let ev = |c: &[f64; 4]| {
        // c0 + c1 (jw) + c2 (jw)^2 + c3 (jw)^3
        let re = c[0] - c[2] * w * w;
        let im = c[1] * w - c[3] * w * w * w;
        re.hypot(im)
    };
    ev(b) / ev(a)
}

/// A third-order section from the bilinear transform (Yeh & Smith's own discretization).
struct Third {
    b: [f64; 4],
    a: [f64; 4],
    x: [f64; 3],
    y: [f64; 3],
}

impl Third {
    fn new(b: [f64; 4], a: [f64; 4], rate: f64) -> Third {
        let c = 2.0 * rate;
        let (c2, c3) = (c * c, c * c * c);
        let bz = [
            b[0] + b[1] * c + b[2] * c2 + b[3] * c3,
            3.0 * b[0] + b[1] * c - b[2] * c2 - 3.0 * b[3] * c3,
            3.0 * b[0] - b[1] * c - b[2] * c2 + 3.0 * b[3] * c3,
            b[0] - b[1] * c + b[2] * c2 - b[3] * c3,
        ];
        let az = [
            a[0] + a[1] * c + a[2] * c2 + a[3] * c3,
            3.0 * a[0] + a[1] * c - a[2] * c2 - 3.0 * a[3] * c3,
            3.0 * a[0] - a[1] * c - a[2] * c2 + 3.0 * a[3] * c3,
            a[0] - a[1] * c + a[2] * c2 - a[3] * c3,
        ];
        Third { b: bz.map(|v| v / az[0]), a: az.map(|v| v / az[0]), x: [0.0; 3], y: [0.0; 3] }
    }

    fn run(&mut self, x: f64) -> f64 {
        let y = self.b[0] * x + self.b[1] * self.x[0] + self.b[2] * self.x[1] + self.b[3] * self.x[2]
            - self.a[1] * self.y[0]
            - self.a[2] * self.y[1]
            - self.a[3] * self.y[2];
        self.x = [x, self.x[0], self.x[1]];
        self.y = [y, self.y[0], self.y[1]];
        y
    }
}

fn lowpass(hz: f64, q: f64, rate: f64) -> Biquad {
    let w0 = TAU * hz;
    Biquad::from_analog([1.0, 0.0, 0.0], [1.0, 1.0 / (q * w0), 1.0 / (w0 * w0)], w0 / (w0 / (2.0 * rate)).tan())
}

fn highpass(hz: f64, q: f64, rate: f64) -> Biquad {
    let w0 = TAU * hz;
    Biquad::from_analog([0.0, 0.0, 1.0 / (w0 * w0)], [1.0, 1.0 / (q * w0), 1.0 / (w0 * w0)], w0 / (w0 / (2.0 * rate)).tan())
}

/// Peaking EQ: `db` at `hz`, bandwidth set by `q`.
fn peak(hz: f64, q: f64, db: f64, rate: f64) -> Biquad {
    let w0 = TAU * hz;
    let g = 10f64.powf(db / 40.0);
    Biquad::from_analog([1.0, g / (q * w0), 1.0 / (w0 * w0)], [1.0, 1.0 / (g * q * w0), 1.0 / (w0 * w0)], w0 / (w0 / (2.0 * rate)).tan())
}

/// First-order high shelf: unity at low frequencies, `db` well above `hz`.
fn high_shelf(hz: f64, db: f64, rate: f64) -> Biquad {
    let g = 10f64.powf(db / 20.0);
    let w = TAU * hz;
    // (1 + s g / wp) / (1 + s / wp): 1 at low frequencies, g at high; the corner sits about hz.
    let wp = w * g.sqrt();
    Biquad::from_analog([1.0, g / wp, 0.0], [1.0, 1.0 / wp, 0.0], 2.0 * rate)
}

/// First-order shelf, unity at high frequencies and wz / wp at low ones:
/// (wz / wp) (1 + s / wz) / (1 + s / wp).
fn shelf(hz_zero: f64, hz_pole: f64, rate: f64) -> Biquad {
    let (wz, wp) = (TAU * hz_zero, TAU * hz_pole);
    Biquad::from_analog([wz / wp, 1.0 / wp, 0.0], [1.0, 1.0 / wp, 0.0], 2.0 * rate)
}

/// Windowed-sinc low-pass for 4x resampling: cutoff 0.11 of the high rate (19.4 kHz at 176.4 kHz).
fn resampling_filter() -> Vec<f64> {
    let taps = 128;
    let fc = 0.11;
    (0..taps)
        .map(|i| {
            let x = i as f64 - (taps - 1) as f64 / 2.0;
            let sinc = if x == 0.0 { 2.0 * fc } else { (TAU * fc * x).sin() / (PI * x) };
            // Blackman window.
            let w = 0.42 - 0.5 * (TAU * i as f64 / (taps - 1) as f64).cos() + 0.08 * (2.0 * TAU * i as f64 / (taps - 1) as f64).cos();
            sinc * w
        })
        .collect()
}

pub const OVERSAMPLE: usize = 4;

/// Guitar signal (volts from the pickup) in, microphone signal out, at `rate`.
pub fn marshall(x: &[f64], rate: f64, s: &Settings) -> Vec<f64> {
    marshall_with(x, rate, s, OVERSAMPLE)
}

/// `marshall` at a chosen oversampling factor (1 = none: for tests of aliasing).
pub fn marshall_with(x: &[f64], rate: f64, s: &Settings, over: usize) -> Vec<f64> {
    marshall_design(x, rate, s, &Design::default(), over)
}

/// The whole amp with a given design: preamp and tone stack, power stage and speaker load, cabinet.
pub fn marshall_design(x: &[f64], rate: f64, s: &Settings, d: &Design, over: usize) -> Vec<f64> {
    cabinet(&power(&preamp(x, rate, s, d, over), rate, s, d, over), rate)
}

/// Run `stage` on each sample at `over` times the rate: upsample (zero-stuff, filter), process,
/// filter, keep every `over`-th sample.
fn oversampled(x: &[f64], over: usize, mut stage: impl FnMut(f64) -> f64) -> Vec<f64> {
    if over == 1 {
        return x.iter().map(|&v| stage(v)).collect();
    }
    let h = resampling_filter();
    let mut hist = vec![0.0; h.len()];
    let mut down_hist = vec![0.0; h.len()];
    let mut out = Vec::with_capacity(x.len());
    let mut pos = 0;
    for i in 0..x.len() * over {
        hist[pos] = if i % over == 0 { x[i / over] * over as f64 } else { 0.0 };
        // Only every `over`-th history sample is non-zero: the taps that meet them.
        down_hist[pos] = stage(fir(&h, &hist, pos, i % over, over));
        if i % over == over - 1 {
            out.push(fir(&h, &down_hist, pos, 0, 1));
        }
        pos = (pos + 1) & (h.len() - 1);
    }
    out
}

/// Pickup volts in; the tone stack's output (volts) out: three triode stages and the tone stack.
pub fn preamp(x: &[f64], rate: f64, s: &Settings, d: &Design, over: usize) -> Vec<f64> {
    let hi = rate * over as f64;
    let dt = 1.0 / hi;
    let v1a = Triode::new(100e3, d.rk1, true, d.b_plus);
    let v1b = Triode::new(100e3, d.rk2, false, d.b_plus);
    let v2a = Triode::new(100e3, d.rk3, true, d.b_plus);
    // Stage 1's cathode capacitor only bypasses above its shelf: less bass gain.
    let mut v1a_shelf = shelf(d.shelf1_z, d.shelf1_p, hi);
    // The mixing resistor and its bright capacitor into the gain pot.
    let mut peaker = shelf(d.peak_z, d.peak_p, hi);
    let gain_pot = taper(s.gain, d.gain_taper);
    let mut g1 = Grid::new(68e3, 1e6, 1.0, &v1a);
    let mut g2 = Grid::new(d.g2_rs, d.g2_rg, d.g2_c, &v1b);
    let mut g3 = Grid::new(d.g3_rs, d.g3_rg, d.g3_c, &v2a);
    let mut v2a_shelf = shelf(d.shelf3_z, d.shelf3_p, hi);
    let (tb, ta) = tone_stack_of(d, s.treble.clamp(0.0, 1.0), s.mid.clamp(0.0, 1.0), taper(s.bass, d.bass_taper));
    let mut stack = Third::new(tb, ta, hi);
    oversampled(x, over, |up| {
        let a = v1a.run(g1.run(v1a_shelf.run(up), dt));
        // Gain pot, then the cold clipper V1B
        let b = v1b.run(g2.run(peaker.run(a) * gain_pot, dt));
        // V2A, then the cathode follower into the tone stack (unity, not modelled as clipping)
        let c = v2a.run(g3.run(v2a_shelf.run(b * d.pre_pad), dt));
        stack.run(c)
    })
}

/// Tone-stack volts in; the voltage across the speaker out: presence, master, the push-pull power
/// stage (soft clip, optionally lopsided) with sag, and the speaker load's shaping.
pub fn power(x: &[f64], rate: f64, s: &Settings, d: &Design, over: usize) -> Vec<f64> {
    let hi = rate * over as f64;
    let dt = 1.0 / hi;
    // Presence lifts the treble (it cuts negative feedback there).
    let mut presence = shelf(d.presence_hz / (1.0 + d.presence_span * s.presence.clamp(0.0, 1.0)), d.presence_hz, hi);
    let master = taper(s.master, 2.0) * d.master_scale;
    let mut sag = 0.0;
    let mut out_hp = highpass(30.0, 0.7, hi);
    let mut load = peak(d.load_hz, d.load_q, d.load_db, hi);
    let mut load_hf = high_shelf(d.load_hf_hz, d.load_hf_db, hi);
    let a = d.power_asym;
    oversampled(x, over, |v| {
        let e = presence.run(v) * master;
        let headroom = 1.0 - d.sag_depth * sag;
        // tanh(u + a) - tanh(a): a lopsided soft clip when a != 0, still zero at rest.
        let y = headroom * ((e / headroom + a).tanh() - a.tanh());
        sag += (y.abs().min(1.0) - sag) * dt / d.sag_tau.max(1e-4);
        load_hf.run(load.run(out_hp.run(y)))
    })
}

/// Convolution with a circular history (length a power of two) ending at `pos`, using taps
/// first, first + step, ...
fn fir(h: &[f64], hist: &[f64], pos: usize, first: usize, step: usize) -> f64 {
    let mask = h.len() - 1;
    let mut acc = 0.0;
    for k in (first..h.len()).step_by(step) {
        acc += h[k] * hist[(pos + h.len() - k) & mask];
    }
    acc
}


/// A synthetic closed-back 4x12 with Celestion-type speakers, close-miked (hypothesis throughout):
/// cone resonance ~80 Hz, a mid dip, cone break-up and mic presence around 2.5 kHz, steep above 5 kHz.
pub fn cabinet(x: &[f64], rate: f64) -> Vec<f64> {
    let mut chain = [
        highpass(80.0, 1.2, rate),
        peak(120.0, 1.0, 2.0, rate),
        peak(800.0, 0.8, -3.0, rate),
        peak(2500.0, 2.0, 4.0, rate),
        lowpass(5000.0, 0.541, rate),
        lowpass(5000.0, 1.307, rate),
    ];
    x.iter().map(|&v| chain.iter_mut().fold(v, |acc, f| f.run(acc))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fft::{C, fft};

    fn spectrum(x: &[f64]) -> Vec<f64> {
        let n = x.len().next_power_of_two();
        let mut b = vec![C::default(); n];
        for (i, &v) in x.iter().enumerate() {
            b[i] = C::new(v * (PI * i as f64 / x.len() as f64).sin().powi(2), 0.0);
        }
        fft(&mut b, false);
        b[..n / 2].iter().map(|c| c.abs()).collect()
    }

    fn sine(hz: f64, volts: f64, secs: f64, rate: f64) -> Vec<f64> {
        (0..(secs * rate) as usize).map(|i| volts * (TAU * hz * i as f64 / rate).sin()).collect()
    }

    /// Level of the harmonic `k` of `hz` relative to the fundamental, dB, from the steady part.
    fn harmonic_db(y: &[f64], rate: f64, hz: f64, k: f64) -> f64 {
        let s = spectrum(&y[y.len() / 2..y.len() / 2 + 16_384]);
        let at = |f: f64| {
            let bin = (f * 16_384.0 / rate).round() as usize;
            s[bin - 2..=bin + 2].iter().cloned().fold(0.0, f64::max)
        };
        20.0 * (at(k * hz) / at(hz)).log10()
    }

    #[test]
    fn the_tone_stack_has_its_known_shape() {
        let (b, a) = tone_stack_analog(0.5, 0.5, 0.5);
        let r = |hz: f64| analog_response(&b, &a, hz);
        // A zero at DC; the mid scoop between bass and treble.
        assert!(r(0.1) < 0.02 * r(100.0), "DC zero: {:.4} at 0.1 Hz, {:.4} at 100 Hz", r(0.1), r(100.0));
        assert!(r(500.0) < r(100.0) && r(500.0) < r(3000.0), "mid scoop: 100 Hz {:.3}, 500 Hz {:.3}, 3 kHz {:.3}", r(100.0), r(500.0), r(3000.0));
        // Each knob does its job.
        let at = |t: f64, m: f64, l: f64, hz: f64| {
            let (b, a) = tone_stack_analog(t, m, l);
            analog_response(&b, &a, hz)
        };
        assert!(at(0.5, 0.5, 1.0, 80.0) > 1.5 * at(0.5, 0.5, 0.05, 80.0), "bass");
        assert!(at(1.0, 0.5, 0.5, 4000.0) > 1.5 * at(0.0, 0.5, 0.5, 4000.0), "treble");
        assert!(at(0.5, 1.0, 0.5, 600.0) > 1.5 * at(0.5, 0.0, 0.5, 600.0), "mid");
        // Yeh & Smith: the treble control moves only the zeros.
        assert_eq!(tone_stack_analog(0.0, 0.3, 0.7).1, tone_stack_analog(1.0, 0.3, 0.7).1);
        // The digital version matches the analog within 0.5 dB to 5 kHz (at 4x 44.1 kHz).
        let rate = 176_400.0;
        for hz in [100.0, 400.0, 1000.0, 2500.0, 5000.0] {
            let mut f = Third::new(b, a, rate);
            let x = sine(hz, 1.0, 0.5, rate);
            let y: Vec<f64> = x.iter().map(|&v| f.run(v)).collect();
            let g = (y[44_100..].iter().map(|v| v * v).sum::<f64>() / x[44_100..].iter().map(|v| v * v).sum::<f64>()).sqrt();
            let db = 20.0 * (g / r(hz)).log10();
            assert!(db.abs() < 0.5, "{hz} Hz: digital vs analog {db:.2} dB");
        }
    }

    #[test]
    fn the_triode_is_linear_when_quiet_and_clips_unevenly_when_driven() {
        let v1a = Triode::new(100e3, 2700.0, true, 300.0);
        // Small-signal gain around 60 (a bypassed 12AX7 stage with 100 k: ~35 dB).
        let gain = (v1a.run(0.01) - v1a.run(-0.01)) / 0.02;
        assert!((-75.0..-45.0).contains(&gain), "small-signal gain {gain:.1} (inverting)");
        let rate = 44_100.0;
        let quiet: Vec<f64> = sine(220.0, 0.02, 1.0, rate).iter().map(|&v| v1a.run(v)).collect();
        let loud: Vec<f64> = sine(220.0, 3.0, 1.0, rate).iter().map(|&v| v1a.run(v)).collect();
        let h2_quiet = harmonic_db(&quiet, rate, 220.0, 2.0);
        let h2_loud = harmonic_db(&loud, rate, 220.0, 2.0);
        assert!(h2_quiet < -40.0, "quiet 2nd harmonic {h2_quiet:.1} dB");
        // A triode's clipping is lopsided: strong even harmonics.
        assert!(h2_loud > -20.0, "driven 2nd harmonic {h2_loud:.1} dB");
        // Control: a symmetric clipper driven as hard makes almost none.
        let sym: Vec<f64> = sine(220.0, 3.0, 1.0, rate).iter().map(|&v| (v * 2.0).tanh()).collect();
        assert!(harmonic_db(&sym, rate, 220.0, 2.0) < -60.0, "the measurement can tell symmetric from lopsided");
    }

    #[test]
    fn oversampling_keeps_aliases_down() {
        let rate = 44_100.0;
        let hz = 3100.0;
        let x = sine(hz, 0.3, 1.5, rate);
        let s = Settings { gain: 0.9, ..Settings::default() };
        // Energy away from the harmonics of 3.1 kHz (which all sit above the cab's 5 kHz fall but
        // the first) is aliasing. Measure before the cabinet: compare at the same chain with and without 4x.
        let alias_db = |over: usize| {
            let y = marshall_with(&x, rate, &s, over);
            let sp = spectrum(&y[22_050..22_050 + 32_768]);
            let bin_hz = rate / 32_768.0;
            let mut harm = 0.0f64;
            let mut alias = 0.0f64;
            for (k, &m) in sp.iter().enumerate() {
                let f = k as f64 * bin_hz;
                if !(200.0..4800.0).contains(&f) {
                    continue;
                }
                let near_harmonic = ((f / hz).round() - f / hz).abs() * hz < 30.0;
                if near_harmonic { harm += m * m } else { alias += m * m }
            }
            10.0 * (alias / harm).log10()
        };
        let (with, without) = (alias_db(4), alias_db(1));
        assert!(with < without - 10.0, "aliasing with 4x {with:.1} dB, without {without:.1} dB");
        assert!(with < -35.0, "aliasing with 4x {with:.1} dB");
    }

    /// Total harmonic distortion of the whole amp for a 330 Hz sine of `volts` peak.
    fn amp_thd(volts: f64, gain: f64) -> f64 {
        let rate = 44_100.0;
        let y = marshall(&sine(330.0, volts, 1.0, rate), rate, &Settings { gain, ..Settings::default() });
        let seg = &y[22_050..];
        let rms2 = seg.iter().map(|a| a * a).sum::<f64>() / seg.len() as f64;
        let (mut c, mut d) = (0.0, 0.0);
        for (i, a) in seg.iter().enumerate() {
            let ph = TAU * 330.0 * (i + 22_050) as f64 / rate;
            c += a * ph.sin();
            d += a * ph.cos();
        }
        let fund2 = 2.0 * (c * c + d * d) / (seg.len() * seg.len()) as f64;
        ((rms2 - fund2).max(0.0) / fund2).sqrt()
    }

    #[test]
    fn soft_playing_is_clean_and_hard_playing_breaks_up() {
        // A grid that conducted from 0 V instead of from its cathode's bias once made every level
        // distort alike (56% at 10 mV); this catches that.
        let quiet = amp_thd(0.01, 0.3);
        let hard = amp_thd(1.0, 0.3);
        assert!(quiet < 0.03, "10 mV at gain 3: {:.1}% THD", quiet * 100.0);
        assert!(hard > 0.2, "1 V at gain 3: {:.1}% THD", hard * 100.0);
        // More gain, more distortion for the same playing.
        assert!(amp_thd(0.1, 0.9) > 3.0 * amp_thd(0.1, 0.3), "gain knob");
    }

    #[test]
    fn a_loud_note_leaves_the_preamp_briefly_blocked() {
        // Grid current charges the coupling capacitor: right after a loud burst, a quiet note
        // comes through quieter than it does later (bias shift that recovers).
        let rate = 44_100.0;
        let s = Settings { gain: 0.8, ..Settings::default() };
        let mut x = sine(330.0, 1.5, 0.5, rate);
        x.extend(sine(330.0, 0.02, 1.5, rate));
        let y = marshall(&x, rate, &s);
        let rms = |a: f64, b: f64| (y[(a * rate) as usize..(b * rate) as usize].iter().map(|v| v * v).sum::<f64>() / ((b - a) * rate)).sqrt();
        let (just_after, later) = (rms(0.52, 0.6), rms(1.6, 1.9));
        assert!(just_after < 0.8 * later, "just after the burst {just_after:.4}, later {later:.4}");
        // Control: with no burst first, the quiet note is steady.
        let q = marshall(&sine(330.0, 0.02, 2.0, rate), rate, &s);
        let rq = |a: f64, b: f64| (q[(a * rate) as usize..(b * rate) as usize].iter().map(|v| v * v).sum::<f64>() / ((b - a) * rate)).sqrt();
        assert!((rq(0.52, 0.6) / rq(1.6, 1.9) - 1.0).abs() < 0.1, "steady without a burst");
    }
}
