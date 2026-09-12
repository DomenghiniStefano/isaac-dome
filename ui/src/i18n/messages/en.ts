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
    search: 'Search',
    nextSteps: 'Next steps',
    completion: 'Completion',
    unlock: 'Unlock',
    plan: 'Plan',
    collection: 'Collection',
    runs: 'Runs',
    live: 'Live',
    wiki: 'Wiki',
    profile: 'Game profile',
    appearance: 'Appearance',
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
    wikiOverview: 'Overview',
    settingsTitle: 'Settings',
    settingsHint: 'The save you are playing, and how it opens tabs.',
  },
  collection: {
    intro:
      "The items this save's collection doesn't hold yet, by quality and pool: not a wall of icons. Quality and pool come from the game's files, not guessed from the name. Trinkets aren't here: the save keeps no collection of them.",
    state: {
      inCollection: 'in the collection',
      available: 'to find',
      locked: 'locked',
      unknown: 'unreadable',
    },
    facets: 'Filters',
    facet: {
      state: 'State',
      quality: 'Quality',
      pool: 'Pool',
      kind: 'Kind',
      origin: 'Origin DLC',
    },
    qualityUnrated: 'unrated',
    poolNone: 'no pool',
    items: 'items',
    search: 'search an item',
    sortBy: 'sort by',
    sort: {
      quality: 'quality',
      id: 'id',
      name: 'name',
    },
    columns: {
      item: 'Item',
      quality: 'Quality',
      pools: 'Pool',
      origin: 'DLC',
      state: 'State',
    },
    id: 'id',
    lockedBy: 'unlocked by',
    achievement: 'achievement',
    noResults: 'No items with these filters.',
    resetFilters: 'Reset the filters',
    noFilters: 'no filters',
    activeFilters: 'active filters',
    reset: 'Reset',
    diagnostics: {
      noCatalogTitle: 'The catalogue is missing',
      noCatalog:
        "Without the game installed the items have no name and no quality: the save's collection reads, but there's no telling what it holds.",
      noCollectionSectionTitle: "The save's collection can't be read",
      noCollectionSection:
        "Every item reads as unreadable: it doesn't mean you never found it, it means that part of the save wasn't read.",
      noAchievementSectionTitle: "The save's achievements can't be read",
      noAchievementSection:
        "There's no telling which items are still locked: the ones an achievement unlocks read as unreadable.",
      itemsBeyondSlots:
        "catalogue items beyond the save's collection, shown as unreadable",
    },
  },
  wiki: {
    intro:
      "The dataset is compiled into the binary: it works without the game installed and without a save chosen. It's the one piece of the app that depends on nothing.",
    provenance: {
      title: 'Where it comes from',
      snapshot: 'Snapshot of',
      patch: 'Last known patch',
      patchUnknown: 'unknown',
      newerGame:
        'The game is newer than this snapshot: recent changes may be missing.',
      license: 'wiki.gg · CC BY-SA 4.0',
      licenseLong:
        'The wiki text is CC BY-SA 4.0, from bindingofisaacrebirth.wiki.gg',
      unresolved: 'unresolved references',
      unknownTemplates: 'unknown templates',
    },
    categories: 'Categories',
    pages: 'pages',
    noCatalog:
      'Without the game installed the pages have no picture: names come from the wiki titles.',
    search: 'search a page',
    noResults: 'No page with this name.',
    resetFilters: 'Clear the search',
    back: 'Back to the category',
    id: 'id',
    kind: {
      item: 'Item',
      trinket: 'Trinket',
      achievement: 'Achievement',
      boss: 'Boss',
      challenge: 'Challenge',
      character: 'Character',
    },
    revision: 'rev.',
    section: {
      effects: 'Effects',
      notes: 'Notes',
      synergies: 'Synergies',
      interactions: 'Interactions',
      bugs: 'Bugs',
      behavior: 'Behavior',
      championVersions: 'Champion versions',
      damageScaling: 'Damage scaling',
      strategies: 'Strategies',
      difficulty: 'Difficulty',
      reward: 'Reward',
      unlockable: 'Unlockable',
    },
    infobox: {
      title: 'Card',
      description: 'Description',
      requirements: 'Requirements',
      unlocks: 'Unlocks',
      unlockedBy: 'Unlocked by',
      baseHp: 'Base HP',
      environment: 'Where',
      pool: 'Pool',
      goal: 'Goal',
      items: 'Items',
      trinkets: 'Trinkets',
      pickups: 'Pickups',
      health: 'Health',
      curse: 'Curse',
      restrictions: 'Restrictions',
      blindfolded: 'blindfolded',
      noShops: 'no shops',
      noTreasureRooms: 'no treasure rooms',
      noRestrictions: 'no restrictions',
      damage: 'Damage',
      range: 'Range',
      speed: 'Speed',
      luck: 'Luck',
      shotSpeed: 'Shot speed',
      collectibles: 'Starting items',
      none: 'none',
    },
    states: {
      unknownTitle: 'Unknown page',
      unknown: "The dataset doesn't know this page.",
      unknownHint:
        "The tab's key matches no page of the embedded dataset: it may come from a different version of the dataset.",
      noSections: 'This page has no text sections in the dataset.',
      failedTitle: "The Wiki didn't answer",
      missingTitle: "The wiki dataset didn't load",
      missing:
        "The binary carries a dataset that doesn't read: the Wiki has no pages to show.",
    },
  },
  search: {
    intro:
      "One index over everything the app knows by name or by text: the game's names, achievement conditions, wiki titles and the body of wiki pages.",
    placeholder: 'Search screens, achievements, items, wiki pages…',
    empty: 'No results.',
    allResults: 'All results ({count})',
    shown: '{shown} rows',
    limit: 'Showing {shown} of {total}: narrow the search.',
    groups: {
      screens: 'Screens',
      wiki: 'Wiki',
      unlock: 'Unlock',
      collection: 'Collection',
    },
    progress: {
      done: 'done',
      pending: 'to do',
    },
    hint: {
      open: 'open',
      newTab: 'open in a new tab',
    },
    diagnostics: {
      noProfileTitle: 'No profile chosen',
      noCatalogTitle: 'Game not installed',
      noWikiTitle: 'Wiki dataset not loaded',
      noAchievementSectionTitle: 'Achievement section not read',
      noCollectionSectionTitle: 'Collection section not read',
      noProfile:
        'The results do not say what you have already done: choose one in the settings.',
      noCatalog:
        'Only wiki titles and text are searched, with no pictures, and no result opens Unlock or the Collection.',
      noWiki: "Only the game's names are searched, and no result opens a page.",
      noAchievementSection:
        'What is done is unknown for achievements: unread is not undone.',
      noCollectionSection:
        'What you already have is unknown for items: unread is not not-found.',
    },
  },
  appearance: {
    intro:
      'Choose how large the interface is. It moves everything at once — text, icons, rows, the window — and it stays that way between launches.',
    scaleTitle: 'Size',
    scaleLabel: 'Interface size',
    preview: 'Preview',
    previewHint:
      'It stays here while you scroll: the app, drawn at the size you picked.',
    shortcut: 'From any screen:',
    shortcutReset: 'back to 100%',
    saveFailedTitle: "The size wasn't saved",
    saveFailed:
      "The interface is already this size, but we couldn't write it down: the next launch starts as it was.",
    sample: {
      kpi: 'Achievements done',
      item: 'The Sad Onion',
      itemHint: 'passive item · quality 2',
      button: 'A button',
      badge: 'unlockable now',
    },
  },
  about: {
    fanMade:
      'IsaacDome is a fan-made project and is not affiliated with, endorsed or sponsored by Nicalis or Edmund McMillen.',
    version: 'Version',
    versionUnknown: 'development server',
    promisesTitle: 'The three promises',
    promises: {
      readOnlyTitle: 'Saves are read only',
      readOnly:
        'Saves are only ever read. The module that opens the .dat files holds no write code, and the checksum is never recomputed.',
      offlineTitle: 'No account, no server, no telemetry',
      offline:
        'IsaacDome works offline. It needs no account and uses no server and no telemetry. The network is used only for an optional dataset update.',
      oneFileTitle: 'One file written',
      oneFile:
        "The app writes one file, isaacdome.db, in the app's data folder. No other file on your disk is changed or written.",
    },
    creditsTitle: 'Credits and licences',
    wikiText:
      'The wiki text is licensed CC BY-SA 4.0 and comes from bindingofisaacrebirth.wiki.gg.',
    assets:
      "The game's images are neither included nor distributed with the app. They are extracted from the user's own copy of the game.",
    font: 'The Determination Mono typeface is licensed CC BY 3.0.',
  },
  placeholder: {
    runArchive: 'Arrives with the run archive (M4).',
    tabs: 'Arrives with tabs that survive closing.',
  },
  plan: {
    intro:
      'The goals are the set of what you want, the queue is the order you mean to do it in: the rows you asked for, plus the prerequisites they dragged in. Drag a row and the queue repairs itself around the constraint: prerequisites are a wall it stops against, not a refusal.',
    summary: {
      rows: 'rows',
      wanted: 'asked for',
      pulledIn: 'pulled in',
    },
    queueTitle: 'The queue',
    hint: {
      idle: 'drag to reorder — a move repairs, it never fails',
      dragging: 'drop it anywhere: the queue repairs itself',
      stoppedUnder: 'it stopped under',
      prerequisite: "that's a prerequisite",
    },
    row: {
      move: 'Move the row (Alt and the up or down arrow)',
      wanted: 'asked for',
      serves: 'serves',
      unlocks: 'unlocks',
      fanOut: 'unlocks',
      outsideQueue: 'steps outside the queue',
      hint: "the game's hint:",
    },
    achievement: 'achievement',
    empty: 'The queue is empty.',
    emptyHint: 'Add a row from the proposal beside it, or from Unlock.',
    completed: {
      closed: 'rows closed by playing',
      wanted: 'among the ones asked for',
    },
    unresolved: 'is no longer in the catalogue',
    alerts: {
      storeUnavailableTitle: "The plan isn't available",
      unreadableTitle: "This version can't read the saved queue",
      unreadable:
        'It stays as it is in the file and is never overwritten: a newer version of the app may be able to read it.',
      noCatalogTitle: 'The game has to be installed',
      noCatalog:
        "Without the catalogue there's no telling which achievement each row is or what it still needs: the queue stays saved and comes back as soon as the game is there.",
      goalsPendingTitle: 'Saved goals to import',
      goalsPending:
        'Goals saved before the queue existed: nothing moves them in on its own.',
      import: 'Import into the queue',
    },
    aside: {
      title: 'Next steps',
      intro:
        'Rows unlockable right now, ordered by how much they open. Not your queue: the proposal.',
      empty: 'Nothing to propose right now.',
    },
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
      mark: 'Completion marks',
      counter: 'Bosses to beat',
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
    // One cell of the matrix: the boss, and the character to beat it with.
    markName: '{boss} as {character}',
    originNone: 'not stated',
    unknownAchievement: 'Unknown achievement',
    slot: 'slot',
    taintedName: 'Tainted {name}',
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
    facets: 'Filters',
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
    title: 'Game profile',
    intro:
      'Choose the save you are playing with: every number on the other screens is read from it. You can change it whenever you like.',
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
