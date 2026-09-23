import type { Block, Inline, UpdateView } from '../types'
import { Style } from '../types'

// Development only: a newer version already downloaded and verified, so the Updates screen
// can be looked at in a browser in the one phase that draws the most — the version, the notes
// and the install button. The notes are the shape Rust reads them into (`ipc::release_notes`),
// trimmed from what 0.2.0 shipped with.
const text = (value: string, style: Style = Style.Plain): Inline => ({
  kind: 'text',
  text: value,
  style,
})

const notes: Block[] = [
  {
    kind: 'paragraph',
    inline: [
      text(
        'The first release with new things in it since the app learned to update itself.',
        Style.Bold,
      ),
    ],
  },
  { kind: 'heading', level: 3, inline: [text('What changed')] },
  {
    kind: 'list',
    ordered: false,
    items: [
      {
        inline: [
          text('The app opens on Completamento.', Style.Bold),
          text(' Its matrix opens on a summary band.'),
        ],
        children: [],
      },
      {
        inline: [
          text('Obiettivi and the Plan are one screen.', Style.Bold),
          text(' The queue comes first and the suggestions follow it.'),
        ],
        children: [],
      },
    ],
  },
  {
    kind: 'paragraph',
    inline: [
      text(
        'Installer signed and verified before anything runs: public key FF274BE76BC3575C.',
      ),
    ],
  },
]

export const updateAnswer = (): UpdateView => ({
  currentVersion: '0.1.3',
  phase: { kind: 'ready', version: '0.2.0', notes },
  unavailable: null,
})
