//! Minimal WAV writer: 32-bit float, which hound, browsers and DAWs all read.

use std::io::Write;
use std::path::Path;

pub fn write_mono_f32(path: &Path, rate: u32, data: &[f32]) -> std::io::Result<()> {
    write_f32(path, rate, 1, data)
}

/// 32-bit float WAV; `data` is interleaved when `channels` > 1.
pub fn write_f32(path: &Path, rate: u32, channels: u16, data: &[f32]) -> std::io::Result<()> {
    let bytes = (data.len() * 4) as u32;
    let mut out = Vec::with_capacity(bytes as usize + 44);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + bytes).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&3u16.to_le_bytes()); // IEEE float
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 4 * channels as u32).to_le_bytes());
    out.extend_from_slice(&(4 * channels).to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&bytes.to_le_bytes());
    for s in data {
        out.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::File::create(path)?.write_all(&out)
}
