//! The guitar body seen from the bridge: a set of damped modes. It gives
//! - the bridge mobility Y(f) (velocity per unit force), which sets how fast
//!   each string partial loses energy into the body and how far it is detuned;
//! - impulse responses for the bridge's motion (which drives the other strings)
//!   and for what radiates (each mode weighted by how well it moves air), so a
//!   note's bridge force becomes motion and sound by FFT convolution.

use std::f64::consts::{PI, TAU};

use crate::Rng;
use crate::fft::C;

#[derive(Clone, Debug)]
pub struct Mode {
    /// Natural frequency, Hz.
    pub f: f64,
    pub q: f64,
    /// Effective mass at the bridge, kg (peak mobility is q / (m * 2 pi f)).
    pub m: f64,
    /// Radiation efficiency relative to the bridge's own motion: 1 for modes that
    /// breathe (the whole top moving one way), less for bending modes whose parts
    /// move in opposite directions and largely cancel in the far field.
    pub rad: f64,
}

#[derive(Clone, Debug)]
pub struct Body {
    pub modes: Vec<Mode>,
}

/// Parameters of the dense, statistical part of the body above the named modes.
#[derive(Clone, Debug)]
pub struct Dense {
    pub from_hz: f64,
    pub to_hz: f64,
    /// Mean spacing between modes, Hz.
    pub spacing: f64,
    pub q: f64,
    /// Target mean real mobility at `from_hz`, m/(N s).
    pub mobility: f64,
    /// Above this frequency the bridge's mass makes the mobility fall (1/f).
    pub mass_corner: f64,
    pub seed: u64,
}

impl Body {
    /// Named low modes plus a dense statistical set above them.
    pub fn new(named: &[Mode], dense: &Dense) -> Body {
        let mut modes = named.to_vec();
        let mut r = Rng::new(dense.seed);
        let mut f = dense.from_hz;
        while f < dense.to_hz {
            f += dense.spacing * (0.4 + 1.2 * r.unit());
            // Mean Re(Y) of a mode family is pi * n_omega / (2 m) with n_omega modes per rad/s.
            let n_omega = 1.0 / (TAU * dense.spacing);
            let target = dense.mobility / (1.0 + (f / dense.mass_corner).powi(2)).sqrt();
            let m = PI * n_omega / (2.0 * target) * (0.6 + 0.8 * r.unit());
            modes.push(Mode { f, q: dense.q * (0.7 + 0.6 * r.unit()), m, rad: 1.0 });
        }
        Body { modes }
    }

    /// Bridge mobility (velocity / force) at `f` Hz.
    pub fn mobility(&self, f: f64) -> C {
        let w = TAU * f;
        let mut y = C::default();
        for m in &self.modes {
            let wm = TAU * m.f;
            // Y = j w / (m (wm^2 - w^2 + j w wm / q))
            let den = C::new(m.m * (wm * wm - w * w), m.m * w * wm / m.q);
            let num = C::new(0.0, w);
            let d2 = den.re * den.re + den.im * den.im;
            y = y.add(C::new((num.re * den.re + num.im * den.im) / d2, (num.im * den.re - num.re * den.im) / d2));
        }
        y
    }

    /// Responses to a unit force impulse at the bridge (area 1 N s), sampled at `rate`:
    /// (bridge velocity, radiating velocity = each mode weighted by its radiation efficiency).
    pub fn responses(&self, rate: u32) -> (Vec<f64>, Vec<f64>) {
        let dt = 1.0 / rate as f64;
        let nyq = rate as f64 * 0.5;
        let slowest = self.modes.iter().map(|m| PI * m.f / m.q).fold(f64::INFINITY, f64::min);
        let len = ((1e5f64.ln() / slowest) * rate as f64).ceil() as usize;
        let mut vb = vec![0.0; len];
        let mut rad = vec![0.0; len];
        for m in self.modes.iter().filter(|m| m.f < nyq * 0.95) {
            let w0 = TAU * m.f;
            let s = w0 / (2.0 * m.q);
            let wd = (w0 * w0 - s * s).sqrt();
            // Impulse response of s / (m (s^2 + 2 s sigma + w0^2)): e^{-st}(cos wd t - (s/wd) sin wd t) / m.
            let rot = C::new((-s * dt).exp() * (wd * dt).cos(), (-s * dt).exp() * (wd * dt).sin());
            let mut z = C::new(1.0, 0.0);
            let k = s / wd;
            for (o, r) in vb.iter_mut().zip(rad.iter_mut()) {
                let v = (z.re - k * z.im) * dt / m.m;
                *o += v;
                *r += v * m.rad;
                z = z.mul(rot);
                if z.abs() < 1e-7 {
                    break;
                }
            }
        }
        (vb, rad)
    }
}
