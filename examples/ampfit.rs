//! Calibration of the amp model against recordings of a real amplifier (the Marshall JVM410H
//! dataset, Miklánek et al. 2023, CC BY 4.0: guitar input, preamp-with-tone-stack output and
//! speaker output, recorded at many knob settings).
//!
//!   cargo run --release --example ampfit -- <dir>
//!
//! `<dir>` holds `settings.txt` (lines: name gain bass mid treble train|hold) and, per setting,
//! `<name>.input.f32`, `<name>.preamp.f32`, `<name>.speakerout.f32` (mono f32 at 44.1 kHz, made of
//! 2.5 s segments; the first 0.5 s of each only warms the model up).
//!
//! It reads commands on stdin, one per line, and answers each with one line:
//!   pre  key=value ...   the preamp + tone stack (input × in_scale) against the preamp recording
//!   pow  key=value ...   the power stage (preamp recording, as recorded) against the speaker output
//! keys are `amp::Design` fields plus `in_scale` for `pre`. The answer is
//!   <train dB> <held-out dB> <offset dB> <per-setting dB ...>
//! where dB is the mean absolute difference of log spectra (three STFT sizes, 30 Hz-16 kHz), after
//! one level offset shared by every setting (fitted on the training settings).

use std::f64::consts::PI;
use std::io::BufRead;

use instruments::fft::{C, fft};
use instruments::guitar::amp::{self, Design, Settings};

const RATE: f64 = 44_100.0;
const SEG: usize = 110_250; // 2.5 s
const WARM: usize = 22_050; // 0.5 s

struct Setting {
    name: String,
    knobs: Settings,
    train: bool,
    input: Vec<f64>,
    pre: Vec<f64>,
    spk: Vec<f64>,
}

fn read_f32(path: &str) -> Vec<f64> {
    let b = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64).collect()
}

/// Log-magnitude spectra (dB) of x past the warm-up, for one STFT size: frames × bins (30 Hz-16 kHz).
fn log_spec(x: &[f64], n: usize) -> Vec<f64> {
    let hop = n / 4;
    let (lo, hi) = ((30.0 * n as f64 / RATE).ceil() as usize, (16_000.0 * n as f64 / RATE) as usize);
    let mut out = vec![];
    let mut buf = vec![C::default(); n];
    for seg in x.chunks(SEG) {
        let mut t = WARM;
        while t + n <= seg.len() {
            for i in 0..n {
                let w = (PI * i as f64 / n as f64).sin().powi(2);
                buf[i] = C::new(seg[t + i] * w, 0.0);
            }
            fft(&mut buf, false);
            out.extend(buf[lo..hi].iter().map(|c| 20.0 * (c.abs() + 1e-12).log10()));
            t += hop;
        }
    }
    out
}

const SIZES: [usize; 3] = [256, 1024, 4096];

fn main() {
    let dir = std::env::args().nth(1).expect("usage: ampfit <dir>");
    let sets: Vec<Setting> = std::fs::read_to_string(format!("{dir}/settings.txt"))
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let v = |i: usize| f[i].parse::<f64>().unwrap();
            Setting {
                name: f[0].to_string(),
                knobs: Settings { gain: v(1), bass: v(2), mid: v(3), treble: v(4), presence: 0.5, master: 0.5 },
                train: f[5] == "train",
                input: read_f32(&format!("{dir}/{}.input.f32", f[0])),
                pre: read_f32(&format!("{dir}/{}.preamp.f32", f[0])),
                spk: read_f32(&format!("{dir}/{}.speakerout.f32", f[0])),
            }
        })
        .collect();
    // Targets' spectra, once.
    let targets: Vec<[Vec<f64>; 2]> = sets.iter().map(|s| [SIZES.iter().flat_map(|&n| log_spec(&s.pre, n)).collect(), SIZES.iter().flat_map(|&n| log_spec(&s.spk, n)).collect()]).collect();
    eprintln!("ampfit: {} settings loaded", sets.len());

    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut words = line.split_whitespace();
        let Some(stage) = words.next() else { continue };
        let mut d = Design::default();
        let mut in_scale = 1.0;
        let mut bad = None;
        for kv in words {
            let Some((k, v)) = kv.split_once('=') else { bad = Some(kv.to_string()); break };
            let Ok(v) = v.parse::<f64>() else { bad = Some(kv.to_string()); break };
            if k == "in_scale" {
                in_scale = v;
            } else if !d.set(k, v) {
                bad = Some(k.to_string());
                break;
            }
        }
        if let Some(b) = bad {
            println!("error unknown or malformed parameter {b}");
            continue;
        }
        let pre_stage = match stage {
            "pre" => true,
            "pow" => false,
            _ => {
                println!("error unknown stage {stage}");
                continue;
            }
        };
        // Model output per setting (segments run separately, each from rest), in parallel.
        let models: Vec<Vec<f64>> = std::thread::scope(|sc| {
            let jobs: Vec<_> = sets
                .iter()
                .map(|s| {
                    let d = &d;
                    sc.spawn(move || {
                        let src = if pre_stage { &s.input } else { &s.pre };
                        let mut y = Vec::with_capacity(src.len());
                        for seg in src.chunks(SEG) {
                            let x: Vec<f64> = seg.iter().map(|v| v * if pre_stage { in_scale } else { 1.0 }).collect();
                            let out = if pre_stage { amp::preamp(&x, RATE, &s.knobs, d, amp::OVERSAMPLE) } else { amp::power(&x, RATE, &s.knobs, d, amp::OVERSAMPLE) };
                            y.extend(out);
                        }
                        SIZES.iter().flat_map(|&n| log_spec(&y, n)).collect::<Vec<f64>>()
                    })
                })
                .collect();
            jobs.into_iter().map(|j| j.join().unwrap()).collect()
        });
        let k = if pre_stage { 0 } else { 1 };
        // One level offset for all settings: the median difference over the training settings.
        let mut diffs: Vec<f64> = sets.iter().zip(&models).zip(&targets).filter(|((s, _), _)| s.train).flat_map(|((_, m), t)| m.iter().zip(&t[k]).map(|(a, b)| b - a).collect::<Vec<_>>()).collect();
        if diffs.iter().any(|v| !v.is_finite()) {
            println!("1e9 1e9 0");
            continue;
        }
        diffs.sort_by(|a, b| a.total_cmp(b));
        let offset = diffs[diffs.len() / 2];
        // Bins far below the target's loudest are noise in the recording: floor both there.
        let per: Vec<f64> = models
            .iter()
            .zip(&targets)
            .map(|(m, t)| {
                let t = &t[k];
                let floor = t.iter().cloned().fold(f64::MIN, f64::max) - 80.0;
                m.iter().zip(t).map(|(a, b)| ((a + offset).max(floor) - b.max(floor)).abs()).sum::<f64>() / t.len() as f64
            })
            .collect();
        let mean = |train: bool| {
            let v: Vec<f64> = sets.iter().zip(&per).filter(|(s, _)| s.train == train).map(|(_, p)| *p).collect();
            v.iter().sum::<f64>() / v.len().max(1) as f64
        };
        println!("{:.4} {:.4} {:.3} {}", mean(true), mean(false), offset, sets.iter().zip(&per).map(|(s, p)| format!("{}={p:.2}", s.name)).collect::<Vec<_>>().join(" "));
    }
}
