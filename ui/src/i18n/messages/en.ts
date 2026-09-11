import type { MessageSchema } from './it'

export const en: MessageSchema = {
  ui: {
    close: 'Close',
  },
  shell: {
    newTab: 'New tab',
    closeTab: 'Close tab',
    minimize: 'Minimize',
    maximize: 'Maximize',
    closeWindow: 'Close window',
    search: 'Search everything',
    settings: 'Settings',
    about: 'About',
    resizeSidebar: 'Resize the sidebar',
    sections: {
      wiki: 'Wiki',
      progress: 'Progress',
    },
  },
  marks: {
    thirdLevel: 'third level, meaning unconfirmed',
  },
  routes: {
    nextSteps: 'Next steps',
    completion: 'Completion',
    unlock: 'Unlock',
    plan: 'Plan',
    collection: 'Collection',
    runs: 'Runs',
    live: 'Live',
    wiki: 'Wiki',
    profile: 'Game profile',
    tabsSettings: 'Tabs',
    about: 'About',
  },
  wikiCategories: {
    items: 'Items',
    trinkets: 'Trinkets',
    achievements: 'Achievements',
    bosses: 'Bosses',
    challenges: 'Challenges',
    characters: 'Characters',
  },
  sidebar: {
    progressTitle: 'Progress',
    progressHint: 'Every entry reads the active profile.',
    wikiTitle: 'Wiki',
    wikiHint: 'The Wiki works without the game or a save.',
    settingsTitle: 'Settings',
    settingsHint: 'Where the app finds the game and the saves.',
  },
  placeholder: {
    graph: 'Arrives with Next steps, Unlock and Plan.',
    collection: 'Arrives with the Collection screen.',
    runArchive: 'Arrives with the run archive (M4).',
    wiki: 'Arrives with wiki pages in tabs and search.',
    settings: 'Arrives with Settings and About.',
    tabs: 'Arrives with tabs that survive closing.',
  },
  completion: {
    intro:
      'Every character, every mark, and how much of it the save lets us read.',
    kpi: {
      started: 'marks started',
      startedExplain:
        'Cells with at least one mark, normal or hard, among those the save lets us read. Unreadable cells stay out of the denominator.',
      both: 'normal + hard',
      bothExplain: 'Cells holding both the normal and the hard mark.',
      cells: 'cells',
      complete: 'complete characters',
      completeExplain:
        'Characters who started every one of their readable cells: a character with unreadable cells is complete over fewer columns.',
      unknown: 'unreadable',
      unknownExplain:
        "Cells whose column isn't located in the save for that character. It doesn't mean \"never done\": we can't read it.",
    },
    card: {
      title: 'Marks matrix',
    },
    legend: {
      empty: 'never done',
      normal: 'normal',
      hard: 'hard',
      third: 'third level',
      unknown: 'unreadable',
    },
    grid: {
      character: 'Character',
      started: 'started',
      unreadable: 'unreadable',
      columnTotals: 'Characters with the mark',
    },
    groups: {
      base: 'Base characters',
      tainted: 'Tainted',
    },
    cell: {
      empty: 'never done',
      normal: 'normal',
      hard: 'hard',
      both: 'normal and hard',
      unknown: "unreadable: the column isn't located for this character",
      unexpected: 'a value outside the expected ones:',
    },
    nothingReadable:
      'The save lets us read no mark: the counters section is missing or cut short.',
  },
  graph: {
    state: {
      done: 'done',
      now: 'unlockable now',
      blocked: 'blocked by',
      partial: 'partial graph',
    },
    why: {
      title: 'What it still needs',
      character: 'Characters',
      boss: 'Bosses',
      challenge: 'Challenges',
      item: 'Items',
      gate: 'Conditions',
      unknown: "Requirements we can't interpret",
    },
    kinds: {
      passive: 'passive item',
      active: 'active item',
      familiar: 'familiar',
      trinket: 'trinket',
      character: 'character',
      boss: 'boss',
      challenge: 'challenge',
      nothing: 'nothing catalogued',
    },
    stateName: {
      done: 'done',
      now: 'unlockable now',
      blocked: 'blocked',
      partial: 'partial graph',
    },
    originNone: 'not stated',
    unknownAchievement: 'Unknown achievement',
    slot: 'slot',
  },
  nextSteps: {
    intro:
      "At most five rows, all unlockable now: the five that open the most further down. A node the graph can only call partial isn't a step, because we can't vouch for it.",
    unlocks: 'unlocks',
    noCatalogTitle: 'No steps: the catalogue is missing',
    noCatalog:
      "Without the game installed there's no telling what unlocks what: the list is empty on purpose, not five guessed rows.",
    nothingNow:
      'Nothing is unlockable right now: everything is done, or everything waits on something else.',
  },
  unlock: {
    intro:
      'Every node of the graph, filterable. One filter matters more than the rest: unlockable now.',
    rows: 'rows',
    search: 'search name, condition or what it unlocks',
    sortBy: 'sort by',
    sort: {
      fanOut: 'unlocks',
      steps: 'steps missing',
      name: 'name',
    },
    facets: 'Facets',
    noFilters: 'no filters',
    activeFilters: 'active filters',
    reset: 'Reset',
    resetFilters: 'Reset the filters',
    noResults: 'No rows with these filters.',
    facet: {
      state: 'State',
      unlocks: 'What it unlocks',
      origin: 'Origin DLC',
      character: 'Required character',
    },
    columns: {
      achievement: 'Achievement',
      unlocks: 'What it unlocks',
      condition: 'Condition',
      state: 'State',
      fanOut: 'Unlocks',
    },
    unlocksNothing: 'nothing catalogued',
    noCondition: 'no condition in the file',
    diagnostics: {
      noCatalogTitle: 'The catalogue is missing',
      noCatalog:
        'Without the game installed the achievements have no name and no condition: the rows only say which slots are done.',
      noAchievementSectionTitle: "The achievements section can't be read",
      noAchievementSection:
        "Zero rows doesn't mean zero achievements done: it means that part of the save wasn't read.",
      slotsBeyondCatalog:
        'save slots beyond the catalogue, shown as unknown achievements',
      catalogBeyondSlots:
        "catalogue achievements beyond the save, which don't appear",
    },
  },
  gate: {
    needsProfile:
      'Progress depends on the active profile: choose one to see this screen.',
  },
  indicator: {
    noProfile: 'No active profile',
    notFound: 'Saves not found',
    slot: 'slot',
  },
  profile: {
    eyebrow: 'Screen 0',
    title: 'Game profile',
    intro:
      'Not a step to go through once: it is the state that decides every number in the app. It stays open to read and change.',
    chain: {
      title: 'The chain of three requirements',
      summary: 'each level can be missing on its own',
      steam: 'Steam',
      game: 'Game',
      saves: 'Saves',
      found: 'found',
      missing: 'not found',
      yourChoice: 'your choice',
      several: 'more than one',
      chosen: 'chosen',
      candidates: 'candidate files',
    },
    none: {
      title: 'No save found',
      steamNotFound:
        "Steam isn't in the system registry: without Steam we can't find the game folder, and without that we can't find the saves.",
      gameNotFound:
        "Steam is there, but the game isn't in any of its libraries: without the game folder we can't find the saves.",
      noSaves:
        'The game is there, but there is no save file in the known places.',
      retry: 'Search again',
      diagnostics: 'Diagnostics — what we tried',
    },
    diagnostics: {
      steamNotFound: 'Steam: no installation found',
      gameNotFound: "Game: not in Steam's libraries",
      noSavesFound: 'Saves: no file in the known places',
      unreadablePath: 'Unreadable path',
      malformedManifest: 'Unreadable Steam manifest',
    },
    pick: {
      title: 'A choice is needed',
      summary: 'until you choose, no profile is active',
      savedGone:
        "The profile you used no longer exists where it was. We didn't pick another in its place: a different profile's numbers, shown without saying so, are the error you never notice you have.",
      edition: 'Edition',
      slot: 'Slot',
      foundIn: 'Found here',
      modified: 'Modified',
      size: 'Size',
      suggested: 'most recent',
      hint: 'The most recent is only a suggestion: you choose which profile to read.',
      use: 'Use this profile',
      cancel: 'Cancel',
    },
    sources: {
      steamCloud: 'Steam Cloud',
      documents: 'Documents',
      manual: 'Chosen by hand',
    },
    active: {
      title: 'Active profile',
      autoSelected: 'chosen by us · it was the only one',
      modified: 'Modified',
      size: 'Size',
      dlcs: 'DLC',
      foundIn: 'Found in',
      change: 'Change profile',
      reload: 'Read the file again',
      unknownDate: 'unknown',
      bytes: 'bytes',
    },
    read: {
      title: 'What we could read',
      sections: 'sections',
      note: "Some sections we don't know the content of yet, and the counts change with every patch: no number is hardcoded, here or anywhere in the app. The screens show what the file declares today.",
      diagnostics: "The file holds something we didn't expect",
    },
    saveDiagnostics: {
      unexpectedKind: "A section isn't the expected one",
      sectionOverrun: 'A section runs past the end of the file',
      trailingBytes: 'There are extra bytes at the end of the file',
    },
    sections: {
      achievements: 'Achievements and secrets',
      counters: 'Counters and marks',
      levelCounters: 'Per-floor counters',
      items: 'Item collection',
      bosses: 'Bosses met',
      challenges: 'Challenges',
      bestiary: 'Bestiary',
      unknown: 'To identify',
    },
    errors: {
      title: "We can't read the profile",
      retry: 'Try again',
    },
  },
  queue: {
    inQueue: 'queued',
    inPlan: "already in the Plan's queue",
    add: 'Add to the queue',
    addShort: 'Add',
    remove: 'Remove from the queue',
    removeShort: 'Remove',
    errorTitle: "The queue didn't change",
  },
  ipcErrors: {
    noBackend: "The backend didn't answer.",
    noActiveProfile: 'No active profile.',
    unknownProfile: 'The chosen profile no longer exists.',
    unreadableSave: "The save can't be read.",
    settingsNotWritable: "We can't remember the choice.",
    unknownTarget: "What you're looking for doesn't exist.",
    catalogUnavailable: "The game's catalogue isn't available.",
    storeUnavailable: "The app's database isn't available.",
    wikiUnavailable: "The wiki dataset isn't available.",
  },
}
