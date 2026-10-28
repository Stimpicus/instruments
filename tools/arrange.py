"""Turn a song description (JSON) into guitar plucks for `guitar perform`, and check renders.

    python -I tools/arrange.py events <song.json | -> <out.csv | -> [--part <name>]
    python -I tools/arrange.py check  <song.json> <render (dry).wav>

Song JSON:
  title, bpm, beats_per_bar (4), swing (0 straight .. 1 triplet), capo, tool (pick|finger),
  strum_ms (spread between strings on a downstroke), seed,
  thumb_weight (fingerstyle: bass strings E A D against the trebles, 1 = even),
  tuning ([39, 44, 49, 54, 58, 63] = open pitches as MIDI, low E first; strings keep their letters),
  part_tuning ({part: tuning} when parts differ; one part's events carry its own tuning, and all
        parts together say "#tunings-differ" so a renderer can tune a guitar per part),
  shapes:   {"Am": "x 0 2 2 1 0", ...}   frets low E to high E, relative to the capo; x = not played
  patterns: {"name": {"beats": 4, "steps": [[beat, what, velocity], ...]}}
            what: D (strum all, low to high), d (down, top four), U (up, top four), u (up, top three),
                  B (bass: lowest played string), B2 (next played string up), 1-6 (strings, 1 = low E),
                  joined with + to pluck together, X (damp everything)
  sections: {"verse": {"pattern": "name", "chords": "Am:1 G:0.5 F:0.5", "repeat": 2, "tool": "finger"}}
            chord lengths are in bars
            or a tab section: {"tab": "0:D0+G0+B1 0.5:D2h 1.5:E3@0.6", "beats": 4, "label": "F", "velocity": 0.55}
            (optionally "bpm" for its own tempo)
            with "shape_damping": true (here or song-wide) a bar labelled with a known shape lifts
            fretted notes that don't fit it (for chordal picking; leave it off for lead lines)
            tokens are beat:notes; a note is string (E A D G B e, low to high) + fret (relative to
            the capo), then h = hammer-on, p = pull-off or s = slide there (not plucked: the ringing
            string changes fret) or x = dead note, b<n> = pick and bend up n semitones, r = release the bend,
            ~ or ~<depth> = vibrato, then optional @velocity; a leading / slides into the note from
            two frets below; + joins notes played together; a bare X damps everything (a rest);
            t continues a tied note without picking it; [B0:0;B0.5:2;V0.25:5.5:0.3;M0.7;D1.5] adds raw moves
            (bend points in beats from this note : semitones, vibrato, palm mute, written duration in beats)
  form:     ["intro", "verse", ...]   (names may be groups)
  groups:   {"riff1": ["riff1.1", "riff1.2"], ...}   named runs of sections (tools/tabimport.py makes them)
  parts:    {"rhythm": [form...], "lead": [form...]}   instead of form: players with their own hands, together
  ending:   {"chord": "Am", "what": "D", "velocity": 0.7, "ring": 4}

A note rings until its string is plucked again, everything is damped (X), or the
chord changes and the fretting finger leaves that string.
"""

import json
import random
import sys

import numpy as np

OPEN = [40, 45, 50, 55, 59, 64]


def shape(spec, name):
    raw = spec["shapes"][name].split()
    if len(raw) != 6:
        raise SystemExit(f"shape {name}: need 6 entries, got {raw}")
    return [None if f.lower() == "x" else int(f) for f in raw]


def chord_list(text):
    out = []
    for tok in text.split():
        name, _, bars = tok.partition(":")
        out.append((name, float(bars or 1)))
    return out


def strings_for(what, frets):
    played = [s for s in range(6) if frets[s] is not None]
    if not played:
        return [], "none"
    if what == "D":
        return played, "down"
    if what == "d":
        return [s for s in played if s >= 2], "down"
    if what == "U":
        return [s for s in reversed(played) if s >= 2], "up"
    if what == "u":
        return [s for s in reversed(played) if s >= 3], "up"
    out = []
    for part in what.split("+"):
        if part == "B":
            out.append(played[0])
        elif part == "B2":
            up = [s for s in played if s > played[0]]
            out.append(up[0] if up else played[0])
        elif part.isdigit() and 1 <= int(part) <= 6:
            if frets[int(part) - 1] is not None:
                out.append(int(part) - 1)
        else:
            raise SystemExit(f"unknown pattern step '{what}'")
    return out, "pinch"


def events(spec, form=None):
    rng = random.Random(spec.get("seed", 1))
    bpm = spec["bpm"]
    beat = 60.0 / bpm
    bpb = spec.get("beats_per_bar", 4)
    capo = spec.get("capo", 0)
    swing = spec.get("swing", 0.0)
    strum = spec.get("strum_ms", 12) / 1000.0
    ev = []  # [t, string, fret, vel, tool, end, distance, angle]
    ringing = {}  # string -> index into ev
    held = {}  # string -> fret the finger stops it at now (slides move it)
    spans = []  # (t0, t1, chord) for checking

    def stop(s, t):
        i = ringing.pop(s, None)
        if i is not None and ev[i][5] > t:
            ev[i][5] = max(ev[i][0] + 0.03, t)

    thumb = spec.get("thumb_weight", 1.0)

    def pluck(t, s, fret, vel, tool):
        stop(s, t)
        if tool == "finger" and thumb != 1.0:
            # Fingerstyle: the thumb takes the three bass strings and plays them heavier.
            vel *= thumb ** 0.5 if s <= 2 else thumb ** -0.5
        base_d = 0.13 if tool == "pick" else 0.15
        ev.append([t, s, fret, min(1.0, max(0.05, vel + rng.uniform(-0.04, 0.04))), tool, t + 6.0,
                   base_d + rng.uniform(-0.01, 0.01), 0.6 + rng.uniform(-0.12, 0.12), []])
        ringing[s] = len(ev) - 1
        held[s] = fret

    def tab_note(t, tok, vel, tool):
        """One tab note: [/]string fret [h|p|s] [b<semis>] [r] [~[depth]] [@velocity]."""
        import re
        m = re.fullmatch(r"(/?)([EADGBe])(\d+)([hpsxt]?)(?:b([0-9.]+))?(r?)(~[0-9.]*)?(\[[^\]]*\])?(?:@([0-9.]+))?", tok)
        if not m:
            raise SystemExit(f"bad tab note '{tok}' (e.g. D2, D2h, D7s, /D7, G7b2, G7r, G7~, G7~0.4, D2@0.7, G7[B0:0;B0.5:2], G7t[B0:2;B1:0])")
        slide_in, letter, fr, legato, bend, release, vib, raw, vel_s = m.groups()
        s = "EADGBe".index(letter)
        fret = capo + int(fr)
        v = float(vel_s) if vel_s else vel
        cur = ev[ringing[s]] if s in ringing else None

        def add_raw(e):
            """Raw moves in brackets, times in beats from this token: B<beats>:<semis>, V<beats>:<rate>:<depth>, M<mute>."""
            if not raw:
                return
            for part in raw[1:-1].split(";"):
                part = part.strip()
                if not part:
                    continue
                kind, nums = part[0], part[1:].split(":")
                if kind == "D":
                    # Written duration in beats: the note stops then (a tie sets the end of the whole tied note).
                    end = t + float(nums[0]) * beat + 0.02
                    e[5] = end if legato == "t" else min(e[5], end)
                elif kind == "M":
                    e[8].append(f"M{float(nums[0]):.3f}")
                elif kind in "BV":
                    at = (t - e[0]) + float(nums[0]) * beat
                    e[8].append(f"{kind}{max(at, 0):.4f}:" + ":".join(f"{float(v):.3f}" for v in nums[1:]))
                else:
                    raise SystemExit(f"'{tok}': unknown raw move '{part}'")

        if legato == "t":
            # Tied: the string keeps ringing as it was; only its moves (a held or released bend) apply.
            if cur is None:
                print(f"warning: '{tok}' at {t:.2f} s: tie with nothing ringing; picked instead", file=sys.stderr)
                pluck(t, s, fret, v, tool)
                add_raw(ev[ringing[s]])
            else:
                add_raw(cur)
            return
        if legato == "x":
            # Dead note: the string is struck while the fretting hand mutes it, a short thud.
            pluck(t, s, fret, v * 0.6, tool)
            ev[ringing[s]][5] = t + 0.035
            ringing.pop(s)
            return
        if legato == "s" or release:
            # Moves on the ringing note: no new pluck.
            if cur is None:
                # Nothing to move (a tab can slide out of a note we never heard): pick it instead.
                print(f"warning: '{tok}' at {t:.2f} s: nothing ringing on that string; picked instead", file=sys.stderr)
                if release:
                    return
                pluck(t, s, fret, v, tool)
                return
            rel = t - cur[0]
            if legato == "s":
                # The finger travels over the 90 ms before the beat (or half the time since the pluck).
                dur = min(0.09, rel / 2)
                cur[8].append(f"S{rel - dur:.4f}:{rel:.4f}:{fret - held[s]}")
                held[s] = fret
            if release:
                cur[8].append(f"B{max(rel - 0.12, 0):.4f}:{bend_now(cur, rel - 0.12):.3f}")
                cur[8].append(f"B{rel:.4f}:0")
            return
        if legato in ("h", "p"):
            # Legato: the string keeps sounding at the new fret, a little quieter than it was.
            prev = cur[3] if cur else v
            dist = cur[6] if cur else 0.14
            stop(s, t)
            ev.append([t, s, fret, min(1.0, prev * 0.8), "hammer" if legato == "h" else "pull", t + 6.0, dist, 0.6, []])
            ringing[s] = len(ev) - 1
            held[s] = fret
        elif slide_in:
            # Slide into the note from two frets below: plucked there, arriving on the beat.
            pluck(t - 0.07, s, max(capo, fret - 2), v, tool)
            ev[ringing[s]][8].append(f"S0.0:0.07:{fret - max(capo, fret - 2)}")
            held[s] = fret
        else:
            pluck(t, s, fret, v, tool)
        e = ev[ringing[s]]
        start = t - e[0]
        if bend:
            # Pick, then push the string up over 0.15 s.
            e[8] += [f"B{start + 0.04:.4f}:0", f"B{start + 0.19:.4f}:{float(bend):.3f}"]
        if vib:
            e[8].append(f"V{start + 0.25:.4f}:5.5:{float(vib[1:]) if len(vib) > 1 else 0.25:.3f}")
        add_raw(e)

    def bend_now(e, rel):
        """The bend (semitones) an event's moves hold at time `rel` after its pluck."""
        pts = sorted((float(a), float(b)) for a, b in (x[1:].split(":") for x in e[8] if x.startswith("B")))
        val = 0.0
        for a, b in pts:
            if a <= rel:
                val = b
        return val

    def step(t, what, vel, frets, tool):
        if what == "X":
            for s in list(ringing):
                stop(s, t)
            return
        strs, kind = strings_for(what, frets)
        gap = strum if kind == "down" else strum * 0.75 if kind == "up" else 0.0
        for k, s in enumerate(strs):
            v = vel * (1.0 - 0.035 * k) if kind != "pinch" else vel
            pluck(t + k * gap, s, capo + frets[s], v, tool)

    def expand(names):
        for n in names:
            if n in spec.get("groups", {}):
                yield from expand(spec["groups"][n])
            else:
                yield n

    t = 0.0
    for name in expand(form if form is not None else spec["form"]):
        sec = spec["sections"][name]
        tool = sec.get("tool", spec.get("tool", "pick"))
        if "tab" in sec:
            beats = sec.get("beats", bpb)
            # A section can carry its own tempo (a tempo change in the score).
            beat = 60.0 / sec.get("bpm", bpm)
            for _ in range(sec.get("repeat", 1)):
                label = sec.get("label", "tab")
                if sec.get("shape_damping", spec.get("shape_damping", False)) and label in spec.get("shapes", {}):
                    # The fretting hand moves to this bar's shape: fretted notes that don't fit it stop
                    # (strings at the capo aren't held, so they ring on).
                    frets = shape(spec, label)
                    for s in list(ringing):
                        now = held.get(s, ev[ringing[s]][2])
                        if now != capo and (frets[s] is None or capo + frets[s] != now):
                            stop(s, t + 0.02)
                spans.append((t, t + beats * beat, label))
                for tok in sec["tab"].split():
                    at, _, notes = tok.partition(":")
                    pos = float(at)
                    if abs(pos % 1 - 0.5) < 1e-9:
                        pos += swing / 6.0
                    tt = t + pos * beat + rng.gauss(0, 0.004)
                    for n in notes.split("+"):
                        if n == "X":
                            # A rest in the notation: the hand damps whatever is ringing.
                            for st in list(ringing):
                                stop(st, tt)
                            continue
                        tab_note(tt, n, sec.get("velocity", 0.55) + rng.uniform(-0.04, 0.04), tool)
                t += beats * beat
            beat = 60.0 / bpm
            continue
        pat = spec["patterns"][sec["pattern"]]
        for _ in range(sec.get("repeat", 1)):
            sec_start = t
            for chord, bars in chord_list(sec["chords"]):
                frets = shape(spec, chord)
                c0, c1 = t, t + bars * bpb * beat
                # Chord change: fingers leave strings whose fret changes or that go silent.
                for s in list(ringing):
                    if frets[s] is None or capo + frets[s] != held.get(s, ev[ringing[s]][2]):
                        stop(s, c0 + 0.02)
                spans.append((c0, c1, chord))
                # Pattern steps at section-relative positions, looping every pattern length.
                n0 = int(np.floor((c0 - sec_start) / beat / pat["beats"] + 1e-9))
                n1 = int(np.ceil((c1 - sec_start) / beat / pat["beats"] - 1e-9))
                for n in range(n0, n1):
                    for b, what, vel in pat["steps"]:
                        pos = n * pat["beats"] + b
                        if abs(pos % 1 - 0.5) < 1e-9:
                            pos += swing / 6.0
                        tt = sec_start + pos * beat
                        if c0 - 1e-9 <= tt < c1 - 1e-9:
                            step(tt + rng.gauss(0, 0.004), what, vel, frets, tool)
                t = c1
    end = spec.get("ending")
    if end:
        frets = shape(spec, end["chord"])
        for s in list(ringing):
            stop(s, t)
        spans.append((t, t + end.get("ring", 4), end["chord"]))
        step(t, end.get("what", "D"), end.get("velocity", 0.7), frets, spec.get("tool", "pick"))
        for s, i in ringing.items():
            ev[i][5] = min(ev[i][5], t + end.get("ring", 4))
    for e in ev:
        e[5] = min(e[5], e[0] + 6.0)
    ev.sort(key=lambda e: e[0])
    return ev, spans


def all_events(spec, only=None):
    """Every player's plucks merged (or just part `only`); chord spans from the first part."""
    if "parts" not in spec:
        return events(spec)
    if only is not None and only not in spec["parts"]:
        raise SystemExit(f"no part '{only}' (parts: {', '.join(spec['parts'])})")
    ev, spans = [], None
    for i, (name, form) in enumerate(spec["parts"].items()):
        e, sp = events({**spec, "seed": spec.get("seed", 1) + 101 * i}, form)
        if only is not None and name != only:
            # Chord spans still come from the first part, so the display keeps its chords.
            spans = spans or sp
            continue
        ev += e
        spans = spans or sp
    ev.sort(key=lambda e: e[0])
    return ev, spans


def write_events(spec, path, only=None):
    """Plucks as `guitar perform` reads them; path "-" writes to stdout (the workbench uses that)."""
    ev, spans = all_events(spec, only)
    lines = [f"# {spec.get('title', '')}: {len(ev)} plucks, {spans[-1][1]:.1f} s"]
    if "parts" in spec:
        lines.append("#parts, " + ", ".join(spec["parts"]))
    tuning = spec.get("part_tuning", {}).get(only, spec.get("tuning")) if only else spec.get("tuning")
    if tuning:
        # Open-string pitches (MIDI, low E first) for the renderer.
        lines.append("#tuning, " + ", ".join(f"{m:g}" for m in tuning))
    if only is None and len({tuple(t) for t in spec.get("part_tuning", {}).values()}) > 1:
        lines.append("#tunings-differ")
    # Chord spans as comments (the renderer skips them; the workbench shows them).
    lines += [f"#span, {t0:.4f}, {t1:.4f}, {name}" for t0, t1, name in spans]
    for e in ev:
        moves = ";".join(e[8]) if len(e) > 8 and e[8] else ""
        lines.append(f"{max(0.0, e[0]):.4f}, {e[1]}, {e[2]}, {e[3]:.3f}, {e[4]}, {e[5]:.4f}, {e[6]:.4f}, {e[7]:.3f}" + (f", {moves}" if moves else ""))
    text = "\n".join(lines) + "\n"
    if path == "-":
        sys.stdout.write(text)
        return
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)
    print(f"{len(ev)} plucks, {spans[-1][1]:.1f} s -> {path}")


def check(spec, wav):
    """Per chord span: is the strongest chord template the one that should be sounding?"""
    from scipy.io import wavfile
    rate, x = wavfile.read(wav)
    x = x.astype(np.float64)
    if x.ndim > 1:
        x = x.mean(axis=1)
    capo = spec.get("capo", 0)
    templates = {}
    for name in spec["shapes"]:
        fr = shape(spec, name)
        templates[name] = {(OPEN[s] + capo + f) % 12 for s, f in enumerate(fr) if f is not None}
    _, spans = all_events(spec)
    ok = n = 0
    misses = []
    for t0, t1, chord in spans:
        if chord not in templates:
            continue
        a, b = int((t0 + 0.08) * rate), int(min(t1 - 0.02, t0 + 2.0) * rate)
        if b - a < 2048:
            continue
        seg = x[a:b]
        if len(seg) < 2048:
            # The render ends before this chord: a failure, not a skip.
            n += 1
            misses.append(f"{t0:.1f}s {chord}->(no audio)")
            continue
        sp = np.abs(np.fft.rfft(seg * np.hanning(len(seg)))) ** 2
        f = np.fft.rfftfreq(len(seg), 1 / rate)
        sel = (f > 70) & (f < 1200)
        chroma = np.zeros(12)
        np.add.at(chroma, np.round(69 + 12 * np.log2(f[sel] / 440)).astype(int) % 12, sp[sel])
        # Score: share of energy on the chord's pitch classes, normalised by how many it has.
        score = {k: sum(chroma[p] for p in pcs) / chroma.sum() / np.sqrt(len(pcs)) for k, pcs in templates.items()}
        best = max(score, key=score.get)
        same = templates[best] == templates[chord]
        n += 1
        ok += same
        if not same:
            misses.append(f"{t0:.1f}s {chord}->{best}")
    print(f"{ok}/{n} chord spans sound as charted" + (f"; misses: {', '.join(misses[:8])}" if misses else ""))
    return ok, n


if __name__ == "__main__":
    args = sys.argv[1:]
    only = None
    if "--part" in args:
        i = args.index("--part")
        only = args[i + 1]
        del args[i:i + 2]
    if len(args) != 3 or args[0] not in ("events", "check"):
        print(__doc__)
        sys.exit(2)
    if args[1] == "-":
        spec = json.load(sys.stdin)  # e.g. piped from tools/gpimport.py
    else:
        with open(args[1], encoding="utf-8") as f:
            spec = json.load(f)
    if args[0] == "events":
        write_events(spec, args[2], only)
    else:
        check(spec, args[2])
