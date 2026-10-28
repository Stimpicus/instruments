"""Compare real notes (from guitarset_notes.py) with the model, string by string.

    python -I tools/compare.py <ref dir> <guitar binary> <work dir> [--velocity 0.55] [--max 12]

For every reference note the model renders the same string and fret (into
<work dir>), and both are measured the same way:
- third-octave band levels over the first 250 ms, relative to the band sum
  (spectral shape only, since recording level and pluck strength are unknown);
- spectral centroid over 0-100 ms and 300-700 ms;
- overall decay slope, dB/s, between 0.1 s and 0.9 s (the clip ends at 1 s).
Prints per-string means for reference and model, and the mean band difference
(model minus reference, dB) so a systematic tilt shows up.
"""

import argparse
import csv
import os
import subprocess
import sys

import numpy as np
from scipy.io import wavfile

BANDS = [100 * 2 ** (i / 3) for i in range(0, 22)]  # 100 Hz .. ~12.7 kHz


def load(path):
    rate, x = wavfile.read(path)
    x = x.astype(np.float64)
    if x.ndim > 1:
        x = x.mean(axis=1)
    return rate, x


def align(x, rate):
    env = np.abs(x)
    i = int(np.argmax(env > env.max() * 0.2))
    return x[max(i - int(0.002 * rate), 0):]


def features(x, rate):
    x = align(x, rate)
    seg = x[: int(0.25 * rate)]
    sp = np.abs(np.fft.rfft(seg * np.hanning(len(seg)), 1 << 15)) ** 2
    fr = np.fft.rfftfreq(1 << 15, 1 / rate)
    bands = []
    for f in BANDS:
        lo, hi = f / 2 ** (1 / 6), f * 2 ** (1 / 6)
        bands.append(sp[(fr >= lo) & (fr < hi)].sum())
    bands = np.array(bands)
    bands_db = 10 * np.log10(bands / bands.sum() + 1e-30)

    def centroid(t0, t1):
        s = x[int(t0 * rate): int(t1 * rate)]
        if len(s) < 64:
            return np.nan
        m = np.abs(np.fft.rfft(s * np.hanning(len(s))))
        f = np.fft.rfftfreq(len(s), 1 / rate)
        return (m * f).sum() / (m.sum() + 1e-30)

    hop = int(0.01 * rate)
    rms = np.array([np.sqrt(np.mean(x[i:i + hop] ** 2)) for i in range(0, len(x) - hop, hop)])
    t = np.arange(len(rms)) * 0.01
    sel = (t >= 0.1) & (t <= 0.9)
    slope = np.polyfit(t[sel], 20 * np.log10(rms[sel] + 1e-30), 1)[0] if sel.sum() > 10 else np.nan
    return bands_db, centroid(0, 0.1), centroid(0.3, 0.7), slope


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("ref")
    ap.add_argument("binary")
    ap.add_argument("work")
    ap.add_argument("--velocity", type=float, default=0.55)
    ap.add_argument("--max", type=int, default=12, help="reference notes per string at most")
    ap.add_argument("--tool", default="pick")
    ap.add_argument("--min-dur", type=float, default=0.9, help="skip reference notes annotated shorter (player damped them)")
    a = ap.parse_args()
    os.makedirs(a.work, exist_ok=True)
    with open(os.path.join(a.ref, "notes.csv"), encoding="utf-8") as f:
        rows = list(csv.DictReader(f))
    per = {s: [] for s in range(1, 7)}
    for r in rows:
        if float(r["annotated_dur"]) < a.min_dur:
            continue
        if len(per[int(r["string"])]) < a.max:
            per[int(r["string"])].append(r)
    all_diff = []
    per_string = []
    print("str  n | centroid 0-100ms ref/model | 300-700ms ref/model | decay dB/s ref/model")
    for s, rs in per.items():
        if not rs:
            continue
        ref_f, mod_f = [], []
        for r in rs:
            rate, x = load(os.path.join(a.ref, r["file"]))
            ref_f.append(features(x, rate))
            out = os.path.join(a.work, f"m_s{s}f{int(r['fret']):02d}.wav")
            if not os.path.exists(out):
                subprocess.run([a.binary, "note", str(s - 1), r["fret"], str(a.velocity), out, a.tool], check=True, capture_output=True)
            rate, y = load(out)
            mod_f.append(features(y[: int(1.02 * rate)], rate))
        rb = np.mean([f[0] for f in ref_f], axis=0)
        mb = np.mean([f[0] for f in mod_f], axis=0)
        all_diff.append(mb - rb)
        octs = [(mb - rb)[i:i + 3].mean() for i in range(0, 21, 3)]
        per_string.append((s, octs))
        m = lambda fs, i: np.nanmean([f[i] for f in fs])
        print(f" {s}  {len(rs):2d} | {m(ref_f, 1):6.0f} / {m(mod_f, 1):6.0f}         | {m(ref_f, 2):6.0f} / {m(mod_f, 2):6.0f}      | {m(ref_f, 3):6.1f} / {m(mod_f, 3):6.1f}")
    print("octave bands (Hz):  " + " ".join(f"{int(BANDS[i + 1]):>5}" for i in range(0, 21, 3)))
    for s, octs in per_string:
        print(f"  string {s} model-ref: " + " ".join(f"{v:+5.1f}" for v in octs))
    d = np.mean(all_diff, axis=0)
    print("band (Hz):  " + " ".join(f"{int(f):>5}" for f in BANDS))
    print("model-ref:  " + " ".join(f"{v:+5.1f}" for v in d))
    return 0


if __name__ == "__main__":
    sys.exit(main())
