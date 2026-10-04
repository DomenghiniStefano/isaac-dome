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
    noMatch: 'No matching values.',
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
    collection: 'Collection',
    challenges: 'Challenges',
    roll: 'Tonight',
    runs: 'Runs',
    live: 'Live',
    floor: 'Floor map',
    wiki: 'Wiki',
    profile: 'Game profile',
    appearance: 'Appearance',
    background: 'Background',
    tabsSettings: 'Tabs',
    updates: 'Updates',
    data: 'Data',
  },
  wikiCategories: {
    items: 'Items',
    trinkets: 'Trinkets',
    achievements: 'Achievements',
    bosses: 'Bosses',
    challenges: 'Challenges',
    characters: 'Characters',
    transformations: 'Transformations',
    monsters: 'Monsters',
    cardsAndRunes: 'Cards and Runes',
    pickups: 'Pickups',
    stages: 'Stages',
    versions: 'Versions',
  },
  sidebar: {
    progressTitle: 'Progress',
    progressHint: 'Everything here is based on your save.',
    toolTitle: 'Tools',
    toolHint:
      'No save needed: they work from the run in progress, or from what you draw.',
    wikiTitle: 'Wiki',
    wikiHint: 'Available even without the game installed.',
    wikiOverview: 'Overview',
    settingsTitle: 'Settings',
    settingsHint: 'Which save to read, how the app looks and how it behaves.',
  },
  live: {
    intro:
      'The run you are playing, as it happens, and what finishing it would unlock. Keep IsaacDome open while you play: the game wipes its log at every launch.',
    floors: 'floors',
    collected: 'collected',
    wouldOpen: 'If you finish this run',
    missing: '{n} marks still to get',
    secondLevel: {
      hard: 'on hard',
      ultraGreedier: 'on Ultra Greedier',
    },
    noCondition: 'condition not given by the game',
    column: {
      achievement: 'Achievement',
      cell: 'Mark needed',
      condition: 'How to earn it',
      opens: 'Unlocks',
    },
    marks: 'Marks for the character in play',
    items: 'Your items',
    startingItems: 'Started with',
    collectedItems: 'Collected',
    unlockedHere: 'Unlocked in this run',
    opens: 'unlocks {count}',
    opensNothingMore: 'unlocks nothing more',
    nothing:
      'This run cannot unlock any new achievement on its own: the ones you are missing need marks from other characters.',
    diagnostic: {
      noRun:
        'No run in progress. Start a game and IsaacDome will follow it automatically.',
      characterNotNamed:
        'The character will show up as soon as you pick up your first item.',
      unknownCharacter: 'The character “{name}” is not in the game’s catalog.',
      ambiguousCharacter:
        'The log says “{name}”, a name the game uses for {forms} characters: the base form and the Tainted one. Both are shown below.',
      noGraph: 'Install the game to see what you would unlock.',
      noProfile: 'Choose a save to see what you are missing.',
      saveUnreadable:
        'We could not read the chosen save, so we cannot tell what you are missing. The run in progress stays on screen.',
    },
  },
  floor: {
    intro:
      'Draw the floor the way you see it on the minimap: empty cells light up wherever the game’s rules allow a Secret, Super Secret or Ultra Secret Room.',
    clear: 'Clear the grid',
    clearConfirm: {
      title: 'Clear the grid?',
      body: 'The floor you drew will be deleted, and it cannot be recovered.',
      cancel: 'Cancel',
      confirm: 'Clear',
    },
    neighbours: '{n} adjacent rooms',
    empty: 'empty',
    cell: 'Row {row}, column {column}: {room}',
    at: 'Row {row}, column {column}',
    rooms: 'Rooms',
    startRoomMissing: 'The start room is missing',
    cellCandidate: '{cell} — {target}, rank {rank}',
    move: {
      title: 'Move the drawing',
      note: 'The arrows shift your whole drawing by one cell, so you don’t have to redraw it. Use them when the start room is too close to an edge and the map doesn’t fit. A grey arrow means a room is already touching that edge.',
      left: 'Move everything left',
      up: 'Move everything up',
      down: 'Move everything down',
      right: 'Move everything right',
    },
    rank: {
      title: 'What the colours mean',
      note: 'The fuller the square, the likelier the spot. Beyond third place the rules make no distinction.',
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
        'It depends on the shape of the room on that side, and every room on this grid is a single square: L-shaped and narrow rooms cannot be drawn, so this rule cannot be checked.',
    },
    unresolved: 'Rules the grid cannot check',
    none: 'With what you have drawn so far, no cell fits the rules.',
    failed:
      'We could not work out the possible spots. Your drawing is left as it was.',
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
      gridEmpty: 'Draw at least one room to get started.',
      noStartRoom:
        'Mark the start room: without it, the Super Secret Room can only be partly placed.',
      rulesUnreadable:
        'The placement rules did not load. That is a fault in the app, not in your drawing.',
      gridMalformed:
        'The grid received is not valid: {cells} cells instead of 169.',
    },
  },
  runs: {
    intro:
      'Your run history: this session’s runs, earlier ones and the online games the game has recorded, newest first.',
    rows: 'runs',
    search: 'search a seed or a character',
    facet: {
      outcome: 'Outcome',
      character: 'Character',
      online: 'Mode',
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
      solo: 'solo',
    },
    source: {
      live: 'this session',
      launch: 'earlier session',
      session: 'online session',
    },
    totals: {
      runs: 'runs',
      won: 'won',
      died: 'died',
      abandoned: 'abandoned',
    },
    column: {
      date: 'Date',
      character: 'Character',
      outcome: 'Outcome',
      floors: 'Floors',
      seed: 'Seed',
      source: 'Source',
    },
    date: {
      started: 'started at {time}',
      written: 'log last written at {time}',
      undated: 'read before IsaacDome kept dates',
      none: '—',
    },
    noCharacter: 'unknown character',
    noItems: 'none',
    itemId: 'item {id}',
    killedBy: 'killed by {killer}',
    endedWith: 'ending {ending}',
    startingItems: 'Starting items',
    collected: 'Collected',
    heldActive: 'Active held',
    passives: 'Passives',
    familiars: 'Familiars',
    pool: {
      treasure: 'treasure room',
      shop: 'shop',
      boss: 'boss',
      devil: 'devil deal',
      angel: 'angel room',
      secret: 'secret room',
      library: 'library',
      curse: 'curse room',
      goldenChest: 'golden chest',
      redChest: 'red chest',
      beggar: 'beggar',
      greedTreasure: 'Greed treasure room',
      greedShop: 'Greed shop',
      greedBoss: 'Greed boss',
    },
    itemView: {
      asLogged: 'As logged',
      byType: 'By type',
      byOrigin: 'By origin',
      byFloor: 'By floor',
    },
    copy: 'Copy',
    copied: 'Copied',
    copyFailed: 'Could not copy',
    page: {
      backToList: 'All runs',
      noSuchRun:
        'This run is not in the archive any more: it may have been read again since the link was made.',
      killedBy: 'killed by',
      spawnedBy: 'spawned by',
      floors: '{count} floors',
      achievements: 'Achievements unlocked',
      achievementId: 'achievement {id}',
      route: 'Route',
      items: 'Items',
      rooms: '{count} rooms',
      floorNumbers: 'stage {stage}, type {type}',
      beforeFirstFloor: 'Before the first floor read',
    },
    unnamedItem: 'item {id}',
    empty:
      'No runs yet: play a game with IsaacDome open and it will show up here.',
    noMatch: 'No runs match these filters.',
    diagnostic: {
      noLogFolder:
        'We cannot find the game’s log folder, so there are no runs to show.',
      storeUnavailable:
        'We could not open the app’s database, so the run history is unavailable.',
      unreadableEvents:
        '{count} log lines were not recognised: some runs may be incomplete.',
      unreadableSessions:
        '{count} online sessions could not be read, and their runs are missing from the history.',
      liveLogUnreadable:
        'We cannot read the log of the game in progress: the current run is not in the history.',
      noCatalog:
        'The game is not installed: items show their number instead of their name.',
    },
  },
  challenges: {
    intro:
      'Every challenge in the game: which ones you have completed, what it takes to unlock them and what they reward. Character, goal and blindfold come from the wiki; everything else is on each challenge’s page.',
    rows: 'challenges',
    search: 'search a challenge or its number',
    state: {
      done: 'completed',
      available: 'to do',
      blocked: 'locked',
      unknown: 'unreadable',
    },
    blockedBy: 'missing requirements: {count}',
    facet: {
      state: 'State',
      character: 'Character',
      rewards: 'Reward',
      blindfolded: 'Blindfolded',
    },
    rewardsSome: 'has a reward',
    rewardsNone: 'no reward',
    blindfoldedYes: 'blindfolded',
    blindfoldedNo: 'not blindfolded',
    blindfolded: 'blindfolded',
    noCondition: 'not given by the wiki',
    unlocksNothing: 'unlocks nothing',
    unnamedReward: 'achievement {id}',
    columns: {
      number: '#',
      challenge: 'Challenge',
      character: 'Character',
      goal: 'Goal',
      state: 'State',
    },
    empty: 'No challenges to show.',
    noResults: 'No challenges match these filters.',
    diagnostics: {
      noCatalogTitle: 'We can’t find the game',
      noCatalog:
        'Challenges are described in the game’s own files: without them we know how many you have completed, but not which. Install The Binding of Isaac from Steam and reopen the app.',
      noChallengesSectionTitle: 'We don’t know which challenges you completed',
      noChallengesSection:
        'We could not read the part of the save that holds challenges: they are all listed below, but without knowing which ones you have completed.',
      noAchievementSectionTitle: 'We don’t know what you have unlocked',
      noAchievementSection:
        'Without the achievements we cannot tell which challenges are already unlocked, or which rewards you already have.',
      noWiki:
        'The wiki data did not load: challenges are listed without their character, goal and blindfold.',
    },
  },
  roll: {
    intro:
      'Not sure what to play tonight? Draw a target from the marks matrix and let chance decide.',
    draw: 'Draw',
    drawAgain: 'Draw again',
    notDrawnYet: 'Press “Draw” to find out what to play tonight.',
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
        'The deck is empty: {count} targets are left out because you already have them.',
      unreadable:
        'The deck is empty: we cannot read {count} targets from the save.',
      locked:
        'The deck is empty: {count} targets need a character you have not unlocked yet.',
      filtered:
        'The deck is empty: {count} targets are left out by the filters below.',
      nothing: 'The deck is empty: there is nothing to offer right now.',
    },
    panel: {
      characters: 'Characters',
      columns: 'Columns',
      includeTaken: 'Include targets you already have',
      includeTakenHint:
        'Otherwise the draw only picks from what is still to do.',
      onlyPlayable: 'Only unlocked characters',
      onlyPlayableHint: 'Leaves out the characters you cannot pick yet.',
    },
    diagnostics: {
      noCounterSectionTitle: 'We don’t know which marks you have',
      noCounterSection:
        'We could not read the part of the save that holds marks: the matrix is there, but every cell shows as unreadable.',
      documentUnreadableTitle: 'Your preferences did not load',
      documentUnreadable:
        'The saved preferences could not be read: we restored the defaults.',
      documentFromTheFutureTitle: 'Your preferences come from a newer version',
      documentFromTheFuture:
        'They are saved in format {version}, and this version of the app reads up to {supported}: the defaults are used for now.',
      noCatalogTitle: 'We can’t find the game',
      noCatalog:
        'Without the game installed there are no names or icons, and we cannot tell which characters you have unlocked.',
      playabilityUnknownTitle:
        'We don’t know which characters you have unlocked',
      playabilityUnknown:
        'The “only unlocked characters” filter was turned off for this draw: without the game’s catalog or the achievements we cannot check it.',
      storeUnavailableTitle: 'We can’t save your preferences',
    },
  },
  collection: {
    intro:
      'The items you are missing, with their quality and the pools they appear in. Trinkets are not included: the game does not record which ones you have found.',
    state: {
      inCollection: 'collected',
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
    noResults: 'No items match these filters.',
    diagnostics: {
      noCatalogTitle: 'We can’t find the game',
      noCatalog:
        'Without the game files, items have no name and no quality: we know how many you have, but not which. Install The Binding of Isaac from Steam and reopen the app.',
      noCollectionSectionTitle: 'We don’t know which items you have collected',
      noCollectionSection:
        'We could not read the part of the save that holds items, so they all show as unreadable. It does not mean you have never found them.',
      noAchievementSectionTitle: 'We don’t know what you have unlocked',
      noAchievementSection:
        'We cannot tell which items are still locked: the ones tied to an achievement stay uncertain.',
      itemsBeyondSlots:
        '{count} of the game’s items do not appear in this save: we show them as unreadable',
    },
  },
  wiki: {
    intro:
      'The Isaac wiki, built into the app and linked to your progress: items, characters, bosses, challenges, achievements, transformations and much more. Always at hand, offline too, even without the game installed.',
    provenance: {
      title: 'Source',
      snapshot: 'Up to date as of',
      patch: 'Latest patch covered',
      patchUnknown: 'unknown',
      newerGame:
        'Your game is newer than this edition of the wiki: the latest changes may be missing.',
      license: 'wiki.gg · CC BY-SA 4.0',
      licenseLong:
        'Wiki text licensed CC BY-SA 4.0, from bindingofisaacrebirth.wiki.gg',
      unresolved: 'unresolved references',
      unknownTemplates: 'unknown templates',
    },
    categories: 'Categories',
    pages: '{n} pages',
    pagesOf: '{shown} / {total} pages',
    noCatalog:
      'Without the game installed the pages have no pictures: the artwork comes from your own installation of Isaac.',
    search: 'search a page',
    noResults: 'No pages found.',
    emptyCategory: 'This category has no pages.',
    resetFilters: 'Clear the search',
    back: 'Back to the category',
    id: 'id {id}',
    list: {
      rows: 'pages',
      sortBy: 'sort by',
      noData: 'No data',
      qualityUnrated: 'Unrated',
      baseForm: 'Base form',
      progressOf: '{done} / {total}',
      grid: 'Cards',
      table: 'Table',
      ascending: 'Ascending',
      descending: 'Descending',
      facet: {
        profile: 'Your progress',
        edition: 'Edition',
        quality: 'Quality',
        template: 'Type',
        tag: 'Tag',
        tainted: 'Tainted',
        articleCategory: 'Card or Rune',
      },
      sort: {
        name: 'Name',
        id: 'Id',
        edition: 'Edition',
      },
    },
    kind: {
      item: 'Item',
      trinket: 'Trinket',
      achievement: 'Achievement',
      boss: 'Boss',
      challenge: 'Challenge',
      character: 'Character',
      transformation: 'Transformation',
      monster: 'Monster',
      cardOrRune: 'Card or Rune',
      pickup: 'Pickup',
      stage: 'Stage',
      version: 'Version',
    },
    revision: 'rev. {revision}',
    qualityChip: 'Quality {quality}',
    addedIn: 'Added in {edition}',
    removedIn: 'Removed in {edition}',
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
      startingItems: 'Unlockable starting items',
      other: 'Other',
    },
    infobox: {
      title: 'Overview',
      description: 'Description',
      requirements: 'Requirements',
      notes: 'Notes',
      unlocks: 'Unlocks',
      unlockedBy: 'Unlocked by',
      baseHp: 'Base HP',
      environment: 'Where',
      behavior: 'Behavior',
      pool: 'Pool',
      replace: 'Replaced by',
      replaceChance: 'Replace chance',
      replaceNotes: 'Replace notes',
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
      obtainedFrom: 'Obtained from',
      tags: 'Tags',
      none: 'none',
    },
    facts: {
      activated: 'Active',
      passive: 'Passive',
      recharge: 'Recharge {value}',
      shopPrice: 'Shop price {value}',
      devilPrice: 'Devil price {value}',
      tag: '{value}',
      requirement: 'Requirement: {value}',
      baseHp: 'Base HP {value}',
      floors: 'Floors {value}',
      character: 'Character {value}',
      goal: 'Goal {value}',
      curse: 'Curse {value}',
      health: 'Health {value}',
      damage: 'Damage {value}',
      tears: 'Tears {value}',
      range: 'Range {value}',
      speed: 'Speed {value}',
      luck: 'Luck {value}',
      shotSpeed: 'Shot speed {value}',
      tainted: 'Tainted',
      requires: 'Items needed {value}',
      contributors: 'Items that count {value}',
      category: 'Category',
      categoryCard: 'Card',
      categoryRune: 'Rune',
      versionNumber: 'Version {value}',
      versionDate: 'Released {value}',
      template: 'Type',
    },
    progress: {
      done: 'Done',
      notDone: 'To do',
      itemCollected: 'Collected',
      itemNotCollected: 'Not collected',
      unlocked: 'Unlocked',
      locked: 'Locked',
      lockedBy: 'Unlocked by achievement {id}',
      marks: 'Marks {done}/{total}',
      available: 'Available',
      blocked: 'Locked ({count} missing)',
      unknownState: 'Unknown',
      bestiaryMet: 'Met {count}',
      bestiaryKilled: 'Killed {count}',
      bestiaryKilledYou: 'Killed you {count}',
    },
    landing: {
      totalPages: 'Pages',
      overallProgress: 'Overall progress',
      categoryProgress: '{category} progress',
      progressOf: '{done} / {total}',
    },
    states: {
      unknownTitle: 'Page not found',
      unknown: 'This page is not in IsaacDome’s wiki yet.',
      unknownHint:
        'It may have been added recently: the wiki is current as of the date above, and it updates along with the app.',
      noSections: 'This page only has an overview, with no text.',
      failedTitle: 'The Wiki is not responding',
      pageFailedTitle: 'This page didn’t open',
      missingTitle: 'The Wiki didn’t open',
      missing:
        'We could not load the wiki, so there are no pages to show. Restarting the app usually fixes it; if it keeps happening, please let us know.',
    },
  },
  search: {
    intro:
      'Search the whole app: screens, items, achievements and the text of every wiki page.',
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
      noProfileTitle: 'No save chosen',
      noCatalogTitle: 'Game not installed',
      noWikiTitle: 'The Wiki didn’t open',
      noAchievementSectionTitle: 'We don’t know what you have unlocked',
      noCollectionSectionTitle: 'We don’t know which items you have collected',
      noProfile:
        'Results do not show your progress: choose a save in the settings.',
      noCatalog:
        'Only the wiki is searched, without pictures, and results do not open Unlock or the Collection.',
      noWiki:
        'Only the game’s names are searched, and results do not open wiki pages.',
      noAchievementSection:
        'Achievements show no “done” or “to do”: we cannot read that part of the save.',
      noCollectionSection:
        'Items do not show whether you have them: we cannot read that part of the save.',
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
    nothingToSearch: 'There is nothing to search here.',
  },
  background: {
    intro:
      'When IsaacDome starts, and what happens when you close the last window.',
    startTitle: 'Start with Windows',
    startHint:
      'Keep IsaacDome open while you play so no run is lost: the game wipes its log at every launch. A session played with the app closed can still be recovered, but only until you restart the game.',
    startDev:
      'Not available in development builds: it would register a temporary path at login that would soon stop working.',
    startWithoutBackground:
      'With “Keep running in the background” off, IsaacDome starts at login as an icon next to the clock, with no window; once you close the first window you open, the app quits and stops following your games.',
    startFailedTitle: 'Windows did not accept the startup entry',
    stayTitle: 'Keep running in the background',
    stayHint:
      'When you close the last window, IsaacDome stays in the notification area next to the clock: one click on the icon brings it back. When off, closing the last window quits the app.',
    saveFailedTitle: 'The setting was not saved',
    saveFailed:
      'The change is already in effect, but we could not save it: it will be back to how it was next time you start the app.',
  },
  data: {
    intro:
      'The two files IsaacDome writes, and where. Copy them to keep a backup; delete them to start over. The game and your saves are never among them.',
    database: 'Database',
    databaseHint:
      'Your goals, the queue, the runs read from the log, and the roll.',
    settings: 'Settings',
    settingsHint:
      'The active profile, the scale, the switches, and the folders you chose.',
    folder: 'Folder',
    size: 'Size',
    notCreated: 'Not created yet: nothing has been saved.',
    folderUnknown: 'Windows did not say which folder this is.',
    unreadable: 'The file is there and will not open.',
    reveal: 'Show in folder',
    revealFailed: 'Could not open the folder',
    contents: {
      goals: 'Goals: {count}',
      queue: 'In the queue: {count}',
      queueUnknown: 'In the queue: unreadable',
      sessions: 'Log sessions read: {count}',
      runs: 'Runs: {count}',
      rollSaved: 'Roll preset saved',
    },
  },
  updates: {
    intro:
      'Your installed version and any available updates. IsaacDome downloads updates while you keep using it, and installs them when you choose.',
    currentVersion: 'Installed version: {version}',
    autoTitle: 'Update automatically',
    autoHint:
      'When on, the app checks GitHub for a new version at every start and downloads it. When off, no request is made: nothing leaves your computer unless you press “Check now”.',
    unsupported: 'This build does not update itself.',
    unsupportedHint:
      'Not available in development builds: updating would replace it with a release build.',
    idle: 'No check yet since you opened the app.',
    checking: 'Checking…',
    upToDate: 'You have the latest version.',
    downloading: 'Downloading {version}…',
    ready: '{version} is ready to install.',
    // Not bad luck like the others: what arrived is not what it claims to be.
    failedRejected:
      'Update refused: the signature does not match, and nothing was installed. Download the installer from the releases page instead of retrying here.',
    failedOffline:
      'Could not reach GitHub. Try again once you are back online.',
    failedNotPublished: 'There are no published versions to download yet.',
    failedInstall:
      'The update was downloaded, but the installation did not start. The file is still available: you can try again.',
    failedUnknown: 'The update did not complete.',
    failedTitle: 'Installation failed',
    notesTitle: 'What’s new',
    check: 'Check now',
    install: 'Restart and install',
    installHint: 'Closes the app, installs the new version and opens it again.',
    saveFailedTitle: 'The setting was not saved',
    saveFailed:
      'The change is already in effect, but we could not save it: it will be back to how it was next time you start the app.',
  },
  tabsSettings: {
    intro: 'What you find when you reopen IsaacDome.',
    resumeTitle: 'Reopen the last session’s tabs',
    resumeHint:
      'At startup you get back the windows and tabs you had open. When off, the app starts on the home screen and the saved session is deleted right away.',
    keptTitle: 'What is saved',
    keptWindows: 'The windows you had open, with their position and size.',
    keptTabs: 'The tabs of each window, in order, and which one was in front.',
    keptReading:
      'The state of each tab: filters, sort order, selected row and how far you had scrolled.',
    keptSidebar: 'The width of the sidebar.',
    stoppedTitle: 'The session is no longer being saved',
    stopped:
      'Too many tabs are open to save them all: they stay where they are, but will not come back at the next start. Close a few, then open any tab, and saving resumes.',
    saveFailedTitle: 'The setting was not saved',
    saveFailed:
      'The change is already in effect, but we could not save it: it will be back to how it was next time you start the app.',
  },
  appearance: {
    intro:
      'Choose the size of the interface: text, icons, rows and the window all scale together, and your choice is kept between launches.',
    scaleTitle: 'Size',
    scaleLabel: 'Interface size',
    preview: 'Preview',
    previewHint:
      'Stays in view while you scroll: the app at the size you picked.',
    shortcut: 'From any screen:',
    shortcutReset: 'back to 100%',
    saveFailedTitle: 'The size was not saved',
    saveFailed:
      'The new size is already in effect, but we could not save it: it will be back to how it was next time you start the app.',
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
    promisesTitle: 'Our promises',
    promises: {
      readOnlyTitle: 'Your saves are never modified',
      readOnly:
        'IsaacDome only reads your saves: the code that opens the .dat files contains no write function at all.',
      offlineTitle: 'No account, no server, no telemetry',
      offline:
        'IsaacDome works offline, with no account, no server and no telemetry. The network is used only for updates, and you can turn them off.',
      oneFileTitle: 'It only writes to its own folders',
      oneFile:
        'IsaacDome keeps its settings and its database, isaacdome.db, in its own folders. The game, your saves and the rest of your disk are never touched.',
    },
    creditsTitle: 'Credits and licences',
    wikiText:
      'The wiki text is licensed CC BY-SA 4.0 and comes from bindingofisaacrebirth.wiki.gg.',
    assets:
      'The game’s images are not included with the app: they are extracted from your own installation of the game.',
    font: 'The Determination Mono typeface is licensed CC BY 3.0.',
  },

  plan: {
    opens: 'unlocks {count}',
    opensNothing: 'unlocks nothing else',
    detail: 'Show details',
    queueCount: 'Your queue',
    addPane: 'Suggested goals',
    hint: {
      idle: 'drag to reorder: every move is allowed',
      dragging: 'drop it anywhere: the queue rearranges itself',
      stoppedUnder: 'it stopped under “{name}”, which it depends on',
    },
    row: {
      move: 'Move the row (Alt and the up or down arrow)',
      wanted: 'added by you',
      serves: 'needed for “{name}”',
      outsideQueue: 'prerequisites not in the queue',
    },
    achievementNumbered: 'achievement {id}',
    empty: 'The queue is empty.',
    emptyHint: 'Add a goal from the suggestions alongside, or from Unlock.',
    completed: {
      closed: 'completed by playing: {count}',
      closedWanted:
        'completed by playing: {count} · among the ones you added: {names}',
    },
    unresolved: 'achievement {id}: no longer in the game',
    alerts: {
      storeUnavailableTitle: 'The plan is not available',
      unreadableTitle: 'This version can’t read the saved queue',
      unreadable:
        'The queue stays saved exactly as it is and is never overwritten: a newer version of the app will be able to read it.',
      noCatalogTitle: 'The game needs to be installed',
      noCatalog:
        'Without the game files we cannot tell which achievement each row is, or what it still needs. The queue stays saved and comes back as soon as the game is installed.',
      goalsPendingTitle: 'Saved goals to import: {count}',
      goalsPending:
        'Goals saved with an earlier version of the app: import them to add them to the queue.',
      import: 'Import into the queue',
    },
  },
  completion: {
    intro: 'Every character and every completion mark, at a glance.',
    headline: 'marks taken on hard',
    kpi: {
      columns: 'complete bosses',
      columnsExplain:
        'Bosses beaten on hard by every character (Greed on Ultra Greedier), counting only readable cells. It is the same count as “complete characters”, read down a column instead of across a row, and it is what the card alongside draws: a symbol turns hard only once its column is full.',
      normal: 'marks on normal',
      normalExplain:
        'Cells with at least one mark, among the readable ones. A mark taken on hard counts here too. Unreadable cells are left out of the total.',
      hard: 'marks on hard',
      hardExplain:
        'Cells with the second-level mark, over the same total: hard for the bosses, Ultra Greedier for Greed. Never higher than the number alongside, because every hard mark also counts as a normal one.',
      complete: 'complete characters',
      completeExplain:
        'Characters with every readable cell on hard (Greed on Ultra Greedier): the game’s own full row. For characters with unreadable cells, only the others count.',
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
      unreadable: '{n} unreadable',
      columnTotals: 'Characters with the mark',
      columnTotalsHard: 'Of those, on hard',
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
      unknown: 'unreadable for this character',
      unexpected: 'unexpected value:',
    },
    nothingReadable:
      'We cannot read any mark from this save: the part that holds them is missing or incomplete.',
  },
  // The landing screen: what you get, how, why it is worth it, and the one action.
  goals: {
    intro:
      'What to unlock and in what order: pick from the suggestions or search for what you want, and add it to the queue.',
    fanOut: 'Unlock the most',
    closeness: 'Almost there',
    seeAll: 'See all',
    noCatalogTitle: 'We can’t find the game',
    noCatalog:
      'IsaacDome reads The Binding of Isaac’s own files to know what each achievement unlocks, but it cannot find them on this computer. Install the game from Steam and reopen the app: this is where you will find what is worth playing tonight.',
    nothingNow:
      'There is nothing to unlock right now: either you have done it all, or every goal needs something else first.',
  },
  // B37: the same screen asked from the other end. You name what you want, and the answer is
  // the series to play, in the order the Plan would play it.
  want: {
    placeholder: 'I want… (an item, a character, a challenge, an achievement)',
    clear: 'Cancel',
    chain: 'What you need to play',
    availableNow: 'You can play it now',
    done: 'You already have it',
    noProfile: 'We don’t know how far along you are',
    wayOf: 'Path {index} of {total}',
    unknown:
      'Plus {count} requirements we cannot interpret: the path may be longer.',
    addAll: 'Add all to the Plan',
    diagnostics: {
      noCatalog:
        'Without the game files we don’t know what each achievement unlocks. Install The Binding of Isaac and reopen the app.',
      noProfile:
        'No save chosen: we can tell you how to get it, but not how far along you are.',
      nothingUnlocks:
        'No achievement unlocks it: it is either available from the start, or earned by playing.',
      notUnlockable: 'This is not something you unlock.',
    },
  },
  // The block an achievement's wiki page gains when a profile is active. It says only where
  // you stand: what the thing is and what it asks is already in the infobox below.
  profileBlock: {
    title: 'Your progress',
    missing: 'What you are missing',
    unlocks: 'What you get',
    opens: 'Unlocking it opens up {count} more things.',
    // A node already done will not open: it opened. The future tense under an "Already
    // done" reads as if something were still left to do.
    opened: 'It unlocked {count} more things.',
    opensNothing: 'It unlocks nothing else.',
    openedNothing: 'It unlocked nothing else.',
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
      unknown: 'Conditions we can’t interpret',
    },
    kinds: {
      passive: 'passive item',
      active: 'active item',
      familiar: 'familiar',
      trinket: 'trinket',
      character: 'character',
      boss: 'boss',
      challenge: 'challenge',
      nothing: 'no unlock listed by the game',
    },
    stateName: {
      done: 'done',
      now: 'unlockable now',
      blocked: 'locked',
      partial: 'requirements unclear',
    },
    // One cell of the matrix: the boss, and the character to beat it with.
    markName: '{boss} as {character}',
    thresholdName: '{name} — {current} of {atLeast}',
    originNone: 'not stated',
    unknownAchievement: 'Unknown achievement',
    unknownAchievementInSlot: 'Unknown achievement · slot {slot}',
    slot: 'slot {slot}',
    taintedName: 'Tainted {name}',
  },
  unlock: {
    intro:
      'Every achievement in the game, filtered however you like. The most useful filter is “unlockable now”.',
    rows: 'rows',
    search: 'search name, condition or what it unlocks',
    sortBy: 'sort by',
    sort: {
      fanOut: 'how many it unlocks',
      steps: 'steps missing',
      name: 'name',
    },
    empty: 'No achievements to show.',
    noResults: 'No rows match these filters.',
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
    unlocksNothing: 'no unlock listed by the game',
    noCondition: 'condition not given by the game',
    diagnostics: {
      noCatalogTitle: 'We can’t find the game',
      noCatalog:
        'Without the game files achievements have no name and no condition: you can only see which ones you have already done.',
      noAchievementSectionTitle: 'We don’t know what you have unlocked',
      noAchievementSection:
        'We cannot read the part of the save that holds achievements: if none show as done, it does not mean you have none.',
      slotsBeyondCatalog:
        '{count} achievements are in the save but not in the installed game: they are probably from a newer version',
      catalogBeyondSlots:
        '{count} of the game’s achievements do not appear in this save and are left out of the list',
    },
  },
  welcome: {
    title: 'Which save are you playing with?',
    subtitle:
      'All the data in the app comes from the save you choose here. You can change it whenever you like.',
    savedGone:
      'The save you were using is no longer where it was. We have not picked another one for you: choose which one to read.',
    hint: 'We suggest the most recent one, but the choice is yours.',
    use: 'Continue',
    cancel: 'Cancel',
    card: {
      slot: 'slot {slot}',
      achievements: 'achievements',
      items: 'items',
      marks: 'marks',
      unread: 'unreadable',
      suggested: 'most recent',
      never: 'date unknown',
      unreadableCells: '{count} unreadable cells',
      unreadable: 'We could not read this file',
    },
    nothing: {
      title: 'No saves found',
      retry: 'Search again',
      diagnostics: 'Details — where we looked',
      chooseGame: 'Choose the game folder',
      chooseSaves: 'Choose the saves folder',
    },
    failed: {
      title: 'Could not read the save',
      retry: 'Try again',
    },
  },
  indicator: {
    noProfile: 'No save chosen',
    notFound: 'Saves not found',
    slot: 'slot {slot}',
  },
  profile: {
    title: 'Game profile',
    intro:
      'The save you are using, where we found it and what we could read from it. To change it, use the indicator at the top.',
    chain: {
      title: 'What it takes',
      summary: 'Steam, the game and the saves',
      steam: 'Steam',
      game: 'Game',
      saves: 'Saves',
      found: 'found',
      missing: 'not found',
      yourChoice: 'your choice',
      several: 'more than one',
      chosen: 'chosen',
      candidates: '{n} files found',
    },
    none: {
      steamNotFound:
        'We can’t find Steam on this computer: that is where we start looking for the game and its saves.',
      gameNotFound:
        'Steam is installed, but the game is not in any of its libraries: without the game folder we cannot find the saves.',
      noSaves:
        'The game is installed, but we found no saves in the usual places.',
      noSavesInChosenFolder:
        'There is no save in the folder you picked. The game names them `rep_persistentgamedata1.dat` or `rep+persistentgamedata1.dat`: if they are not there, the right folder is a different one.',
    },
    diagnostics: {
      steamNotFound: 'Steam: no installation found',
      gameNotFound: 'Game: not in Steam’s libraries',
      noSavesFound: 'Saves: no file in the usual places',
      noSavesInChosenFolder: 'Saves: no file in the folder you picked',
      unreadablePath: 'Unreadable path · {name} · {reason}',
      malformedManifest: 'Unreadable Steam manifest · {name}',
    },
    sources: {
      steamCloud: 'Steam Cloud',
      documents: 'Documents',
      manual: 'Chosen by hand',
    },
    active: {
      title: 'Active save',
      autoSelected: 'chosen automatically · it was the only one',
      modified: 'Modified',
      size: 'Size',
      dlcs: 'DLC',
      foundIn: 'Found in {source}',
      change: 'Change save',
      reload: 'Read the file again',
      unknownDate: 'unknown',
      bytes: '{count} bytes',
    },
    read: {
      title: 'What we read',
      sections: 'sections',
      note: 'Some parts of the save are not decoded yet, and the numbers change with the game’s patches. What you see here and across the app always comes from your own file, never from our estimates.',
      diagnostics: 'The file contains unexpected data',
    },
    saveDiagnostics: {
      unexpectedKind: 'A section is not the expected one ({expected}, {found})',
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
      title: 'We can’t read the save',
      retry: 'Try again',
    },
  },
  table: {
    actions: 'Actions',
  },
  queue: {
    inQueue: 'queued',
    inPlan: 'already in the Plan’s queue',
    add: 'Add to the queue',
    remove: 'Remove from the queue',
    removeShort: 'Remove from the queue',
    errorTitle: 'The queue did not change',
  },
  // Why a command could not answer. Since N2 these are variants on the wire, not a sentence
  // built in Rust: the numbers arrive as numbers and the wording lives here.
  ipcReasons: {
    ioNotFound: 'The file does not exist.',
    ioPermissionDenied: 'Windows does not allow us to open it.',
    ioOther: 'The system could not open it.',
    saveTooShort: 'It is too short to be a save.',
    saveBadMagic: 'It does not look like an Isaac save.',
    settingsConfigDirUnknown: 'Windows does not say where to store settings.',
    settingsEncoding: 'We could not write the settings.',
    storeDataDirUnknown: 'Windows does not say where to store app data.',
    storeDataDirNotCreatable: 'We could not create the app’s folder.',
    storeUnreadable: 'The file does not open, or is not a database.',
    storeNewerSchema:
      'It comes from a newer version of the app ({found} against {supported}).',
    storeQueueUnparseable: 'We could not read the saved plan.',
    autostartWriteRefused:
      'Windows refused the entry: a system policy or an antivirus may be blocking it. IsaacDome will not start at login.',
    autostartWriteIgnored:
      'The entry was written, but Windows keeps blocking it. Check Task Manager, under Startup apps: if IsaacDome is disabled there, it has to be re-enabled there.',
  },
  ipcErrors: {
    noBackend: 'The app did not respond.',
    noActiveProfile: 'No save chosen.',
    unknownProfile: 'The chosen save no longer exists.',
    unreadableSave: 'We could not read the save.',
    settingsNotWritable: 'We could not save your choice.',
    unknownTarget: 'What you are looking for does not exist.',
    catalogUnavailable: 'The game’s catalog is not available.',
    storeUnavailable: 'The app’s database is not available.',
    wikiUnavailable: 'The wiki is not available.',
    sessionTooLarge: 'The tab session is too large to be saved.',
    autostartNotWritable: 'Windows did not accept the start-at-login entry.',
    updateNotReady: 'There is no downloaded update to install.',
    folderNotOpenable: 'Explorer would not open on this folder.',
  },
}
