//! Diagnostic: every note's level on the electric, at the pickup and after the amp, to find outliers.
//!   cargo run --release --example note_levels [eb] [acoustic]
use instruments::guitar::{Pluck, amp, electric};
fn main() {
    let rate = 44_100u32;
    let args: Vec<String> = std::env::args().collect();
    let acoustic = args.iter().any(|a| a == "acoustic");
    let mut g = if acoustic { instruments::guitar::steel_string(rate) } else { electric::les_paul(rate, electric::Selector::Bridge) };
    if args.iter().any(|a| a == "eb") {
        g.retune([39.0, 44.0, 49.0, 54.0, 58.0, 63.0]);
    }
    let rms = |x: &[f64], a: f64, b: f64| {
        let s = &x[(a * rate as f64) as usize..((b * rate as f64) as usize).min(x.len())];
        20.0 * ((s.iter().map(|v| v * v).sum::<f64>() / s.len() as f64).sqrt()).log10()
    };
    println!("midi string fret  pickup dB(0.05-0.5s)  amp dB(0.05-0.5s)  amp dB(1-2s)");
    for s in 0..6 {
        for fret in 0..=17u32 {
            let p = Pluck::new(s, fret, 0.6);
            let di = g.render_for(&p, 2.0);
            let y = if acoustic { di.clone() } else { amp::marshall(&di, rate as f64, &amp::Settings::default()) };
            println!("{:4.0} {s} {fret:3}  {:7.1}  {:7.1}  {:7.1}", g.strings[s].open + fret as f64, rms(&di, 0.05, 0.5), rms(&y, 0.05, 0.5), rms(&y, 1.0, 2.0));
        }
    }
}
