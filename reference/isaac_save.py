#!/usr/bin/env python3
"""
M0 — lettore del salvataggio di The Binding of Isaac: Repentance+ (sola lettura).

Struttura scoperta empiricamente sui backup di Stefano (ISAACNGSAVE09R).

  0x00  magic "ISAACNGSAVE09R  "  (16 byte)
  0x10  u32   sconosciuto (cambia a ogni salvataggio)
  0x14  prima intestazione di sezione

  intestazione di sezione = 3 x u32 little-endian:
      kind   progressivo 1..10
      f2     dimensione "in memoria" = count * 4
      count  numero di voci
  poi i dati: count voci, la cui dimensione su disco dipende dalla sezione.

  ultimi 4 byte: checksum (CRC32 con polinomio custom, non identificato —
  irrilevante finché si legge soltanto).
"""

import struct
import sys

MAGIC = b"ISAACNGSAVE09R"

# kind -> (nome, byte per voce; None = sezione a lunghezza variabile fino a fine file)
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
            raise ValueError(f"magic inatteso: {self.raw[:16]!r}")
        self.path = path
        self.sections = {}
        self._parse()

    def _parse(self):
        end = len(self.raw) - 4          # esclude il checksum finale
        off = 0x14
        for kind, (name, entry) in SECTIONS.items():
            k, f2, count = struct.unpack("<III", self.raw[off:off + 12])
            if k != kind:
                raise ValueError(f"attesa sezione {kind}, trovata {k} a 0x{off:04x}")
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
                out.append(f"{name:13s} {on:5d} / {s['count']:<5d} sbloccati")
            elif entry == 4:
                vals = self.u32s(name)
                nz = sum(1 for v in vals if v)
                out.append(f"{name:13s} {nz:5d} / {s['count']:<5d} contatori diversi da zero")
            else:
                out.append(f"{name:13s} {s['length']:5d} byte variabili "
                           f"({s['length'] / 8:.0f} record da 8)")
        return "\n".join(out)


def diff(a, b):
    """Cosa e' cambiato fra due salvataggi: achievement, oggetti, sfide."""
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
        print("\n--- differenze ---")
        for k in ("achievements", "items", "challenges", "cards_pills"):
            if d[k]:
                print(f"{k}: nuovi indici {d[k]}")
        print(f"contatori cambiati: {len(d['counters'])}")
