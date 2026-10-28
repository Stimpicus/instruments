//! Radix-2 FFT and FFT convolution, enough for convolving notes with the body.

use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct C {
    pub re: f64,
    pub im: f64,
}

impl C {
    pub fn new(re: f64, im: f64) -> C {
        C { re, im }
    }
    pub fn mul(self, o: C) -> C {
        C::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
    pub fn add(self, o: C) -> C {
        C::new(self.re + o.re, self.im + o.im)
    }
    pub fn sub(self, o: C) -> C {
        C::new(self.re - o.re, self.im - o.im)
    }
    pub fn scale(self, k: f64) -> C {
        C::new(self.re * k, self.im * k)
    }
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
    pub fn expi(phase: f64) -> C {
        C::new(phase.cos(), phase.sin())
    }
}

/// In-place FFT; `inverse` also divides by n. `a.len()` must be a power of two.
pub fn fft(a: &mut [C], inverse: bool) {
    let n = a.len();
    assert!(n.is_power_of_two(), "fft length {n} is not a power of two");
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            a.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = 2.0 * PI / len as f64 * if inverse { 1.0 } else { -1.0 };
        let w = C::expi(ang);
        for chunk in a.chunks_mut(len) {
            let mut wk = C::new(1.0, 0.0);
            let (lo, hi) = chunk.split_at_mut(len / 2);
            for k in 0..len / 2 {
                let u = lo[k];
                let v = hi[k].mul(wk);
                lo[k] = u.add(v);
                hi[k] = u.sub(v);
                wk = wk.mul(w);
            }
        }
        len <<= 1;
    }
    if inverse {
        let k = 1.0 / n as f64;
        for x in a.iter_mut() {
            *x = x.scale(k);
        }
    }
}

/// A kernel prepared for repeated overlap-add convolution.
#[derive(Clone, Debug)]
pub struct Convolver {
    block: usize,
    size: usize,
    kernel: Vec<C>,
    klen: usize,
}

impl Convolver {
    pub fn new(kernel: &[f64]) -> Convolver {
        let klen = kernel.len().max(1);
        let block = klen.next_power_of_two().max(4096);
        let size = (block + klen).next_power_of_two();
        let mut k = vec![C::default(); size];
        for (d, &s) in k.iter_mut().zip(kernel) {
            d.re = s;
        }
        fft(&mut k, false);
        Convolver { block, size, kernel: k, klen }
    }

    /// Full linear convolution, truncated to `x.len()` samples.
    pub fn apply(&self, x: &[f64]) -> Vec<f64> {
        let mut out = vec![0.0; x.len() + self.klen];
        let mut buf = vec![C::default(); self.size];
        for (b, chunk) in x.chunks(self.block).enumerate() {
            buf.iter_mut().for_each(|c| *c = C::default());
            for (d, &s) in buf.iter_mut().zip(chunk) {
                d.re = s;
            }
            fft(&mut buf, false);
            for (d, k) in buf.iter_mut().zip(&self.kernel) {
                *d = d.mul(*k);
            }
            fft(&mut buf, true);
            let at = b * self.block;
            for (i, c) in buf.iter().enumerate().take(chunk.len() + self.klen) {
                out[at + i] += c.re;
            }
        }
        out.truncate(x.len());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convolution_matches_direct() {
        let x: Vec<f64> = (0..10_000).map(|i| ((i * 7919) % 101) as f64 / 50.0 - 1.0).collect();
        let k: Vec<f64> = (0..300).map(|i| (-(i as f64) / 40.0).exp() * if i % 3 == 0 { 1.0 } else { -0.5 }).collect();
        let fast = Convolver::new(&k).apply(&x);
        for n in [0, 1, 299, 300, 4095, 4096, 4097, 9999] {
            let direct: f64 = (0..k.len()).filter(|&j| j <= n).map(|j| k[j] * x[n - j]).sum();
            assert!((fast[n] - direct).abs() < 1e-9, "n={n}: {} vs {direct}", fast[n]);
        }
    }

    #[test]
    fn convolution_detects_a_wrong_kernel() {
        // The check above must be able to fail: a shifted kernel gives a different answer.
        let x: Vec<f64> = (0..5000).map(|i| (i as f64 * 0.37).sin()).collect();
        let a = Convolver::new(&[1.0, 0.5]).apply(&x);
        let b = Convolver::new(&[0.0, 1.0, 0.5]).apply(&x);
        assert!(a.iter().zip(&b).any(|(p, q)| (p - q).abs() > 0.1));
    }
}
