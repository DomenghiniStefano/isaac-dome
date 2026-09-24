import type { MessageSchema } from './it'

export const en: MessageSchema = {
  ui: {
    close: 'Close',
    explain: 'What this means',
  },
  filters: {
    more: 'More filters',
    fewer: 'Fewer filters',
    active: 'active filters',
    reset: 'Clear filters',
    inMenu: 'filter the values',
    noMatch: 'No value matches that text.',
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
    collapseSidebar: 'Collapse the sidebar',
    expandSidebar: 'Expand the sidebar',
    sections: {
      progress: 'Progress',
      tool: 'Tools',
      wiki: 'Wiki',
    },
  },
  marks: {
    wonOnline: 'also won online',
  },
  routes: {
    search: 'Search',
    goals: 'Goals',
    completion: 'Completion',
    unlock: 'Unlock',
    plan: 'Plan',
    collection: 'Collection',
    challenges: 'Challenges',
    roll: 'Tonight',
    runs: 'Runs',
    live: 'Live',
    floor: 'Floor',
    wiki: 'Wiki',
    profile: 'Game profile',
    appearance: 'Appearance',
    background: 'Background',
    tabsSettings: 'Tabs',
    updates: 'Updates',
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
    missing: 'marks still to take',
    secondLevel: {
      hard: 'on hard',
      ultraGreedier: 'on Ultra Greedier',
    },
    noCondition: 'the game does not say',
    column: {
      achievement: 'Achievement',
      cell: 'Mark needed',
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
      saveUnreadable:
        'The chosen save does not read, so we do not know what you are missing. The run stays here.',
    },
  },
  floor: {
    intro:
      'Draw the floor the way you see it on the minimap: empty cells light up where the game’s own rules allow a Secret, Super Secret or Ultra Secret Room.',
    clear: 'Clear the grid',
    clearConfirm: {
      title: 'Clear the grid?',
      body: 'The floor you drew goes, and it cannot be brought back.',
      cancel: 'Cancel',
      confirm: 'Clear',
    },
    neighbours: 'adjacent rooms',
    empty: 'empty',
    cell: 'Row {row}, column {column}: {room}',
    at: 'Row {row}, column {column}',
    rooms: 'Rooms',
    startRoomMissing: 'The start room is missing',
    cellCandidate: '{cell} — {target}, place {rank}',
    move: {
      title: 'Move the drawing',
      note: 'The arrows shift the whole floor you drew one cell in that direction, without redrawing it. They are for when the start room ended up too close to an edge and the map does not fit. A grey arrow means a room is against that edge: moving further would push it off the grid.',
      left: 'Move everything left',
      up: 'Move everything up',
      down: 'Move everything down',
      right: 'Move everything right',
    },
    rank: {
      title: 'What a colour means',
      note: 'The fuller the square, the likelier the place. Past the third the rules say no more.',
    },
    // What each rule of `crates/floor/rules/placement.json` says, keyed by its id in camel case
    // (`lib/floor/ruleText.ts`). In English it is the wiki's quote, word for word.
    ruleText: {
      secretNeighbours:
        'Secret Rooms are equally as likely to be in a valid location with 3 neighbors, as it is with 4 neighbors.',
      secretNeighboursTwo:
        '2 neighbor locations are rare but possible, even when there are locations with 3+ neighbors available.',
      secretNeighboursOne:
        '1 neighbor locations can only happen if there are no valid 3+ neighbor locations, and are very rare.',
      secretForbiddenNeighbours:
        'Secret Rooms can exist next to all types of rooms except Boss Rooms, Super Secret Rooms, and other Secret Rooms',
      superSecretDeadEnd:
        'Super Secret Rooms are only located next to one other room',
      superSecretNeighbourNotSpecial:
        "this room can't be a Special Room; in other words, it is placed on one of the floor's dead ends, like any other Special Room",
      superSecretNotNextToSecret: 'cannot be connected to the Secret Room',
      superSecretSecondLongest:
        'Super Secret Rooms replace the dead-end room that would require the 2nd most rooms walked through from the start room to access',
      ultraSecretConnections:
        'Ultra Secret rooms are most likely generated in spots that connect to 3+ non-red rooms through its adjacent red rooms (different squares in L rooms count as 2).',
      ultraSecretConnectionsTwo:
        'They can be connected to 2 or 1 non-red rooms through its adjacent red rooms, but a specific 3+ room location is 11.5x more likely than a specific 2 room location',
      ultraSecretConnectionsOne:
        'If there is no 3+ room location available then a specific 2 room location is 11.5x more likely than a specific 1 room location.',
      ultraSecretNotConnected:
        'Ultra Secret Rooms are special rooms that are not connected to any other room on the map directly.',
      ultraSecretRedRoomInvalid:
        "Ultra Secret Rooms can't be connected to red rooms that connect to Secret Rooms, Super Secret Rooms, or Curse Rooms, and can't be in a location where any of its adjacent red rooms are invalid, such as next to a Boss Room",
      ultraSecretShapes:
        "next to the sides of narrow rooms, or any room that can't have a red room opened on that specific side, however locations on the 13x13 border where a red room would normally open to an I AM ERROR room are allowed",
    },
    // Why the grid cannot judge a rule, for the rules that can reach the screen unjudged.
    ruleNote: {
      superSecretSecondLongest:
        'There is no start room: without one, the rooms walked through cannot be counted.',
      ultraSecretShapes:
        'It depends on the shape of the room behind that side, and every room on this grid is a single square: L-shaped and narrow rooms cannot be drawn, so this rule cannot be judged.',
    },
    unresolved: 'What the grid cannot judge',
    none: 'No cell the rules allow, with what you have drawn so far.',
    failed: 'The candidates could not be computed. What you drew stays.',
    target: {
      secret: 'Secret Room',
      superSecret: 'Super Secret Room',
      ultraSecret: 'Ultra Secret Room',
    },
    targetShort: {
      secret: 'Secret',
      superSecret: 'Super',
      ultraSecret: 'Ultra',
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
      rulesUnreadable:
        'The placement rules did not load. This is a fault in the build, not in your floor.',
      gridMalformed:
        'The grid that arrived is not a floor: {cells} cells instead of 169.',
    },
  },
  runs: {
    intro:
      'The runs the app has read: this session’s, and the online sessions the game had already recorded. The log carries no clock, so the order is by session and not by time.',
    rows: 'runs',
    search: 'search a seed or a character',
    facet: {
      outcome: 'Outcome',
      character: 'Character',
      online: 'With whom',
      source: 'Source',
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
      source: 'Source',
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
  challenges: {
    intro:
      'The game’s 45 challenges: which ones you have finished, what it takes to open them and what they unlock. The conditions — who you play as, how far you go, whether it is blindfolded — come from the wiki; each challenge’s page has the rest.',
    rows: 'challenges',
    search: 'search a challenge or its number',
    state: {
      done: 'done',
      available: 'to do',
      blocked: 'blocked',
      unknown: 'unreadable',
    },
    blockedBy: 'missing requirements: {count}',
    facet: {
      state: 'State',
      character: 'Character',
      rewards: 'Unlocks',
      blindfolded: 'Blindfolded',
    },
    rewardsSome: 'unlocks something',
    rewardsNone: 'unlocks nothing',
    blindfoldedYes: 'blindfolded',
    blindfoldedNo: 'not blindfolded',
    blindfolded: 'blindfolded',
    noCondition: 'the wiki does not say',
    unlocksNothing: 'unlocks nothing',
    unnamedReward: 'achievement {id}',
    columns: {
      number: '#',
      challenge: 'Challenge',
      character: 'Character',
      goal: 'Finish line',
      state: 'State',
    },
    empty: 'No challenge to show.',
    noResults: 'No challenge matches these filters.',
    diagnostics: {
      noCatalogTitle: 'We cannot find the game',
      noCatalog:
        'The challenges live in the game’s own files: without them we know how many you have finished, not which. Install The Binding of Isaac from Steam and reopen the app.',
      noChallengesSectionTitle:
        'The save does not say which challenges you finished',
      noChallengesSection:
        'The section holding the challenges could not be read: every challenge is listed below, and none of them can say whether you have done it.',
      noAchievementSectionTitle: 'The save does not say what you have unlocked',
      noAchievementSection:
        'Without the achievements we do not know which challenges are already open, nor which rewards you already hold.',
      noWiki:
        'The wiki dataset did not load: the challenges stay, without their character, goal and blindfold.',
    },
  },
  roll: {
    intro:
      'One target, drawn from the 442-cell matrix: what is worth playing tonight, without choosing between hundreds of cells yourself.',
    draw: 'Draw',
    drawAgain: 'Draw again',
    notDrawnYet: 'Nothing drawn yet tonight.',
    status: {
      missing: 'to do',
      taken: 'already taken',
      unreadable: 'unreadable',
    },
    card: {
      mark: '{character} against {column}',
      greedier: '{character} — Greedier!',
      drawnFrom: 'Drawn from {size} possible targets',
    },
    emptyDeck: {
      taken:
        'The deck is empty: {count} targets are left out because they are already taken.',
      unreadable:
        'The deck is empty: {count} targets cannot be read from the save.',
      locked:
        'The deck is empty: {count} targets are locked to a character that is not playable yet.',
      filtered:
        'The deck is empty: {count} targets are left out by the filters below.',
      nothing: 'The deck is empty: there is nothing to offer yet.',
    },
    panel: {
      characters: 'Characters',
      columns: 'Columns',
      includeTaken: 'Include already-taken targets',
      includeTakenHint:
        'Otherwise the deck only draws from what is still to do.',
      onlyPlayable: 'Only already-playable characters',
      onlyPlayableHint:
        'Hides the characters the save does not let you pick yet.',
    },
    diagnostics: {
      noCounterSectionTitle: 'The save does not say which marks you took',
      noCounterSection:
        'The section holding the marks could not be read: the matrix stays, but every cell reads as unreadable.',
      documentUnreadableTitle: "Tonight's choices could not be read",
      documentUnreadable:
        'The saved document could not be parsed: the default preset was used instead.',
      documentFromTheFutureTitle: 'Your choices come from a newer version',
      documentFromTheFuture:
        'The document is at version {version}, and this app reads up to {supported}: the default preset was used instead.',
      noCatalogTitle: "We can't find the game",
      noCatalog:
        'Without the game installed there are no names, no art, and no way to tell who is already playable.',
      playabilityUnknownTitle: "We don't know who is already playable",
      playabilityUnknown:
        'The "only already-playable characters" filter was turned off for this draw: without the catalog or the achievements there is no way to tell.',
      storeUnavailableTitle: 'Your choices cannot be saved',
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
    empty: 'No items to show.',
    noResults: 'No items with these filters.',
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
    emptyCategory: 'This category has no pages.',
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
    outline: 'On this page',
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
      requires: 'Items needed',
      contributors: 'Items that count',
      target: 'Acts on',
      quality: 'Quality',
      recharge: 'Recharge',
      devilPrice: 'Devil price',
      shopPrice: 'Shop price',
      pools: 'Pools',
      tags: 'Tags',
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
  find: {
    label: 'Find on this page',
    placeholder: 'Find on this page',
    count: '{position} of {total}',
    empty: 'No matches',
    previous: 'Previous match',
    next: 'Next match',
    close: 'Close find',
    nothingToSearch: 'There is nothing here to search.',
  },
  background: {
    intro:
      'When the app starts, and what it does when you close the last window.',
    startTitle: 'Start with Windows',
    startHint:
      'IsaacDome has to be running while you play: the game rewrites its log at every launch. A session you played before opening the app is still recoverable — until the game is launched again, and then it is gone.',
    startDev:
      'Not available in this build. A development build would put its own temporary path in your login, where it would fail at every boot without saying so.',
    startWithoutBackground:
      'With "keep running in the background" off, you get the icon next to the clock at login and no window; the first window you open and close ends the app, and the reading stops with it.',
    startFailedTitle: 'Windows did not accept the entry',
    stayTitle: 'Keep running in the background',
    stayHint:
      'Closing the last window leaves the app next to the clock, in the notification area: one click on the icon brings the window back. Off, closing the last window closes the app.',
    saveFailedTitle: 'The setting was not saved',
    saveFailed:
      'The app already behaves this way, but we could not write it down: it will be back as it was next time you start it.',
  },
  updates: {
    intro:
      'Which version you have, and whether a newer one is out. The app downloads the update while you keep using it, and installs it when you say so.',
    currentVersion: 'Installed version: {version}',
    autoTitle: 'Update automatically',
    autoHint:
      'On, the app asks GitHub at every start whether there is a newer version and downloads it if there is. Off, no request is made at all: nothing leaves this computer unless you press “Check now”.',
    unsupported: 'This build does not update itself.',
    unsupportedHint:
      'Not available in this build. An installer run from here would silently replace the development build with a release, and nobody would notice.',
    idle: 'We have not checked yet since this app started.',
    checking: 'Checking…',
    upToDate: 'You have the newest version.',
    downloading: 'Downloading {version}…',
    ready: '{version} is ready to install.',
    // Not bad luck like the others: what arrived is not what it claims to be.
    failedRejected:
      'The update was refused: the signature does not match. Nothing was installed. Download the installer from the releases page rather than trying again here.',
    failedOffline:
      'We could not reach GitHub. Try again when you are back online.',
    failedNotPublished: 'There is no published version to download.',
    failedInstall:
      'The update was downloaded and the installation did not start. The file is still here: you can try again.',
    failedUnknown: 'The update did not work.',
    failedTitle: 'Could not install',
    notesTitle: 'What changes',
    check: 'Check now',
    install: 'Restart and install',
    installHint: 'Closes the app, installs the new version and opens it again.',
    saveFailedTitle: 'The setting was not saved',
    saveFailed:
      'The app already behaves this way, but we could not write it down: it will be back as it was next time you start it.',
  },
  tabsSettings: {
    intro: 'What you find when you open the app again.',
    resumeTitle: "Reopen the last session's tabs",
    resumeHint:
      'At startup you find the windows you had, holding the same tabs. Off, the app starts on the landing screen and what was saved is deleted at once.',
    keptTitle: 'What is saved',
    keptWindows:
      'The windows you had open, where they were and how big they were.',
    keptTabs:
      'The tabs of each window, in their order, and which one was in front.',
    keptReading:
      'How you were reading each tab: the filters, the sort, the selected row and how far you had scrolled.',
    keptSidebar: 'The width of the sidebar.',
    stoppedTitle: 'The session is no longer being saved',
    stopped:
      'The open tabs stay where they are, but they will not come back at the next start: what the windows are showing is past the room the session has. Closing a few tabs and opening any one again resumes the saving.',
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
    tabs: 'Arrives with tabs that survive closing.',
  },
  plan: {
    intro:
      'The order you mean to do things in: what you added, plus what has to come first. Drag a row wherever you like — if something it needs is still missing it settles just below, instead of refusing the move.',
    opens: 'opens {count}',
    opensNothing: 'opens nothing',
    detail: 'Show the detail',
    queueCount: 'Your queue',
    addPane: 'Suggested goals',
    queueTitle: 'The queue',
    hint: {
      idle: 'drag to reorder — no move is ever refused',
      dragging: 'drop it anywhere: the queue sorts itself out',
      stoppedUnder: "it stopped under «{name}»: that's a prerequisite",
    },
    row: {
      move: 'Move the row (Alt and the up or down arrow)',
      wanted: 'added by you',
      serves: 'needed for «{name}»',
      unlocks: 'unlocks',
      fanOut: 'unlocks',
      outsideQueue: 'prerequisites not queued',
      condition: 'how to get it:',
    },
    achievementNumbered: 'achievement {id}',
    empty: 'The queue is empty.',
    emptyHint: 'Add a row from the proposal beside it, or from Unlock.',
    completed: {
      closed: 'rows closed by playing: {count}',
      closedWanted:
        'rows closed by playing: {count} · among the ones you added: {names}',
    },
    unresolved: 'achievement {id}: the game no longer knows it',
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
    headline: 'marks taken on hard',
    kpi: {
      columns: 'complete bosses',
      columnsExplain:
        'Bosses every character has beaten on hard (Greed on Ultra Greedier), among that column’s readable cells. The same count as “complete characters”, read down a column instead of across a row — and it is what the paper beside it draws: a symbol appears at hard only once its column is full.',
      normal: 'marks at normal',
      normalExplain:
        'Cells with a mark, among those the save lets us read. A mark taken on hard counts here too, because beating a boss on hard is the harder of the two. Unreadable cells stay out of the denominator.',
      hard: 'marks at hard',
      hardExplain:
        'Cells with the second level, over the same denominator: hard in the boss columns, Ultra Greedier in Greed’s. Never more than the count beside it, because a mark at the second level counts as a normal one too.',
      complete: 'complete characters',
      completeExplain:
        'Characters with every one of their readable cells at hard, and Greed at Ultra Greedier, which is what the game’s own widget means by a full row. A character with unreadable cells is complete over fewer columns.',
    },
    card: {
      title: 'Marks matrix',
    },
    legend: {
      empty: 'never done',
      normal: 'normal',
      hard: 'hard',
      online: 'also won online',
      unknown: 'unreadable',
    },
    grid: {
      character: 'Character',
      normal: 'normal',
      hard: 'hard',
      unreadable: 'unreadable',
      columnTotals: 'Characters with the mark',
      columnTotalsHard: 'Of those, at hard',
    },
    groups: {
      base: 'Base characters',
      tainted: 'Tainted',
    },
    cell: {
      empty: 'never done',
      normal: 'normal',
      hard: 'hard',
      ultraGreedier: 'Ultra Greedier',
      unknown: "we can't tell for this character",
      unexpected: 'a value we did not expect:',
    },
    nothingReadable:
      'We cannot read a single mark from this save: the part that holds them is missing, or stops half way.',
  },
  // The landing screen: what you get, how, why it is worth it, and the one action.
  goals: {
    intro:
      'What to unlock and in what order: take from the suggestions, or ask for what you want, and queue it.',
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
      blocked: 'missing requirements: {count}',
      partial: 'requirements unclear',
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
      partial: 'requirements unclear',
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
      fanOut: 'how many it opens',
      steps: 'steps missing',
      name: 'name',
    },
    empty: 'No achievements to show.',
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
      fanOut: 'Opens',
    },
    unlocksNothing: 'nothing the game names',
    noCondition: 'the game does not say',
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
  welcome: {
    title: 'Which save are you playing with?',
    subtitle:
      'Every number in the app comes from the file you choose here. You can change it whenever you like.',
    savedGone:
      'The save you were using is no longer where it was. We have not picked another one in its place: the numbers of a different profile, shown without saying so, are the mistake you never notice you have.',
    hint: 'The most recent one is only a suggestion: you choose which to read.',
    use: 'Continue',
    cancel: 'Cancel',
    card: {
      slot: 'slot',
      achievements: 'achievements',
      items: 'items',
      marks: 'marks',
      unread: 'unreadable',
      suggested: 'most recent',
      never: 'date unknown',
      unreadableCells: '{count} cells not read',
      unreadable: 'This file would not let us read it',
    },
    nothing: {
      title: 'We found no save at all',
      retry: 'Search again',
      diagnostics: 'Diagnostics — what we tried',
      chooseGame: 'Choose the game folder',
      chooseSaves: 'Choose the saves folder',
    },
    failed: {
      title: 'We could not read it',
      retry: 'Try again',
    },
  },
  indicator: {
    noProfile: 'No active profile',
    notFound: 'Saves not found',
    slot: 'slot {slot}',
  },
  profile: {
    title: 'Game profile',
    intro:
      'Where the numbers come from: which save is being read, where we found it, and what the file let us read. To change it, the indicator at the top.',
    chain: {
      title: 'The three things it takes',
      summary: 'Steam, the game and the saves',
      steam: 'Steam',
      game: 'Game',
      saves: 'Saves',
      found: 'found',
      missing: 'not found',
      yourChoice: 'your choice',
      several: 'more than one',
      chosen: 'chosen',
      candidates: '{n} candidate files',
    },
    none: {
      steamNotFound:
        "We can't find Steam on this computer, and that is where we start to reach the game folder and then the saves.",
      gameNotFound:
        "Steam is there, but the game isn't in any of its libraries: without the game folder we can't find the saves.",
      noSaves:
        'The game is there, but there is no save file in the known places.',
      noSavesInChosenFolder:
        'The folder you pointed at holds no save. The game calls them `rep_persistentgamedata1.dat` or `rep+persistentgamedata1.dat`: if they are not there, it is a different folder.',
    },
    diagnostics: {
      steamNotFound: 'Steam: no installation found',
      gameNotFound: "Game: not in Steam's libraries",
      noSavesFound: 'Saves: no file in the known places',
      noSavesInChosenFolder: 'Saves: no file in the folder you pointed at',
      unreadablePath: 'Unreadable path · {name} · {reason}',
      malformedManifest: 'Unreadable Steam manifest · {name}',
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
      bytes: '{count} bytes',
    },
    read: {
      title: 'What we could read',
      sections: 'sections',
      note: 'Some parts of the save we do not know the content of yet, and the quantities change with every patch of the game. What you see here and everywhere else in the app is always what your own file declares, never a number we decided.',
      diagnostics: "The file holds something we didn't expect",
    },
    saveDiagnostics: {
      unexpectedKind: "A section isn't the expected one ({expected}, {found})",
      sectionOverrun: 'A section runs past the end of the file ({section})',
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
    removeShort: 'Remove from the queue',
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
    autostartWriteRefused:
      'Windows refused the entry: a system policy or an antivirus may be standing in the way. Nothing will start at login.',
    autostartWriteIgnored:
      'The entry was written and Windows goes on saying no. Look in Task Manager, under Startup apps: switched off there, IsaacDome stays off whatever is written here.',
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
    autostartNotWritable: 'Windows did not accept the start-at-login entry.',
    updateNotReady: 'There is no downloaded update to install.',
  },
}
