//! guitar note <string 0-5> <fret> <velocity 0-1> <out.wav> [pick|finger] [seed]
//!   one note, peak-normalized to -6 dBFS (prints the raw peak and timing);
//!   set GUITAR_GAIN=<k> for a fixed gain instead, so dynamics can be compared
//! guitar set <dir> <name>
//!   the standard multisample set: <dir>/<name>.instrument.json + <dir>/<name>/*.wav
//! guitar perform <events.csv> <out prefix> [sympathetic 0.5] [electric[:neck|bridge|both]]
//!   a performance (see guitar::perform): writes "<prefix> (dry).wav" and "<prefix> (room).wav";
//!   electric: a Les Paul into the Marshall (guitar::amp) at its default settings
//! guitar losses <string> <fret>
//!   per-partial loss breakdown

use std::path::PathBuf;
use std::time::Instant;

use instruments::guitar::{self, Pluck, Tool};

const RATE: u32 = 44_100;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("note") if args.len() >= 5 => {
            let t0 = Instant::now();
            let g = guitar::steel_string(RATE);
            let built = t0.elapsed();
            let mut p = Pluck::new(args[1].parse().unwrap(), args[2].parse().unwrap(), args[3].parse().unwrap());
            if args.get(5).map(String::as_str) == Some("finger") {
                p.tool = Tool::Finger;
            }
            if let Some(seed) = args.get(6) {
                p.seed = seed.parse().unwrap();
            }
            let t1 = Instant::now();
            let x = g.render(&p);
            let took = t1.elapsed();
            let peak = x.iter().fold(0.0f64, |m, v| m.max(v.abs()));
            // GUITAR_GAIN fixes the gain (to hear dynamics); otherwise peak-normalize to -6 dBFS.
            let k = std::env::var("GUITAR_GAIN").ok().and_then(|g| g.parse().ok()).unwrap_or(0.5 / peak.max(1e-30));
            let data: Vec<f32> = x.iter().map(|v| (v * k) as f32).collect();
            instruments::wav::write_mono_f32(&PathBuf::from(&args[4]), RATE, &data).unwrap();
            println!("raw peak {peak:.4e}, {:.2} s, guitar built in {built:?}, note rendered in {took:?}", x.len() as f64 / RATE as f64);
        }
        Some("set") if args.len() >= 3 => {
            let t0 = Instant::now();
            let g = guitar::steel_string(RATE);
            let r = guitar::set::render_set(&g, &guitar::set::SetSpec::standard());
            println!("render_set took {:?}", t0.elapsed());
            let secs: f64 = r.samples.iter().map(|(_, d)| d.len() as f64).sum::<f64>() / RATE as f64;
            instruments::export::write_set(&PathBuf::from(&args[1]), &args[2], RATE, &r.samples, &r.regions).unwrap();
            println!("{} samples, {secs:.0} s of audio ({:.1} MB as f32), rendered in {:?}", r.samples.len(), secs * RATE as f64 * 4.0 / 1e6, t0.elapsed());
        }
        Some("perform") if args.len() >= 3 => {
            // guitar perform <events.csv> <out prefix> [sympathetic]
            let text = std::fs::read_to_string(&args[1]).unwrap_or_else(|e| panic!("{}: {e}", args[1]));
            let events = guitar::perform::parse(&text).unwrap_or_else(|e| panic!("{e}"));
            let sym: f64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.5);
            let t0 = Instant::now();
            let electric = args.get(4).filter(|a| a.starts_with("electric"));
            let mut g = match electric {
                Some(a) => {
                    let sel = a.split(':').nth(1).map(|s| guitar::electric::Selector::parse(s).expect("pickup: neck, bridge or both"));
                    guitar::electric::les_paul(RATE, sel.unwrap_or(guitar::electric::Selector::Bridge))
                }
                None => guitar::steel_string(RATE),
            };
            if let Some(open) = guitar::perform::tuning(&text).unwrap_or_else(|e| panic!("{e}")) {
                g.retune(open);
            }
            if text.lines().any(|l| l.trim() == "#tunings-differ") {
                eprintln!("warning: the parts have different tunings; this renders all of them in the first part's. Use arrange.py --part for each.");
            }
            let mut x = guitar::perform::perform(&g, &events, sym);
            if electric.is_some() {
                let peak = x.iter().fold(0.0f64, |m, v| m.max(v.abs()));
                let rms = (x.iter().map(|v| v * v).sum::<f64>() / x.len() as f64).sqrt();
                println!("pickup signal: peak {peak:.3} V, rms {rms:.3} V");
                x = guitar::amp::marshall(&x, RATE as f64, &guitar::amp::Settings::default());
            }
            let took = t0.elapsed();
            let (l, r) = guitar::perform::room(&x, RATE, 1.3, 0.22);
            // One gain per file: the louder of the two versions peaks at -1 dBFS.
            let peak = x.iter().chain(&l).chain(&r).fold(0.0f64, |m, v| m.max(v.abs()));
            let k = 0.891 / peak.max(1e-30);
            let dry: Vec<f32> = x.iter().map(|v| (v * k) as f32).collect();
            let wet: Vec<f32> = l.iter().zip(&r).flat_map(|(a, b)| [(a * k) as f32, (b * k) as f32]).collect();
            instruments::wav::write_mono_f32(&PathBuf::from(format!("{} (dry).wav", args[2])), RATE, &dry).unwrap();
            instruments::wav::write_f32(&PathBuf::from(format!("{} (room).wav", args[2])), RATE, 2, &wet).unwrap();
            println!("{} notes, {:.1} s of music, rendered in {took:?}", events.len(), x.len() as f64 / RATE as f64);
        }
        Some("losses") if args.len() >= 3 => {
            let g = guitar::steel_string(RATE);
            println!("   Hz     sigma_int  sigma_bridge   ReY       ImY      T60 s");
            for (f, si, sb, re, im) in g.loss_table(args[1].parse().unwrap(), args[2].parse().unwrap(), 12) {
                println!("{f:8.1} {si:9.3} {sb:9.3}  {re:9.5} {im:9.5} {:7.2}", 6.91 / (si + sb));
            }
        }
        _ => {
            eprintln!("usage: guitar note <string 0-5> <fret> <velocity 0-1> <out.wav> [pick|finger] [seed]");
            std::process::exit(2);
        }
    }
}
