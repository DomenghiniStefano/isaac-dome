#!/usr/bin/env python3
"""
M0 — reader for The Binding of Isaac: Repentance+ save file (read-only).

Structure discovered empirically from Stefano's backups (ISAACNGSAVE09R).

  0x00  magic "ISAACNGSAVE09R  "  (16 bytes)
  0x10  u32   unknown, and sometimes zero -- see docs/save-format.md, measured 2026-09-16.
              This line read "changes on every save" until then; it is zero on 4 of the 16
              saves of the 2024 series, every one of which changed payload.
  0x14  first section header

  section header = 3 x little-endian u32:
      kind   sequential 1..10
      f2     "in-memory" size = count * 4
      count  number of entries
  then the data: count entries, whose on-disk size depends on the section.

  last 4 bytes: checksum (CRC32 with a custom, unidentified polynomial —
  irrelevant as long as we're only reading).
"""

import struct
import sys

MAGIC = b"ISAACNGSAVE09R"

# kind -> (name, bytes per entry; None = variable-length section running to end of file)
SECTIONS = {
    1:  ("achievements", 1),
    2:  ("counters",     4),
    3:  ("per_char",     4),
    4:  ("items",        1),
    5:  ("unknown_5",    1),
    6:  ("cards_pills",  1),
    7:  ("challenges",   1),
    8:  ("unknown_8",    4),
    9:  ("unknown_9",    4),
    10: ("bestiary",  None),
}


class Save:
    def __init__(self, path):
        with open(path, "rb") as fh:
            self.raw = fh.read()
        if not self.raw.startswith(MAGIC):
            raise ValueError(f"unexpected magic: {self.raw[:16]!r}")
        self.path = path
        self.sections = {}
        self._parse()

    def _parse(self):
        end = len(self.raw) - 4          # excludes the final checksum
        off = 0x14
        for kind, (name, entry) in SECTIONS.items():
            k, f2, count = struct.unpack("<III", self.raw[off:off + 12])
            if k != kind:
                raise ValueError(f"expected section {kind}, found {k} at 0x{off:04x}")
            data = off + 12
            length = (end - data) if entry is None else count * entry
            self.sections[name] = {
                "kind": kind, "count": count, "declared": f2,
                "offset": data, "length": length,
                "blob": self.raw[data:data + length],
            }
            off = data + length

    def flags(self, name):
        s = self.sections[name]
        return [b != 0 for b in s["blob"]]

    def u32s(self, name):
        s = self.sections[name]
        return list(struct.unpack(f"<{s['count']}I", s["blob"]))

    def summary(self):
        out = []
        for name, s in self.sections.items():
            _, entry = SECTIONS[s["kind"]]
            if entry == 1:
                on = sum(s["blob"])
                out.append(f"{name:13s} {on:5d} / {s['count']:<5d} unlocked")
            elif entry == 4:
                vals = self.u32s(name)
                nz = sum(1 for v in vals if v)
                out.append(f"{name:13s} {nz:5d} / {s['count']:<5d} nonzero counters")
            else:
                out.append(f"{name:13s} {s['length']:5d} variable bytes "
                           f"({s['length'] / 8:.0f} 8-byte records)")
        return "\n".join(out)


def diff(a, b):
    """What changed between two saves: achievements, items, challenges."""
    res = {}
    for name in ("achievements", "items", "challenges", "cards_pills"):
        fa, fb = a.flags(name), b.flags(name)
        res[name] = [i for i, (x, y) in enumerate(zip(fa, fb)) if y and not x]
    ca, cb = a.u32s("counters"), b.u32s("counters")
    res["counters"] = {i: (x, y) for i, (x, y) in enumerate(zip(ca, cb)) if x != y}
    return res


if __name__ == "__main__":
    s = Save(sys.argv[1])
    print(f"{s.path}  ({len(s.raw)} byte)\n")
    print(s.summary())
    if len(sys.argv) > 2:
        d = diff(s, Save(sys.argv[2]))
        print("\n--- differences ---")
        for k in ("achievements", "items", "challenges", "cards_pills"):
            if d[k]:
                print(f"{k}: new indices {d[k]}")
        print(f"counters changed: {len(d['counters'])}")
