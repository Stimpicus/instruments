"""Read an ASCII guitar tab (the common e|-B|-G|-D|-A|-E| block format) into arranger tab sections.

    python -I tools/tabimport.py <tab.txt> [--capo N]

Prints JSON: {"sections": {...}, "groups": {...}} where each tab bar becomes a tab section
"<block>.<n>" (4 beats; a short first bar is a pickup) and each block "<block>" is a group
listing its bars in order. Blocks are named by the nearest heading above them, e.g.
"(Riff 1)" -> "riff1", "[Solo 2]" -> "solo2". Copy what you need into a song file.

Timing: tabs carry no note values, so a bar's notes are spread by their character column over
the bar (or, for a pickup, at the block's usual characters per beat) and snapped to sixteenths.
Techniques: h hammer-on, p pull-off, / and \\ slides (to a following fret on the same string;
with no earlier note, a slide into the note), ~ vibrato, b bend (a whole tone; the next same
fret on that string releases it), x dead note.
"""

import json
import re
import statistics
import sys

STRINGS = "eBGDAE"  # top to bottom in the tab
LETTER = {"e": "e", "B": "B", "G": "G", "D": "D", "A": "A", "E": "E"}


def systems(lines):
    """Yield (heading, chord_line, [6 tab lines]) for every tab system."""
    heading = "tab"
    i = 0
    while i < len(lines):
        line = lines[i].rstrip("\n")
        m = re.match(r"\s*[\[(]([^\])]+)[\])]\s*$", line)
        if m:
            heading = re.sub(r"[^a-z0-9]", "", m.group(1).lower())
        if line.startswith("e|") and i + 5 < len(lines) and all(lines[i + k].lstrip().startswith(STRINGS[k] + "|") for k in range(6)):
            chord_line = lines[i - 1].rstrip("\n") if i > 0 and "|" not in lines[i - 1] else ""
            yield heading, chord_line, [lines[i + k].rstrip("\n").lstrip() for k in range(6)]
            i += 6
            continue
        i += 1


def bars_of(tab):
    """Split a system into bars: [(start column, end column)] using the top line's bar lines."""
    top = tab[0]
    cuts = [m.start() for m in re.finditer(r"\|", top)]
    return [(a + 1, b) for a, b in zip(cuts, cuts[1:]) if b - a > 1]


def notes_in(line, a, b):
    """Notes on one string between columns a and b: (column, fret or None for x, flags)."""
    out = []
    seg = line[a:b] if len(line) > a else ""
    k = 0
    while k < len(seg):
        c = seg[k]
        if c.isdigit():
            j = k
            while j < len(seg) and seg[j].isdigit():
                j += 1
            # what leads into this note (skipping dashes for a hanging "9-h-10")
            back = seg[:k].rstrip("-")
            lead = back[-1:] if back and back[-1] in "hp/\\" else ""
            if lead in "hp" and back and len(seg[:k]) - len(back) > 2:
                lead = ""
            trail = seg[j:j + 1]
            after = seg[j:].lstrip("~-")
            slide_out = trail in "/\\" or (seg[j:j + 2].startswith("~") and seg[j + 1:j + 2] in "/\\")
            out.append([a + k, int(seg[k:j]), {"lead": lead, "vib": trail == "~", "bend": trail == "b",
                                               "slide_out": slide_out and not any(ch.isdigit() for ch in after)}])
            k = j
        elif c == "x":
            out.append([a + k, None, {"lead": "", "vib": False, "bend": False, "slide_out": False}])
            k += 1
        else:
            k += 1
    return out


def import_tab(text, capo=0):
    lines = text.splitlines()
    sections, groups, counts, carry = {}, {}, {}, {}
    for heading, chord_line, tab in systems(lines):
        spans = bars_of(tab)
        if not spans:
            continue
        lens = [b - a for a, b in spans]
        full = statistics.median(lens)
        name = heading
        groups.setdefault(name, [])
        last_fret = {}
        bent = {}
        # strings whose last note slid off the end of the previous bar (carries across lines of a block)
        slide_into = carry.setdefault(name, set())
        # Most tabs space eighth notes three characters apart; bars that fit that grid use it.
        grid = lambda length: abs((length - 1) / 6 - 4) < 0.75
        for bi, (a, b) in enumerate(spans):
            length = b - a
            pickup = bi == 0 and len(spans) > 1 and length < 0.6 * full
            on_grid = grid(full) if pickup else grid(length)
            last_col = max((col for si, line in enumerate(tab) for col, _, _ in notes_in(line, a, b)), default=a)
            if on_grid and not pickup and (last_col - a - 1) / 6.0 > 3.5 + 1e-9:
                on_grid = False  # notes run past the grid's last eighth: spread the bar instead
            per_beat = 6.0 if on_grid else (full - 1) / 4 if pickup else (length - 1) / 4
            beats = max(0.5, round((length - 1) / per_beat * 2) / 2) if pickup else 4.0
            step = 0.5 if on_grid else 0.25
            # Gather every note in time order first, then place them.
            found = []
            for si, line in enumerate(tab):
                for col, fret, fl in notes_in(line, a, b):
                    found.append((col, si, fret, fl))
            found.sort()
            tokens = {}
            prev_t = {}
            for col, si, fret, fl in found:
                letter = LETTER[STRINGS[si]]
                raw = max(0, col - a - 1) / per_beat
                legato = fret is not None and (fl["lead"] in ("h", "p", "/", "\\") and letter in last_fret)
                if legato:
                    # Ornaments follow their parent note by at least a sixteenth.
                    pos = max(round(raw / 0.25) * 0.25, prev_t.get(letter, -1) + 0.25)
                else:
                    pos = round(raw / step) * step
                pos = min(pos, beats - 0.125)
                prev_t[letter] = pos
                if fret is None:
                    tok = f"{letter}{last_fret.get(letter, 0)}x"
                elif fl["lead"] in ("h", "p") and letter in last_fret:
                    tok = f"{letter}{fret}{fl['lead']}"
                elif letter in slide_into and fret is not None:
                    tok = f"{letter}{fret}s"
                elif fl["lead"] in ("/", "\\"):
                    tok = f"{letter}{fret}s" if letter in last_fret else f"/{letter}{fret}"
                elif bent.get(letter) == fret:
                    tok = f"{letter}{fret}r"
                    bent.pop(letter)
                else:
                    tok = f"{letter}{fret}"
                if fl["bend"] and fret is not None:
                    if not tok.endswith(("h", "p", "s")):
                        tok += "b2"
                    bent[letter] = fret
                if fl["vib"]:
                    tok += "~"
                if fret is not None:
                    last_fret[letter] = fret
                slide_into.discard(letter)
                if fl.get("slide_out"):
                    slide_into.add(letter)
                tokens.setdefault(pos, []).append(tok)
            # chord names written above this bar (first one wins as the label)
            chords = [m.group(0) for m in re.finditer(r"\S+", chord_line[a:b])] if chord_line else []
            n = counts.get(name, 0)
            counts[name] = n + 1
            key = f"{name}.{n}"
            sections[key] = {
                "tab": " ".join(f"{p:g}:" + "+".join(ts) for p, ts in sorted(tokens.items())),
                "beats": beats,
                "label": chords[0] if chords else (sections[groups[name][-1]]["label"] if groups[name] else "tab"),
                "chords_above": " ".join(chords),
            }
            groups[name].append(key)
    return {"sections": sections, "groups": groups}


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    capo = int(sys.argv[sys.argv.index("--capo") + 1]) if "--capo" in sys.argv else 0
    with open(sys.argv[1], encoding="utf-8") as f:
        print(json.dumps(import_tab(f.read(), capo), indent=1))
