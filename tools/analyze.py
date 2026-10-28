"""Measure a plucked-string recording or render.

    python -I tools/analyze.py <file.wav> <nominal f0 Hz> [--png out.png] [--partials N]

Reports: measured f0 (and cents off nominal), partial frequencies and the
inharmonicity B they imply, per-partial T60 from a straight-line fit to the
decay in dB, the fit residual (beating between polarizations shows up here),
spectral centroid early and late, and peak level. Works the same on real
recordings, so renders and references are measured by one ruler.
"""

import argparse
import struct
import sys
import zlib

import numpy as np
from scipy.io import wavfile


def load(path):
    rate, x = wavfile.read(path)
    x = x.astype(np.float64)
    if x.ndim > 1:
        x = x.mean(axis=1)
    if np.issubdtype(np.asarray(wavfile.read(path)[1]).dtype, np.integer):
        x /= np.iinfo(wavfile.read(path)[1].dtype).max
    return rate, x


def onset(x, rate):
    env = np.abs(x)
    thr = env.max() * 0.1
    return int(np.argmax(env > thr))


def peak_near(spec, freqs, f, tol):
    lo, hi = np.searchsorted(freqs, [f - tol, f + tol])
    if hi <= lo + 2:
        return None, None
    i = lo + int(np.argmax(spec[lo:hi]))
    if 0 < i < len(spec) - 1:
        a, b, c = np.log(spec[i - 1] + 1e-30), np.log(spec[i] + 1e-30), np.log(spec[i + 1] + 1e-30)
        d = 0.5 * (a - c) / (a - 2 * b + c) if (a - 2 * b + c) != 0 else 0.0
        return freqs[i] + d * (freqs[1] - freqs[0]), spec[i]
    return freqs[i], spec[i]


def png(path, img):
    """Write a greyscale PNG from a 2-D array in [0, 1] (row 0 = top)."""
    h, w = img.shape
    raw = b"".join(b"\x00" + (np.clip(img[r], 0, 1) * 255).astype(np.uint8).tobytes() for r in range(h))

    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 0, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("wav")
    ap.add_argument("f0", type=float)
    ap.add_argument("--png")
    ap.add_argument("--partials", type=int, default=16)
    ap.add_argument("--glide", action="store_true", help="track f0 over time (cents vs the late pitch)")
    a = ap.parse_args()

    rate, x = load(a.wav)
    x = x[onset(x, rate):]
    dur = len(x) / rate
    print(f"{a.wav}: {dur:.2f} s from onset, peak {20 * np.log10(np.abs(x).max() + 1e-30):.1f} dBFS")

    # f0 and partials from a long window after the attack.
    start, length = int(0.15 * rate), min(int(1.5 * rate), len(x) - int(0.15 * rate))
    seg = x[start:start + length] * np.hanning(length)
    nfft = 1 << 20
    spec = np.abs(np.fft.rfft(seg, nfft))
    freqs = np.fft.rfftfreq(nfft, 1 / rate)
    f1, _ = peak_near(spec, freqs, a.f0, a.f0 * 0.06)
    if f1 is None:
        print("no fundamental found")
        return 1
    print(f"f0 {f1:.3f} Hz ({1200 * np.log2(f1 / a.f0):+.1f} cents vs {a.f0:.3f})")
    parts = []
    b_est = 0.0
    for k in range(1, a.partials + 1):
        guess = k * f1 * np.sqrt((1 + b_est * k * k) / (1 + b_est))
        if guess > rate * 0.45:
            break
        fk, mag = peak_near(spec, freqs, guess, f1 * 0.25)
        if fk is None:
            break
        parts.append((k, fk, mag))
        ks = np.array([p[0] for p in parts], float)
        ratio = np.array([p[1] for p in parts]) / (ks * f1)
        if len(parts) >= 4:
            # ratio^2 ~ (1 + B k^2) / (1 + B): fit the slope of ratio^2 against k^2.
            slope = np.polyfit(ks ** 2, ratio ** 2, 1)[0]
            b_est = max(slope, 0.0)
    print(f"inharmonicity B ~ {b_est:.2e} (from {len(parts)} partials)")

    # Per-partial decay from an STFT.
    win, hop = 4096, 256
    frames = (len(x) - win) // hop
    if frames < 8:
        print("too short for decay fits")
        return 0
    w = np.hanning(win)
    stft = np.array([np.abs(np.fft.rfft(x[i * hop:i * hop + win] * w)) for i in range(frames)])
    sf = np.fft.rfftfreq(win, 1 / rate)
    times = (np.arange(frames) * hop + win / 2) / rate
    floor_db = 20 * np.log10(np.median(stft[-max(frames // 10, 1):]) + 1e-30)
    print(" k   freq Hz   level dB   T60 s   resid dB")
    for k, fk, mag in parts:
        bi = int(round(fk / sf[1]))
        band = stft[:, max(bi - 2, 0):bi + 3].max(axis=1)
        db = 20 * np.log10(band + 1e-30)
        top = db.max()
        t0 = int(np.argmax(db))
        # Fit from the peak to where it falls 40 dB or meets the noise floor + 10 dB.
        end = t0 + 1
        limit = max(top - 40, floor_db + 10)
        while end < frames and db[end] > limit:
            end += 1
        if end - t0 < 6:
            print(f"{k:2d} {fk:9.2f}  {top:8.1f}   (too short)")
            continue
        tt, yy = times[t0:end], db[t0:end]
        slope, icpt = np.polyfit(tt, yy, 1)
        resid = np.sqrt(np.mean((yy - (slope * tt + icpt)) ** 2))
        t60 = -60 / slope if slope < 0 else float("inf")
        print(f"{k:2d} {fk:9.2f}  {top:8.1f}  {t60:6.2f}   {resid:6.2f}")

    def centroid(t_from, t_to):
        i0, i1 = int(t_from * rate), int(t_to * rate)
        s = np.abs(np.fft.rfft(x[i0:i1] * np.hanning(i1 - i0)))
        f = np.fft.rfftfreq(i1 - i0, 1 / rate)
        return (s * f).sum() / (s.sum() + 1e-30)

    late = min(1.0, dur - 0.2)
    print(f"spectral centroid: {centroid(0, 0.1):.0f} Hz (0-100 ms), {centroid(late, late + 0.2):.0f} Hz ({late:.1f}-{late + 0.2:.1f} s)")

    if a.glide:
        # f0 in 120 ms windows; reference = the pitch at 1.5-1.8 s.
        def f0_at(t):
            i0 = int(t * rate)
            ln = int(0.12 * rate)
            if i0 + ln > len(x):
                return None
            sp = np.abs(np.fft.rfft(x[i0:i0 + ln] * np.hanning(ln), 1 << 19))
            fr = np.fft.rfftfreq(1 << 19, 1 / rate)
            return peak_near(sp, fr, f1, f1 * 0.05)[0]
        ref = f0_at(min(1.5, dur - 0.2))
        pts = [(t, f0_at(t)) for t in (0.02, 0.05, 0.1, 0.2, 0.4, 0.8)]
        print("glide (cents vs pitch at 1.5 s): " + ", ".join(f"{t:.2f}s {1200 * np.log2(f / ref):+.1f}" for t, f in pts if f))
    if a.png:
        top_hz = 8000
        nb = int(top_hz / sf[1])
        img = 20 * np.log10(stft[:, :nb].T + 1e-30)
        img = (img - (img.max() - 80)) / 80
        img = img[::-1]
        # squash to at most 1200 px wide, 600 tall
        step = max(1, img.shape[1] // 1200)
        img = img[:, ::step]
        rstep = max(1, img.shape[0] // 600)
        img = img[::rstep]
        png(a.png, img)
        print(f"spectrogram 0-{top_hz} Hz, 80 dB range -> {a.png}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
