//! A performance: timed plucks rendered straight from the model (no sampler), so
//! every note is on the string and fret a player would use and rings until that
//! string is played again or damped. Plus a small synthetic room for listening.
//!
//! Event file (`#` lines are comments), one pluck per line:
//! `time_s, string 0-5, fret, velocity 0-1, pick|finger|hammer|pull, end_s[, distance_m, angle_rad[, moves]]`
//! where moves (see `Moves::parse`) are bends, slides and vibrato, e.g. `B0.05:0;B0.2:2;V0.4:5.5:0.3`
//! where `end_s` is when the note is damped (next pluck on that string, or the
//! fretting finger leaving).

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{Guitar, Pluck, Tool};
use crate::Rng;
use crate::fft::Convolver;

#[derive(Clone, Debug)]
pub struct Event {
    pub t: f64,
    pub string: usize,
    pub fret: u32,
    pub velocity: f64,
    pub tool: Tool,
    pub end: f64,
    pub distance: Option<f64>,
    pub angle: Option<f64>,
    pub moves: super::Moves,
}

/// Open-string pitches from a `#tuning, 39, 44, 49, 54, 58, 63` line (MIDI, low E first), if any.
pub fn tuning(text: &str) -> Result<Option<[f64; 6]>, String> {
    let Some(line) = text.lines().find(|l| l.trim_start().starts_with("#tuning")) else { return Ok(None) };
    let nums: Vec<f64> = line.split(|c: char| c == ',' || c.is_whitespace()).skip(1).filter(|s| !s.is_empty()).map(|s| s.parse::<f64>().map_err(|_| format!("bad tuning line: {line}"))).collect::<Result<_, _>>()?;
    if nums.len() != 6 || nums.iter().any(|m| !(20.0..=80.0).contains(m)) {
        return Err(format!("tuning needs six MIDI pitches: {line}"));
    }
    Ok(Some([nums[0], nums[1], nums[2], nums[3], nums[4], nums[5]]))
}

pub fn parse(text: &str) -> Result<Vec<Event>, String> {
    let mut out = vec![];
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split(',').map(str::trim).collect();
        let err = |what: &str| format!("line {}: {what}: {line}", n + 1);
        if f.len() < 6 {
            return Err(err("need at least 6 fields"));
        }
        let num = |i: usize| f[i].parse::<f64>().map_err(|_| err("not a number"));
        let string = num(1)? as usize;
        if string > 5 {
            return Err(err("string must be 0-5"));
        }
        let tool = Tool::parse(f[4]).ok_or_else(|| err("tool must be pick, finger, hammer or pull"))?;
        out.push(Event {
            t: num(0)?,
            string,
            fret: num(2)? as u32,
            velocity: num(3)?.clamp(0.0, 1.0),
            tool,
            end: num(5)?,
            distance: f.get(6).and_then(|s| s.parse().ok()),
            angle: f.get(7).and_then(|s| s.parse().ok()),
            moves: match f.get(8) {
                Some(m) => super::Moves::parse(m).map_err(|e| err(&e))?,
                None => super::Moves::default(),
            },
        });
    }
    Ok(out)
}

/// Seconds of release tail rendered after a damped note (the damping itself is physical).
const TAIL: f64 = 4.0;

/// Mono mix of all events. `sympathetic` scales each note's open-string ringing
/// (lower than for single notes, since the other strings are often busy).
pub fn perform(g: &Guitar, events: &[Event], sympathetic: f64) -> Vec<f64> {
    let rate = g.rate as f64;
    // Room for every note's release tail (see `tail` below), so the last notes fade instead of stopping.
    let total = events.iter().map(|e| e.end.max(e.t)).fold(0.0, f64::max) + TAIL + 0.1;
    let mix = Mutex::new(vec![0.0f64; (total * rate) as usize + 1]);
    let next = AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(e) = events.get(i) else { break };
                    let held = (e.end - e.t).max(0.02);
                    // Every note is held for its written length and then released: the physical damping
                    // (Pluck::damp_at) and its full tail, always (the founder's rule, 2026-10-27).
                    let tail = TAIL;
                    let mut p = Pluck::new(e.string, e.fret, e.velocity);
                    p.tool = e.tool;
                    p.seed = i as u64 + 1;
                    p.sympathetic = sympathetic;
                    if let Some(d) = e.distance {
                        p.distance = d;
                    }
                    if let Some(a) = e.angle {
                        p.angle = a;
                    }
                    p.moves = e.moves.clone();
                    p.damp_at = Some(held);
                    let mut x = g.render_for(&p, held + tail);
                    // Only a 10 ms taper at the very end, against a click where the render stops.
                    let n = x.len();
                    let f = ((0.01 * rate) as usize).min(n);
                    for (k, v) in x[n - f..].iter_mut().enumerate() {
                        *v *= 1.0 - k as f64 / f as f64;
                    }
                    let at = (e.t.max(0.0) * rate) as usize;
                    let mut m = mix.lock().unwrap();
                    for (k, v) in x.iter().enumerate() {
                        if let Some(o) = m.get_mut(at + k) {
                            *o += v;
                        }
                    }
                }
            });
        }
    });
    let mut out = mix.into_inner().unwrap();
    let last = out.iter().rposition(|v| v.abs() > 1e-9).unwrap_or(0);
    out.truncate(last + (0.05 * rate) as usize);
    out
}

/// A small synthetic room: stereo decaying noise (high frequencies dying faster),
/// a few early reflections, `rt60` seconds. Returns (left, right) = dry + wet.
pub fn room(x: &[f64], rate: u32, rt60: f64, wet: f64) -> (Vec<f64>, Vec<f64>) {
    let r = rate as f64;
    let len = (rt60 * 1.2 * r) as usize;
    let ir = |seed: u64| -> Vec<f64> {
        let mut rng = Rng::new(seed);
        let mut ir = vec![0.0; len];
        let pre = (0.012 * r) as usize;
        for (k, (d, g)) in [(0.0, 0.5), (0.0063, 0.35), (0.0117, 0.3), (0.0171, 0.22), (0.0236, 0.18)].iter().enumerate() {
            let at = pre + ((d + 0.002 * rng.unit()) * r) as usize;
            ir[at] += g * if k % 2 == 0 { 1.0 } else { -1.0 };
        }
        let mut lp = 0.0;
        for (n, v) in ir.iter_mut().enumerate().skip(pre) {
            let t = (n - pre) as f64 / r;
            // Air and walls absorb highs: the low-pass closes as the tail ages.
            let a = (0.9 * (-t / (rt60 * 0.35)).exp()).max(0.08);
            lp += (rng.signed() - lp) * a;
            *v += lp * 0.12 * (-6.91 * t / rt60).exp();
        }
        ir
    };
    let (l, rr) = (Convolver::new(&ir(11)).apply(x), Convolver::new(&ir(23)).apply(x));
    let dry = 1.0 - wet;
    (x.iter().zip(&l).map(|(d, w)| d * dry + w * wet).collect(), x.iter().zip(&rr).map(|(d, w)| d * dry + w * wet).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_parse_and_reject_bad_lines() {
        let ev = parse("# c\n0.0, 1, 3, 0.7, pick, 0.5\n0.25, 5, 0, 0.5, finger, 1.0, 0.14, 0.8\n").unwrap();
        assert_eq!(ev.len(), 2);
        assert_eq!((ev[1].string, ev[1].tool, ev[1].distance), (5, Tool::Finger, Some(0.14)));
        assert!(parse("0, 6, 0, 0.5, pick, 1").unwrap_err().contains("string"));
        assert!(parse("0, 1, 0, 0.5, plectrum, 1").unwrap_err().contains("tool"));
        assert!(parse("0, 1, 0, 0.5, pick").unwrap_err().contains("6 fields"));
    }

    #[test]
    fn a_damped_note_fades_over_seconds_highs_first() {
        let r = 22_050.0;
        let g = super::super::steel_string(22_050);
        let ev = |end: f64| vec![Event { t: 0.0, string: 3, fret: 2, velocity: 0.6, tool: Tool::Pick, end, distance: None, angle: None, moves: Default::default() }];
        let damped = perform(&g, &ev(0.5), 0.0);
        let ringing = perform(&g, &ev(5.0), 0.0);
        let rms = |x: &[f64], a: f64, b: f64| {
            let s = &x[((a * r) as usize).min(x.len())..((b * r) as usize).min(x.len())];
            (s.iter().map(|v| v * v).sum::<f64>() / s.len().max(1) as f64).sqrt().max(1e-12)
        };
        let db = |a: f64, b: f64| 20.0 * (a / b).log10();
        let before = rms(&damped, 0.47, 0.5);
        // A gentle start: 0.3 s after the release it is quieter than the undamped note, but not gone.
        let (d03, r03) = (rms(&damped, 0.78, 0.8), rms(&ringing, 0.78, 0.8));
        assert!(db(d03, r03) < -4.0 && db(d03, before) > -25.0, "0.3 s after: {:.1} dB under the undamped note, {:.1} dB under the release", db(d03, r03), db(d03, before));
        // Gone a few seconds later.
        assert!(db(rms(&damped, 3.4, 3.5), before) < -40.0, "3 s after: {:.1} dB", db(rms(&damped, 3.4, 3.5), before));
        // Highs die first: darker than the undamped note at the same moment (first-difference energy share).
        let bright = |x: &[f64], a: f64, b: f64| {
            let s = &x[(a * r) as usize..(b * r) as usize];
            s.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum::<f64>() / s.iter().map(|v| v * v).sum::<f64>().max(1e-30)
        };
        assert!(bright(&damped, 1.0, 1.1) < bright(&ringing, 1.0, 1.1) * 0.8, "the tail is darker than the undamped note");
    }

    /// Level (dB re peak) of the 50 ms before the sound ends: perform pads 50 ms of silence after
    /// the last sample, and that padding must not be what gets measured.
    fn end_level(x: &[f64], r: f64) -> f64 {
        let peak = x.iter().fold(0.0f64, |m, v| m.max(v.abs()));
        let last = x.iter().rposition(|v| v.abs() > 1e-9).unwrap_or(0);
        let n = ((0.05 * r) as usize).min(last);
        let rms = (x[last - n..last].iter().map(|v| v * v).sum::<f64>() / n.max(1) as f64).sqrt();
        20.0 * (rms / peak).log10()
    }

    #[test]
    fn the_last_note_fades_out_instead_of_stopping() {
        // The mix once ended 1 s after the last release, cutting its tail at -33 dB (heard as
        // abrupt silence, especially through the amp).
        let r = 22_050.0;
        let mut g = super::super::electric::les_paul(22_050, super::super::electric::Selector::Bridge);
        g.max_secs = 12.0;
        let x = perform(&g, &[Event { t: 0.0, string: 0, fret: 5, velocity: 0.6, tool: Tool::Pick, end: 1.0, distance: None, angle: None, moves: Default::default() }], 0.0);
        let db = end_level(&x, r);
        assert!(db < -55.0, "the sound ends {db:.0} dB under the peak, {:.2} s in", x.len() as f64 / r);
    }

    #[test]
    fn every_release_gets_its_tail_even_when_the_string_is_picked_again() {
        let r = 22_050.0;
        let g = super::super::steel_string(22_050);
        let e = |t: f64, string: usize, fret: u32, end: f64| Event { t, string, fret, velocity: 0.6, tool: Tool::Pick, end, distance: None, angle: None, moves: Default::default() };
        // Fret 5 held to 0.5 s, then the same string at fret 7 from 0.6 s: the first note's tail carries on
        // under the second (it was once cut at the re-pick, heard as notes cutting to silence).
        let x = perform(&g, &[e(0.0, 3, 5, 0.5), e(0.6, 3, 7, 3.0)], 0.0);
        let alone = perform(&g, &[e(0.0, 3, 5, 0.5)], 0.0);
        let f1 = super::super::midi_hz(60.0); // G string (55) at fret 5
        let level = |x: &[f64], a: f64, b: f64| {
            let s = &x[(a * r) as usize..(b * r) as usize];
            let w = std::f64::consts::TAU * f1 / r;
            let (c, d) = s.iter().enumerate().fold((0.0, 0.0), |(c, d), (i, v)| (c + v * (w * i as f64).cos(), d + v * (w * i as f64).sin()));
            (c * c + d * d).sqrt() / s.len() as f64
        };
        let (with, tail) = (level(&x, 0.7, 1.0), level(&alone, 0.7, 1.0));
        assert!(with > 0.5 * tail, "first note's pitch after the re-pick {with:.2e}, its tail alone {tail:.2e}");
    }

    #[test]
    fn a_long_held_note_still_tails_out() {
        // Notes once stopped at max_secs without their tail. The electric sustains long enough to show it.
        let r = 22_050.0;
        let g = super::super::electric::les_paul(22_050, super::super::electric::Selector::Bridge);
        let x = perform(&g, &[Event { t: 0.0, string: 0, fret: 0, velocity: 0.6, tool: Tool::Pick, end: g.max_secs + 1.0, distance: None, angle: None, moves: Default::default() }], 0.0);
        let db = end_level(&x, r);
        assert!(db < -55.0, "the sound ends {db:.0} dB under the peak, {:.2} s in", x.len() as f64 / r);
    }
}
