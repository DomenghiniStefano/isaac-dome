<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { selectProfile, setupState } from './lib/ipc/setup'
import { completion, saveSummary } from './lib/ipc/save'
import { extractionReport } from './lib/ipc/resources'
import { nextSteps, unlock } from './lib/ipc/graph'
import { wikiEntry } from './lib/ipc/wiki'
import type {
  ArchiveMode,
  Cell,
  Entry,
  ExtractionReport,
  IpcError,
  MarksMatrix,
  NextSteps,
  SaveSummary,
  SetupState,
  Target,
  UnlockView,
  WikiInfo,
} from './lib/ipc/types'
import { StepsBasis } from './lib/ipc/types'
import { assertNever } from './lib/assertNever'
import WikiBlocks from './components/WikiBlocks.vue'

const state = ref<SetupState | null>(null)
const summary = ref<SaveSummary | null>(null)
const matrix = ref<MarksMatrix | null>(null)
const extraction = ref<ExtractionReport | null>(null)
const unlockView = ref<UnlockView | null>(null)
const steps = ref<NextSteps | null>(null)
const error = ref<IpcError | null>(null)

// The default target to load: Binge Eater, so there's right away an entry with
// nested notes to eyeball.
const wikiTarget = ref<Target>({ kind: 'item', id: 664 })
const wikiEntryView = ref<Entry | null>(null)
const wikiId = ref('664')

const load = async () => {
  state.value = await setupState()
  // Extraction doesn't depend on the active profile: it reads the game's archives,
  // not the save file. So it must be loaded even when no profile is selected.
  extraction.value = await extractionReport()
  await loadWiki(wikiTarget.value)
  if (state.value.active.kind !== 'active') return
  summary.value = await saveSummary()
  matrix.value = await completion()
  unlockView.value = await unlock()
  steps.value = await nextSteps()
}

// `null` means "the dataset doesn't know this target": the view stays empty, with no error.
const loadWiki = async (target: Target) => {
  wikiTarget.value = target
  try {
    wikiEntryView.value = await wikiEntry(target)
  } catch (e) {
    handleIpcError(e)
  }
}

// `noActiveProfile` isn't an error to display: for the UI it means
// "go to profile selection". Every other case is an error to render.
const handleIpcError = (e: unknown) => {
  const err = e as IpcError
  switch (err.kind) {
    case 'noActiveProfile':
      summary.value = null
      matrix.value = null
      unlockView.value = null
      steps.value = null
      return
    case 'unknownProfile':
    case 'unreadableSave':
    case 'settingsNotWritable':
    case 'unknownTarget':
    case 'catalogUnavailable':
    case 'storeUnavailable':
    case 'wikiUnavailable':
      error.value = err
      return
    default:
      return assertNever(err)
  }
}

const choose = async (id: string) => {
  error.value = null
  try {
    state.value = await selectProfile(id)
    await load()
  } catch (e) {
    handleIpcError(e)
  }
}

const cellText = (c: Cell) => {
  switch (c.kind) {
    case 'known':
      return String(c.bits)
    case 'unknown':
      return '?'
    case 'unexpected':
      return `!${c.value}`
    default:
      return assertNever(c)
  }
}

const cellClass = (c: Cell) => {
  switch (c.kind) {
    case 'known':
      return c.bits === 0 ? 'text-mark-none' : 'text-mark-done'
    case 'unknown':
      return 'text-mark-unknown italic'
    case 'unexpected':
      return 'text-mark-partial font-bold'
    default:
      return assertNever(c)
  }
}

const modeText = (m: ArchiveMode) => {
  switch (m.kind) {
    case 'bogocrypt1':
      return 'Bogocrypt1'
    case 'lzw':
      return 'LZW'
    case 'miniZ':
      return 'MiniZ'
    case 'bogocrypt2':
      return 'Bogocrypt2'
    case 'unknown':
      return `sconosciuto (${m.value})`
    default:
      return assertNever(m)
  }
}

// Until the graph (M2) exists, steps arrive in slot order: the basis states
// this, and the screen says so instead of making the order look computed.
const basisText = (b: StepsBasis) => {
  switch (b) {
    case StepsBasis.Stub:
      return 'ordine di slot: il grafo non esiste ancora'
    case StepsBasis.FanOut:
      return 'per fan-out'
    default:
      return assertNever(b)
  }
}

// `null` covers both "the game isn't installed" and "installed but not via
// Steam": without a reliable installation date there's no way to tell whether
// the snapshot is behind.
const freshnessText = (newerThanSnapshot: boolean | null) => {
  if (newerThanSnapshot === null)
    return 'gioco non da Steam: freschezza non verificabile'
  return newerThanSnapshot
    ? 'il gioco è più recente dello snapshot'
    : 'snapshot al passo col gioco'
}

const countsText = (counts: Extract<WikiInfo, { kind: 'loaded' }>['counts']) =>
  `oggetti ${counts.items} · trinket ${counts.trinkets} · achievement ${counts.achievements} · boss ${counts.bosses} · sfide ${counts.challenges} · personaggi ${counts.characters}`

onMounted(() => load().catch(handleIpcError))
</script>

<template>
  <main class="flex flex-col gap-6 p-6 font-mono text-sm">
    <p v-if="error" class="text-mark-partial">{{ JSON.stringify(error) }}</p>

    <section v-if="state" class="flex flex-col gap-2">
      <h1 class="text-lg font-bold">Stato</h1>
      <p>Steam: {{ state.steam?.rootHint ?? 'non trovato' }}</p>
      <p>
        Gioco: {{ state.game?.dirHint ?? 'non trovato' }} ({{
          state.game?.edition ?? '—'
        }})
      </p>
      <p>Profilo attivo: {{ state.active.kind }}</p>
      <ul class="flex flex-col gap-1">
        <li v-for="c in state.candidates" :key="c.id">
          <button class="underline" @click="choose(c.id)">
            {{ c.prefix }} slot {{ c.slot }} — {{ c.sizeBytes }} byte
            <span v-if="c.suggested">(suggerito)</span>
          </button>
        </li>
      </ul>
    </section>

    <section
      v-if="state && state.diagnostics.length"
      class="flex flex-col gap-1"
    >
      <h2 class="font-bold">Diagnostica setup</h2>
      <p v-for="(d, i) in state.diagnostics" :key="i">
        {{ JSON.stringify(d) }}
      </p>
    </section>

    <section v-if="summary" class="flex flex-col gap-1">
      <h2 class="font-bold">Sezioni</h2>
      <p v-for="s in summary.sections" :key="s.kind">
        {{ s.kind }}: {{ s.count }}
      </p>
    </section>

    <section
      v-if="summary && summary.diagnostics.length"
      class="flex flex-col gap-1"
    >
      <h2 class="font-bold">Diagnostica salvataggio</h2>
      <p v-for="(d, i) in summary.diagnostics" :key="i">
        {{ JSON.stringify(d) }}
      </p>
    </section>

    <section v-if="extraction" class="flex flex-col gap-2">
      <h2 class="font-bold">
        Archivi del gioco: {{ extraction.archives.length }} aperti,
        {{ extraction.totalEntries }} voci indicizzate
      </h2>
      <p v-if="!extraction.archives.length" class="opacity-muted">
        Nessun archivio: il gioco non risulta installato.
      </p>
      <p v-for="a in extraction.archives" :key="a.name">
        {{ a.name }} — {{ modeText(a.mode) }} — {{ a.entries }} voci
      </p>
    </section>

    <section v-if="extraction?.catalog" class="flex flex-col gap-2">
      <h2 class="font-bold">
        Catalogo: {{ extraction.catalog.total }} oggetti —
        {{ extraction.catalog.counts.passives }} passivi,
        {{ extraction.catalog.counts.actives }} attivi,
        {{ extraction.catalog.counts.familiars }} familiari,
        {{ extraction.catalog.counts.trinkets }} trinket
      </h2>
      <p class="opacity-muted">
        Nomi da stringtable.sta in
        {{ extraction.catalog.languages.length }} lingue;
        {{ extraction.catalog.unresolvedNames }} senza stringa (si mostra la
        chiave).
      </p>
    </section>

    <section v-if="extraction?.sprites.length" class="flex flex-col gap-2">
      <h2 class="font-bold">
        Sprite estratti a runtime ({{ extraction.sprites.length }} dei
        {{ extraction.catalog?.total ?? 0 }} oggetti)
      </h2>
      <ul class="flex flex-row flex-wrap gap-4">
        <li
          v-for="s in extraction.sprites"
          :key="s.id"
          class="flex w-sprite flex-col items-center gap-1"
        >
          <img :src="s.dataUrl" :alt="s.name" class="size-sprite pixelated" />
          <span class="text-center break-words">{{ s.name }}</span>
          <span class="opacity-muted">{{ s.id }}</span>
        </li>
      </ul>
    </section>

    <section v-if="extraction" class="flex flex-col gap-2">
      <h2 class="font-bold">Wiki</h2>
      <template v-if="extraction.wiki.kind === 'loaded'">
        <p>
          snapshot {{ extraction.wiki.snapshotAt }} · patch
          {{ extraction.wiki.lastKnownPatch?.number ?? '?' }} · irrisolti
          (occorrenze) {{ extraction.wiki.unresolved }} · template ignoti
          (occorrenze) {{ extraction.wiki.unknownTemplates }}
        </p>
        <p>{{ freshnessText(extraction.wiki.gameNewerThanSnapshot) }}</p>
        <p>{{ countsText(extraction.wiki.counts) }}</p>
      </template>
      <p v-else>dataset assente: {{ extraction.wiki.reason }}</p>
      <label class="flex gap-2">
        oggetto id
        <input
          v-model="wikiId"
          class="border"
          @keyup.enter="
            Number.isFinite(Number(wikiId)) &&
            loadWiki({ kind: 'item', id: Number(wikiId) })
          "
        />
      </label>
      <template v-if="wikiEntryView">
        <h3 class="font-bold">
          {{ wikiEntryView.title }} (revid {{ wikiEntryView.revid }})
        </h3>
        <div
          v-for="s in wikiEntryView.sections"
          :key="s.kind"
          class="flex flex-col gap-1"
        >
          <h4 class="font-bold">{{ s.kind }}</h4>
          <WikiBlocks :blocks="s.blocks" @navigate="loadWiki" />
        </div>
      </template>
    </section>

    <section v-if="matrix" class="flex flex-col gap-2 overflow-x-auto">
      <h2 class="font-bold">
        Marchi: {{ matrix.totals.started }} iniziati su
        {{ matrix.totals.readable }} leggibili ({{
          matrix.totals.unknown
        }}
        ignoti, {{ matrix.totals.unexpected }} sospetti)
      </h2>
      <table>
        <thead>
          <tr>
            <th class="text-left"></th>
            <th v-for="b in matrix.bosses" :key="b" class="px-2 text-left">
              {{ b }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in matrix.characters" :key="row.character">
            <td class="pr-2">{{ row.character }}</td>
            <td
              v-for="(c, i) in row.cells"
              :key="i"
              :class="cellClass(c)"
              class="px-2"
            >
              {{ cellText(c) }}
            </td>
          </tr>
        </tbody>
      </table>
    </section>

    <section v-if="unlockView" class="flex flex-col gap-2">
      <h2 class="font-bold">
        Unlock: {{ unlockView.totals.done }} fatti su
        {{ unlockView.totals.slots }} ({{ unlockView.totals.unknown }}
        sconosciuti al catalogo)
      </h2>
      <p
        v-for="(d, i) in unlockView.diagnostics"
        :key="i"
        class="opacity-muted"
      >
        {{ JSON.stringify(d) }}
      </p>
    </section>

    <section v-if="steps" class="flex flex-col gap-2">
      <h2 class="font-bold">Prossimi passi ({{ basisText(steps.basis) }})</h2>
      <ul class="flex flex-col gap-1">
        <li
          v-for="(n, i) in steps.steps"
          :key="i"
          class="flex flex-row items-center gap-2"
        >
          <template v-if="n.achievement.kind === 'known'">
            <img
              v-if="n.achievement.iconUrl"
              :src="n.achievement.iconUrl"
              :alt="n.achievement.text"
              class="h-achievement"
            />
            <span>{{ n.achievement.text }}</span>
            <span v-if="n.achievement.hint" class="opacity-muted">
              — {{ n.achievement.hint }}
            </span>
          </template>
          <span v-else class="opacity-muted">
            slot {{ n.achievement.slot }}: fatto o no, il catalogo non sa cos'è
          </span>
        </li>
      </ul>
    </section>
  </main>
</template>
