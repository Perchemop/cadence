#!/usr/bin/env python3
"""Neutralise Phosphor Icons' legacy ASCII placeholder mappings.

Cadence registers the Phosphor face first in the proportional font family so
the Private Use Area icons are not shadowed by Inter's private-use mappings
(PROGRESS.md, 2026-11-13 icon-shadowing fix). The upstream face also maps a
handful of ASCII characters to *blank* placeholder glyphs for compatibility
with legacy icon-font tooling:

    U+0000, U+0001, U+0020 (space), U+002D (hyphen-minus), U+0061..U+007A

egui resolves a character against the family list in order and keeps the
first face whose character map contains it, so with Phosphor first every
lowercase letter, hyphen and space resolved to a blank Phosphor glyph and
vanished, while capitals and digits fell through to Inter. That was the
rendered-evidence defect recorded on 2026-11-22.

This script rewrites the cmap so those characters map to .notdef (glyph 0).
egui treats a .notdef mapping as "this face has no such glyph" and continues
down the family list, so Latin text resolves to Inter while the icon face
keeps every Private Use Area glyph it had.

Only the cmap table is modified; every other table is copied byte for byte
and verified afterwards. The transformation is deterministic.

Usage:
    python3 scripts/patch-phosphor.py <source.ttf> <output.ttf>

The source is the font shipped by the egui-phosphor crate (MIT licence).
The output is vendored at crates/ui-egui/assets/fonts/Phosphor.ttf.
"""

from __future__ import annotations

import struct
import sys
from pathlib import Path

# Characters whose upstream placeholder mapping is neutralised. The face
# keeps every other character, including the whole Private Use Area.
NEUTRALISE = {0x0000, 0x0001, 0x0020, 0x002D} | set(range(0x0061, 0x007B))

HEADER = 12
RECORD = 16


class FontError(Exception):
    """Raised when the input font is not the shape this script expects."""


def read_tables(data):
    """Return the table directory as (tag, payload) pairs, in file order."""
    if len(data) < HEADER + RECORD:
        raise FontError("file is too short to be an sfnt font")
    count = struct.unpack(">H", data[4:6])[0]
    tables = []
    for index in range(count):
        start = HEADER + RECORD * index
        tag, _checksum, offset, length = struct.unpack(">4sIII", data[start:start + 16])
        tables.append((tag, data[offset:offset + length]))
    return tables


def parse_format4(subtable):
    """Parse a cmap format 4 subtable into its segments."""
    fmt, length, _lang, seg_x2, search, selector, shift = struct.unpack(">HHHHHHH", subtable[:14])
    if fmt != 4:
        raise FontError("expected cmap format 4, found %d" % fmt)
    count = seg_x2 // 2
    ends = struct.unpack(">%dH" % count, subtable[14:14 + seg_x2])
    starts = struct.unpack(">%dH" % count, subtable[16 + seg_x2:16 + 2 * seg_x2])
    deltas = struct.unpack(">%dh" % count, subtable[16 + 2 * seg_x2:16 + 3 * seg_x2])
    offsets = struct.unpack(">%dH" % count, subtable[16 + 3 * seg_x2:16 + 4 * seg_x2])
    return {
        "length": length,
        "segments": list(zip(ends, starts, deltas, offsets)),
        "glyph_ids": subtable[16 + 4 * seg_x2:],
    }


def build_format4(segments):
    """Serialise segments into a cmap format 4 subtable."""
    count = len(segments)
    seg_x2 = 2 * count
    power = 1
    while power * 2 <= count:
        power *= 2
    search = 2 * power
    selector = power.bit_length() - 1
    shift = seg_x2 - search
    header = struct.pack(">HHHHHHH", 4, 16 + 8 * count, 0, seg_x2, search, selector, shift)
    ends = b"".join(struct.pack(">H", end) for end, _, _, _ in segments)
    starts = b"".join(struct.pack(">H", start) for _, start, _, _ in segments)
    deltas = b"".join(struct.pack(">h", delta) for _, _, delta, _ in segments)
    ranges = b"".join(struct.pack(">H", off) for _, _, _, off in segments)
    return header + ends + struct.pack(">H", 0) + starts + deltas + ranges


def neutralise(segments):
    """Split ASCII segments so the neutralised characters map to .notdef.

    A format 4 segment maps code -> (code + idDelta) & 0xFFFF while
    idRangeOffset is zero, so a whole range cannot be pointed at .notdef with
    one delta. Each neutralised character therefore becomes its own segment
    with idDelta = -code, which maps exactly that character to glyph 0.
    """
    result = []
    for end, start, delta, range_offset in segments:
        if range_offset != 0:
            raise FontError("unexpected non-zero idRangeOffset")
        if end == 0xFFFF:
            result.append((end, start, delta, range_offset))
            continue
        codes = list(range(start, end + 1))
        neutral = [code for code in codes if code in NEUTRALISE]
        if not neutral:
            result.append((end, start, delta, range_offset))
            continue
        keep = [code for code in codes if code not in NEUTRALISE]
        if keep:
            result.append((keep[-1], keep[0], delta, 0))
        for code in neutral:
            result.append((code, code, -code, 0))
    return result


def checksum(payload):
    """The sfnt per-table checksum: sum of big-endian uint32s, zero padded."""
    padded = payload + b"\x00" * (-len(payload) % 4)
    total = 0
    for (value,) in struct.iter_unpack(">I", padded):
        total = (total + value) & 0xFFFFFFFF
    return total


def rebuild_font(tables):
    """Rebuild an sfnt file from tables, recomputing all checksums."""
    count = len(tables)
    power = 1
    while power * 2 <= count:
        power *= 2
    search = 16 * power
    selector = power.bit_length() - 1
    shift = 16 * count - search
    header = struct.pack(">IHHHH", 0x00010000, count, search, selector, shift)
    offset = HEADER + RECORD * count
    records = []
    body = bytearray()
    for tag, payload in tables:
        records.append((tag, offset, len(payload)))
        body += payload
        body += b"\x00" * (-len(payload) % 4)
        offset += len(payload) + (-len(payload) % 4)
    head_index = next(i for i, item in enumerate(tables) if item[0] == b"head")
    head_offset = records[head_index][1]
    body[head_offset + 8:head_offset + 12] = b"\x00\x00\x00\x00"
    directory = bytearray()
    for tag, at, length in records:
        directory += struct.pack(">4sIII", tag, checksum(bytes(body[at:at + length])), at, length)
    out = bytearray(header + bytes(directory) + bytes(body))
    total = 0
    for (value,) in struct.iter_unpack(">I", bytes(out) + b"\x00" * (-len(out) % 4)):
        total = (total + value) & 0xFFFFFFFF
    out[head_offset + 8:head_offset + 12] = struct.pack(">I", (0xB1B0AFBA - total) & 0xFFFFFFFF)
    return bytes(out)


def lookup(cmap_table, code):
    """Return the glyph id for a code through the shared format 4 subtable."""
    count = struct.unpack(">H", cmap_table[2:4])[0]
    for index in range(count):
        offset = struct.unpack(">HHI", cmap_table[4 + 8 * index:12 + 8 * index])[2]
        if struct.unpack(">H", cmap_table[offset:offset + 2])[0] != 4:
            continue
        parsed = parse_format4(cmap_table[offset:])
        for end, start, delta, range_offset in parsed["segments"]:
            if start <= code <= end:
                if range_offset == 0:
                    return (code + delta) & 0xFFFF
                base = 16 + 4 * len(parsed["segments"])
                slot = range_offset // 2 + (code - start)
                glyph = struct.unpack(">H", parsed["glyph_ids"][2 * slot:2 * slot + 2])[0]
                return (glyph + delta) & 0xFFFF if glyph else 0
        break
    return 0


def main(argv):
    if len(argv) != 3:
        print(__doc__.strip())
        return 2
    source_path = Path(argv[1])
    output_path = Path(argv[2])
    source = source_path.read_bytes()
    tables = read_tables(source)
    tags = [tag for tag, _ in tables]
    if b"cmap" not in tags or b"head" not in tags:
        raise FontError("font has no cmap or head table")
    cmap_index = tags.index(b"cmap")
    cmap_table = tables[cmap_index][1]
    count = struct.unpack(">H", cmap_table[2:4])[0]
    shared = set()
    for index in range(count):
        shared.add(struct.unpack(">HHI", cmap_table[4 + 8 * index:12 + 8 * index])[2])
    if len(shared) != 1:
        raise FontError("expected every cmap subtable to be shared")
    subtable_offset = shared.pop()
    parsed = parse_format4(cmap_table[subtable_offset:])
    if parsed["glyph_ids"]:
        raise FontError("expected an empty glyph id array (delta-mapped font)")
    new_segments = neutralise(parsed["segments"])
    new_cmap = cmap_table[:4] + cmap_table[4:4 + 8 * count] + build_format4(new_segments)
    tables[cmap_index] = (b"cmap", new_cmap)
    output = rebuild_font(tables)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_bytes(output)
    new_tables = read_tables(output)
    if [tag for tag, _ in new_tables] != tags:
        raise FontError("table set changed")
    for (tag, before), (_, after) in zip(tables, new_tables):
        if tag == b"head":
            # checkSumAdjustment is recomputed for the rebuilt file.
            before = before[:8] + before[12:]
            after = after[:8] + after[12:]
        if tag != b"cmap" and before != after:
            raise FontError("table %r changed unexpectedly" % tag)
    new_cmap_table = dict((tag, payload) for tag, payload in new_tables)[b"cmap"]
    for code in sorted(NEUTRALISE):
        if lookup(new_cmap_table, code) != 0:
            raise FontError("U+%04X is still mapped" % code)
    for code in list(range(0, 0x2000)) + list(range(0xE000, 0xF900)) + [0x2014, 0x2192, 0x2713]:
        if code in NEUTRALISE:
            continue
        if lookup(cmap_table, code) != lookup(new_cmap_table, code):
            raise FontError("U+%04X changed mapping" % code)
    print("patched %s -> %s" % (source_path, output_path))
    print("  neutralised %d ASCII mappings (a-z, space, hyphen, 2 controls)" % len(NEUTRALISE))
    print("  segments: %d -> %d" % (len(parsed["segments"]), len(new_segments)))
    print("  every other checked mapping verified unchanged")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv))
    except FontError as error:
        print("error: %s" % error, file=sys.stderr)
        raise SystemExit(1)
