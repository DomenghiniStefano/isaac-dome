import { assertNever } from '@/lib/assertNever'
import type { PageProgress, Target, WikiPageRef } from '@/lib/ipc/types'
import type { WikiCategory } from '@/router/routeTable'

// The one definition of "this page is done", one rule per `PageProgress` variant (design
// decision 6). `null` when the variant itself carries no answer — an item the save never
// resolved a `collected` bit for, a character whose unlock bit the save doesn't say — never
// guessed as `false`: a category that only ever sees `null` here has nothing to report, and
// `categoryProgress` below reads that as "no state", not as "none done".
export const isComplete = (p: PageProgress): boolean | null => {
  switch (p.kind) {
    case 'achievement':
      return p.done
    case 'item':
      return p.collected
    case 'unlockable':
      return p.unlocked
    case 'character':
      return p.unlocked
    case 'challenge':
      return p.state.kind === 'done'
    case 'bestiary':
      return p.killed > 0
    default:
      return assertNever(p)
  }
}

export interface Progress {
  done: number
  total: number
}

// A category's "N of M", over only the pages that have a progress entry whose `isComplete`
// answers something: a category the save says nothing about (transformations — decision 6's
// table names none) or a window with no save chosen (every `progressFor` answers `null`)
// both read as `null`, never as `0 of 0` — that would draw an empty bar where there ought to
// be none at all.
export const categoryProgress = (
  pages: WikiPageRef[],
  category: WikiCategory,
  progressFor: (target: Target) => PageProgress | null,
): Progress | null => {
  let done = 0
  let total = 0
  for (const page of pages) {
    if (page.category !== category) continue
    const progress = progressFor(page.target)
    if (progress === null) continue
    const complete = isComplete(progress)
    if (complete === null) continue
    total += 1
    if (complete) done += 1
  }
  return total === 0 ? null : { done, total }
}

// The landing hero's total: the sum of every category that has a progress of its own,
// derived straight from the pages' own `category` field rather than a list passed in, so a
// category nobody put state behind is excluded the same way `categoryProgress` excludes it —
// by construction, not by a second list somebody has to keep in step.
export const overallProgress = (
  pages: WikiPageRef[],
  progressFor: (target: Target) => PageProgress | null,
): Progress | null => {
  const categories = new Set<WikiCategory>()
  for (const page of pages) {
    if (page.category !== null) categories.add(page.category)
  }
  let done = 0
  let total = 0
  let any = false
  for (const category of categories) {
    const result = categoryProgress(pages, category, progressFor)
    if (result === null) continue
    any = true
    done += result.done
    total += result.total
  }
  return any ? { done, total } : null
}
