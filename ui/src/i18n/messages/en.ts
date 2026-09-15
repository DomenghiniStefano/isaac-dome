import type { MessageSchema } from './it'

export const en: MessageSchema = {
  ui: {
    close: 'Close',
  },
  shell: {
    newTab: 'New tab',
    closeTab: 'Close tab',
    back: 'Back',
    forward: 'Forward',
    minimize: 'Minimize',
    maximize: 'Maximize',
    closeWindow: 'Close window',
    search: 'Search everything',
    settings: 'Settings',
    about: 'About',
    resizeSidebar: 'Resize the sidebar',
    sections: {
      progress: 'Progress',
      tool: 'Tools',
      wiki: 'Wiki',
    },
  },
  marks: {
    thirdLevel: 'third level, meaning unconfirmed',
  },
  routes: {
    search: 'Search',
    goals: 'Suggested goals',
    completion: 'Completion',
    unlock: 'Unlock',
    plan: 'Plan',
    collection: 'Collection',
    runs: 'Runs',
    live: 'Live',
    floor: 'Floor',
    wiki: 'Wiki',
    profile: 'Game profile',
    appearance: 'Appearance',
    background: 'Background',
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
    transformations: 'Transformations',
  },
  sidebar: {
    progressTitle: 'Progress',
    progressHint: 'Every entry reads the active profile.',
    toolTitle: 'Tools',
    toolHint: 'They work without a save: they read the log, or what you draw.',
    wikiTitle: 'Wiki',
    wikiHint: 'The Wiki works without the game or a save.',
    wikiOverview: 'Overview',
    settingsTitle: 'Settings',
    settingsHint: 'The save you are playing, and how it opens tabs.',
  },
  live: {
    intro:
      'What you are playing now, and what finishing it would open. The app has to be running while you play: the game rewrites the log on every launch.',
    run: 'The run in progress',
    floors: 'floors',
    heldActive: 'held',
    collected: 'collected',
    wouldOpen: 'If you finish this run',
    missing: 'still to take',
    secondLevel: 'second level',
    noCondition: 'the game does not say',
    column: {
      achievement: 'Achievement',
      cell: 'Needs',
      condition: 'How it is earned',
      opens: 'Opens',
    },
    marks: 'The marks of who you are playing',
    items: 'What you are holding',
    startingItems: 'Started with',
    collectedItems: 'Collected',
    unlockedHere: 'Unlocked in this run',
    opens: 'opens {count}',
    opensNothingMore: 'opens nothing more',
    beat: 'Beat {column} with {character}',
    nothing:
      'Nothing this run can open on its own: what the achievements are missing is not a mark for this character.',
    diagnostic: {
      noRun:
        'No run in progress: the app is watching, and you are not playing.',
      characterNotNamed:
        'We do not know who you are playing yet: the name appears when you pick up the first item.',
      unknownCharacter: 'The character “{name}” is not in the game’s catalog.',
      ambiguousCharacter:
        'The log writes “{name}”, and the game calls {forms} characters that: the base form and the Tainted one. Both are below, because guessing which would be an inference.',
      noGraph: 'Without the game installed we do not know what it would open.',
      noProfile: 'Without a save chosen we do not know what you are missing.',
    },
  },
  floor: {
    intro:
      'Draw the floor the way you see it on the minimap: empty cells light up where the game’s own rules allow a secret room. Every lit cell says which rule lit it and where that rule was read.',
    erase: 'Erase',
    clear: 'Clear the grid',
    painted: 'rooms drawn',
    neighbours: 'adjacent rooms',
    source: 'Source',
    unresolved: 'What the grid cannot judge',
    none: 'No cell the rules allow, with what you have drawn so far.',
    failed: 'The candidates could not be computed. What you drew stays.',
    target: {
      secret: 'Secret Room',
      superSecret: 'Super Secret Room',
      ultraSecret: 'Ultra Secret Room',
    },
    room: {
      start: 'Start',
      normal: 'Normal',
      boss: 'Boss',
      treasure: 'Treasure',
      shop: 'Shop',
      curse: 'Curse',
      challenge: 'Challenge',
      sacrifice: 'Sacrifice',
      arcade: 'Arcade',
      library: 'Library',
      miniboss: 'Miniboss',
      secret: 'Secret',
      superSecret: 'Super Secret',
      ultraSecret: 'Ultra Secret',
    },
    diagnostic: {
      gridEmpty:
        'Draw at least one room: without one there is nothing to judge.',
      noStartRoom:
        'Mark the start room. Without it the rule about rooms walked from the start cannot be read, and the Super Secret Room is only half judged.',
      rulesUnreadable: 'The placement rules did not load: {reason}.',
      gridMalformed:
        'The grid that arrived is not a floor: {cells} cells instead of 169.',
    },
  },
  runs: {
    intro:
      'The runs the app has read: this session’s, and the online sessions the game had already recorded. The log carries no clock, so the order is by session and not by time.',
    rows: 'runs',
    facets: 'Filters',
    activeFilters: 'active filters',
    noFilters: 'no filter',
    reset: 'Reset the filters',
    search: 'search a seed or a character',
    facet: {
      outcome: 'Outcome',
      character: 'Character',
      online: 'With whom',
      source: 'From where',
    },
    outcome: {
      won: 'won',
      died: 'died',
      abandoned: 'abandoned',
      open: 'in progress',
    },
    online: {
      online: 'online',
      solo: 'alone',
    },
    source: {
      live: 'this session',
      session: 'online session',
    },
    totals: {
      runs: 'runs read',
      won: 'won',
      died: 'died',
      abandoned: 'abandoned',
      open: 'in progress',
    },
    column: {
      character: 'Character',
      outcome: 'Outcome',
      floors: 'Floors',
      seed: 'Seed',
      source: 'From where',
    },
    noCharacter: 'character not named',
    noItems: 'none',
    openPage: 'Click for the page, Ctrl-click to open it beside',
    itemId: 'item {id}',
    floors: 'floors',
    killedBy: 'killed by {killer}',
    endedWith: 'ending {ending}',
    startingItems: 'Starting items',
    collected: 'Collected',
    heldActive: 'Active held',
    unnamedItem: 'item {id}',
    empty:
      'Nothing in the archive: no run has been played since the app was installed.',
    noMatch: 'No run matches these filters.',
    diagnostic: {
      noLogFolder:
        'We cannot find the folder the game writes its logs to, so there is nothing to read.',
      storeUnavailable:
        'The app’s database will not open, so the archive cannot be read.',
      unreadableEvents:
        '{count} log lines were not understood: the runs holding them may be incomplete.',
      noCatalog:
        'The game is not installed: items carry their number and not their name.',
    },
  },
  collection: {
    intro:
      "The items this save doesn't hold yet, with their quality and the pools they turn up in. Trinkets aren't here: the game keeps no record of which ones you have found.",
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
      noCatalogTitle: "We can't find the game",
      noCatalog:
        'Without the game files the items have no name and no quality: we know how many you have, not which. Install The Binding of Isaac from Steam and reopen the app.',
      noCollectionSectionTitle: "The save doesn't say what you picked up",
      noCollectionSection:
        "Every item reads as unreadable. It doesn't mean you never found it: it means that part of the save would not open.",
      noAchievementSectionTitle: "The save doesn't say what you unlocked",
      noAchievementSection:
        'We cannot tell which items are still locked: the ones that come from an achievement stay uncertain.',
      itemsBeyondSlots:
        '{count} items the game knows and this save does not name: we show them as unreadable',
    },
  },
  wiki: {
    intro:
      'A copy of the Isaac wiki inside the app: items, characters, bosses, challenges, achievements and transformations. It works without the game installed and without a save chosen.',
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
      'Without the game installed the pages have no picture: the artwork comes from your own copy of Isaac, not from the app.',
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
      transformation: 'Transformation',
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
      notes: 'Notes',
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
      tears: 'Tears',
      range: 'Range',
      speed: 'Speed',
      luck: 'Luck',
      shotSpeed: 'Shot speed',
      collectibles: 'Starting items',
      requires: 'Needs',
      contributors: 'Counts',
      target: 'Acts on',
      none: 'none',
    },
    states: {
      unknownTitle: 'Unknown page',
      unknown: 'This page is not in our copy of the wiki.',
      unknownHint:
        'It may have been added since: the copy is from the date above, and it updates with the app.',
      noSections: 'For this page we only have the card, with no text.',
      failedTitle: "The Wiki didn't answer",
      missingTitle: "The Wiki didn't open",
      missing:
        'The copy of the wiki inside the app will not read, so there are no pages to show. Restarting the app usually clears it; if it does not, it is worth reporting.',
    },
  },
  search: {
    intro:
      "One search over everything: the game's names, achievement conditions, wiki titles and the body of wiki pages.",
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
      noWikiTitle: "The Wiki didn't open",
      noAchievementSectionTitle: "The save doesn't say what you unlocked",
      noCollectionSectionTitle: "The save doesn't say what you picked up",
      noProfile:
        'The results do not say what you have already done: choose a save in the settings.',
      noCatalog:
        'Only wiki titles and text are searched, with no pictures, and no result opens Unlock or the Collection.',
      noWiki: "Only the game's names are searched, and no result opens a page.",
      noAchievementSection:
        'Achievements carry no "done" or "to do". It does not mean you have not done them: it means we cannot tell.',
      noCollectionSection:
        'Items do not say whether you already have them. It does not mean you do not: it means we cannot tell.',
    },
  },
  background: {
    intro:
      'What the app does when you close the last window, and what you find when you open it again.',
    stayTitle: 'Keep running in the background',
    stayHint:
      'Closing the last window leaves the app next to the clock, in the notification area: one click on the icon brings the window back. Off, closing the last window closes the app.',
    resumeTitle: "Reopen the last session's tabs",
    resumeHint:
      'The first window reopens on the tabs it had. Off, it starts on the landing screen and what was saved is deleted.',
    saveFailedTitle: 'The setting was not saved',
    saveFailed:
      'The app already behaves this way, but we could not write it down: it will be back as it was next time you start it.',
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
      'The order you mean to do things in: what you asked for, plus what has to come first. Drag a row wherever you like — if something it needs is still missing it settles just below, instead of refusing the move.',
    summary: {
      rows: 'rows',
      wanted: 'asked for',
      pulledIn: 'pulled in',
    },
    queueTitle: 'The queue',
    hint: {
      idle: 'drag to reorder — no move is ever refused',
      dragging: 'drop it anywhere: the queue sorts itself out',
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
      condition: 'how to get it:',
    },
    achievement: 'achievement',
    empty: 'The queue is empty.',
    emptyHint: 'Add a row from the proposal beside it, or from Unlock.',
    completed: {
      closed: 'rows closed by playing',
      wanted: 'among the ones asked for',
    },
    unresolved: 'the game no longer knows it',
    alerts: {
      storeUnavailableTitle: "The plan isn't available",
      unreadableTitle: "This version can't read the saved queue",
      unreadable:
        'It stays as it is in the file and is never overwritten: a newer version of the app may be able to read it.',
      noCatalogTitle: 'The game has to be installed',
      noCatalog:
        'Without the game files we cannot tell which achievement each row is, or what it still needs. The queue stays saved: it comes back as it was as soon as the game is there.',
      goalsPendingTitle: 'Saved goals to import: {count}',
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
        'Cells we cannot yet find in the save. It does not mean "never done": it means we cannot tell.',
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
      unknown: "we can't tell for this character",
      unexpected: 'a value we did not expect:',
    },
    nothingReadable:
      'We cannot read a single mark from this save: the part that holds them is missing, or stops half way.',
  },
  // The landing screen: what you get, how, why it is worth it, and the one action.
  goals: {
    intro:
      'Things you can unlock right now. At the top the ones that open the most, below the ones you are closest to.',
    fanOut: 'They open the most',
    closeness: 'Almost there',
    inPlan: 'In your Plan',
    openPlan: 'Open the Plan',
    seeAll: 'See them all',
    opens: 'Opens {count} more things',
    opensNothing: 'Opens nothing else',
    noCatalogTitle: "We can't find the game",
    noCatalog:
      "IsaacDome reads The Binding of Isaac's own files to know which achievement unlocks what, and it cannot find them on this computer. Install the game from Steam and reopen the app: this is where you'll find what's worth playing tonight.",
    nothingNow:
      'There is nothing to unlock right now: either it is all done, or everything waits on something else.',
  },
  // B37: the same screen asked from the other end. You name what you want, and the answer is
  // the series to play, in the order the Plan would play it.
  want: {
    placeholder: 'I want… (an item, a character, a challenge, an achievement)',
    clear: 'Cancel',
    chain: 'What you have to play',
    availableNow: 'You can play it now',
    done: 'You already have it',
    noProfile: 'We don’t know where you stand',
    wayOf: 'Way {index} of {total}',
    unknown:
      'Plus {count} requirements we cannot read: the series may be longer.',
    addAll: 'Put it all in the Plan',
    diagnostics: {
      noCatalog:
        'Without the game files we don’t know which achievement unlocks what. Install The Binding of Isaac and reopen the app.',
      noProfile:
        'No profile read: we can tell you how it is obtained, not where you stand.',
      nothingUnlocks:
        'Nothing unlocks it: either you have always had it, or it comes from playing rather than from an achievement.',
      notUnlockable: 'This is not something you unlock.',
    },
  },
  // The block an achievement's wiki page gains when a profile is active. It says only where
  // you stand: what the thing is and what it asks is already in the infobox below.
  profileBlock: {
    title: 'Your profile',
    missing: 'What you are missing',
    unlocks: 'What you get',
    opens: 'Unlocking it opens {count} more things.',
    // A node already done will not open: it opened. The future tense under an "Already
    // done" reads as if something were still left to do.
    opened: 'It opened {count} more things.',
    opensNothing: 'It opens nothing else: this is the end of a branch.',
    openedNothing: 'It opened nothing else: this is the end of a branch.',
    stepsMissing: '{count} unlocks are needed first.',
    done: 'Already done.',
  },
  graph: {
    state: {
      done: 'done',
      now: 'unlockable now',
      blocked: 'blocked by',
      partial: "we can't tell",
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
      threshold: 'Transformations',
      unknown: "Conditions we can't read",
    },
    kinds: {
      passive: 'passive item',
      active: 'active item',
      familiar: 'familiar',
      trinket: 'trinket',
      character: 'character',
      boss: 'boss',
      challenge: 'challenge',
      nothing: 'nothing the game names',
    },
    stateName: {
      done: 'done',
      now: 'unlockable now',
      blocked: 'blocked',
      partial: "we can't tell",
    },
    // One cell of the matrix: the boss, and the character to beat it with.
    markName: '{boss} as {character}',
    thresholdName: '{name} — {current} of {atLeast}',
    originNone: 'not stated',
    unknownAchievement: 'Unknown achievement',
    slot: 'slot',
    taintedName: 'Tainted {name}',
  },
  unlock: {
    intro:
      'Every achievement in the game, filter it however you like. The filter that matters most is "unlockable now".',
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
    unlocksNothing: 'nothing the game names',
    noCondition: "we don't know",
    diagnostics: {
      noCatalogTitle: "We can't find the game",
      noCatalog:
        'Without the game files the achievements have no name and no condition: the rows only say which ones you have already done.',
      noAchievementSectionTitle: "The save doesn't say what you unlocked",
      noAchievementSection:
        "Zero rows done doesn't mean zero achievements: it means that part of the save would not open.",
      slotsBeyondCatalog:
        '{count} achievements the save knows and your copy of the game does not: they are probably from a newer version',
      catalogBeyondSlots:
        '{count} achievements the game has and this save does not name: they are left out of the list',
    },
  },
  gate: {
    needsProfile:
      'Progress depends on the active profile: choose one to see this screen.',
    blocked: {
      title: 'Progress needs a save',
      settings: 'Open the profile settings',
    },
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
      title: 'The three things it takes',
      summary: 'any one of them can be the one missing',
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
        "We can't find Steam on this computer, and that is where we start to reach the game folder and then the saves.",
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
      note: 'Some parts of the save we do not know the content of yet, and the quantities change with every patch of the game. What you see here and everywhere else in the app is always what your own file declares, never a number we decided.',
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
  // Why a command could not answer. Since N2 these are variants on the wire, not a sentence
  // built in Rust: the numbers arrive as numbers and the wording lives here.
  ipcReasons: {
    ioNotFound: "The file isn't there.",
    ioPermissionDenied: "Windows won't let us open it.",
    ioOther: "The system wouldn't open it.",
    saveTooShort: "It's too short to hold a save.",
    saveBadMagic: "It doesn't look like an Isaac save.",
    settingsConfigDirUnknown: "Windows won't say where settings go.",
    settingsEncoding: "The settings wouldn't be written out.",
    storeDataDirUnknown: "Windows won't say where app data goes.",
    storeDataDirNotCreatable: "The app's folder can't be created.",
    storeUnreadable: "The file won't open, or isn't a database.",
    storeNewerSchema:
      'It comes from a newer version of the app ({found} against {supported}).',
    storeQueueUnparseable: "The saved plan can't be read.",
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
    sessionTooLarge: 'The tab session is too large to be saved.',
  },
}
