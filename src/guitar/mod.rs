//! A steel-string acoustic guitar, modelled physically and rendered offline.
//!
//! Each string is a set of modes (partials) in two polarizations: vertical
//! (into the top plate) and horizontal (parallel to it). Every partial has the
//! stiff-string frequency `n f0 sqrt(1 + B n^2)` and its own decay rate from
//! the string's internal losses (Valette/Woodhouse model) plus the loss into
//! the body at the bridge, computed from the body's mobility at that frequency.
//! So the body shapes the strings the way a real one does: notes near strong
//! body modes die fast.
//!
//! The partials' bridge forces drive the body (convolution with its impulse
//! responses); the sound is the body's radiating motion (each mode weighted by
//! how well it moves air) through a breathing-sphere radiation filter (Woodhouse 2004). The bridge motion also
//! drives the other, open strings: sympathetic resonance. Large plucks start
//! slightly sharp and settle (tension modulation).
//!
//! Numbers, sources and which ones are hypotheses: `docs/acoustic-guitar.md`.

pub mod amp;
pub mod body;
pub mod electric;
pub mod perform;
pub mod set;

use std::f64::consts::{PI, TAU};

use crate::Rng;
use crate::fft::{C, Convolver};
use body::{Body, Dense, Mode};

/// Young's modulus of steel, Pa (hypothesis 190-210 GPa; see the doc).
const E_STEEL: f64 = 2.0e11;

/// Extra loss once a note is damped, per second: DAMP_BASE + DAMP_PER_W * w (w in rad/s). Hypothesis.
pub const DAMP_BASE: f64 = 2.2;
pub const DAMP_PER_W: f64 = 0.0004;

#[derive(Clone, Debug)]
pub struct StringSpec {
    /// Open-string pitch as a MIDI number.
    pub open: f64,
    /// Tension at pitch, N.
    pub tension: f64,
    /// Diameter that gives the string's bending stiffness as a solid steel
    /// wire, m: the gauge for plain strings, an effective value for wound ones.
    pub stiff_d: f64,
    /// Steel cross-section carrying longitudinal stretch (tension modulation), m^2.
    pub steel_area: f64,
    /// Internal loss coefficients (Woodhouse 2012, Table I).
    pub eta_f: f64,
    pub eta_a: f64,
    pub eta_b: f64,
}

/// How the string is set in motion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tool {
    Pick,
    Finger,
    /// Hammer-on: a fretting finger comes down on a string that is already
    /// ringing. Nothing plucks it; the vibration carries on at the new length.
    /// By then the upper partials have mostly died, so the new note starts darker
    /// and softer than a pluck, with only a faint tap from the fingertip.
    Hammer,
    /// Pull-off: the fretting finger leaves sideways and plucks the string as it goes.
    Pull,
}

impl Tool {
    pub fn parse(s: &str) -> Option<Tool> {
        Some(match s {
            "pick" => Tool::Pick,
            "finger" => Tool::Finger,
            "hammer" => Tool::Hammer,
            "pull" => Tool::Pull,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Pluck {
    /// 0 = low E ... 5 = high E.
    pub string: usize,
    pub fret: u32,
    /// 0..1, how hard.
    pub velocity: f64,
    pub tool: Tool,
    /// Where the string is plucked, metres from the bridge saddle.
    pub distance: f64,
    /// Direction of the pluck: 0 = parallel to the top, PI/2 = into it.
    pub angle: f64,
    /// Scales the open strings' sympathetic ringing (0 = all muted).
    pub sympathetic: f64,
    /// Varies the small random details (exact angle, pick noise).
    pub seed: u64,
    /// Pitch movements while the note rings: bends, slides, vibrato.
    pub moves: Moves,
    /// When the player damps the note, seconds after the pluck (a finger lifting or touching the string).
    pub damp_at: Option<f64>,
}

impl Pluck {
    pub fn new(string: usize, fret: u32, velocity: f64) -> Pluck {
        Pluck { string, fret, velocity, tool: Tool::Pick, distance: 0.13, angle: 0.6, sympathetic: 1.0, seed: 1, moves: Moves::default(), damp_at: None }
    }
}

/// How a ringing note's pitch moves (times in seconds from the pluck). All three
/// scale every partial together: a bend or vibrato raises the tension, a slide
/// shortens the string.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Moves {
    /// Bend: (time, semitones) points, linear between them, held after the last;
    /// before the first point the string is unbent.
    pub bend: Vec<(f64, f64)>,
    /// Slides: (start, end, frets). The finger travels `frets` frets (negative =
    /// down) between start and end. The frets stop the string, so the pitch steps
    /// a semitone at each one rather than gliding.
    pub slides: Vec<(f64, f64, i32)>,
    /// Vibrato: (start, rate Hz, depth semitones). Steel-string vibrato is a
    /// repeated small bend, so it only goes sharp; it eases in over 0.25 s.
    pub vibrato: Option<(f64, f64, f64)>,
    /// Palm mute, 0..1: the side of the picking hand rests on the strings by the
    /// saddle and chokes them, the upper partials most.
    pub mute: f64,
}

impl Moves {
    pub fn is_empty(&self) -> bool {
        self.bend.is_empty() && self.slides.is_empty() && self.vibrato.is_none() && self.mute == 0.0
    }

    /// (semitones above the plucked fret, frets crossed by slides so far) at `t`.
    pub fn at(&self, t: f64) -> (f64, u32) {
        let mut semis = 0.0;
        if let Some(&(t0, _)) = self.bend.first()
            && t >= t0
        {
            let i = self.bend.iter().rposition(|&(bt, _)| bt <= t).unwrap_or(0);
            semis += match self.bend.get(i + 1) {
                Some(&(t1, s1)) => {
                    let (ta, sa) = self.bend[i];
                    sa + (s1 - sa) * (t - ta) / (t1 - ta).max(1e-9)
                }
                None => self.bend[i].1,
            };
        }
        let mut crossed = 0;
        for &(t0, t1, frets) in &self.slides {
            let k = (((t - t0) / (t1 - t0).max(1e-9)).clamp(0.0, 1.0) * frets.unsigned_abs() as f64).floor() as u32;
            crossed += k;
            semis += k as f64 * frets.signum() as f64;
        }
        if let Some((t0, rate, depth)) = self.vibrato
            && t > t0
        {
            let ease = ((t - t0) / 0.25).min(1.0);
            semis += depth * ease * 0.5 * (1.0 - (std::f64::consts::TAU * rate * (t - t0)).cos());
        }
        (semis, crossed)
    }

    /// `B<t>:<semitones>`, `S<start>:<end>:<frets>`, `V<start>:<rate>:<depth>`, `M<palm mute 0-1>`, joined by `;`.
    pub fn parse(s: &str) -> Result<Moves, String> {
        let mut m = Moves::default();
        for tok in s.split(';').map(str::trim).filter(|t| !t.is_empty()) {
            let (kind, rest) = tok.split_at(1);
            let nums: Vec<f64> = rest.split(':').map(|x| x.parse::<f64>().map_err(|_| format!("bad move '{tok}'"))).collect::<Result<_, _>>()?;
            match (kind, nums.as_slice()) {
                ("B", [t, semis]) => m.bend.push((*t, *semis)),
                ("S", [t0, t1, frets]) => m.slides.push((*t0, *t1, *frets as i32)),
                ("V", [t0, rate, depth]) => m.vibrato = Some((*t0, *rate, *depth)),
                ("M", [amount]) => m.mute = amount.clamp(0.0, 1.0),
                _ => return Err(format!("bad move '{tok}' (expected B<t>:<semis>, S<t0>:<t1>:<frets>, V<t>:<rate>:<depth> or M<amount>)")),
            }
        }
        m.bend.sort_by(|a, b| a.0.total_cmp(&b.0));
        Ok(m)
    }
}

#[derive(Clone, Debug)]
pub struct Guitar {
    pub rate: u32,
    pub strings: [StringSpec; 6],
    /// Nut-to-saddle length, m.
    pub scale: f64,
    pub body: Body,
    /// Horizontal-polarization mobility relative to vertical: the top is far
    /// stiffer in its own plane, so that polarization barely couples or radiates.
    pub h_mobility: f64,
    /// Fractional frequency split between polarizations (vertical higher), from
    /// the string rolling on fret and saddle.
    pub split: f64,
    /// Corner of the radiation high-pass, Hz.
    pub radiation_hz: f64,
    /// Pick-noise force relative to the pluck.
    pub pick_noise: f64,
    /// Seconds rendered at most.
    pub max_secs: f64,
    /// An electric guitar's magnetic pickup. When set, a note's output is the pickup's voltage
    /// (the string's motion over the poles, through the coil's circuit), not the body's sound.
    pub pickup: Option<electric::Pickup>,
    vb_ir: Convolver,
    rad_ir: Convolver,
}

/// One partial of one polarization, as an exponentially decaying phasor.
#[derive(Clone, Debug)]
pub struct Partial {
    pub w: f64,
    pub sigma: f64,
    /// Bridge force amplitude, N.
    pub force: f64,
    /// Modal displacement amplitude, m (for tension modulation).
    pub disp: f64,
    /// n pi / L, rad/m.
    pub kn: f64,
}

impl Guitar {
    pub fn new(rate: u32, strings: [StringSpec; 6], scale: f64, body: Body) -> Guitar {
        let (vb, rad) = body.responses(rate);
        Guitar {
            rate,
            strings,
            scale,
            body,
            h_mobility: 0.1,
            split: 5e-4,
            radiation_hz: 250.0,
            pick_noise: 0.02,
            max_secs: 6.0,
            pickup: None,
            vb_ir: Convolver::new(&vb),
            rad_ir: Convolver::new(&rad),
        }
    }

    /// Tune the strings to other open pitches (MIDI, low E first): the same strings at
    /// a different tension (f is proportional to the square root of T).
    pub fn retune(&mut self, open: [f64; 6]) {
        for (s, &m) in self.strings.iter_mut().zip(open.iter()) {
            s.tension *= 2f64.powf((m - s.open) / 6.0);
            s.open = m;
        }
    }

    pub fn open_freq(&self, s: usize) -> f64 {
        midi_hz(self.strings[s].open)
    }

    /// The (string, fret) a player in open position would use for `midi`:
    /// the highest string that can reach it, so the lowest fret.
    pub fn fingering(&self, midi: i32) -> Option<(usize, u32)> {
        (0..6).rev().find_map(|s| {
            let fret = midi - self.strings[s].open as i32;
            (0..=20).contains(&fret).then_some((s, fret as u32))
        })
    }

    /// Inharmonicity coefficient of string `s` stopped at `fret`.
    pub fn inharmonicity(&self, s: usize, fret: u32) -> f64 {
        let spec = &self.strings[s];
        let len = self.scale * 2f64.powf(-(fret as f64) / 12.0);
        let ei = E_STEEL * PI * spec.stiff_d.powi(4) / 64.0;
        PI * PI * ei / (spec.tension * len * len)
    }

    /// Partials of string `s` stopped at `fret`, for one polarization.
    /// `mobility_scale` is 1 for vertical, `h_mobility` for horizontal;
    /// `detune` multiplies every frequency (the polarization split).
    pub fn partials(&self, s: usize, fret: u32, mobility_scale: f64, detune: f64, amp: impl Fn(usize, f64) -> f64) -> Vec<Partial> {
        let spec = &self.strings[s];
        let f0 = midi_hz(spec.open + fret as f64);
        let len = self.scale * 2f64.powf(-(fret as f64) / 12.0);
        let t = spec.tension;
        // Linear density from the open string: f = sqrt(T / mu) / 2L.
        let mu = t / (4.0 * self.scale * self.scale * midi_hz(spec.open).powi(2));
        let z0 = (t * mu).sqrt();
        let b = self.inharmonicity(s, fret);
        let nyq = self.rate as f64 * 0.47;
        let mut out = vec![];
        for n in 1.. {
            let nf = n as f64;
            let f = nf * f0 * (1.0 + b * nf * nf).sqrt() / (1.0 + b).sqrt();
            if f > nyq {
                break;
            }
            let w = TAU * f;
            // Woodhouse 2012 eq. 4: eta = (eta_F + eta_A / w + eta_B B n^2) / (1 + B n^2).
            let eta = (spec.eta_f + spec.eta_a / w + spec.eta_b * b * nf * nf) / (1.0 + b * nf * nf);
            let y = self.body.mobility(f).scale(mobility_scale);
            // Bridge: |r| ~ 1 - 2 Z0 Re(Y) per round trip, phase ~ -2 Z0 Im(Y).
            let sigma = eta * w / 2.0 + 2.0 * f0 * z0 * y.re.max(0.0);
            let dw = -2.0 * f0 * z0 * y.im;
            let a = amp(n, f);
            let kn = nf * PI / len;
            out.push(Partial { w: (w + dw) * detune, sigma, force: t * kn * a, disp: a, kn });
        }
        out
    }

    /// Render one plucked note, mono, at `self.rate`, in the model's own pressure units.
    pub fn render(&self, p: &Pluck) -> Vec<f64> {
        self.render_for(p, self.max_secs)
    }

    /// `render`, computing at most `secs` seconds (cheaper for notes that will be damped).
    pub fn render_for(&self, p: &Pluck, secs: f64) -> Vec<f64> {
        let rate = self.rate as f64;
        let dt = 1.0 / rate;
        let n = (secs.max(0.05) * rate) as usize;
        let mut rng = Rng::new(p.seed);
        let spec = &self.strings[p.string];
        let len = self.scale * 2f64.powf(-(p.fret as f64) / 12.0);
        let beta = (p.distance / len).clamp(0.02, 0.5);
        let v = p.velocity.clamp(0.0, 1.0);
        // Pluck amplitude (displacement at the plucking point), m. Hypothesis.
        // A pick or finger applies a force; the string's stiffness at that point is T len / (d (len - d)),
        // so a shorter (fretted) string deflects less. pluck_amplitude is the open string's deflection;
        // scale by the fretted length's compliance relative to the open string's (same pluck distance).
        // Without this, high-fret notes were plucked as far as open ones: too loud and, through tension
        // modulation, 37 cents sharp gliding down for seconds (the founder heard it as "wah", 2026-10-27).
        let d = beta * len;
        let open_d = d.min(0.5 * self.scale);
        let compliance = (d * (len - d) / len) / (open_d * (self.scale - open_d) / self.scale);
        let h = pluck_amplitude(v) * compliance;
        // Width of the contact, as Woodhouse's Gaussian exp(-(n pi w / L)^2).
        // He used 7.5 mm for a finger. The pick width was fitted to GuitarSet's
        // high bands; the velocity dependence (harder = brighter) is a hypothesis.
        let width = match p.tool {
            Tool::Pick => 2.5e-3 - 1.5e-3 * v,
            Tool::Finger => 9.0e-3 - 3.0e-3 * v,
            // Hypotheses for the ear: a hammer carries an already-smoothed shape
            // (wide), a pull-off is a fingertip flick (between finger and pick).
            Tool::Hammer => 12.0e-3,
            Tool::Pull => 6.0e-3,
        };
        let angle = p.angle + 0.15 * rng.signed();
        let amp = |k: usize, _f: f64| {
            let kf = k as f64;
            // Triangle released from rest: 2h sin(n pi beta) / (n^2 pi^2 beta (1 - beta)).
            let tri = 2.0 * h * (kf * PI * beta).sin() / (kf * kf * PI * PI * beta * (1.0 - beta));
            tri * (-(kf * PI * width / len).powi(2)).exp()
        };
        let mut vert = self.partials(p.string, p.fret, 1.0, 1.0, |k, f| amp(k, f) * angle.sin());
        let mut horiz = self.partials(p.string, p.fret, self.h_mobility, 1.0 - self.split, |k, f| amp(k, f) * angle.cos());
        // The player tunes by ear to what sounds over time: weight each polarization's
        // fundamental by its radiated amplitude times its decay time, and put that
        // average on pitch. (Near a strong body mode the vertical one dies within a
        // second and the horizontal one carries the sustain.)
        let wv = vert[0].force.abs() / vert[0].sigma;
        let wh = self.h_mobility * horiz[0].force.abs() / horiz[0].sigma;
        let heard = ((wv * vert[0].w.ln() + wh * horiz[0].w.ln()) / (wv + wh).max(1e-30)).exp();
        let fix = TAU * midi_hz(spec.open + p.fret as f64) / heard;
        vert.iter_mut().chain(horiz.iter_mut()).for_each(|q| q.w *= fix);

        // Tension modulation: the string stretches by (1/4) sum (a_n n pi / L)^2,
        // raising tension by EA * strain and frequency by half that, fractionally.
        let ea_over_t = E_STEEL * spec.steel_area / spec.tension;
        let mut force = vec![0.0; n];
        const BLOCK: usize = 32;
        // Each phasor also carries what the pickup (if any) hears of it: modal amplitude times the
        // pickup's weighting of that mode shape, for its polarization.
        let heard_by = |q: &Partial, polar: f64| self.pickup.as_ref().map_or(0.0, |pu| q.disp * pu.weight(q.kn) * pu.string_gain[p.string] * polar);
        let (pv, ph) = self.pickup.as_ref().map_or((0.0, 0.0), |pu| (1.0, pu.horizontal));
        let mut phasors: Vec<(C, &Partial, f64, f64)> = vert
            .iter()
            .map(|q| (C::new(1.0, 0.0), q, 1.0, heard_by(q, pv)))
            .chain(horiz.iter().map(|q| (C::new(1.0, 0.0), q, self.h_mobility, heard_by(q, ph))))
            .collect();
        let peak_force = phasors.iter().map(|(_, q, g, _)| (q.force * g).abs()).fold(0.0, f64::max);
        let mut pick = vec![0.0; if self.pickup.is_some() { n } else { 0 }];
        // On an electric guitar a mode matters if the pickup hears it, however little it moves the bridge.
        let peak_pick = phasors.iter().map(|p| p.3.abs()).fold(0.0, f64::max);
        let keep = |q: &Partial, g: f64, h: f64, z: f64| {
            if self.pickup.is_some() { z * h.abs() > peak_pick * 1e-6 } else { z * (q.force * g).abs() > peak_force * 1e-6 }
        };
        phasors.retain(|(_, q, g, h)| keep(q, *g, *h, 10.0));
        let all: Vec<&Partial> = vert.iter().chain(horiz.iter()).collect();
        let nyq_w = TAU * rate * 0.47;
        let mut crossed_before = 0;
        let mut start = 0;
        while start < n && !phasors.is_empty() {
            let t = start as f64 * dt;
            let strain: f64 = all.iter().map(|q| (q.disp * q.kn).powi(2) * (-2.0 * q.sigma * t).exp()).sum::<f64>() / 4.0;
            let glide = 1.0 + 0.5 * ea_over_t * strain;
            // Bends, slides and vibrato scale every partial together.
            let (semis, crossed) = if p.moves.is_empty() { (0.0, 0) } else { p.moves.at(t) };
            let moved = 2f64.powf(semis / 12.0);
            // Each fret a slide crosses takes a little energy (finger friction, fret contact). Hypothesis.
            let loss = 0.97f64.powi((crossed - crossed_before) as i32);
            crossed_before = crossed;
            let end = (start + BLOCK).min(n);
            for (z, q, g, heard) in phasors.iter_mut() {
                let w = q.w * glide * moved;
                // Palm mute: extra damping growing with frequency (hypothesis: at full mute a
                // 100 Hz partial dies in ~0.35 s, a 2 kHz one in ~0.15 s).
                let mut choke = p.moves.mute * (18.0 + 0.0025 * w);
                // Damping: a finger lifting off the fret, or touching the string, adds losses that grow
                // with frequency, so the highs go first and a tail is left. The founder asked for 10x the
                // first version's tail (hypothesis: 60 dB down in ~2.8 s at 100 Hz, ~1.5 s at 1 kHz,
                // ~0.7 s at 3 kHz, on top of the string's own losses).
                if p.damp_at.is_some_and(|d| t >= d) {
                    choke += DAMP_BASE + DAMP_PER_W * w;
                }
                let rot = C::new((-(q.sigma + choke) * dt).exp(), 0.0).mul(C::expi(w * dt));
                *z = z.scale(loss);
                // A partial pushed above the band by a bend or slide falls silent instead of aliasing.
                let a = if w < nyq_w { q.force * *g } else { 0.0 };
                if pick.is_empty() {
                    for o in force[start..end].iter_mut() {
                        *o += a * z.re;
                        *z = z.mul(rot);
                    }
                } else {
                    // The pickup senses velocity: d/dt Re(z) = Re((-sigma + i w) z), about -w Im(z).
                    let b = if w < nyq_w { -*heard * w } else { 0.0 };
                    for o in pick[start..end].iter_mut() {
                        *o += b * z.im;
                        *z = z.mul(rot);
                    }
                }
            }
            phasors.retain(|(z, q, g, h)| keep(q, *g, *h, z.abs()));
            start = end;
        }

        // Slide squeak: a finger sliding along a wound string rubs over its windings, a
        // buzz at (slide speed / winding pitch). Winding pitch ~0.5 mm and level are hypotheses.
        // EJ16's four lowest strings are wound (phosphor bronze over steel).
        if p.string <= 3 {
            for &(t0, t1, frets) in &p.moves.slides {
                let (a, b) = ((t0.max(0.0) * rate) as usize, ((t1 * rate) as usize).min(n));
                if b <= a {
                    continue;
                }
                let travel = (self.scale * (2f64.powf(-(p.fret as f64) / 12.0) - 2f64.powf(-((p.fret as i32 + frets) as f64) / 12.0))).abs();
                let hz = travel / (t1 - t0).max(1e-3) / 0.5e-3;
                let mut phase = 0.0f64;
                let mut lp = 0.0;
                for (i, f) in force[a..b].iter_mut().enumerate() {
                    let x = i as f64 / (b - a) as f64;
                    phase += TAU * hz * dt;
                    lp += (rng.signed() - lp) * 0.3;
                    *f += 0.012 * peak_force * (x * PI).sin() * (phase.sin() * 0.7 + lp * 0.3);
                }
            }
        }

        // Pick noise: a short burst as the pick scrapes off, into the bridge. Hypothesis.
        let noise_len = (0.004 * rate) as usize;
        let noise_amp = self.pick_noise
            * peak_force
            * match p.tool {
                Tool::Pick => 1.0,
                Tool::Finger => 0.3,
                Tool::Hammer => 0.15,
                Tool::Pull => 0.25,
            };
        let mut lp = 0.0;
        for (i, f) in force.iter_mut().take(noise_len).enumerate() {
            lp += (rng.signed() - lp) * 0.5;
            *f += noise_amp * lp * (-(i as f64) / (0.0008 * rate)).exp();
        }

        if let Some(pu) = &self.pickup {
            // A solid body barely radiates and the pickup only hears the strings: the output is the
            // pickup's voltage. (Pick and slide noise reach a pickup only as string motion; not modelled.)
            let mut out = pu.circuit(&pick, rate);
            trim_tail(&mut out, rate);
            return out;
        }

        let vb = self.vb_ir.apply(&force);
        let mut radiating = self.rad_ir.apply(&force);

        // Sympathetic: open strings driven by the bridge's acceleration, their
        // forces fed back through the body.
        if p.sympathetic > 0.0 {
            let accel = differentiate(&vb, rate);
            let played: Vec<f64> = vert.iter().filter(|q| q.force.abs() > peak_force * 1e-4).map(|q| q.w).collect();
            let mut sym = vec![0.0; n];
            let damp_from = p.damp_at.map(|d| (d * rate) as usize);
            for s in (0..6).filter(|&s| s != p.string) {
                let t = self.strings[s].tension;
                for (k, q) in self.partials(s, 0, 1.0, 1.0, |_, _| 1.0).iter().enumerate() {
                    let near = q.w < TAU * 600.0 || played.iter().any(|&w| (w - q.w).abs() < (TAU * 2.0).max(4.0 * q.sigma));
                    if !near {
                        continue;
                    }
                    let kf = (k + 1) as f64;
                    // q'' + 2 sigma q' + w^2 q = -(2 / (n pi)) a(t); force back on the bridge = T n pi / L q.
                    let g = -2.0 / (kf * PI) * p.sympathetic * dt;
                    let pole = C::new((-q.sigma * dt).exp(), 0.0).mul(C::expi(q.w * dt));
                    // When the player damps the note, the same hand stops the strings it set ringing.
                    let damped = pole.scale((-(DAMP_BASE + DAMP_PER_W * q.w) * dt).exp());
                    let from = damp_from.unwrap_or(n);
                    // Impulse response of 1 / ((s - p)(s - p*)) is e^{-sigma t} sin(w t) / w.
                    let fk = t * q.kn / q.w;
                    let mut z = C::default();
                    for i in 0..n {
                        z = z.mul(if i < from { pole } else { damped });
                        z.re += accel[i] * g;
                        sym[i] += fk * z.im;
                    }
                }
            }
            let sym_rad = self.rad_ir.apply(&sym);
            radiating.iter_mut().zip(&sym_rad).for_each(|(a, b)| *a += b);
        }

        // Sound = the radiating velocity through the breathing-sphere filter (i w / wc) / (1 + i w / wc):
        // pressure follows acceleration below wc and velocity above it.
        let mut out = highpass(&radiating, self.radiation_hz, rate);
        trim_tail(&mut out, rate);
        out
    }
}

/// Pluck displacement at the plucking point for a 0..1 velocity, m (hypothesis).
pub fn pluck_amplitude(v: f64) -> f64 {
    0.3e-3 + 2.7e-3 * v.clamp(0.0, 1.0).powf(1.5)
}

fn differentiate(x: &[f64], rate: f64) -> Vec<f64> {
    (0..x.len()).map(|i| (x[i] - if i > 0 { x[i - 1] } else { 0.0 }) * rate).collect()
}

/// First-order high-pass (bilinear, prewarped): (s / wc) / (1 + s / wc).
fn highpass(x: &[f64], hz: f64, rate: f64) -> Vec<f64> {
    let k = (PI * hz / rate).tan();
    let (b0, a1) = (1.0 / (1.0 + k), (1.0 - k) / (1.0 + k));
    let mut y = vec![0.0; x.len()];
    let (mut px, mut py) = (0.0, 0.0);
    for (o, &xi) in y.iter_mut().zip(x) {
        *o = b0 * (xi - px) + a1 * py;
        px = xi;
        py = *o;
    }
    y
}

/// Cut the tail once it stays 70 dB under the peak, with a short fade.
fn trim_tail(x: &mut Vec<f64>, rate: f64) {
    let peak = x.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    let floor = peak * 10f64.powf(-70.0 / 20.0);
    let last = x.iter().rposition(|v| v.abs() > floor).unwrap_or(0);
    let fade = (0.03 * rate) as usize;
    let end = (last + fade).min(x.len());
    x.truncate(end);
    let from = end.saturating_sub(fade);
    for (i, v) in x[from..].iter_mut().enumerate() {
        *v *= 1.0 - i as f64 / fade as f64;
    }
}

pub fn midi_hz(m: f64) -> f64 {
    440.0 * 2f64.powf((m - 69.0) / 12.0)
}

/// A steel-string flat-top: D'Addario EJ16 light phosphor bronze on a 25.4"
/// scale, Woodhouse's measured string losses (new strings), and a body built on
/// Carcagno et al.'s steel-string mode measurements. Sources: `docs/acoustic-guitar.md`.
pub fn steel_string(rate: u32) -> Guitar {
    let inch = 0.0254;
    let area = |d_in: f64| PI * (d_in * inch).powi(2) / 4.0;
    // (open, tension N @25.4", stiffness diameter in, steel area, eta_F, eta_A, eta_B)
    let s = |open: f64, t: f64, stiff_in: f64, steel_in: f64, ef: f64, ea: f64, eb: f64| StringSpec {
        open,
        tension: t,
        stiff_d: stiff_in * inch,
        steel_area: area(steel_in),
        eta_f: ef,
        eta_a: ea,
        eta_b: eb,
    };
    let strings = [
        s(40.0, 110.1, 0.0186, 0.0186, 5e-5, 1.7, 0.3e-2),
        s(45.0, 127.7, 0.0156, 0.0156, 11e-5, 1.2, 0.4e-2),
        s(50.0, 132.1, 0.0145, 0.0145, 9e-5, 1.3, 1.1e-2),
        s(55.0, 132.6, 0.0142, 0.0142, 11e-5, 1.2, 1.8e-2),
        s(59.0, 102.9, 0.016, 0.016, 6e-5, 1.0, 1.1e-2),
        s(64.0, 103.1, 0.012, 0.012, 7e-5, 2.5, 0.7e-2),
    ];
    let d = BodyDesign::steel_string();
    Guitar::new(rate, strings, 25.4 * inch, d.build())
}

/// The body as editable numbers, so it can be varied (the workbench) and rebuilt.
#[derive(Clone, Debug)]
pub struct BodyDesign {
    /// The low "triad": (Hz, Q, peak mobility m/(N s), radiation efficiency).
    pub triad: [(f64, f64, f64, f64); 3],
    pub dense: Dense,
}

impl BodyDesign {
    pub fn steel_string() -> BodyDesign {
        BodyDesign {
            // Carcagno 2018: means of six steel-string guitars; peaks of about 0.03 m/(N s) (Fig. 1).
            // The third is "a transverse bending mode of the top plate": its halves move in opposite
            // directions, so it moves the bridge (and damps strings) strongly but radiates poorly.
            // Its 0.3 is a hypothesis, set after the founder heard F4 (on that mode) as harsh and too loud.
            triad: [(99.0, 37.0, 0.03, 1.0), (182.0, 23.0, 0.03, 1.0), (350.0, 40.0, 0.02, 0.3)],
            // Above the triad: "a relatively smooth and lower-level trace" (Carcagno 2018). Its level
            // and the bridge-mass corner were fitted to GuitarSet recordings (docs, "Fitted").
            dense: Dense { from_hz: 420.0, to_hz: 20_000.0, spacing: 35.0, q: 35.0, mobility: 0.0015, mass_corner: 8000.0, seed: 7 },
        }
    }

    pub fn build(&self) -> Body {
        // Peak mobility of a mode is q / (m w).
        let named: Vec<Mode> = self.triad.iter().map(|&(f, q, peak, rad)| Mode { f, q, m: q / (peak * TAU * f), rad }).collect();
        Body::new(&named, &self.dense)
    }
}

/// A steel-string guitar with a different body (strings and everything else as `steel_string`).
pub fn steel_string_with(rate: u32, body: &BodyDesign) -> Guitar {
    let g = steel_string(rate);
    Guitar::new(rate, g.strings, g.scale, body.build())
}

impl Guitar {
    /// Diagnostic: per partial of string `s` at `fret`: (Hz, internal sigma, bridge sigma, Re Y, Im Y).
    pub fn loss_table(&self, s: usize, fret: u32, count: usize) -> Vec<(f64, f64, f64, f64, f64)> {
        let spec = &self.strings[s];
        let f0 = midi_hz(spec.open + fret as f64);
        let t = spec.tension;
        let mu = t / (4.0 * self.scale * self.scale * midi_hz(spec.open).powi(2));
        let z0 = (t * mu).sqrt();
        let b = self.inharmonicity(s, fret);
        (1..=count)
            .map(|k| {
                let nf = k as f64;
                let f = nf * f0;
                let w = TAU * f;
                let eta = (spec.eta_f + spec.eta_a / w + spec.eta_b * b * nf * nf) / (1.0 + b * nf * nf);
                let y = self.body.mobility(f);
                (f, eta * w / 2.0, 2.0 * f0 * z0 * y.re.max(0.0), y.re, y.im)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fundamental of a rendered note, Hz: FFT peak near `near` over 0.8-1.6 s, interpolated.
    fn measured_f0(x: &[f64], rate: f64, near: f64) -> f64 {
        let (a, b) = ((0.8 * rate) as usize, (1.6 * rate) as usize);
        let size = 1 << 19;
        let mut buf = vec![C::default(); size];
        for (i, v) in x[a..b].iter().enumerate() {
            let w = 0.5 - 0.5 * (TAU * i as f64 / (b - a) as f64).cos();
            buf[i] = C::new(v * w, 0.0);
        }
        crate::fft::fft(&mut buf, false);
        let bin = rate / size as f64;
        let (lo, hi) = ((near * 0.97 / bin) as usize, (near * 1.03 / bin) as usize);
        let k = (lo..hi).max_by(|&i, &j| buf[i].abs().total_cmp(&buf[j].abs())).unwrap();
        let (p, q, r) = (buf[k - 1].abs().ln(), buf[k].abs().ln(), buf[k + 1].abs().ln());
        (k as f64 + 0.5 * (p - r) / (p - 2.0 * q + r)) * bin
    }

    fn cents(f: f64, target: f64) -> f64 {
        1200.0 * (f / target).log2()
    }

    #[test]
    fn notes_away_from_body_modes_sound_on_pitch() {
        let g = steel_string(44_100);
        // Open A, D and high E, and the 7th fret of the G: none sits on a strong body mode.
        for (s, f) in [(1, 0), (2, 0), (5, 0), (3, 7)] {
            let x = g.render(&Pluck::new(s, f, 0.5));
            let target = midi_hz(g.strings[s].open + f as f64);
            let c = cents(measured_f0(&x, 44_100.0, target), target);
            assert!(c.abs() < 2.0, "string {s} fret {f}: {c:+.2} cents");
        }
    }

    #[test]
    fn the_pitch_check_can_fail() {
        // Same measurement against a target 20 cents off must report the error.
        let g = steel_string(44_100);
        let x = g.render(&Pluck::new(5, 0, 0.5));
        let wrong = midi_hz(64.2);
        assert!(cents(measured_f0(&x, 44_100.0, wrong), wrong) < -15.0);
    }

    #[test]
    fn inharmonicity_matches_published_values() {
        // Woodhouse 2012: low E ~1.1e-4 (two independent measurements), high E ~2.0e-5.
        let g = steel_string(44_100);
        let (low, high) = (g.inharmonicity(0, 0), g.inharmonicity(5, 0));
        assert!((low / 1.1e-4 - 1.0).abs() < 0.1, "low E B {low:.2e}");
        assert!((high / 2.0e-5 - 1.0).abs() < 0.1, "high E B {high:.2e}");
        // Stopping at the 12th fret halves the length: four times the B.
        assert!((g.inharmonicity(0, 12) / low - 4.0).abs() < 1e-9);
    }

    #[test]
    fn renders_are_deterministic_finite_and_seeded() {
        let g = steel_string(44_100);
        let p = Pluck::new(0, 3, 0.8);
        let (a, b) = (g.render(&p), g.render(&p));
        assert_eq!(a, b, "same pluck, same samples");
        assert!(a.iter().all(|v| v.is_finite()));
        let c = g.render(&Pluck { seed: 2, ..p });
        assert!(a.iter().zip(&c).any(|(x, y)| (x - y).abs() > 1e-9), "a different seed varies the pluck");
    }

    #[test]
    fn harder_plucks_are_louder_and_brighter() {
        let g = steel_string(44_100);
        let measure = |v: f64| {
            let x = g.render(&Pluck::new(2, 2, v));
            let head = &x[..4410];
            let rms = (head.iter().map(|v| v * v).sum::<f64>() / head.len() as f64).sqrt();
            // Brightness: share of energy in the first difference (a crude high-pass).
            let hi = head.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum::<f64>() / head.iter().map(|v| v * v).sum::<f64>();
            (rms, hi)
        };
        let (soft, hard) = (measure(0.2), measure(0.9));
        assert!(hard.0 > soft.0 * 2.0, "loudness {soft:?} vs {hard:?}");
        assert!(hard.1 > soft.1, "brightness {soft:?} vs {hard:?}");
    }

    #[test]
    fn moves_follow_their_shapes() {
        let m = Moves::parse("B0.1:0;B0.3:2;S1:1.4:3;V2:5:0.5").unwrap();
        assert_eq!(m.at(0.05), (0.0, 0), "before the bend starts");
        assert!((m.at(0.2).0 - 1.0).abs() < 1e-9, "halfway up the bend");
        assert!((m.at(0.9).0 - 2.0).abs() < 1e-9, "bend held");
        // The slide steps a whole semitone at each fret: never in between.
        for t in [1.05, 1.15, 1.25, 1.35] {
            let (s, crossed) = m.at(t);
            assert!(((s - 2.0) - (s - 2.0).round()).abs() < 1e-9 && (s - 2.0) as u32 == crossed, "t={t}: {s}");
        }
        assert_eq!(m.at(1.5), (5.0, 3));
        // Vibrato only goes sharp, never below the held pitch.
        let lowest = (0..400).map(|i| m.at(2.0 + i as f64 * 0.005).0).fold(f64::INFINITY, f64::min);
        let highest = (0..400).map(|i| m.at(2.0 + i as f64 * 0.005).0).fold(0.0, f64::max);
        assert!((lowest - 5.0).abs() < 1e-6 && (highest - 5.5).abs() < 1e-3, "{lowest}..{highest}");
        assert!(Moves::parse("X1:2").is_err() && Moves::parse("B1").is_err());
    }

    #[test]
    fn a_bend_and_a_slide_land_on_their_pitches() {
        let g = steel_string(44_100);
        let target = midi_hz(g.strings[3].open + 7.0);
        let mut p = Pluck::new(3, 7, 0.6);
        p.moves = Moves::parse("B0.1:0;B0.3:2").unwrap();
        let bent = g.render(&p);
        let c = cents(measured_f0(&bent, 44_100.0, target * 2f64.powf(2.0 / 12.0)), target);
        assert!((c - 200.0).abs() < 4.0, "bend up a tone: {c:+.1} cents");
        p.moves = Moves::parse("S0.2:0.5:3").unwrap();
        let slid = g.render(&p);
        let c = cents(measured_f0(&slid, 44_100.0, target * 2f64.powf(3.0 / 12.0)), target);
        assert!((c - 300.0).abs() < 4.0, "slide up three frets: {c:+.1} cents");
        // The check can fail: the unmoved note is nowhere near +300.
        let plain = g.render(&Pluck::new(3, 7, 0.6));
        assert!(cents(measured_f0(&plain, 44_100.0, target), target).abs() < 4.0);
    }

    #[test]
    fn retuning_moves_the_pitch_and_lowers_tension() {
        let mut g = steel_string(44_100);
        let before = g.strings[0].tension;
        g.retune([39.0, 44.0, 49.0, 54.0, 58.0, 63.0]); // E flat tuning
        assert!((g.strings[0].tension / before - 2f64.powf(-1.0 / 6.0)).abs() < 1e-12, "a semitone down is 0.89 x tension");
        let x = g.render(&Pluck::new(0, 0, 0.5));
        let target = midi_hz(39.0);
        assert!(cents(measured_f0(&x, 44_100.0, target), target).abs() < 2.0, "open low string sounds E flat 2");
    }

    #[test]
    fn palm_mute_chokes_the_note() {
        let g = steel_string(44_100);
        let open = g.render(&Pluck::new(0, 0, 0.6));
        let mut p = Pluck::new(0, 0, 0.6);
        p.moves = Moves::parse("M1").unwrap();
        let muted = g.render(&p);
        let rms = |x: &[f64], a: f64, b: f64| (x[(a * 44_100.0) as usize..(b * 44_100.0) as usize].iter().map(|v| v * v).sum::<f64>() / ((b - a) * 44_100.0)).sqrt();
        let (o, m) = (rms(&open, 0.5, 0.6), rms(&muted, 0.5, 0.6).max(1e-12));
        assert!(20.0 * (o / m).log10() > 20.0, "half a second in, the muted note is 20 dB quieter: {:.1} dB", 20.0 * (o / m).log10());
        assert!(rms(&muted, 0.0, 0.05) > rms(&open, 0.0, 0.05) * 0.3, "but its attack is still there");
    }

    #[test]
    fn fingering_covers_the_range_in_open_position() {
        let g = steel_string(44_100);
        for m in 40..=84 {
            let (s, f) = g.fingering(m).unwrap_or_else(|| panic!("{m} unplayable"));
            assert_eq!(g.strings[s].open as i32 + f as i32, m);
        }
        assert_eq!(g.fingering(45), Some((1, 0)), "A2 is the open A string");
        assert_eq!(g.fingering(39), None);
    }
}
