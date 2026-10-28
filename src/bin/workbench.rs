//! Listening workbench: a local web page for playing and tuning the instruments.
//!
//!   cargo run --release --bin workbench -- [--port 8765] [--refs <dir>] [--log <file>] [--gp-dir <folder of .gp files>]
//!
//! References default to `reference/guitarset-notes` in this repo when present.
//!
//! Then open http://127.0.0.1:8765. It binds to 127.0.0.1 only. Endpoints:
//! - `GET /api/render?string=&fret=&velocity=&...` renders one note (32-bit float WAV, fixed gain);
//!   every model knob can be overridden in the query (see `Knobs`). `instrument=electric` plays the
//!   Les Paul (`pickup=neck|bridge|both`) through the Marshall (`amp_gain`, `amp_bass`, `amp_mid`,
//!   `amp_treble`, `amp_presence`, `amp_master`, each 0..1); songs take the same parameters.
//! - `GET /api/defaults` the knobs' default values.
//! - `GET /api/refs`, `GET /api/ref?file=` reference recordings, if `--refs` was given.
//! - `GET /api/songs`, `GET /api/song?name=&room=0|1&<knobs>` song descriptions in the `covers` folder
//!   next to the log, arranged (tools/arrange.py) and performed in memory with the given knobs;
//!   `GET /api/song-events?name=` the same arrangement's plucks and chord spans, for the display.
//! - `POST /api/log` appends one JSON line (a blind-trial answer or a note) to the log
//!   (default `listening/log.jsonl` in this repo).

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use instruments::guitar::{self, BodyDesign, Pluck, Tool, amp, electric};

const RATE: u32 = 44_100;
/// Model units to playback level: a hard open low E peaks around 0.8.
const GAIN: f64 = 160.0;
/// The amp's output to playback level (a hard chord at the default settings peaks around 0.8).
const AMP_GAIN: f64 = 1.5;

/// `instrument=electric`: the pickup selector and the amp's front panel from the query.
fn electric_from(q: &HashMap<String, String>) -> Option<(electric::Selector, amp::Settings)> {
    if q.get("instrument").map(String::as_str) != Some("electric") {
        return None;
    }
    let sel = q.get("pickup").and_then(|p| electric::Selector::parse(p)).unwrap_or(electric::Selector::Bridge);
    let mut s = amp::Settings::default();
    for (k, v) in [("amp_gain", &mut s.gain), ("amp_bass", &mut s.bass), ("amp_mid", &mut s.mid), ("amp_treble", &mut s.treble), ("amp_presence", &mut s.presence), ("amp_master", &mut s.master)] {
        if let Some(x) = q.get(k).and_then(|x| x.parse::<f64>().ok()).filter(|x| x.is_finite()) {
            *v = x.clamp(0.0, 1.0);
        }
    }
    Some((sel, s))
}
const PAGE: &str = include_str!("../../workbench/index.html");

/// Every adjustable value, with its default. Multipliers are 1.0 at the model's own values.
const KNOBS: &[(&str, f64)] = &[
    ("velocity", 0.55),
    ("distance", 0.13),
    ("angle", 0.6),
    ("sympathetic", 1.0),
    ("seed", 1.0),
    ("h_mobility", 0.1),
    ("split", 5e-4),
    ("radiation_hz", 250.0),
    ("pick_noise", 0.02),
    ("eta_b", 1.0),
    ("body_freq", 1.0),
    ("body_q", 1.0),
    ("body_peak", 1.0),
    ("bending_rad", 0.3),
    ("dense_mobility", 0.0015),
    ("mass_corner", 8000.0),
    ("max_secs", 4.0),
];

struct Knobs(HashMap<String, f64>);

impl Knobs {
    fn from_query(q: &HashMap<String, String>) -> Knobs {
        Knobs(KNOBS.iter().map(|&(k, d)| (k.to_string(), q.get(k).and_then(|v| v.parse::<f64>().ok()).filter(|v| v.is_finite()).unwrap_or(d))).collect())
    }
    fn get(&self, k: &str) -> f64 {
        self.0[k]
    }
}

/// The model with every knob applied.
fn guitar_from(k: &Knobs) -> guitar::Guitar {
    let mut body = BodyDesign::steel_string();
    for m in body.triad.iter_mut() {
        m.0 *= k.get("body_freq").clamp(0.5, 2.0);
        m.1 *= k.get("body_q").clamp(0.1, 10.0);
        m.2 *= k.get("body_peak").clamp(0.05, 20.0);
    }
    body.triad[2].3 = k.get("bending_rad").clamp(0.0, 2.0);
    body.dense.mobility = k.get("dense_mobility").clamp(1e-5, 0.1);
    body.dense.mass_corner = k.get("mass_corner").clamp(500.0, 40_000.0);
    let mut g = guitar::steel_string_with(RATE, &body);
    g.h_mobility = k.get("h_mobility").clamp(0.0, 1.0);
    g.split = k.get("split").clamp(-0.05, 0.05);
    g.radiation_hz = k.get("radiation_hz").clamp(10.0, 5000.0);
    g.pick_noise = k.get("pick_noise").clamp(0.0, 1.0);
    g.max_secs = k.get("max_secs").clamp(0.5, 12.0);
    for s in g.strings.iter_mut() {
        s.eta_b *= k.get("eta_b").clamp(0.01, 100.0);
    }
    g
}

fn render(q: &HashMap<String, String>) -> Result<(Vec<u8>, String), String> {
    let k = Knobs::from_query(q);
    let string: usize = q.get("string").and_then(|v| v.parse().ok()).filter(|&s| s < 6).ok_or("string must be 0-5")?;
    let fret: u32 = q.get("fret").and_then(|v| v.parse().ok()).filter(|&f| f <= 24).ok_or("fret must be 0-24")?;
    let elec = electric_from(q);
    let g = match &elec {
        Some((sel, _)) => electric::les_paul(RATE, *sel),
        None => guitar_from(&k),
    };
    let mut p = Pluck::new(string, fret, k.get("velocity").clamp(0.0, 1.0));
    p.tool = q.get("tool").and_then(|t| Tool::parse(t)).unwrap_or(Tool::Pick);
    p.distance = k.get("distance").clamp(0.01, 0.5);
    p.angle = k.get("angle").clamp(0.0, std::f64::consts::FRAC_PI_2);
    p.sympathetic = k.get("sympathetic").clamp(0.0, 4.0);
    p.seed = k.get("seed").max(0.0) as u64;
    let t = Instant::now();
    let x = match &elec {
        // A single note: 4 s is plenty to hear it through the amp.
        Some((_, s)) => amp::marshall(&g.render_for(&p, 4.0), RATE as f64, s).iter().map(|v| v * AMP_GAIN).collect(),
        None => g.render(&p).iter().map(|v| v * GAIN).collect::<Vec<f64>>(),
    };
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let data: Vec<f32> = x.iter().map(|&v| v as f32).collect();
    let peak = data.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    let info = format!(
        "{{\"ms\": {ms:.1}, \"peak\": {peak:.4}, \"seconds\": {:.3}, \"hz\": {:.3}, \"b\": {:.3e}}}",
        data.len() as f64 / RATE as f64,
        guitar::midi_hz(g.strings[string].open + fret as f64),
        g.inharmonicity(string, fret)
    );
    Ok((wav_bytes(&data), info))
}

/// Song descriptions (`*.json` for tools/arrange.py) in the `covers` folder next to the log.
fn songs_dir(log: &Mutex<PathBuf>) -> PathBuf {
    log.lock().unwrap().parent().map(|p| p.join("covers")).unwrap_or_else(|| PathBuf::from("covers"))
}

/// Guitar Pro files are read from here (--gp-dir; default reference/tracks in this repo).
static GP_DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

fn gp_dir() -> &'static PathBuf {
    GP_DIR.get_or_init(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/reference/tracks")))
}

fn files_with(dir: &Path, ext: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|d| d.flatten().filter(|e| e.path().is_file()).map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| n.to_ascii_lowercase().ends_with(ext)).collect())
        .unwrap_or_default();
    names.sort();
    names
}

/// Guitar Pro file types tools/gpimport.py reads.
const GP_EXTS: [&str; 5] = [".gp", ".gpx", ".gp3", ".gp4", ".gp5"];

/// Everything playable: arrangements (covers/*.json) and Guitar Pro files (GP_EXTS).
fn songs(log: &Mutex<PathBuf>) -> Vec<String> {
    let mut names = files_with(&songs_dir(log), ".json");
    for ext in GP_EXTS {
        names.extend(files_with(gp_dir(), ext));
    }
    names
}

/// The song's plucks and chord spans as text, from tools/arrange.py (run with -I). Deterministic,
/// so the event list the page draws is the one the audio was rendered from.
fn arrange(name: &str, part: Option<&str>, log: &Mutex<PathBuf>) -> Result<String, String> {
    if !songs(log).iter().any(|n| n == name) {
        return Err(format!("no such song: {name}"));
    }
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tools/arrange.py");
    let part_args = part.filter(|p| !p.is_empty() && *p != "all").map(|p| vec!["--part", p]).unwrap_or_default();
    let lower = name.to_ascii_lowercase();
    let out = if GP_EXTS.iter().any(|e| lower.ends_with(e)) {
        // A Guitar Pro file: import it in memory (tools/gpimport.py) and pipe the song into the arranger.
        let import = concat!(env!("CARGO_MANIFEST_DIR"), "/tools/gpimport.py");
        // UTF-8 mode: on Windows, piped output is otherwise the ANSI code page, and part names like
        // "Guitare électrique" arrive mangled and match no part.
        let song = std::process::Command::new("python")
            .args(["-X", "utf8", "-I", import])
            .arg(gp_dir().join(name))
            .arg("-")
            .output()
            .map_err(|e| format!("could not run python: {e}"))?;
        if !song.status.success() {
            return Err(format!("Guitar Pro import failed: {}", String::from_utf8_lossy(&song.stderr)));
        }
        let mut child = std::process::Command::new("python")
            .args(["-X", "utf8", "-I", script, "events", "-", "-"])
            .args(&part_args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("could not run python: {e}"))?;
        let mut stdin = child.stdin.take().ok_or("no stdin")?;
        let feed = std::thread::spawn(move || stdin.write_all(&song.stdout));
        let out = child.wait_with_output().map_err(|e| e.to_string())?;
        let _ = feed.join();
        out
    } else {
        std::process::Command::new("python")
            .args(["-X", "utf8", "-I", script, "events"])
            .arg(songs_dir(log).join(name))
            .arg("-")
            .args(&part_args)
            .output()
            .map_err(|e| format!("could not run python: {e}"))?
    };
    if !out.status.success() {
        return Err(format!("arrange failed: {}", String::from_utf8_lossy(&out.stderr)));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Arrange and perform a song in memory with the current knobs.
fn render_song(q: &HashMap<String, String>, log: &Mutex<PathBuf>) -> Result<(Vec<u8>, String), String> {
    let name = decode(q.get("name").map(String::as_str).unwrap_or(""));
    let k = Knobs::from_query(q);
    let t = Instant::now();
    let part = q.get("part").map(|p| decode(p));
    let text = arrange(&name, part.as_deref(), log)?;
    let events = guitar::perform::parse(&text)?;
    let tuning = guitar::perform::tuning(&text)?;
    let arranged = t.elapsed().as_secs_f64();
    let elec = electric_from(q);
    // The instrument tuned for one part's events (or the whole song's).
    let tuned = |tuning: Option<[f64; 6]>| {
        let mut g = match &elec {
            Some((sel, _)) => electric::les_paul(RATE, *sel),
            None => guitar_from(&k),
        };
        if let Some(open) = tuning {
            g.retune(open);
        }
        // Final chords ring longer than single test notes.
        g.max_secs = g.max_secs.max(6.0);
        g
    };
    let sym = k.get("sympathetic").clamp(0.0, 4.0) * 0.5;
    // One part on one guitar: through the amp if electric.
    let play = |g: &guitar::Guitar, ev: &[guitar::perform::Event]| {
        let x = guitar::perform::perform(g, ev, sym);
        match &elec {
            Some((_, s)) => amp::marshall(&x, RATE as f64, s),
            None => x,
        }
    };
    let parts: Vec<String> = text.lines().find_map(|l| l.strip_prefix("#parts,")).map(|l| l.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()).unwrap_or_default();
    let whole = part.as_deref().is_none_or(|p| p.is_empty() || p == "all");
    // Render part by part when each guitarist has their own amp (distortion mixes notes within a
    // part, not across parts) or their own tuning; then mix.
    let x = if whole && parts.len() > 1 && (elec.is_some() || text.lines().any(|l| l.trim() == "#tunings-differ")) {
        let each = std::thread::scope(|sc| {
            let jobs: Vec<_> = parts
                .iter()
                .map(|name_p| {
                    let (name, tuned, play) = (&name, &tuned, &play);
                    sc.spawn(move || -> Result<Vec<f64>, String> {
                        let text_p = arrange(name, Some(name_p), log)?;
                        let g = tuned(guitar::perform::tuning(&text_p)?);
                        Ok(play(&g, &guitar::perform::parse(&text_p)?))
                    })
                })
                .collect();
            jobs.into_iter().map(|j| j.join().map_err(|_| "a part's render panicked".to_string())?).collect::<Result<Vec<_>, String>>()
        })?;
        let mut mix = vec![0.0; each.iter().map(Vec::len).max().unwrap_or(0)];
        for y in &each {
            mix.iter_mut().zip(y).for_each(|(m, v)| *m += v);
        }
        mix
    } else {
        play(&tuned(tuning), &events)
    };
    let room = q.get("room").map(String::as_str) != Some("0");
    let (l, r) = if room { guitar::perform::room(&x, RATE, 1.3, 0.22) } else { (x.clone(), x.clone()) };
    let peak = l.iter().chain(&r).fold(0.0f64, |m, v| m.max(v.abs()));
    let gain = 0.891 / peak.max(1e-30);
    let data: Vec<f32> = l.iter().zip(&r).flat_map(|(a, b)| [(a * gain) as f32, (b * gain) as f32]).collect();
    let info = format!("{{\"plucks\": {}, \"seconds\": {:.1}, \"arrange_s\": {arranged:.2}, \"render_s\": {:.2}}}", events.len(), x.len() as f64 / RATE as f64, t.elapsed().as_secs_f64());
    Ok((wav_bytes_ch(&data, 2), info))
}

fn wav_bytes(data: &[f32]) -> Vec<u8> {
    wav_bytes_ch(data, 1)
}

fn wav_bytes_ch(data: &[f32], channels: u16) -> Vec<u8> {
    let bytes = (data.len() * 4) as u32;
    let mut out = Vec::with_capacity(bytes as usize + 44);
    let ch = channels as u32;
    for chunk in [&b"RIFF"[..], &(36 + bytes).to_le_bytes(), b"WAVEfmt ", &16u32.to_le_bytes(), &3u16.to_le_bytes(), &channels.to_le_bytes(), &RATE.to_le_bytes(), &(RATE * 4 * ch).to_le_bytes(), &(4 * channels).to_le_bytes(), &32u16.to_le_bytes(), b"data", &bytes.to_le_bytes()] {
        out.extend_from_slice(chunk);
    }
    for s in data {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

struct Refs {
    dir: PathBuf,
    /// notes.csv rows as JSON objects, and the file names they allow.
    json: String,
    files: Vec<String>,
}

fn load_refs(dir: &Path) -> Result<Refs, String> {
    let text = std::fs::read_to_string(dir.join("notes.csv")).map_err(|e| format!("{}: {e}", dir.join("notes.csv").display()))?;
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().ok_or("empty notes.csv")?.split(',').collect();
    let mut rows = vec![];
    let mut files = vec![];
    for line in lines.filter(|l| !l.trim().is_empty()) {
        let cells: Vec<&str> = line.split(',').collect();
        let fields: Vec<String> = head.iter().zip(&cells).map(|(h, c)| format!("\"{h}\": \"{}\"", c.replace(['"', '\\'], ""))).collect();
        files.push(cells[0].to_string());
        rows.push(format!("{{{}}}", fields.join(", ")));
    }
    Ok(Refs { dir: dir.to_path_buf(), json: format!("[{}]", rows.join(",\n")), files })
}

fn query(s: &str) -> HashMap<String, String> {
    s.split('&').filter_map(|kv| kv.split_once('=')).map(|(k, v)| (k.to_string(), v.replace("%2E", ".").replace("%2D", "-"))).collect()
}

/// Percent-decoding for query values (file names with spaces and brackets).
fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = vec![];
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < b.len() => {
                match std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                    Some(v) => {
                        out.push(v);
                        i += 2;
                    }
                    None => out.push(b'%'),
                }
            }
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn respond(stream: &mut TcpStream, status: &str, kind: &str, extra: &str, body: &[u8]) {
    let head = format!("HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n{extra}\r\n", body.len());
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

fn handle(mut stream: TcpStream, refs: &Option<Refs>, log: &Mutex<PathBuf>) {
    let mut reader = BufReader::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    });
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let mut parts = line.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/"));
    let mut length = 0usize;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h).is_err() || h.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':')
            && k.eq_ignore_ascii_case("content-length")
        {
            length = v.trim().parse().unwrap_or(0);
        }
    }
    let (path, q) = target.split_once('?').map(|(p, q)| (p, query(q))).unwrap_or((target, HashMap::new()));
    match (method, path) {
        ("GET", "/") => respond(&mut stream, "200 OK", "text/html; charset=utf-8", "", PAGE.as_bytes()),
        ("GET", "/api/defaults") => {
            let fields: Vec<String> = KNOBS.iter().map(|(k, v)| format!("\"{k}\": {v}")).collect();
            let body = format!("{{\"knobs\": {{{}}}, \"refs\": {}}}", fields.join(", "), refs.is_some());
            respond(&mut stream, "200 OK", "application/json", "", body.as_bytes());
        }
        ("GET", "/api/render") => match render(&q) {
            Ok((wav, info)) => respond(&mut stream, "200 OK", "audio/wav", &format!("X-Render: {info}\r\n"), &wav),
            Err(e) => respond(&mut stream, "400 Bad Request", "text/plain", "", e.as_bytes()),
        },
        ("GET", "/api/refs") => match refs {
            Some(r) => respond(&mut stream, "200 OK", "application/json", "", r.json.as_bytes()),
            None => respond(&mut stream, "200 OK", "application/json", "", b"[]"),
        },
        ("GET", "/api/ref") => {
            // Only files listed in notes.csv: no paths from the client reach the disk.
            let name = q.get("file").cloned().unwrap_or_default();
            match refs.as_ref().filter(|r| r.files.contains(&name)).map(|r| std::fs::read(r.dir.join(&name))) {
                Some(Ok(bytes)) => respond(&mut stream, "200 OK", "audio/wav", "", &bytes),
                _ => respond(&mut stream, "404 Not Found", "text/plain", "", b"no such reference"),
            }
        }
        ("GET", "/api/songs") => {
            let body = format!("[{}]", songs(log).iter().map(|n| format!("\"{}\"", n.replace(['"', '\\'], ""))).collect::<Vec<_>>().join(", "));
            respond(&mut stream, "200 OK", "application/json", "", body.as_bytes());
        }
        ("GET", "/api/song") => match render_song(&q, log) {
            Ok((wav, info)) => respond(&mut stream, "200 OK", "audio/wav", &format!("X-Render: {info}
"), &wav),
            Err(e) => respond(&mut stream, "400 Bad Request", "text/plain", "", e.as_bytes()),
        },
        ("GET", "/api/song-events") => match arrange(&decode(q.get("name").map(String::as_str).unwrap_or("")), q.get("part").map(|p| decode(p)).as_deref(), log) {
            Ok(text) => respond(&mut stream, "200 OK", "text/plain; charset=utf-8", "", text.as_bytes()),
            Err(e) => respond(&mut stream, "400 Bad Request", "text/plain", "", e.as_bytes()),
        },
        ("POST", "/api/log") if length <= 65_536 => {
            let mut body = vec![0u8; length];
            if reader.read_exact(&mut body).is_err() {
                return respond(&mut stream, "400 Bad Request", "text/plain", "", b"short body");
            }
            let text = String::from_utf8_lossy(&body).replace(['\r', '\n'], " ");
            let at = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            let path = log.lock().unwrap();
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let ok = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&*path)
                .and_then(|mut f| writeln!(f, "{{\"unix\": {at}, \"entry\": {text}}}"));
            match ok {
                Ok(()) => respond(&mut stream, "200 OK", "text/plain", "", b"logged"),
                Err(e) => respond(&mut stream, "500 Internal Server Error", "text/plain", "", e.to_string().as_bytes()),
            }
        }
        _ => respond(&mut stream, "404 Not Found", "text/plain", "", b"not found"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let port: u16 = arg("--port").and_then(|p| p.parse().ok()).unwrap_or(8765);
    // Default references: reference/guitarset-notes in this repo, if they were extracted there.
    let default_refs = concat!(env!("CARGO_MANIFEST_DIR"), "/reference/guitarset-notes");
    let refs = match arg("--refs") {
        Some(d) => Some(load_refs(Path::new(&d)).unwrap_or_else(|e| panic!("--refs: {e}"))),
        None => load_refs(Path::new(default_refs)).ok(),
    };
    let log = PathBuf::from(arg("--log").unwrap_or_else(|| concat!(env!("CARGO_MANIFEST_DIR"), "/listening/log.jsonl").to_string()));
    if let Some(d) = arg("--gp-dir") {
        let _ = GP_DIR.set(PathBuf::from(d));
    }
    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|e| panic!("port {port}: {e} (try --port)"));
    println!("Guitar Pro files from {}", gp_dir().display());
    println!("workbench: http://127.0.0.1:{port}  (references: {}; log: {})", refs.as_ref().map(|r| format!("{} notes", r.files.len())).unwrap_or("none".into()), log.display());
    let refs = Arc::new(refs);
    let log = Arc::new(Mutex::new(log));
    for stream in listener.incoming().flatten() {
        let (refs, log) = (refs.clone(), log.clone());
        std::thread::spawn(move || handle(stream, &refs, &log));
    }
}
