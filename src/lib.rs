//! Physically modelled instruments, rendered offline into samples a sampler can
//! play (game engines, Strudel via region maps). No dependencies.
//!
//! - [`guitar`]: a steel-string acoustic guitar (modal strings coupled to a
//!   modal body through the bridge). See `docs/acoustic-guitar.md`.
//! - [`export`]: a multisample set (WAV files plus an `.instrument.json`).

pub mod export;
pub mod fft;
pub mod guitar;
pub mod wav;

/// Small deterministic generator (xorshift64*), so every render is reproducible.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in [-1, 1).
    pub fn signed(&mut self) -> f64 {
        self.unit() * 2.0 - 1.0
    }
}
