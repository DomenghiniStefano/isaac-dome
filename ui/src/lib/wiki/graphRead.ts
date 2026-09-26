import { LoadStatus } from '@/stores/loadStatus'

// Whether a wiki page reads the profile's unlock graph itself. Only when nobody has read it in
// this window: a read clears the view before it answers, so reading a graph that is there
// would blank Unlock and Goals for the length of a command, and a failed read is how "no
// profile" answers — the wiki is reachable without one, and it does not ask again on every page.
export const wikiReadsGraph = (status: LoadStatus): boolean =>
  status === LoadStatus.Idle
