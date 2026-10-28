//! Diagnostic: the amp's output level and distortion against input level (330 Hz sine).
use instruments::guitar::amp;
use std::f64::consts::TAU;
fn main() {
    let rate = 44_100.0;
    for gain in [0.3, 0.6, 0.9] {
        let s = amp::Settings { gain, ..amp::Settings::default() };
        print!("gain {gain}: ");
        for v in [0.01, 0.03, 0.1, 0.3, 1.0] {
            let x: Vec<f64> = (0..44_100).map(|i| v * (TAU * 330.0 * i as f64 / rate).sin()).collect();
            let y = amp::marshall(&x, rate, &s);
            let seg = &y[22_050..];
            let rms = (seg.iter().map(|a| a * a).sum::<f64>() / seg.len() as f64).sqrt();
            // fundamental by correlation
            let (mut c, mut d) = (0.0, 0.0);
            for (i, a) in seg.iter().enumerate() { let ph = TAU * 330.0 * (i + 22_050) as f64 / rate; c += a * ph.sin(); d += a * ph.cos(); }
            let fund = (2.0 * (c * c + d * d).sqrt() / seg.len() as f64) / 2f64.sqrt();
            let thd = ((rms * rms - fund * fund).max(0.0)).sqrt() / fund;
            print!("{v} V -> rms {rms:.3}, THD {:.0}% | ", thd * 100.0);
        }
        println!();
    }
}
