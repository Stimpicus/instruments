"""Read a Guitar Pro file into an arranger song.

    python -I tools/gpimport.py <song.gp|.gpx|.gp3|.gp4|.gp5> <song.json | -> [--tracks 2,3,4] [--list]

.gp (Guitar Pro 7/8) and .gpx (Guitar Pro 6) are read here directly; .gp3/.gp4/.gp5 (the older
binary formats) are read with PyGuitarPro (pip install pyguitarpro), then converted the same way.
(.gtp, Guitar Pro 1/2, is not supported: no reader for it exists here.)

Guitar tracks are chosen by their General MIDI instrument (programs 24-31, not on the percussion
channel), since files label voice and keyboard tracks "guitar" too. Parts may have different
tunings: tracks that are another track's tuning raised evenly play it with a capo; otherwise each
part keeps its own tuning ("part_tuning"), and the workbench renders such songs part by part.

Every guitar track becomes a part (pick one in the workbench, or play them together). Each
bar is a tab section with the file's exact rhythm, tempo map, section names and tuning. Notes
carry their written durations, ties (held, not re-picked), hammer-ons and pull-offs, slides
(legato and shift slides, into a note, out of a note), bends with their full curves, vibrato,
palm muting, dead notes, accents and ghost notes, dynamics, grace notes and strummed chords.

Units, from alphaTab's GPIF reader (open source, checked 2026-10-27): bend values are 25 per
quarter tone (so 50 per semitone), bend offsets are percent of the note; slide flags are bits:
1 shift, 2 legato, 4 out down, 8 out up, 16 in from below, 32 in from above.
"""

import json
import sys
import zipfile
import xml.etree.ElementTree as ET

LETTERS = "EADGBe"
NOTE_BEATS = {"Whole": 4.0, "Half": 2.0, "Quarter": 1.0, "Eighth": 0.5, "16th": 0.25, "32nd": 0.125, "64th": 0.0625, "128th": 0.03125}
DYNAMIC = {"PPP": 0.25, "PP": 0.32, "P": 0.4, "MP": 0.48, "MF": 0.56, "F": 0.65, "FF": 0.76, "FFF": 0.88}


def prop(el, name):
    """A <Property name=...> child of el's <Properties>, or None."""
    for p in el.iter("Property"):
        if p.get("name") == name:
            return p
    return None


def gpx_files(data):
    """Files inside a Guitar Pro 6 container (.gpx): {name: bytes}.

    Format as alphaTab's GpxFileSystem reads it (open source, MPL-2.0, read 2026-10-27): "BCFZ"
    + int32 length + a bit stream (MSB first) of literal runs and back-references, giving a
    "BCFS" filesystem of 0x1000-byte sectors; a sector whose first int is 2 is a file entry
    (name at +4, size at +0x8C, data sector numbers from +0x94 until 0)."""
    head, data = data[:4], data[4:]
    if head == b"BCFZ":
        expected = int.from_bytes(data[:4], "little")
        src, pos, out = data[4:], 0, bytearray()

        def bits(n, reverse=False):
            nonlocal pos
            v = 0
            for i in range(n):
                byte = pos >> 3
                if byte >= len(src):
                    raise EOFError
                bit = (src[byte] >> (7 - (pos & 7))) & 1
                pos += 1
                v |= bit << (i if reverse else n - 1 - i)
            return v

        try:
            while len(out) < expected:
                if bits(1):
                    word = bits(4)
                    offset, size = bits(word, True), bits(word, True)
                    start = len(out) - offset
                    out += out[start:start + min(offset, size)]
                else:
                    for _ in range(bits(2, True)):
                        out.append(bits(8))
        except EOFError:
            pass
        if out[:4] != b"BCFS":
            raise ValueError("decompressed data is not a BCFS filesystem")
        fs = bytes(out[4:])
    elif head == b"BCFS":
        fs = data
    else:
        raise ValueError(f"unknown header {head!r}")
    sector, files, offset = 0x1000, {}, 0x1000
    integer = lambda at: int.from_bytes(fs[at:at + 4], "little")
    while offset + 3 < len(fs):
        if integer(offset) == 2:
            name = fs[offset + 4:offset + 4 + 127].split(b"\0")[0].decode("latin-1")
            size = integer(offset + 0x8C)
            body, k, pointers = bytearray(), 0, offset + 0x94
            while True:
                n = integer(pointers + 4 * k)
                k += 1
                if n == 0:
                    break
                body += fs[n * sector:(n + 1) * sector]
                offset = n * sector
            files[name] = bytes(body[:size])
        offset += sector
    return files


def load(path):
    """The score XML inside a Guitar Pro 6 (.gpx) or 7/8 (.gp) file, or a plain error for anything else."""
    if path.lower().endswith(".gpx"):
        try:
            with open(path, "rb") as f:
                return ET.fromstring(gpx_files(f.read())["score.gpif"])
        except (OSError, ValueError, KeyError, ET.ParseError) as e:
            raise SystemExit(f"{path}: not a readable Guitar Pro 6 file (.gpx): {e}")
    try:
        with zipfile.ZipFile(path) as z:
            return ET.fromstring(z.read("Content/score.gpif"))
    except (zipfile.BadZipFile, KeyError, ET.ParseError, OSError) as e:
        raise SystemExit(f"{path}: not a readable Guitar Pro 7/8 file (.gp, a zip holding Content/score.gpif): {e}")


def tracks_of(root):
    out = []
    for t in root.find("Tracks"):
        inst = t.find("InstrumentSet")
        kind = inst.findtext("Type") if inst is not None else ""
        pitches = None
        p = prop(t, "Tuning")
        if p is not None and p.findtext("Pitches"):
            pitches = [int(x) for x in p.findtext("Pitches").split()]
        # Guitar Pro 6 has no InstrumentSet: the General MIDI program says what the track is.
        gm = t.find("GeneralMidi")
        if not kind and gm is not None:
            program = int(gm.findtext("Program") or -1)
            kind = "guitar" if gm.get("table") != "Percussion" and 24 <= program <= 31 else f"midi {program}"
        out.append({"index": int(t.get("id")), "name": t.findtext("Name") or f"track {t.get('id')}", "kind": kind, "tuning": pitches})
    return out


def beats_of_rhythm(r):
    d = NOTE_BEATS[r.findtext("NoteValue")]
    dot = r.find("AugmentationDot")
    if dot is not None:
        n = int(dot.get("count", "1"))
        d *= 2 - 0.5 ** n
    tup = r.find("PrimaryTuplet")
    if tup is not None:
        d *= int(tup.get("den")) / int(tup.get("num"))
    return d


def convert(root, track_ids):
    bars = {b.get("id"): b for b in root.find("Bars")}
    voices = {v.get("id"): v for v in root.find("Voices")}
    beats = {b.get("id"): b for b in root.find("Beats")}
    notes = {n.get("id"): n for n in root.find("Notes")}
    rhythms = {r.get("id"): r for r in root.find("Rhythms")}
    masters = list(root.find("MasterBars"))
    tracks = {t["index"]: t for t in tracks_of(root)}

    for i in track_ids:
        if not tracks[i]["tuning"] or len(tracks[i]["tuning"]) != 6:
            raise SystemExit(f"track {i} ({tracks[i]['name']}) is not a six-string guitar: tuning {tracks[i]['tuning']}")
    tunings = [tracks[i]["tuning"] for i in track_ids]
    tuning = tunings[0]

    # String numbering: check against each note's own MIDI pitch rather than assume.
    hits = {"as is": 0, "reversed": 0}
    pitched = 0
    for n in list(notes.values())[:400]:
        s, f, m = prop(n, "String"), prop(n, "Fret"), prop(n, "Midi")
        if s is None or f is None or m is None:
            continue
        pitched += 1
        si, fr, mi = int(s.findtext("String")), int(f.findtext("Fret")), int(m.findtext("Number"))
        if si < 6:
            hits["as is"] += any(tu[si] + fr == mi for tu in tunings)
            hits["reversed"] += any(tu[5 - si] + fr == mi for tu in tunings)
    if pitched == 0:
        # Guitar Pro 6 notes carry no pitch to check against. GPIF string 0 is the lowest string
        # (alphaTab's GPIF parser and model, read 2026-10-27; and the pitch check on our GP7 files).
        reverse = False
        report_note = "string order from the GPIF convention (no pitches in the file to check)"
    elif max(hits.values()) == 0:
        raise SystemExit(f"cannot tell which string is which (pitch check {hits})")
    else:
        reverse = hits["reversed"] > hits["as is"]
        report_note = f"string order checked against {pitched} note pitches"

    # Tempo map (bar -> bpm) and section names.
    tempo = {}
    for a in root.find("MasterTrack").iter("Automation"):
        if a.findtext("Type") == "Tempo":
            tempo[int(a.findtext("Bar"))] = float(a.findtext("Value").split()[0])
    bpm_at, label_at, bpm, label = [], [], tempo.get(0, 120.0), "Intro"
    for i, mb in enumerate(masters):
        bpm = tempo.get(i, bpm)
        sec = mb.find("Section")
        if sec is not None and (sec.findtext("Text") or "").strip():
            label = sec.findtext("Text").strip()
        bpm_at.append(bpm)
        label_at.append(label)

    song_bpm = bpm_at[0]
    sections, parts, report = {}, {}, {"notes": 0, "bends": 0, "legato": 0, "slides": 0, "muted": 0, "palm": 0, "ties": 0, "grace": 0, "strums": 0, "let_ring": 0, "strings": report_note}
    for ti in track_ids:
        part = []
        last_fret = {}      # string -> fret last played (for hammer vs pull)
        slide_next = set()  # strings whose last note slides into the next one
        for bi, mb in enumerate(masters):
            num, den = (int(x) for x in mb.findtext("Time").split("/"))
            bar_beats = num * 4.0 / den
            bar = bars[mb.findtext("Bars").split()[ti]]
            tokens = {}
            for vid in bar.findtext("Voices").split():
                if vid == "-1":
                    continue
                pos = 0.0
                pending_grace = []
                for bid in (voices[vid].findtext("Beats") or "").split():
                    beat = beats[bid]
                    dur = beats_of_rhythm(rhythms[beat.find("Rhythm").get("ref")])
                    grace = beat.find("GraceNotes") is not None
                    vel = DYNAMIC.get(beat.findtext("Dynamic") or "MF", 0.56)
                    nids = (beat.findtext("Notes") or "").split()
                    if not nids and not grace:
                        tokens.setdefault(round(pos, 4), []).append("X")  # a rest: the hand stops the strings
                    entries = []
                    for nid in nids:
                        n = notes[nid]
                        si = int(prop(n, "String").findtext("String"))
                        s = 5 - si if reverse else si
                        fret = int(prop(n, "Fret").findtext("Fret"))
                        letter = LETTERS[s]
                        tie = n.find("Tie")
                        tie_dest = tie is not None and tie.get("destination") == "true"
                        # "Let ring": no written stop; it rings until the string is played again or a rest.
                        moves, v = ([] if n.find("LetRing") is not None else [f"D{dur:g}"]), vel
                        report["let_ring"] += n.find("LetRing") is not None
                        if n.find("Accent") is not None:
                            v += 0.12
                        if n.find("AntiAccent") is not None:
                            v *= 0.55
                        if prop(n, "Muted") is not None:
                            entries.append((s, f"{letter}{fret}x@{v:.2f}"))
                            report["muted"] += 1
                            last_fret[s] = fret
                            continue
                        if prop(n, "PalmMuted") is not None:
                            moves.append("M0.7")
                            report["palm"] += 1
                        vib = n.findtext("Vibrato")
                        if vib:
                            moves.append(f"V{min(0.25, dur / 3):g}:5.5:{0.55 if vib == 'Wide' else 0.25}")
                        if prop(n, "Bended") is not None:
                            def val(name, default=None):
                                p = prop(n, name)
                                return float(p.findtext("Float")) if p is not None else default
                            pts = [(val("BendOriginOffset", 0.0), val("BendOriginValue", 0.0))]
                            mv = val("BendMiddleValue")
                            if mv is not None:
                                for name in ("BendMiddleOffset1", "BendMiddleOffset2"):
                                    o = val(name)
                                    if o is not None:
                                        pts.append((o, mv))
                            pts.append((val("BendDestinationOffset", 100.0), val("BendDestinationValue", 0.0)))
                            pts = sorted(set(pts))
                            moves += [f"B{o / 100 * dur:.4g}:{v_ / 50:.3g}" for o, v_ in pts]
                            report["bends"] += 1
                        slide = prop(n, "Slide")
                        flags = int(slide.findtext("Flags")) if slide is not None else 0
                        if flags & (4 | 8):
                            # Slide off the note: a quick glide away in its last quarter.
                            moves += [f"B{0.75 * dur:.4g}:0", f"B{dur:.4g}:{-3 if flags & 4 else 3}"]
                        if tie_dest:
                            mark = "t"
                            report["ties"] += 1
                        elif s in slide_next:
                            mark = "s"
                            report["slides"] += 1
                        elif prop(n, "HopoDestination") is not None and s in last_fret:
                            mark = "h" if fret > last_fret[s] else "p"
                            report["legato"] += 1
                        else:
                            mark = ""
                        prefix = "/" if flags & (16 | 32) and not mark else ""
                        slide_next.discard(s)
                        if flags & (1 | 2):
                            slide_next.add(s)
                        last_fret[s] = fret
                        entries.append((s, f"{prefix}{letter}{fret}{mark}[{';'.join(moves)}]@{min(1.0, v):.2f}"))
                        report["notes"] += 1
                    if grace:
                        pending_grace += entries
                        report["grace"] += len(entries)
                        continue
                    # A strum (GP "brush") spreads the chord: down = low string first.
                    brush = prop(beat, "Brush")
                    plucked = [e for e in entries if "t[" not in e[1]]
                    if brush is not None and len(plucked) > 1:
                        up = (brush.findtext("Direction") or "Down") == "Up"
                        order = sorted(plucked, key=lambda e: e[0], reverse=up)
                        step = 0.012 * bpm_at[bi] / 60  # 12 ms between strings, in beats
                        for k, (_, tok) in enumerate(order):
                            tokens.setdefault(round(pos + k * step, 4), []).append(tok)
                        entries = [e for e in entries if "t[" in e[1]]
                        report["strums"] += 1
                    if pending_grace:
                        at = max(0.0, pos - 0.06 * bpm_at[bi] / 60)  # grace notes 60 ms before their beat
                        for _, tok in pending_grace:
                            tokens.setdefault(round(at, 4), []).append(tok.split("[")[0] + "[D0.05]@0.5")
                        pending_grace = []
                    for _, tok in entries:
                        tokens.setdefault(round(pos, 4), []).append(tok)
                    pos += dur
            name = f"t{ti}.{bi + 1}"
            sec = {"tab": " ".join(f"{p:g}:" + "+".join(ts) for p, ts in sorted(tokens.items()) if p < bar_beats),
                   "beats": bar_beats, "label": label_at[bi]}
            if abs(bpm_at[bi] - song_bpm) > 1e-6:
                sec["bpm"] = bpm_at[bi]
            sections[name] = sec
            part.append(name)
        parts[tracks[ti]["name"]] = part
    score = root.find("Score")
    title = f"{score.findtext('Title') or '?'} ({score.findtext('Artist') or '?'}), from a Guitar Pro file"
    song = {"title": title, "note": "PRIVATE PRACTICE ARRANGEMENT, NOT FOR DISTRIBUTION. Imported by tools/gpimport.py.",
            "bpm": song_bpm, "swing": 0.0, "capo": 0, "tool": "pick", "seed": 1, "tuning": tuning,
            "shapes": {}, "patterns": {}, "sections": sections, "parts": parts}
    if len({tuple(t) for t in tunings}) > 1:
        song["part_tuning"] = {tracks[i]["name"]: tracks[i]["tuning"] for i in track_ids}
    return song, report, reverse


# ---------- Guitar Pro 3/4/5, through PyGuitarPro ----------

def legacy_tracks(song):
    out = []
    for i, t in enumerate(song.tracks):
        tuning = [st.value for st in sorted(t.strings, key=lambda x: -x.number)]  # string 1 is the highest
        # Every track is stored as a "guitar"; its General MIDI program says what it really is.
        program = t.channel.instrument
        kind = "percussion" if t.isPercussionTrack else "guitar" if 24 <= program <= 31 and len(t.strings) >= 6 else f"midi {program}"
        out.append({"index": i, "name": t.name, "kind": kind, "tuning": tuning})
    return out


def play_order(headers):
    """Measure indices in playing order, with repeats and alternate endings expanded."""
    order, i, opened, passes = [], 0, 0, {}
    while i < len(headers):
        h = headers[i]
        if h.isRepeatOpen:
            opened = i
        alt = h.repeatAlternative
        n = passes.get(opened, 0) + 1  # which pass through the current repeat
        if alt and not (alt & (1 << (n - 1)) or alt == n):
            i += 1
            continue
        order.append(i)
        if h.repeatClose > 0 and passes.get(opened, 0) < h.repeatClose:
            passes[opened] = passes.get(opened, 0) + 1
            i = opened
            continue
        i += 1
    return order


def convert_legacy(song, track_ids):
    import guitarpro as gp
    tracks = legacy_tracks(song)
    # A track whose tuning is another's raised evenly by k semitones plays that tuning with a capo at k;
    # otherwise it keeps its own tuning (a part tuning).
    capo, tuned, bases = {}, {}, []
    for i in sorted(track_ids, key=lambda i: tracks[i]["tuning"][0]):
        t = tracks[i]["tuning"]
        if len(t) != 6:
            raise SystemExit(f"track {i} ({tracks[i]['name']}) is not a six-string guitar: tuning {t}")
        for b in bases:
            d = [x - y for x, y in zip(t, b)]
            if len(set(d)) == 1 and d[0] >= 0:
                capo[i], tuned[i] = d[0], b
                break
        else:
            bases.append(t)
            capo[i], tuned[i] = 0, t
    base = tuned[track_ids[0]]
    headers = song.measureHeaders
    order = play_order(headers)
    labels, label = [], "Intro"
    for h in headers:
        if h.marker and h.marker.title.strip():
            label = h.marker.title.strip()
        labels.append(label)
    tempo_at = {}  # measure index -> bpm, from mix-table tempo changes in any track
    for t in song.tracks:
        for mi, m in enumerate(t.measures):
            for v in m.voices:
                for b in v.beats:
                    mt = b.effect.mixTableChange if b.effect else None
                    if mt and mt.tempo and mt.tempo.value > 0:
                        tempo_at.setdefault(mi, float(mt.tempo.value))
    song_bpm, bpm_of, bpm = float(song.tempo), [], float(song.tempo)
    for mi in range(len(headers)):
        bpm = tempo_at.get(mi, bpm)
        bpm_of.append(bpm)
    sections, parts = {}, {}
    report = {"notes": 0, "legato": 0, "slides": 0, "ties": 0, "muted": 0, "palm": 0, "bends": 0, "grace": 0, "strums": 0, "capo": {tracks[i]["name"]: capo[i] for i in track_ids if capo[i]}}
    for ti in track_ids:
        t = song.tracks[ti]
        nstr = len(t.strings)
        part, legato_from, slide_from, last_fret = [], set(), set(), {}
        prev = {}  # string -> (fret, start, length) of its last note, in absolute beats
        for k, mi in enumerate(order):
            m = t.measures[mi]
            h = headers[mi]
            bar_start = sum(headers[o].timeSignature.numerator * 4.0 / headers[o].timeSignature.denominator.value for o in order[:k])
            bar_beats = h.timeSignature.numerator * 4.0 / h.timeSignature.denominator.value
            tokens = {}
            for v in m.voices:
                pos = 0.0
                for b in v.beats:
                    dur = float(b.duration.time) / gp.Duration.quarterTime
                    if b.status == gp.BeatStatus.empty:
                        pos += dur
                        continue
                    if b.status == gp.BeatStatus.rest or not b.notes:
                        tokens.setdefault(round(pos, 4), []).append("X")
                        pos += dur
                        continue
                    entries, graces = [], []
                    for n in b.notes:
                        s = nstr - n.string  # string 1 is the highest
                        letter = LETTERS[s]
                        fret = n.value + capo[ti]
                        e = n.effect
                        vel = 0.2 + 0.6 * (min(max(n.velocity, 15), 127) - 15) / 112
                        if e.accentuatedNote:
                            vel += 0.12
                        if e.ghostNote:
                            vel *= 0.55
                        if n.type == gp.NoteType.dead:
                            entries.append((s, f"{letter}{fret}x@{vel:.2f}"))
                            report["muted"] += 1
                            continue
                        moves = [] if e.letRing else [f"D{dur * (0.5 if e.staccato else 1.0):g}"]
                        if e.palmMute:
                            moves.append("M0.7")
                            report["palm"] += 1
                        if e.vibrato:
                            moves.append(f"V{min(0.25, dur / 3):g}:5.5:0.25")
                        if e.bend:
                            moves += [f"B{pt.position / gp.BendEffect.maxPosition * dur:.4g}:{pt.value / 2:.3g}" for pt in e.bend.points]
                            report["bends"] += 1
                        if any(x in (gp.SlideType.outDownwards, gp.SlideType.outUpwards) for x in e.slides):
                            down = gp.SlideType.outDownwards in e.slides
                            moves += [f"B{0.75 * dur:.4g}:0", f"B{dur:.4g}:{-3 if down else 3}"]
                        # Interpretation: some transcribers write a rolled chord as 64th-note beats that repeat
                        # the earlier notes; a repeat straight after such a short note is a continuation.
                        before = prev.get(s)
                        rolled = (n.type == gp.NoteType.normal and before is not None and before[0] == fret
                                  and before[2] <= 0.0625 + 1e-9 and abs(before[1] + before[2] - (bar_start + pos)) < 1e-6)
                        prev[s] = (fret, bar_start + pos, dur)
                        if n.type == gp.NoteType.tie or rolled:
                            mark = "t"
                            report["ties"] += 1
                            report["rolled"] = report.get("rolled", 0) + rolled
                        elif s in slide_from:
                            mark = "s"
                            report["slides"] += 1
                        elif s in legato_from and s in last_fret:
                            mark = "h" if fret > last_fret[s] else "p"
                            report["legato"] += 1
                        else:
                            mark = ""
                        into = any(x in (gp.SlideType.intoFromBelow, gp.SlideType.intoFromAbove) for x in e.slides)
                        prefix = "/" if into and not mark else ""
                        legato_from.discard(s)
                        slide_from.discard(s)
                        if e.hammer:  # GP3-5: the flag sits on the note the hammer-on or pull-off starts from
                            legato_from.add(s)
                        if any(x in (gp.SlideType.shiftSlideTo, gp.SlideType.legatoSlideTo) for x in e.slides):
                            slide_from.add(s)
                        if n.type != gp.NoteType.tie:
                            last_fret[s] = fret
                        if e.grace and n.type != gp.NoteType.tie:
                            graces.append(f"{letter}{e.grace.fret + capo[ti]}[D0.05]@0.5")
                            report["grace"] += 1
                        entries.append((s, f"{prefix}{letter}{fret}{mark}[{';'.join(moves)}]@{min(1.0, vel):.2f}"))
                        report["notes"] += 1
                    if graces:
                        at = max(0.0, pos - 0.06 * bpm_of[mi] / 60)
                        for g in graces:
                            tokens.setdefault(round(at, 4), []).append(g)
                    stroke = b.effect.stroke if b.effect else None
                    plucked = [x for x in entries if "t[" not in x[1]]
                    if stroke and stroke.direction != gp.BeatStrokeDirection.none and len(plucked) > 1:
                        up = stroke.direction == gp.BeatStrokeDirection.up
                        step = 0.012 * bpm_of[mi] / 60
                        for j, (_, tok) in enumerate(sorted(plucked, key=lambda x: x[0], reverse=up)):
                            tokens.setdefault(round(pos + j * step, 4), []).append(tok)
                        entries = [x for x in entries if "t[" in x[1]]
                        report["strums"] += 1
                    for _, tok in entries:
                        tokens.setdefault(round(pos, 4), []).append(tok)
                    pos += dur
            name = f"t{ti}.{k + 1}"
            sec = {"tab": " ".join(f"{p_:g}:" + "+".join(ts) for p_, ts in sorted(tokens.items()) if p_ < bar_beats),
                   "beats": bar_beats, "label": labels[mi]}
            if abs(bpm_of[mi] - song_bpm) > 1e-6:
                sec["bpm"] = bpm_of[mi]
            sections[name] = sec
            part.append(name)
        parts[t.name] = part
    title = f"{song.title or '?'}{' (' + song.artist + ')' if song.artist else ''}, from a Guitar Pro file"
    out = {"title": title, "note": "PRIVATE PRACTICE ARRANGEMENT, NOT FOR DISTRIBUTION. Imported by tools/gpimport.py.",
           "bpm": song_bpm, "swing": 0.0, "capo": 0, "tool": "pick", "seed": 1, "tuning": base,
           "shapes": {}, "patterns": {}, "sections": sections, "parts": parts}
    if len(bases) > 1:
        out["part_tuning"] = {song.tracks[i].name: tuned[i] for i in track_ids}
    return out, report


def legacy(path):
    try:
        import guitarpro
    except ImportError:
        raise SystemExit("reading .gp3/.gp4/.gp5 needs PyGuitarPro: python -m pip install pyguitarpro")
    try:
        return guitarpro.parse(path)
    except Exception as e:  # the binary reader raises many kinds of error on a damaged file
        raise SystemExit(f"{path}: not a readable Guitar Pro 3/4/5 file: {e}")


if __name__ == "__main__":
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        sys.exit(2)
    old_format = args[0].lower().endswith((".gp3", ".gp4", ".gp5"))
    if old_format:
        parsed = legacy(args[0])
        listing = legacy_tracks(parsed)
    else:
        root = load(args[0])
        listing = tracks_of(root)
    if "--list" in args:
        for t in listing:
            print(f"{t['index']}: {t['name']} ({t['kind']}), tuning {t['tuning']}")
        sys.exit(0)
    if "--tracks" in args:
        ids = [int(x) for x in args[args.index("--tracks") + 1].split(",")]
    else:
        ids = [t["index"] for t in listing if "guitar" in t["kind"].lower() and t["tuning"] and len(t["tuning"]) == 6]
    if old_format:
        song, report = convert_legacy(parsed, ids)
        reverse = False
    else:
        song, report, reverse = convert(root, ids)
    if args[1] == "-":
        # To stdout, for piping into tools/arrange.py (the workbench does this).
        json.dump(song, sys.stdout)
        sys.exit(0)
    with open(args[1], "w", encoding="utf-8") as f:
        json.dump(song, f, indent=1)
    print(f"{len(song['parts'])} part(s): {', '.join(song['parts'])}; {len(next(iter(song['parts'].values())))} bars; "
          f"strings numbered {'high to low' if reverse else 'low to high'}; {report}")
