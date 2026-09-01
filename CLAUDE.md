# IsaacDome

App desktop che legge i file locali di *The Binding of Isaac: Repentance+* e risponde a
una domanda: **cosa mi manca, e cosa mi conviene giocare stasera.**

Non è un tool personale. Si distribuisce con un installer e deve funzionare senza
configurazione su qualunque installazione Steam del gioco. Il vincolo "deve funzionare
a casa di uno sconosciuto" governa metà delle decisioni qui sotto.

Il documento di progetto completo è in `docs/progetto.html`.

---

## Vincoli non negoziabili

1. **Sola lettura sui salvataggi.** Nessuna funzione di scrittura nel modulo che apre i
   `.dat`, per costruzione. Il checksum non va mai ricalcolato. Un bug qui distrugge il
   profilo di uno sconosciuto.
2. **Nessuna chiave API**, né richiesta all'utente né incorporata nel binario. Sono
   ammesse solo API pubbliche senza credenziali.
3. **Nessun asset del gioco nel pacchetto.** Le immagini si estraggono dalla copia
   dell'utente a runtime. Vale anche per i dataset derivati dalla wiki, che vanno con la
   loro licenza e attribuzione.
4. **Nessun account, nessun backend, nessuna telemetria.** L'app funziona offline; la
   rete serve solo per aggiornamenti opzionali di dataset.
5. **Degradare, mai fallire.** Se una sezione del salvataggio non si legge, l'app parte
   lo stesso, mostra quello che sa e segnala cosa manca.

## Stack

- **Frontend**: Vue 3 + TypeScript, Vite, **shadcn-vue** su **Reka UI**, **Tailwind v4**,
  **TanStack Table** (griglie filtrabili), **TanStack Virtual** (liste lunghe: 733 oggetti,
  642 achievement), Pinia, Vue Router, vue-i18n, Lucide.
- **Backend**: Rust dentro Tauri 2. Crate: `steamlocate`, `winreg` (fallback),
  `keyvalues-parser`, `quick-xml`, `notify`, `rusqlite` (bundled), `serde`.
- **Tooling**: pnpm, Git Flow con `develop` come branch di integrazione.

**Rust fa tutto ciò che tocca il disco. Vue riceve solo JSON già risolto**: il frontend
non conosce offset, nomi di file né stringhe di log.

## Moduli

| Modulo | Responsabilità |
|---|---|
| `discovery` | Trova Steam, il gioco, i salvataggi. Fallback manuale a ogni passo. |
| `unpack` | Estrae gli archivi `.a` del gioco nella cache locale. |
| `core-save` | Parser del `.dat`, sola lettura. |
| `log-watch` | Segue `log.txt` in append, con i pattern in un file di regole versionato. |
| `catalog` | Normalizza gli XML del gioco. |
| `graph` | Grafo degli sblocchi: stato, "sbloccabile ora", fan-out, piani. |
| `store` | SQLite: archivio run, snapshot, catalogo, piani. |

---

## Formato del salvataggio (decodificato e verificato)

Verificato su 28 salvataggi reali di Repentance+ distribuiti su 14 mesi.
Implementazione di riferimento in `reference/isaac_save.py` — tradurre da lì.

```
0x00   "ISAACNGSAVE09R  "   firma, 16 byte   # i tool esistenti cercano 06R e falliscono
0x10   u32                  cambia a ogni salvataggio, significato ignoto
0x14   prima intestazione di sezione
fine-4 checksum              CRC32 con polinomio custom, NON identificato (irrilevante)
```

Intestazione di sezione: tre `u32` little-endian — `kind` (progressivo 1..10),
`f2` (= count × 4, dimensione "in memoria"), `count`. Poi i dati: `count` voci, la cui
dimensione **su disco** dipende dalla sezione.

| kind | count | byte/voce | contenuto |
|---|---|---|---|
| 1 | 642 | 1 | achievement e segreti |
| 2 | 523 | 4 | contatori di gioco **e marchi di completamento** |
| 3 | 14 | 4 | un valore per personaggio originale, da identificare |
| 4 | 733 | 1 | collezione oggetti |
| 5 | 7 | 1 | da identificare |
| 6 | 104 | 1 | carte e pillole |
| 7 | 46 | 1 | sfide |
| 8 | 27 | 4 | da identificare |
| 9 | 2 | 4 | da identificare |
| 10 | variabile | 8 | bestiario, record chiave/valore |

> **Il numero di voci si legge dal file, MAI si scrive nel codice.** Il salvataggio di
> giugno 2025 dichiara 641 achievement, quelli del 2026 ne dichiarano 642: una patch ne ha
> aggiunto uno. Qualunque conteggio cablato si rompe da solo.

### Contatori e marchi

La sezione 2 corrisponde uno a uno all'enum `EventCounter` di REPENTOGON. Etichette in
`reference/isaac_counters.py`. Le celle `PROGRESSION_*` non sono contatori ma **maschere
di bit**: valori osservati solo 0, 1, 2, 3, 5, 7. Due bit sono i livelli del marchio, il
terzo non è ancora spiegato — **non calcolare percentuali di completamento come se fosse
noto**.

Aperto: i nomi documentati arrivano a 284; lo schema regolare (blocchi da 19 celle per
Bethany, Jacob & Esau e i 17 Tainted) regge fino a Hush, poi si interrompe. L'indice 385 è
un contatore a sé; la coda 404–522 mescola almeno due famiglie e contiene Delirium, Mother
e The Beast. Si chiude raccogliendo altri salvataggi in cui quei valori cambiano.

## log.txt

Riscritto a ogni avvio del gioco: le run non tracciate sono perse per sempre, quindi
l'app deve girare mentre si gioca. Righe verificate:

```
Adding collectible 225 (Gimpy) to player 0 (Cain) from pool treasure
RNG Start Seed: FYQ8 QQ8G (586324166) [New, 1]
Level::Init m_Stage 2, m_StageType 1 Seed 408474304
Game Over. Killed by (9.0) spawned by (84.0) damage flags (0)
playing cutscene 15 (Sheol).
```

Una riga dà ID e nome dell'oggetto, personaggio e pool; la morte dà l'entità che uccide e
quella che l'ha generata. I pattern vanno in un file di regole versionato, aggiornabile
senza ricompilare.

## Percorsi reali

`discovery` deve coprire almeno questi casi, tutti visti su macchine reali:

```
Documents\My Games\Binding of Isaac Repentance\        # senza il +
Documents\My Games\Binding of Isaac Repentance+\       # con il +
Steam\userdata\<id>\250900\remote\rep_persistentgamedata<n>.dat
Steam\userdata\<id>\250900\remote\rep+persistentgamedata<n>.dat
```

Con Steam Cloud attivo il salvataggio **non è** nella cartella Documents. Lì restano
`log.txt`, `options.ini`, e due sottocartelle utili: `save_backups\` (backup datati creati
dal gioco: una serie storica gratuita) e `online_logs\` (una cartella per sessione co-op
online, con log completo e snapshot del profilo prima e dopo).

Nota: il co-op online usa un **profilo condiviso separato** che cresce col gruppo, distinto
da quello personale. È una seconda progressione che il gioco non mostra da nessuna parte.

---

## Da non fare

- Non usare `nom` per il `.dat`: sono tre interi e una fetta di byte, `u32::from_le_bytes`
  e slice bastano.
- Non cablare conteggi di achievement, oggetti, sfide o personaggi.
- Non aprire i salvataggi in scrittura, per nessun motivo.
- Non introdurre una libreria di componenti "completa" (PrimeVue, Element Plus, AG Grid):
  la scelta è shadcn-vue proprio per avere i componenti nel repo.
- Non dare per installato REPENTOGON: è un bonus opzionale, mai una feature promessa.
- Non aprire la schermata su una griglia di oggetti — quello è già il menu del gioco.

## Stato

M0 chiusa: formato decodificato, parser Python funzionante, contatori etichettati, matrice
dei marchi ricostruita, log verificato.

Prossimo: **M1** — parser in Rust, `discovery`, `unpack`, e la prima schermata
(Completamento). Il grafo degli sblocchi, che alimenta "sbloccabile ora" e il piano, è
ancora tutto da costruire ed è il vero collo di bottiglia del progetto.

## Dati di test

Metti un salvataggio reale in `samples/` (la cartella è ignorata da git) e usalo come
riferimento per i test del parser. Idealmente due date diverse, così si prova anche il
diff — che è il meccanismo su cui poggia il piano che si aggiorna da solo.
