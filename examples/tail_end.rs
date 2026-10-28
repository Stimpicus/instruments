//! Diagnostic: a held note released at 1 s, on the electric, through perform and the amp:
//! level (dB re the note's peak) every 0.25 s, to see whether the tail fades or stops.
use instruments::guitar::{amp, electric, perform::{Event, perform}, Tool};
fn main() {
    let rate = 44_100u32;
    let r = rate as f64;
    let g = electric::les_paul(rate, electric::Selector::Bridge);
    for (s, fret) in [(0usize, 5u32), (3, 7), (5, 12)] {
        let ev = vec![Event { t: 0.0, string: s, fret, velocity: 0.6, tool: Tool::Pick, end: 1.0, distance: None, angle: None, moves: Default::default() }];
        let di = perform(&g, &ev, 0.0);
        let y = amp::marshall(&di, r, &amp::Settings::default());
        for (name, x) in [("pickup", &di), ("amp", &y)] {
            let peak = x.iter().fold(0.0f64, |m, v| m.max(v.abs()));
            print!("s{s} f{fret} {name:6} len {:.2}s:", x.len() as f64 / r);
            let mut t = 0.0;
            while t < x.len() as f64 / r {
                let a = (t * r) as usize;
                let b = ((t + 0.05) * r) as usize;
                let seg = &x[a..b.min(x.len())];
                let rms = (seg.iter().map(|v| v * v).sum::<f64>() / seg.len().max(1) as f64).sqrt();
                print!(" {t:.2}:{:.0}", 20.0 * (rms / peak).max(1e-9).log10());
                t += 0.25;
            }
            println!();
        }
    }
}
