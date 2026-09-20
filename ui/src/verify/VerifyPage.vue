<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { selectProfile, setupState } from '@/lib/ipc/setup'
import { runs } from '@/lib/ipc/runs'
import { AppEvent, watchAppEvent } from '@/lib/window/appEvents'
import { completion, saveSummary } from '@/lib/ipc/save'
import { extractionReport } from '@/lib/ipc/resources'
import { graphViews } from '@/lib/ipc/graph'
import { wikiEntry } from '@/lib/ipc/wiki'
import type {
  ArchiveMode,
  Cell,
  Entry,
  ExtractionReport,
  IpcError,
  MarksMatrix,
  NextSteps,
  RunOutcomeView,
  RunSource,
  RunView,
  RunsView,
  SaveSummary,
  SetupState,
  Target,
  UnlockView,
  WikiInfo,
} from '@/lib/ipc/types'
import { StepsBasis } from '@/lib/ipc/types'
import { assertNever } from '@/lib/assertNever'
import WikiBlocks from '@/components/wiki/WikiBlocks.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

const state = ref<SetupState | null>(null)
const summary = ref<SaveSummary | null>(null)
const matrix = ref<MarksMatrix | null>(null)
const extraction = ref<ExtractionReport | null>(null)
const unlockView = ref<UnlockView | null>(null)
const steps = ref<NextSteps | null>(null)
const runsView = ref<RunsView | null>(null)
const error = ref<IpcError | null>(null)

// The default target to load: Binge Eater, so there's right away an entry with
// nested notes to eyeball.
const wikiTarget = ref<Target>({ kind: 'item', id: 664 })
const wikiEntryView = ref<Entry | null>(null)
const wikiId = ref<string | number>('664')

// The archive does not need a profile: it is built from the logs on disk, and it fills itself
// in the background — which is why it is read again on `runs-changed` rather than only here.
const loadRuns = async () => {
  runsView.value = await runs()
}

const load = async () => {
  state.value = await setupState()
  await loadRuns()
  // Extraction doesn't depend on the active profile: it reads the game's archives,
  // not the save file. So it must be loaded even when no profile is selected.
  extraction.value = await extractionReport()
  await loadWiki(wikiTarget.value)
  if (state.value.active.kind !== 'active') return
  summary.value = await saveSummary()
  matrix.value = await completion()
  const views = await graphViews()
  unlockView.value = views.unlock
  steps.value = views.steps
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
    case 'autostartNotWritable':
    case 'sessionTooLarge':
    case 'updateNotReady':
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
      return c.bits === 0
        ? 'text-faint-foreground'
        : 'text-state-done-foreground'
    case 'unknown':
      return 'text-state-unknown-foreground italic'
    case 'unexpected':
      return 'text-state-unexpected-foreground'
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
      return `unknown (${m.value})`
    default:
      return assertNever(m)
  }
}

// The steps are what the graph says is unlockable now, most-opening first. The basis
// travels with them so the screen never has to guess why the order is what it is.
const basisText = (b: StepsBasis) => {
  switch (b) {
    case StepsBasis.FanOut:
      return 'by fan-out'
    case StepsBasis.Closeness:
      return 'by closeness'
    default:
      return assertNever(b)
  }
}

// `null` covers both "the game isn't installed" and "installed but not via
// Steam": without a reliable installation date there's no way to tell whether
// the snapshot is behind.
const freshnessText = (newerThanSnapshot: boolean | null) => {
  if (newerThanSnapshot === null)
    return 'game not from Steam: freshness cannot be checked'
  return newerThanSnapshot
    ? 'the game is newer than the snapshot'
    : 'snapshot in step with the game'
}

const countsText = (counts: Extract<WikiInfo, { kind: 'loaded' }>['counts']) =>
  `items ${counts.items} · trinkets ${counts.trinkets} · achievements ${counts.achievements} · bosses ${counts.bosses} · challenges ${counts.challenges} · characters ${counts.characters} · transformations ${counts.transformations}`

const sourceText = (s: RunSource) => {
  switch (s.kind) {
    case 'live':
      return 'log.txt'
    case 'session':
      return s.name
    default:
      return assertNever(s)
  }
}

const outcomeText = (o: RunOutcomeView) => {
  switch (o.kind) {
    case 'won':
      return `won (${o.ending})`
    case 'died':
      return `died (${o.killer})`
    case 'abandoned':
      return 'abandoned'
    // Not a failure state, and not drawn as one.
    case 'open':
      return 'open'
    default:
      return assertNever(o)
  }
}

const runKey = (r: RunView) => `${sourceText(r.source)}#${r.ordinal}`

let stopRunsEvent: (() => void) | undefined

onMounted(async () => {
  await load().catch(handleIpcError)
  // The archive fills itself in the background: without this the page shows whatever had been
  // imported by the time it mounted, which on a first launch is nothing.
  stopRunsEvent = await watchAppEvent(AppEvent.RunsChanged, () => {
    void loadRuns().catch(handleIpcError)
  })
})

onUnmounted(() => stopRunsEvent?.())
</script>

<template>
  <main class="flex flex-col gap-6 p-6 text-foreground-soft">
    <p v-if="error" class="text-destructive">{{ JSON.stringify(error) }}</p>

    <section v-if="state" class="flex flex-col gap-2">
      <h1 class="text-heading text-foreground">Status</h1>
      <p>Steam: {{ state.steam?.rootHint ?? 'not found' }}</p>
      <p>
        Game: {{ state.game?.dirHint ?? 'not found' }} ({{
          state.game?.edition ?? '—'
        }})
      </p>
      <p>Active profile: {{ state.active.kind }}</p>
      <ul class="flex flex-col gap-1">
        <li v-for="c in state.candidates" :key="c.id">
          <Button :variant="ButtonVariant.Link" @click="choose(c.id)">
            {{ c.prefix }} slot {{ c.slot }} — {{ c.sizeBytes }} bytes
            <span v-if="c.suggested">(suggested)</span>
          </Button>
        </li>
      </ul>
    </section>

    <section
      v-if="state && state.diagnostics.length"
      class="flex flex-col gap-1"
    >
      <h2 class="text-foreground">Setup diagnostics</h2>
      <p v-for="(d, i) in state.diagnostics" :key="i">
        {{ JSON.stringify(d) }}
      </p>
    </section>

    <section v-if="summary" class="flex flex-col gap-1">
      <h2 class="text-foreground">Sections</h2>
      <p v-for="s in summary.sections" :key="s.kind">
        {{ s.kind }}: {{ s.count }}
      </p>
    </section>

    <section
      v-if="summary && summary.diagnostics.length"
      class="flex flex-col gap-1"
    >
      <h2 class="text-foreground">Save diagnostics</h2>
      <p v-for="(d, i) in summary.diagnostics" :key="i">
        {{ JSON.stringify(d) }}
      </p>
    </section>

    <section v-if="extraction" class="flex flex-col gap-2">
      <h2 class="text-foreground">
        Game archives: {{ extraction.archives.length }} open,
        {{ extraction.totalEntries }} entries indexed
      </h2>
      <p v-if="!extraction.archives.length" class="opacity-muted">
        No archives: the game does not appear to be installed.
      </p>
      <p v-for="a in extraction.archives" :key="a.name">
        {{ a.name }} — {{ modeText(a.mode) }} — {{ a.entries }} entries
      </p>
    </section>

    <section v-if="extraction?.catalog" class="flex flex-col gap-2">
      <h2 class="text-foreground">
        Catalog: {{ extraction.catalog.total }} items —
        {{ extraction.catalog.counts.passives }} passives,
        {{ extraction.catalog.counts.actives }} actives,
        {{ extraction.catalog.counts.familiars }} familiars,
        {{ extraction.catalog.counts.trinkets }} trinkets
      </h2>
      <p class="opacity-muted">
        Names from stringtable.sta in
        {{ extraction.catalog.languages.length }} languages;
        {{ extraction.catalog.unresolvedNames }} without a string (the key is
        shown).
      </p>
    </section>

    <section v-if="extraction?.sprites.length" class="flex flex-col gap-2">
      <h2 class="text-foreground">
        Sprites extracted at runtime ({{ extraction.sprites.length }} of
        {{ extraction.catalog?.total ?? 0 }} items)
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
      <h2 class="text-foreground">Wiki</h2>
      <template v-if="extraction.wiki.kind === 'loaded'">
        <p>
          snapshot {{ extraction.wiki.snapshotAt }} · patch
          {{ extraction.wiki.lastKnownPatch?.number ?? '?' }} · unresolved
          (occurrences) {{ extraction.wiki.unresolved }} · unknown templates
          (occurrences) {{ extraction.wiki.unknownTemplates }}
        </p>
        <p>{{ freshnessText(extraction.wiki.gameNewerThanSnapshot) }}</p>
        <p>{{ countsText(extraction.wiki.counts) }}</p>
      </template>
      <p v-else>dataset absent: {{ extraction.wiki.reason }}</p>
      <Label class="w-fit">
        item id
        <Input
          v-model="wikiId"
          class="w-40"
          @keyup.enter="
            Number.isFinite(Number(wikiId)) &&
            loadWiki({ kind: 'item', id: Number(wikiId) })
          "
        />
      </Label>
      <template v-if="wikiEntryView">
        <h3 class="text-foreground">
          {{ wikiEntryView.title }} (revid {{ wikiEntryView.revid }})
        </h3>
        <div
          v-for="s in wikiEntryView.sections"
          :key="s.kind"
          class="flex flex-col gap-1"
        >
          <h4 class="text-foreground">{{ s.kind }}</h4>
          <WikiBlocks :blocks="s.blocks" @navigate="loadWiki" />
        </div>
      </template>
    </section>

    <section v-if="matrix" class="flex flex-col gap-2 overflow-x-auto">
      <h2 class="text-foreground">
        Marks: {{ matrix.totals.normal }} with a level,
        {{ matrix.totals.hard }} hard, out of
        {{ matrix.totals.readable }} readable ({{
          matrix.totals.unknown
        }}
        unknown, {{ matrix.totals.unexpected }} suspect)
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
      <h2 class="text-foreground">
        Unlock: {{ unlockView.totals.done }} done out of
        {{ unlockView.totals.slots }} ({{ unlockView.totals.unknown }}
        unknown to the catalog)
      </h2>
      <p
        v-for="(d, i) in unlockView.diagnostics"
        :key="i"
        class="opacity-muted"
      >
        {{ JSON.stringify(d) }}
      </p>
    </section>

    <section
      v-for="section in steps?.sections ?? []"
      :key="section.basis"
      class="flex flex-col gap-2"
    >
      <h2 class="text-foreground">
        Next steps ({{ basisText(section.basis) }})
      </h2>
      <ul class="flex flex-col gap-1">
        <li
          v-for="(n, i) in section.steps"
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
            <span v-if="n.achievement.condition" class="opacity-muted">
              — {{ n.achievement.condition }}
            </span>
          </template>
          <span v-else class="opacity-muted">
            slot {{ n.achievement.slot }}: done or not, the catalog does not
            know what it is
          </span>
        </li>
      </ul>
    </section>

    <section v-if="runsView" class="flex flex-col gap-2">
      <h2 class="text-foreground">Runs</h2>
      <p>
        {{ runsView.totals.runs }} runs — won {{ runsView.totals.won }}, died
        {{ runsView.totals.died }}, abandoned {{ runsView.totals.abandoned }},
        open {{ runsView.totals.open }}
      </p>
      <p v-for="(d, i) in runsView.diagnostics" :key="i" class="opacity-muted">
        {{ JSON.stringify(d) }}
      </p>
      <ul class="flex flex-col gap-1">
        <li v-for="r in runsView.runs.slice(0, 40)" :key="runKey(r)">
          <span class="opacity-muted">{{ sourceText(r.source) }}</span>
          #{{ r.ordinal }} — {{ r.character ?? '?' }} — {{ r.seedWords }} —
          {{ outcomeText(r.outcome) }} — {{ r.floors }} floors
          <span v-if="r.online" class="opacity-muted">(online)</span>
        </li>
      </ul>
    </section>
  </main>
</template>
