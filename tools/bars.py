"""A transcription written one bar per line, turned into an arranger song file.

    python -I tools/bars.py <transcription.txt> <song.json>

The text format (comments start with #):
    tempo 92            beats per minute (later in the file: a tempo change from the next bar on)
    swing 1.0           0 straight .. 1 triplet feel for eighths written at .5
    capo 0
    tool pick           pick or finger
    beats 4             beats per bar (until changed)
    section Intro       a label for the bars that follow (shown in the workbench)
    12: 0:G7 0.5:G5 ... bar number, then the bar in the arranger's tab notation (see tools/arrange.py)

Bars play in file order; every bar must keep its notes inside its beats.
"""

import json
import re
import sys


def convert(text):
    song = {"bpm": 120, "swing": 0.0, "capo": 0, "tool": "pick", "seed": 1, "shapes": {}, "patterns": {}, "sections": {}, "form": []}
    beats, label, problems, bpm, started = 4.0, "", [], None, False
    for n, raw in enumerate(text.splitlines(), 1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        word, _, rest = line.partition(" ")
        if word == "tempo":
            # Before the first bar it is the song's tempo; later it changes tempo from the next bar on.
            bpm = float(rest)
            if not started:
                song["bpm"] = bpm
        elif word == "swing":
            song["swing"] = float(rest)
        elif word == "capo":
            song["capo"] = int(rest)
        elif word == "tool":
            song["tool"] = rest.strip()
        elif word == "beats":
            beats = float(rest)
        elif word == "section":
            label = rest.strip()
        elif re.fullmatch(r"\d+[a-z]?:", word):
            bar = word[:-1]
            for tok in rest.split():
                at = tok.partition(":")[0]
                try:
                    pos = float(at)
                except ValueError:
                    problems.append(f"line {n}: bar {bar}: bad beat in '{tok}'")
                    continue
                if not 0 <= pos < beats:
                    problems.append(f"line {n}: bar {bar}: '{tok}' is outside the bar's {beats:g} beats")
            name = f"bar{bar}"
            if name in song["sections"]:
                problems.append(f"line {n}: bar {bar} appears twice")
            song["sections"][name] = {"tab": rest.strip(), "beats": beats, "label": label or f"bar {bar}"}
            if started and bpm is not None and bpm != song["bpm"]:
                song["sections"][name]["bpm"] = bpm
            started = True
            song["form"].append(name)
        else:
            problems.append(f"line {n}: not understood: {raw.strip()}")
    return song, problems


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(2)
    with open(sys.argv[1], encoding="utf-8") as f:
        text = f.read()
    song, problems = convert(text)
    first = next((l[1:].strip() for l in text.splitlines() if l.startswith("#")), "")
    song["title"] = first or sys.argv[1]
    if problems:
        print("\n".join(problems), file=sys.stderr)
        sys.exit(1)
    with open(sys.argv[2], "w", encoding="utf-8") as f:
        json.dump(song, f, indent=1)
    print(f"{len(song['form'])} bars at {song['bpm']:g} BPM -> {sys.argv[2]}")
