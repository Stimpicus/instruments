"""Pull isolated single notes out of GuitarSet (CC BY 4.0, Xi et al. 2018,
https://zenodo.org/records/3371780, v1.1.0) to measure a real acoustic guitar.

    python -I tools/guitarset_notes.py <annotation dir> <audio_mono-mic dir> <out dir> [--gap 1.0]

A note counts as isolated when no other note (any string) starts from 0.3 s
before it until `gap` seconds after it. Each one is written as
<out>/s<string>f<fret>_<n>.wav (string 1 = low E, as in the model's files) with
its onset 20 ms in, and listed in <out>/notes.csv with the source file and time.
"""

import argparse
import csv
import json
import os
import sys

import numpy as np
from scipy.io import wavfile

OPEN = [40, 45, 50, 55, 59, 64]


def notes(jams_path):
    with open(jams_path, encoding="utf-8") as f:
        j = json.load(f)
    out = []
    for ann in j["annotations"]:
        if ann["namespace"] != "note_midi":
            continue
        string = int(ann["annotation_metadata"]["data_source"])
        for d in ann["data"]:
            out.append((float(d["time"]), float(d["duration"]), float(d["value"]), string))
    return sorted(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("ann")
    ap.add_argument("audio")
    ap.add_argument("out")
    ap.add_argument("--gap", type=float, default=1.0)
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    rows = []
    count = {}
    for name in sorted(os.listdir(a.ann)):
        if not name.endswith(".jams"):
            continue
        stem = name[:-5]
        wav = os.path.join(a.audio, stem + "_mic.wav")
        if not os.path.exists(wav):
            continue
        ns = notes(os.path.join(a.ann, name))
        rate, x = wavfile.read(wav)
        x = x.astype(np.float64)
        if np.issubdtype(wavfile.read(wav)[1].dtype, np.integer):
            x /= 32768.0
        if x.ndim > 1:
            x = x.mean(axis=1)
        starts = np.array([n[0] for n in ns])
        for t, dur, midi, string in ns:
            others = starts[(starts > t - 0.3) & (starts < t + a.gap)]
            if len(others) != 1:
                continue
            fret = int(round(midi - OPEN[string]))
            if fret < 0 or fret > 20:
                continue
            i0 = int((t - 0.02) * rate)
            i1 = int((t + a.gap) * rate)
            if i0 < 0 or i1 > len(x):
                continue
            key = (string, fret)
            count[key] = count.get(key, 0) + 1
            fn = f"s{string + 1}f{fret:02d}_{count[key]}.wav"
            wavfile.write(os.path.join(a.out, fn), rate, x[i0:i1].astype(np.float32))
            rows.append({"file": fn, "string": string + 1, "fret": fret, "midi": round(midi, 2), "source": stem, "time": round(t, 3), "annotated_dur": round(dur, 3)})
    with open(os.path.join(a.out, "notes.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["file", "string", "fret", "midi", "source", "time", "annotated_dur"])
        w.writeheader()
        w.writerows(rows)
    print(f"{len(rows)} isolated notes from {a.ann}; per string: " + ", ".join(f"s{s + 1} {sum(v for (k, _), v in count.items() if k == s)}" for s in range(6)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
