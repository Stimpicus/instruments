//! The guitar as a multisample instrument: every semitone of the neck rendered
//! on the string a player in open position would use, at several dynamics.
//! Rendering is parallel and deterministic; the whole set shares one gain, so
//! the loudness differences between notes and layers stay physical.

use super::{Guitar, Pluck, Tool, pluck_amplitude as amplitude};
use crate::export::Region;

#[derive(Clone, Debug)]
pub struct Layer {
    /// MIDI velocity range this layer answers.
    pub lv: i32,
    pub hv: i32,
    /// Pluck velocity (0..1) rendered for the layer.
    pub velocity: f64,
}

#[derive(Clone, Debug)]
pub struct SetSpec {
    pub lowest: i32,
    pub highest: i32,
    pub layers: Vec<Layer>,
    pub tool: Tool,
    /// Peak of the loudest sample after normalization.
    pub peak: f32,
    /// Release when a note is cut short, seconds.
    pub release: f64,
    /// Longest sample kept, seconds; longer notes fade out over the last 0.4 s.
    /// Memory against ring-out: a game rarely lets a note ring this long.
    pub length: f64,
}

impl SetSpec {
    pub fn standard() -> SetSpec {
        SetSpec {
            lowest: 40,
            highest: 84,
            layers: vec![
                Layer { lv: 1, hv: 48, velocity: 0.25 },
                Layer { lv: 49, hv: 96, velocity: 0.55 },
                Layer { lv: 97, hv: 127, velocity: 0.9 },
            ],
            tool: Tool::Pick,
            peak: 0.9,
            release: 0.12,
            length: 4.0,
        }
    }
}

pub struct Rendered {
    /// (file-friendly name, samples).
    pub samples: Vec<(String, Vec<f32>)>,
    pub regions: Vec<Region>,
}

pub fn render_set(g: &Guitar, spec: &SetSpec) -> Rendered {
    let mut jobs = vec![];
    for key in spec.lowest..=spec.highest {
        let Some((string, fret)) = g.fingering(key) else { continue };
        for (li, layer) in spec.layers.iter().enumerate() {
            jobs.push((key, li, string, fret, layer.velocity));
        }
    }
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(jobs.len().max(1));
    let mut raw: Vec<Vec<f64>> = vec![vec![]; jobs.len()];
    std::thread::scope(|s| {
        let chunks: Vec<_> = raw.chunks_mut(jobs.len().div_ceil(threads)).zip(jobs.chunks(jobs.len().div_ceil(threads))).collect();
        for (outs, js) in chunks {
            s.spawn(move || {
                for (o, &(key, _, string, fret, velocity)) in outs.iter_mut().zip(js) {
                    let mut p = Pluck::new(string, fret, velocity);
                    p.tool = spec.tool;
                    p.seed = key as u64;
                    *o = g.render(&p);
                    cap(o, spec.length, g.rate as f64);
                }
            });
        }
    });
    let peak = raw.iter().flatten().fold(0.0f64, |m, v| m.max(v.abs()));
    let k = spec.peak as f64 / peak.max(1e-30);
    let mut samples = vec![];
    let mut regions = vec![];
    let keys: Vec<i32> = (spec.lowest..=spec.highest).filter(|&m| g.fingering(m).is_some()).collect();
    for (i, (&(key, li, string, fret, _), data)) in jobs.iter().zip(&raw).enumerate() {
        let layer = &spec.layers[li];
        samples.push((format!("k{key:03}_v{:03}_s{}f{:02}.wav", layer.hv, string + 1, fret), data.iter().map(|v| (v * k) as f32).collect()));
        let lo = if key == keys[0] { 0 } else { key };
        let hi = if key == *keys.last().unwrap() { 127 } else { key };
        let centre = amplitude(layer.velocity);
        let rel = |vel: i32| amplitude(vel as f64 / 127.0) / centre;
        regions.push(Region {
            sample: i,
            lo,
            hi,
            root: key,
            lv: layer.lv,
            hv: layer.hv,
            curve: vec![(layer.lv as f64, rel(layer.lv)), (layer.hv as f64, rel(layer.hv))],
            release: spec.release,
            seq: None,
        });
    }
    Rendered { samples, regions }
}

fn cap(x: &mut Vec<f64>, secs: f64, rate: f64) {
    let n = (secs * rate) as usize;
    if x.len() <= n {
        return;
    }
    x.truncate(n);
    let fade = ((0.4 * rate) as usize).min(n);
    for (i, v) in x[n - fade..].iter_mut().enumerate() {
        let k = 1.0 - i as f64 / fade as f64;
        *v *= k * k;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_and_velocity_maps_to_exactly_one_sample() {
        let g = crate::guitar::steel_string(22_050);
        let spec = SetSpec { lowest: 52, highest: 57, ..SetSpec::standard() };
        let r = render_set(&g, &spec);
        assert_eq!(r.samples.len(), 6 * 3);
        for key in 0..=127 {
            for vel in 1..=127 {
                let hits = r.regions.iter().filter(|x| key >= x.lo && key <= x.hi && vel >= x.lv && vel <= x.hv).count();
                assert_eq!(hits, 1, "key {key} vel {vel}");
            }
        }
        let peak = r.samples.iter().flat_map(|(_, d)| d.iter()).fold(0.0f32, |m, v| m.max(v.abs()));
        assert!((peak - spec.peak).abs() < 1e-6, "set normalized to its loudest sample");
        assert!(r.samples.iter().all(|(_, d)| d.len() <= (spec.length * 22_050.0) as usize));
    }
}
