//! Diagnostic: how much a held chord's tone moves over time (a "wah"/phaser swirl), on the electric.
//! Prints the modulation of band levels (std of dB over 40 ms frames, 0.3-1.5 s) for variants.
use instruments::guitar::{Pluck, amp, electric};
fn main() {
    let rate = 44_100u32;
    let r = rate as f64;
    // The chord at 126.7 s in Sweet Child (E-flat tuning): A3, D2, G0, B1, e0.
    let chord = [(1usize, 3u32), (2, 2), (3, 0), (4, 1), (5, 0)];
    let bands = [(200.0, 500.0), (500.0, 1000.0), (1000.0, 2000.0), (2000.0, 4000.0)];
    for (label, split, horiz) in [("as built", 5e-4, 0.15), ("no polarization split", 0.0, 0.15), ("no horizontal sensing", 5e-4, 0.0), ("neither", 0.0, 0.0)] {
        let mut g = electric::les_paul(rate, electric::Selector::Bridge);
        g.retune([39.0, 44.0, 49.0, 54.0, 58.0, 63.0]);
        g.split = split;
        g.pickup.as_mut().unwrap().horizontal = horiz;
        let mut di = vec![0.0; (2.0 * r) as usize];
        for (k, &(s, f)) in chord.iter().enumerate() {
            let mut p = Pluck::new(s, f, 0.65);
            p.seed = k as u64 + 1;
            for (o, v) in di.iter_mut().zip(g.render_for(&p, 2.0)) { *o += v; }
        }
        for (name, x) in [("pickup", di.clone()), ("amp", amp::marshall(&di, r, &amp::Settings::default()))] {
            print!("{label:24} {name:6}");
            for &(lo, hi) in &bands {
                let frame = (0.04 * r) as usize;
                let mut lv = vec![];
                let mut t = (0.3 * r) as usize;
                while t + frame < (1.5 * r) as usize {
                    // band energy by a crude DFT over the frame on a 25 Hz grid
                    let mut e = 0.0;
                    let mut f = lo;
                    while f < hi {
                        let (mut c, mut d) = (0.0, 0.0);
                        for i in 0..frame { let ph = std::f64::consts::TAU * f * (t + i) as f64 / r; c += x[t + i] * ph.cos(); d += x[t + i] * ph.sin(); }
                        e += c * c + d * d;
                        f += 25.0;
                    }
                    lv.push(10.0 * e.max(1e-30).log10());
                    t += frame;
                }
                // remove the steady decay (linear fit) and report what moves around it
                let n = lv.len() as f64;
                let mx = (n - 1.0) / 2.0;
                let my = lv.iter().sum::<f64>() / n;
                let b = lv.iter().enumerate().map(|(i, y)| (i as f64 - mx) * (y - my)).sum::<f64>() / lv.iter().enumerate().map(|(i, _)| (i as f64 - mx).powi(2)).sum::<f64>();
                let sd = (lv.iter().enumerate().map(|(i, y)| (y - my - b * (i as f64 - mx)).powi(2)).sum::<f64>() / n).sqrt();
                print!("  {lo:.0}-{hi:.0} Hz ±{sd:.1} dB");
            }
            println!();
        }
    }
}
