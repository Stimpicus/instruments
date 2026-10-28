//! Diagnostic: a sustained note's pitch over time (cents from its target), electric vs acoustic.
use instruments::guitar::{self, Pluck, electric, midi_hz};
fn main() {
    let rate = 44_100u32;
    let r = rate as f64;
    // guitar-free args: string fret velocity [standard]
    let a: Vec<String> = std::env::args().skip(1).collect();
    let (s, fret, vel) = if a.len() >= 3 { (a[0].parse().unwrap(), a[1].parse().unwrap(), a[2].parse().unwrap()) } else { (4usize, 17u32, 0.79) };
    let eb = if a.get(3).map(String::as_str) == Some("standard") { [40.0, 45.0, 50.0, 55.0, 59.0, 64.0] } else { [39.0, 44.0, 49.0, 54.0, 58.0, 63.0] };
    let mut e = electric::les_paul(rate, electric::Selector::Bridge);
    e.retune(eb);
    let mut a = guitar::steel_string(rate);
    a.retune(eb);
    let target = midi_hz(eb[s] + fret as f64);
    for (name, g) in [("electric", &e), ("acoustic", &a)] {
        let x = g.render_for(&Pluck::new(s, fret, vel), 4.0);
        print!("{name:9}");
        for t in [0.05, 0.15, 0.3, 0.5, 0.8, 1.0, 1.5, 2.0, 3.0] {
            // fundamental by a fine DFT scan around the target over an 80 ms window
            let (a0, n) = ((t * r) as usize, (0.08 * r) as usize);
            if a0 + n > x.len() { break; }
            let mut best = (0.0, 0.0);
            let mut c = -80.0;
            while c <= 80.0 {
                let f = target * 2f64.powf(c / 1200.0);
                let (mut re, mut im) = (0.0, 0.0);
                for i in 0..n { let w = (std::f64::consts::PI * i as f64 / n as f64).sin().powi(2); let ph = std::f64::consts::TAU * f * i as f64 / r; re += w * x[a0 + i] * ph.cos(); im += w * x[a0 + i] * ph.sin(); }
                let m = re * re + im * im;
                if m > best.0 { best = (m, c); }
                c += 0.5;
            }
            print!("  {t}s {:+.1}c", best.1);
        }
        println!();
    }
}
