//! A multisample set as files: one WAV per sample plus `<name>.instrument.json`,
//! a region map (key and velocity ranges, root keys, velocity curves) that samplers can load.

use std::fmt::Write as _;
use std::path::Path;

/// One playable region: sample `sample` sounds at `root` and covers keys
/// `lo..=hi` and velocities `lv..=hv`.
#[derive(Clone, Debug)]
pub struct Region {
    pub sample: usize,
    pub lo: i32,
    pub hi: i32,
    pub root: i32,
    pub lv: i32,
    pub hv: i32,
    /// Velocity curve points (velocity, amplitude) for loudness inside the layer.
    pub curve: Vec<(f64, f64)>,
    /// Release time, seconds, when the note is cut.
    pub release: f64,
    /// Round robin (position from 1, length).
    pub seq: Option<(u32, u32)>,
}

/// `samples` are file names relative to the JSON file.
pub fn region_map_json(samples: &[String], regions: &[Region]) -> String {
    let mut s = String::from("{\n  \"samples\": [");
    for (i, name) in samples.iter().enumerate() {
        let _ = write!(s, "{}\n    \"{}\"", if i == 0 { "" } else { "," }, name.replace('\\', "/"));
    }
    s.push_str("\n  ],\n  \"regions\": [");
    for (i, r) in regions.iter().enumerate() {
        let curve = r.curve.iter().map(|(v, a)| format!("[{v}, {a:.5}]")).collect::<Vec<_>>().join(", ");
        let _ = write!(
            s,
            "{}\n    {{\"i\": {}, \"lo\": {}, \"hi\": {}, \"c\": {}, \"lv\": {}, \"hv\": {}, \"vc\": [{}], \"rel\": {}",
            if i == 0 { "" } else { "," },
            r.sample,
            r.lo,
            r.hi,
            r.root,
            r.lv,
            r.hv,
            curve,
            r.release
        );
        if let Some((pos, len)) = r.seq {
            let _ = write!(s, ", \"sq\": [{pos}, {len}]");
        }
        s.push('}');
    }
    s.push_str("\n  ]\n}\n");
    s
}

pub fn write_set(dir: &Path, name: &str, rate: u32, samples: &[(String, Vec<f32>)], regions: &[Region]) -> std::io::Result<()> {
    let sub = dir.join(name);
    std::fs::create_dir_all(&sub)?;
    let mut names = vec![];
    for (file, data) in samples {
        crate::wav::write_mono_f32(&sub.join(file), rate, data)?;
        names.push(format!("{name}/{file}"));
    }
    std::fs::write(dir.join(format!("{name}.instrument.json")), region_map_json(&names, regions))
}
