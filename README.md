# IsaacDome

App desktop che legge i salvataggi locali di *The Binding of Isaac: Repentance+* e dice
cosa manca al completamento e cosa conviene giocare adesso. Progetto fan-made, non
affiliato a Nicalis né a Edmund McMillen.

- `CLAUDE.md` — contesto operativo: vincoli, formato del salvataggio, convenzioni.
- `docs/progetto.html` — documento di progetto completo (aprilo nel browser).
- `reference/` — implementazione di riferimento in Python, da tradurre in Rust.
- `samples/` — salvataggi di test, ignorati da git.

## Riferimento Python

```
python3 reference/isaac_save.py samples/<save>.dat            # struttura e conteggi
python3 reference/isaac_save.py samples/<a>.dat samples/<b>.dat   # diff fra due salvataggi
python3 reference/isaac_counters.py samples/<save>.dat        # matrice dei marchi
```

## Stato

M0 completata: formato decodificato e verificato su 28 salvataggi reali.
Prossimo passo: M1 — parser in Rust, discovery, unpack, schermata Completamento.
