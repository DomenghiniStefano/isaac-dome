// Italian is the schema: en.ts is typed against it, so a key missing from English is a
// compile error, not a review note. Item, character and boss names are data, not
// messages: they stay in English and never appear here.
export const it = {
  ui: {
    close: 'Chiudi',
    explain: 'Cosa significa',
  },
  // The filter bar's own words, shared by every list that has one. They were written once per
  // screen until 3.10, with the same values three times over: that is how two of them end up
  // disagreeing. What stays a screen's own is the noun for its rows and what its search reads.
  filters: {
    more: 'Altri filtri',
    fewer: 'Meno filtri',
    active: 'filtri attivi',
    reset: 'Azzera i filtri',
    inMenu: 'filtra i valori',
    noMatch: 'Nessun valore corrisponde.',
  },
  shell: {
    newTab: 'Nuova tab',
    closeTab: 'Chiudi tab',
    back: 'Indietro',
    forward: 'Avanti',
    minimize: 'Riduci a icona',
    maximize: 'Ingrandisci',
    closeWindow: 'Chiudi finestra',
    search: 'Cerca ovunque',
    settings: 'Impostazioni',
    about: 'Informazioni',
    resizeSidebar: 'Ridimensiona la barra laterale',
    collapseSidebar: 'Riduci la barra laterale',
    expandSidebar: 'Espandi la barra laterale',
    sections: {
      progress: 'Progressi',
      tool: 'Strumenti',
      wiki: 'Wiki',
    },
  },
  marks: {
    wonOnline: 'online',
  },
  routes: {
    search: 'Cerca',
    goals: 'Obiettivi',
    completion: 'Completamento',
    unlock: 'Unlock',
    collection: 'Collezione',
    challenges: 'Sfide',
    roll: 'Stasera',
    runs: 'Run',
    live: 'Live',
    floor: 'Mappa del piano',
    wiki: 'Wiki',
    profile: 'Profilo di gioco',
    appearance: 'Aspetto',
    background: 'Background',
    tabsSettings: 'Tab',
    updates: 'Aggiornamenti',
    data: 'Dati',
  },
  wikiCategories: {
    items: 'Oggetti',
    trinkets: 'Trinket',
    achievements: 'Achievement',
    bosses: 'Boss',
    challenges: 'Sfide',
    characters: 'Personaggi',
    transformations: 'Trasformazioni',
    monsters: 'Mostri',
    cardsAndRunes: 'Carte e Rune',
    pickups: 'Oggetti raccoglibili',
    stages: 'Piani',
    versions: 'Versioni',
  },
  sidebar: {
    progressTitle: 'Progressi',
    progressHint: 'Tutto calcolato sul tuo salvataggio.',
    toolTitle: 'Strumenti',
    toolHint:
      'Non serve un salvataggio: lavorano sulla partita in corso o su quello che disegni.',
    wikiTitle: 'Wiki',
    wikiHint: 'Consultabile anche senza il gioco installato.',
    wikiOverview: 'Panoramica',
    settingsTitle: 'Impostazioni',
    settingsHint:
      'Il salvataggio da leggere, l’aspetto e il comportamento dell’app.',
  },
  live: {
    intro:
      'La run che stai giocando, in tempo reale, e cosa sblocchi se la porti a termine. Tieni IsaacDome aperta mentre giochi: il gioco azzera il suo log a ogni avvio.',
    floors: 'piani',
    collected: 'raccolti',
    wouldOpen: 'Se finisci questa run',
    missing: '{n} marchi da prendere',
    secondLevel: {
      hard: 'in hard',
      ultraGreedier: 'in Ultra Greedier',
    },
    noCondition: 'condizione non indicata dal gioco',
    column: {
      achievement: 'Achievement',
      cell: 'Marchio richiesto',
      condition: 'Come si ottiene',
      opens: 'Sblocca',
    },
    marks: 'I marchi del personaggio in gioco',
    items: 'I tuoi oggetti',
    startingItems: 'Di partenza',
    collectedItems: 'Raccolti',
    unlockedHere: 'Sbloccati in questa run',
    opens: 'sblocca {count}',
    opensNothingMore: 'non sblocca altro',
    nothing:
      'Questa run da sola non sblocca nuovi achievement: quelli che ti mancano richiedono i marchi di altri personaggi.',
    diagnostic: {
      noRun:
        'Nessuna run in corso. Avvia una partita: IsaacDome la segue in automatico.',
      characterNotNamed:
        'Il personaggio comparirà appena raccogli il primo oggetto.',
      unknownCharacter:
        'Il personaggio «{name}» non risulta nel catalogo del gioco.',
      ambiguousCharacter:
        'Il log indica «{name}», un nome che il gioco usa per {forms} personaggi: la forma base e quella Tainted. Qui sotto li trovi entrambi.',
      noGraph: 'Installa il gioco per vedere cosa sbloccheresti.',
      noProfile: 'Scegli un salvataggio per vedere cosa ti manca.',
      saveUnreadable:
        'Non riusciamo a leggere il salvataggio scelto, quindi non sappiamo cosa ti manca. La run in corso resta visibile.',
    },
  },
  floor: {
    intro:
      'Disegna il piano come lo vedi sulla minimappa: le celle vuote si illuminano dove le regole del gioco consentono una stanza segreta, super segreta o ultra segreta.',
    clear: 'Svuota la griglia',
    clearConfirm: {
      title: 'Svuotare la griglia?',
      body: 'Il piano che hai disegnato verrà cancellato e non potrà essere recuperato.',
      cancel: 'Annulla',
      confirm: 'Svuota',
    },
    neighbours: '{n} stanze adiacenti',
    empty: 'vuota',
    cell: 'Riga {row}, colonna {column}: {room}',
    at: 'Riga {row}, colonna {column}',
    rooms: 'Stanze',
    startRoomMissing: 'Manca la stanza di partenza',
    cellCandidate: '{cell} — {target}, posto {rank}',
    move: {
      title: 'Sposta il disegno',
      note: 'Le frecce spostano tutto il disegno di una cella, senza doverlo rifare. Servono quando la stanza di partenza è troppo vicina a un bordo e la mappa non ci sta. Una freccia grigia indica che una stanza tocca già quel bordo.',
      left: 'Sposta tutto a sinistra',
      up: 'Sposta tutto in alto',
      down: 'Sposta tutto in basso',
      right: 'Sposta tutto a destra',
    },
    rank: {
      title: 'Cosa indicano i colori',
      note: 'Più il quadrato è pieno, più il punto è probabile. Oltre il terzo posto le regole non fanno distinzioni.',
    },
    // What each rule of `crates/floor/rules/placement.json` says, keyed by its id in camel case
    // (`lib/floor/ruleText.ts`). In English it is the wiki's quote, word for word.
    ruleText: {
      secretNeighbours:
        'Una stanza segreta ha la stessa probabilità di trovarsi in un punto valido con 3 stanze adiacenti che con 4.',
      secretNeighboursTwo:
        'I punti con 2 stanze adiacenti sono rari ma possibili, anche quando ce ne sono con 3 o più.',
      secretNeighboursOne:
        'I punti con una sola stanza adiacente capitano solo se non esiste nessun punto valido con 3 o più adiacenti, e sono molto rari.',
      secretForbiddenNeighbours:
        'Una stanza segreta può stare accanto a stanze di ogni tipo, tranne le stanze del boss, le super segrete e altre stanze segrete.',
      superSecretDeadEnd:
        'Una stanza super segreta sta sempre accanto a una sola altra stanza.',
      superSecretNeighbourNotSpecial:
        'La stanza accanto non può essere una stanza speciale; in altre parole, la super segreta occupa uno dei vicoli ciechi del piano, come ogni altra stanza speciale.',
      superSecretNotNextToSecret:
        'Non può essere collegata alla stanza segreta.',
      superSecretSecondLongest:
        'La super segreta prende il posto del vicolo cieco che, dalla stanza di partenza, richiede di attraversare il secondo numero più alto di stanze per arrivarci.',
      ultraSecretConnections:
        "Un'ultra segreta compare più facilmente nei punti che, attraverso le stanze rosse adiacenti, si collegano a 3 o più stanze non rosse (i quadrati diversi di una stanza a L contano come 2).",
      ultraSecretConnectionsTwo:
        'Può collegarsi anche a 2 o a 1 stanza non rossa attraverso le stanze rosse adiacenti, ma un punto specifico da 3 o più stanze è 11,5 volte più probabile di un punto specifico da 2.',
      ultraSecretConnectionsOne:
        "Se non c'è nessun punto da 3 o più stanze, un punto specifico da 2 stanze è 11,5 volte più probabile di un punto specifico da 1.",
      ultraSecretNotConnected:
        "Le ultra segrete sono stanze speciali che non sono collegate direttamente a nessun'altra stanza della mappa.",
      ultraSecretRedRoomInvalid:
        "Un'ultra segreta non può collegarsi a stanze rosse che confinano con stanze segrete, super segrete o maledette, né stare in un punto dove una delle stanze rosse adiacenti non sarebbe valida, per esempio accanto a una stanza del boss.",
      ultraSecretShapes:
        'Non accanto ai lati delle stanze strette, o di qualsiasi stanza che su quel lato non può aprire una stanza rossa; sono però ammessi i punti sul bordo della griglia 13x13 dove una stanza rossa si aprirebbe normalmente su una stanza I AM ERROR.',
    },
    // Why the grid cannot judge a rule, for the rules that can reach the screen unjudged.
    ruleNote: {
      superSecretSecondLongest:
        'Manca la stanza di partenza: senza, non si possono contare le stanze da attraversare.',
      ultraSecretShapes:
        'Dipende dalla forma della stanza su quel lato, e su questa griglia ogni stanza occupa un solo quadrato: le stanze a L e quelle strette non si possono disegnare, quindi questa regola non si può verificare.',
    },
    unresolved: 'Regole che la griglia non può verificare',
    none: 'Con quello che hai disegnato finora, nessuna cella è compatibile con le regole.',
    failed:
      'Non siamo riusciti a calcolare le posizioni possibili. Il tuo disegno resta com’è.',
    target: {
      secret: 'Stanza segreta',
      superSecret: 'Super segreta',
      ultraSecret: 'Ultra segreta',
    },
    targetShort: {
      secret: 'Segreta',
      superSecret: 'Super',
      ultraSecret: 'Ultra',
    },
    room: {
      start: 'Partenza',
      normal: 'Normale',
      boss: 'Boss',
      treasure: 'Tesoro',
      shop: 'Negozio',
      curse: 'Maledetta',
      challenge: 'Sfida',
      sacrifice: 'Sacrificio',
      arcade: 'Sala giochi',
      library: 'Biblioteca',
      miniboss: 'Miniboss',
      secret: 'Segreta',
      superSecret: 'Super segreta',
      ultraSecret: 'Ultra segreta',
    },
    diagnostic: {
      gridEmpty: 'Disegna almeno una stanza per iniziare.',
      noStartRoom:
        'Segna la stanza di partenza: senza, la posizione della super segreta si può valutare solo in parte.',
      rulesUnreadable:
        'Le regole di piazzamento non si sono caricate: è un errore dell’app, non del tuo disegno.',
      gridMalformed:
        'La griglia ricevuta non è valida: {cells} celle invece di 169.',
    },
  },
  runs: {
    intro:
      'Lo storico delle tue run: quelle di questa sessione e quelle delle partite online registrate dal gioco. Il log non riporta l’ora, quindi le run sono ordinate per sessione.',
    rows: 'run',
    search: 'cerca un seed o un personaggio',
    facet: {
      outcome: 'Esito',
      character: 'Personaggio',
      online: 'Modalità',
      source: 'Origine',
    },
    outcome: {
      won: 'vinta',
      died: 'persa',
      abandoned: 'abbandonata',
      open: 'in corso',
    },
    online: {
      online: 'online',
      solo: 'da solo',
    },
    source: {
      live: 'questa sessione',
      session: 'sessione online',
    },
    totals: {
      runs: 'run',
      won: 'vinte',
      died: 'perse',
      abandoned: 'abbandonate',
    },
    column: {
      character: 'Personaggio',
      outcome: 'Esito',
      floors: 'Piani',
      seed: 'Seed',
      source: 'Origine',
    },
    noCharacter: 'personaggio sconosciuto',
    noItems: 'nessuno',
    openPage:
      'Clic per aprire la pagina, Ctrl+clic per aprirla in una nuova tab',
    itemId: 'oggetto {id}',
    killedBy: 'uccisa da {killer}',
    endedWith: 'finale {ending}',
    startingItems: 'Oggetti iniziali',
    collected: 'Raccolti',
    heldActive: 'Attivo in mano',
    unnamedItem: 'oggetto {id}',
    empty:
      'Ancora nessuna run nello storico: gioca una partita con IsaacDome aperta e la vedrai comparire qui.',
    noMatch: 'Nessuna run con questi filtri.',
    diagnostic: {
      noLogFolder:
        'Non troviamo la cartella dei log del gioco, quindi non ci sono run da mostrare.',
      storeUnavailable:
        'Non riusciamo ad aprire il database dell’app, quindi lo storico non è disponibile.',
      unreadableEvents:
        '{count} righe del log non sono state riconosciute: alcune run potrebbero essere incomplete.',
      unreadableSessions:
        '{count} sessioni online non sono state lette, e le loro run mancano dallo storico.',
      liveLogUnreadable:
        'Non riusciamo a leggere il log della partita in corso: la run attuale non compare nello storico.',
      noCatalog:
        'Il gioco non è installato: gli oggetti compaiono con il loro numero invece del nome.',
    },
  },
  challenges: {
    intro:
      'Tutte le sfide del gioco: quali hai completato, cosa serve per sbloccarle e cosa ti danno in cambio. Personaggio, traguardo e bendatura vengono dalla wiki; tutto il resto è nella pagina di ogni sfida.',
    rows: 'sfide',
    search: 'cerca una sfida o il suo numero',
    state: {
      done: 'completata',
      available: 'da fare',
      blocked: 'bloccata',
      unknown: 'non leggibile',
    },
    blockedBy: 'requisiti mancanti: {count}',
    facet: {
      state: 'Stato',
      character: 'Personaggio',
      rewards: 'Ricompensa',
      blindfolded: 'Bendata',
    },
    rewardsSome: 'con ricompensa',
    rewardsNone: 'senza ricompensa',
    blindfoldedYes: 'bendata',
    blindfoldedNo: 'non bendata',
    blindfolded: 'bendata',
    noCondition: 'non indicato dalla wiki',
    unlocksNothing: 'non sblocca niente',
    unnamedReward: 'achievement {id}',
    columns: {
      number: '#',
      challenge: 'Sfida',
      character: 'Personaggio',
      goal: 'Traguardo',
      state: 'Stato',
    },
    empty: 'Nessuna sfida da mostrare.',
    noResults: 'Nessuna sfida con questi filtri.',
    diagnostics: {
      noCatalogTitle: 'Non troviamo il gioco',
      noCatalog:
        'Le sfide sono descritte nei file del gioco: senza, sappiamo quante ne hai completate ma non quali. Installa The Binding of Isaac da Steam e riapri l’app.',
      noChallengesSectionTitle: 'Non sappiamo quali sfide hai completato',
      noChallengesSection:
        'Non siamo riusciti a leggere la parte del salvataggio con le sfide: le trovi tutte qui sotto, ma senza sapere quali hai già completato.',
      noAchievementSectionTitle: 'Non sappiamo cosa hai sbloccato',
      noAchievementSection:
        'Senza gli achievement non possiamo sapere quali sfide sono già sbloccate, né quali ricompense hai già ottenuto.',
      noWiki:
        'I dati della wiki non si sono caricati: le sfide sono elencate senza personaggio, traguardo e bendatura.',
    },
  },
  roll: {
    intro:
      'Non sai cosa giocare stasera? Pesca un obiettivo dalla matrice dei marchi e lascia decidere al caso.',
    draw: 'Pesca',
    drawAgain: 'Pesca di nuovo',
    notDrawnYet: 'Premi «Pesca» per scoprire cosa giocare stasera.',
    status: {
      missing: 'da fare',
      taken: 'già preso',
      unreadable: 'non leggibile',
    },
    card: {
      mark: '{character} contro {column}',
      greedier: '{character} — Greedier!',
      drawnFrom: 'Pescato tra {size} obiettivi possibili',
    },
    emptyDeck: {
      taken:
        'Il mazzo è vuoto: {count} obiettivi sono esclusi perché già presi.',
      unreadable:
        'Il mazzo è vuoto: non riusciamo a leggere {count} obiettivi dal salvataggio.',
      locked:
        'Il mazzo è vuoto: {count} obiettivi richiedono un personaggio che non hai ancora sbloccato.',
      filtered:
        'Il mazzo è vuoto: {count} obiettivi sono esclusi dai filtri qui sotto.',
      nothing:
        'Il mazzo è vuoto: al momento non c’è nessun obiettivo da proporre.',
    },
    panel: {
      characters: 'Personaggi',
      columns: 'Colonne',
      includeTaken: 'Includi gli obiettivi già presi',
      includeTakenHint: 'Altrimenti si pesca solo tra quelli ancora da fare.',
      onlyPlayable: 'Solo personaggi già sbloccati',
      onlyPlayableHint: 'Esclude i personaggi che non puoi ancora scegliere.',
    },
    diagnostics: {
      noCounterSectionTitle: 'Non sappiamo quali marchi hai preso',
      noCounterSection:
        'Non siamo riusciti a leggere la parte del salvataggio con i marchi: la matrice c’è, ma ogni cella risulta non leggibile.',
      documentUnreadableTitle: 'Le tue preferenze non si sono caricate',
      documentUnreadable:
        'Le preferenze salvate non erano leggibili: abbiamo ripristinato quelle predefinite.',
      documentFromTheFutureTitle:
        'Le tue preferenze vengono da una versione più recente',
      documentFromTheFuture:
        'Sono salvate nel formato {version}, e questa versione dell’app legge fino al {supported}: per ora usiamo le preferenze predefinite.',
      noCatalogTitle: 'Non troviamo il gioco',
      noCatalog:
        'Senza il gioco installato mancano nomi e simboli, e non possiamo sapere quali personaggi hai sbloccato.',
      playabilityUnknownTitle: 'Non sappiamo quali personaggi hai sbloccato',
      playabilityUnknown:
        'Il filtro «solo personaggi già sbloccati» è stato disattivato per questa pescata: senza il catalogo del gioco o gli achievement non possiamo verificarlo.',
      storeUnavailableTitle: 'Non riusciamo a salvare le tue preferenze',
    },
  },
  collection: {
    intro:
      'Gli oggetti che ti mancano, con la loro qualità e i pool in cui compaiono. I trinket non sono inclusi: il gioco non registra quali hai trovato.',
    state: {
      inCollection: 'in collezione',
      available: 'da trovare',
      locked: 'bloccato',
      unknown: 'non leggibile',
    },
    facet: {
      state: 'Stato',
      quality: 'Qualità',
      pool: 'Pool',
      kind: 'Tipo',
      origin: 'DLC di origine',
    },
    qualityUnrated: 'senza qualità',
    poolNone: 'nessun pool',
    items: 'oggetti',
    search: 'cerca un oggetto',
    sortBy: 'ordina per',
    sort: {
      quality: 'qualità',
      id: 'id',
      name: 'nome',
    },
    columns: {
      item: 'Oggetto',
      quality: 'Qualità',
      pools: 'Pool',
      origin: 'DLC',
      state: 'Stato',
    },
    id: 'id',
    lockedBy: 'si sblocca con',
    achievement: 'achievement',
    empty: 'Nessun oggetto da mostrare.',
    noResults: 'Nessun oggetto con questi filtri.',
    diagnostics: {
      noCatalogTitle: 'Non troviamo il gioco',
      noCatalog:
        'Senza i file del gioco gli oggetti non hanno nome né qualità: sappiamo quanti ne hai, ma non quali. Installa The Binding of Isaac da Steam e riapri l’app.',
      noCollectionSectionTitle: 'Non sappiamo quali oggetti hai raccolto',
      noCollectionSection:
        'Non siamo riusciti a leggere la parte del salvataggio con gli oggetti, quindi risultano tutti non leggibili. Non significa che non li hai trovati.',
      noAchievementSectionTitle: 'Non sappiamo cosa hai sbloccato',
      noAchievementSection:
        'Non possiamo sapere quali oggetti sono ancora bloccati: quelli legati a un achievement restano incerti.',
      itemsBeyondSlots:
        '{count} oggetti del gioco non compaiono in questo salvataggio: li mostriamo come non leggibili',
    },
  },
  wiki: {
    intro:
      'La wiki di Isaac, integrata nell’app e collegata ai tuoi progressi: oggetti, personaggi, boss, sfide, achievement, trasformazioni e molto altro. Sempre disponibile, anche offline e senza il gioco installato.',
    provenance: {
      title: 'Fonte',
      snapshot: 'Aggiornata al',
      patch: 'Ultima patch coperta',
      patchUnknown: 'sconosciuta',
      newerGame:
        'Il tuo gioco è più recente di questa versione della wiki: le ultime novità potrebbero mancare.',
      license: 'wiki.gg · CC BY-SA 4.0',
      licenseLong:
        'Testi della wiki con licenza CC BY-SA 4.0, da bindingofisaacrebirth.wiki.gg',
      unresolved: 'riferimenti non risolti',
      unknownTemplates: 'template sconosciuti',
    },
    categories: 'Categorie',
    pages: '{n} pagine',
    pagesOf: '{shown} / {total} pagine',
    noCatalog:
      'Senza il gioco installato le pagine non hanno immagini: le immagini vengono dalla tua installazione di Isaac.',
    search: 'cerca una pagina',
    noResults: 'Nessuna pagina trovata.',
    emptyCategory: 'Questa categoria non ha pagine.',
    resetFilters: 'Azzera la ricerca',
    back: 'Torna alla categoria',
    id: 'id {id}',
    list: {
      rows: 'pagine',
      sortBy: 'ordina per',
      noData: 'Nessun dato',
      qualityUnrated: 'Senza qualità',
      baseForm: 'Forma base',
      progressOf: '{done} / {total}',
      grid: 'Schede',
      table: 'Tabella',
      ascending: 'Crescente',
      descending: 'Decrescente',
      facet: {
        profile: 'I tuoi progressi',
        edition: 'Edizione',
        quality: 'Qualità',
        template: 'Tipo',
        tag: 'Etichetta',
        tainted: 'Tainted',
        articleCategory: 'Carta o Runa',
      },
      sort: {
        name: 'Nome',
        id: 'Id',
        edition: 'Edizione',
      },
    },
    kind: {
      item: 'Oggetto',
      trinket: 'Trinket',
      achievement: 'Achievement',
      boss: 'Boss',
      challenge: 'Sfida',
      character: 'Personaggio',
      transformation: 'Trasformazione',
      monster: 'Mostro',
      cardOrRune: 'Carta o Runa',
      pickup: 'Oggetto raccoglibile',
      stage: 'Piano',
      version: 'Versione',
    },
    revision: 'rev. {revision}',
    qualityChip: 'Qualità {quality}',
    addedIn: 'Aggiunto in {edition}',
    removedIn: 'Rimosso in {edition}',
    outline: 'In questa pagina',
    section: {
      effects: 'Effetti',
      notes: 'Note',
      synergies: 'Sinergie',
      interactions: 'Interazioni',
      bugs: 'Bug',
      behavior: 'Comportamento',
      championVersions: 'Versioni campione',
      damageScaling: 'Scalatura del danno',
      strategies: 'Strategie',
      difficulty: 'Difficoltà',
      reward: 'Ricompensa',
      unlockable: 'Sbloccabile',
      startingItems: 'Oggetti iniziali sbloccabili',
      other: 'Altro',
    },
    infobox: {
      title: 'Scheda',
      description: 'Descrizione',
      requirements: 'Requisiti',
      notes: 'Note',
      unlocks: 'Sblocca',
      unlockedBy: 'Sbloccato da',
      baseHp: 'Vita base',
      environment: 'Dove',
      behavior: 'Comportamento',
      pool: 'Pool',
      replace: 'Sostituito da',
      replaceChance: 'Probabilità di sostituzione',
      replaceNotes: 'Note sulla sostituzione',
      goal: 'Obiettivo',
      items: 'Oggetti',
      trinkets: 'Trinket',
      pickups: 'Oggetti raccoglibili',
      health: 'Salute',
      curse: 'Maledizione',
      restrictions: 'Restrizioni',
      blindfolded: 'bendato',
      noShops: 'senza negozi',
      noTreasureRooms: 'senza stanze del tesoro',
      noRestrictions: 'nessuna restrizione',
      damage: 'Danno',
      tears: 'Lacrime',
      range: 'Gittata',
      speed: 'Velocità',
      luck: 'Fortuna',
      shotSpeed: 'Velocità del colpo',
      collectibles: 'Oggetti iniziali',
      requires: 'Oggetti richiesti',
      contributors: 'Oggetti che contano',
      target: 'Agisce su',
      quality: 'Qualità',
      recharge: 'Ricarica',
      devilPrice: 'Prezzo dal diavolo',
      shopPrice: 'Prezzo in negozio',
      pools: 'Pool',
      obtainedFrom: 'Si ottiene da',
      tags: 'Etichette',
      none: 'nessuno',
    },
    facts: {
      activated: 'Attivo',
      passive: 'Passivo',
      recharge: 'Ricarica {value}',
      shopPrice: 'Prezzo in negozio {value}',
      devilPrice: 'Prezzo dal diavolo {value}',
      tag: '{value}',
      requirement: 'Requisito: {value}',
      baseHp: 'Vita base {value}',
      floors: 'Piani {value}',
      character: 'Personaggio {value}',
      goal: 'Obiettivo {value}',
      curse: 'Maledizione {value}',
      health: 'Salute {value}',
      damage: 'Danno {value}',
      tears: 'Lacrime {value}',
      range: 'Gittata {value}',
      speed: 'Velocità {value}',
      luck: 'Fortuna {value}',
      shotSpeed: 'Velocità del colpo {value}',
      tainted: 'Tainted',
      requires: 'Oggetti richiesti {value}',
      contributors: 'Oggetti che contano {value}',
      category: 'Categoria',
      categoryCard: 'Carta',
      categoryRune: 'Runa',
      versionNumber: 'Versione {value}',
      versionDate: 'Uscita il {value}',
      template: 'Tipo',
    },
    progress: {
      done: 'Fatto',
      notDone: 'Da fare',
      itemCollected: 'Raccolto',
      itemNotCollected: 'Non raccolto',
      unlocked: 'Sbloccato',
      locked: 'Bloccato',
      lockedBy: 'Si sblocca con l’achievement {id}',
      marks: 'Marchi {done}/{total}',
      available: 'Disponibile',
      blocked: 'Bloccato ({count} mancanti)',
      unknownState: 'Sconosciuto',
      bestiaryMet: 'Incontrato {count}',
      bestiaryKilled: 'Ucciso {count}',
      bestiaryKilledYou: 'Ti ha ucciso {count}',
    },
    landing: {
      totalPages: 'Pagine',
      overallProgress: 'Progresso complessivo',
      categoryProgress: 'Progresso in {category}',
      progressOf: '{done} / {total}',
    },
    states: {
      unknownTitle: 'Pagina non trovata',
      unknown: 'Questa pagina non è ancora nella wiki di IsaacDome.',
      unknownHint:
        'Potrebbe essere stata aggiunta di recente: la wiki è aggiornata alla data indicata qui sopra e si aggiorna insieme all’app.',
      noSections: 'Per questa pagina c’è solo la scheda, senza testo.',
      failedTitle: 'La Wiki non risponde',
      pageFailedTitle: 'Questa pagina non si è aperta',
      missingTitle: 'La Wiki non si è aperta',
      missing:
        'Non riusciamo a caricare la wiki, quindi non ci sono pagine da mostrare. Di solito basta riavviare l’app; se il problema continua, segnalacelo.',
    },
  },
  search: {
    intro:
      'Cerca in tutta l’app: schermate, oggetti, achievement e il testo di ogni pagina della wiki.',
    placeholder: 'Cerca schermate, achievement, oggetti, pagine wiki…',
    empty: 'Nessun risultato.',
    allResults: 'Tutti i risultati ({count})',
    shown: '{shown} righe',
    limit: 'Mostrati {shown} di {total}: affina la ricerca.',
    groups: {
      screens: 'Schermate',
      wiki: 'Wiki',
      unlock: 'Unlock',
      collection: 'Collezione',
    },
    progress: {
      done: 'fatto',
      pending: 'da fare',
    },
    hint: {
      open: 'apri',
      newTab: 'apri in una nuova tab',
    },
    diagnostics: {
      noProfileTitle: 'Nessun salvataggio scelto',
      noCatalogTitle: 'Gioco non installato',
      noWikiTitle: 'La Wiki non si è aperta',
      noAchievementSectionTitle: 'Non sappiamo cosa hai sbloccato',
      noCollectionSectionTitle: 'Non sappiamo quali oggetti hai raccolto',
      noProfile:
        'I risultati non mostrano i tuoi progressi: scegli un salvataggio nelle impostazioni.',
      noCatalog:
        'La ricerca copre solo la wiki, senza immagini, e i risultati non aprono Unlock o la Collezione.',
      noWiki:
        'La ricerca copre solo i nomi del gioco, e i risultati non aprono pagine della wiki.',
      noAchievementSection:
        'Accanto agli achievement non compare «fatto» o «da fare»: non riusciamo a leggere quella parte del salvataggio.',
      noCollectionSection:
        'Accanto agli oggetti non compare se li hai già: non riusciamo a leggere quella parte del salvataggio.',
    },
  },
  find: {
    label: 'Cerca in questa pagina',
    placeholder: 'Cerca in questa pagina',
    count: '{position} di {total}',
    empty: 'Nessuna corrispondenza',
    previous: 'Risultato precedente',
    next: 'Risultato successivo',
    close: 'Chiudi la ricerca',
    nothingToSearch: 'Qui non c’è niente da cercare.',
  },
  background: {
    intro:
      'Quando si avvia IsaacDome, e cosa succede quando chiudi l’ultima finestra.',
    startTitle: 'Avvia con Windows',
    startHint:
      'Tieni IsaacDome aperta mentre giochi, così non perdi nessuna run: il gioco azzera il suo log a ogni avvio. Una sessione giocata ad app chiusa si recupera ancora, ma solo finché non riavvii il gioco.',
    startDev:
      'Non disponibile nelle build di sviluppo: registrerebbe all’accesso un percorso temporaneo che smetterebbe presto di funzionare.',
    startWithoutBackground:
      'Con «Resta aperta in background» disattivata, all’accesso IsaacDome parte come icona accanto all’orologio, senza finestre; quando chiudi la prima finestra che apri, l’app si chiude e smette di seguire le partite.',
    startFailedTitle: 'Windows non ha accettato l’avvio automatico',
    stayTitle: 'Resta aperta in background',
    stayHint:
      'Quando chiudi l’ultima finestra, IsaacDome resta attiva nell’area di notifica accanto all’orologio: basta un clic sull’icona per riaprirla. Se disattivata, chiudere l’ultima finestra chiude l’app.',
    saveFailedTitle: 'L’impostazione non è stata salvata',
    saveFailed:
      'La modifica è già attiva, ma non siamo riusciti a salvarla: al prossimo avvio tornerà come prima.',
  },
  data: {
    intro:
      'I due file che IsaacDome scrive, e dove. Copiali per tenerne un backup; cancellali per ripartire da zero. Il gioco e i tuoi salvataggi non sono mai fra questi.',
    database: 'Database',
    databaseHint: 'I tuoi obiettivi, la coda, le run lette dal log e il tiro.',
    settings: 'Impostazioni',
    settingsHint:
      'Il profilo attivo, la scala, gli interruttori e le cartelle che hai scelto.',
    folder: 'Cartella',
    size: 'Dimensione',
    notCreated: 'Non ancora creato: non è stato salvato niente.',
    folderUnknown: 'Windows non ha detto quale cartella sia.',
    unreadable: 'Il file c’è e non si apre.',
    reveal: 'Mostra nella cartella',
    revealFailed: 'Non è stato possibile aprire la cartella',
    contents: {
      goals: 'Obiettivi: {count}',
      queue: 'In coda: {count}',
      queueUnknown: 'In coda: illeggibile',
      sessions: 'Sessioni di log lette: {count}',
      runs: 'Run: {count}',
      rollSaved: 'Preset del tiro salvato',
    },
  },
  updates: {
    intro:
      'La versione installata e gli aggiornamenti disponibili. IsaacDome scarica l’aggiornamento mentre la usi e lo installa quando vuoi tu.',
    currentVersion: 'Versione installata: {version}',
    autoTitle: 'Aggiorna automaticamente',
    autoHint:
      'Se attivo, a ogni avvio l’app controlla su GitHub se c’è una nuova versione e la scarica. Se disattivo, non parte nessuna richiesta: niente lascia il tuo computer finché non premi «Controlla ora».',
    unsupported: 'Questa build non si aggiorna da sola.',
    unsupportedHint:
      'Non disponibile nelle build di sviluppo: l’aggiornamento la sostituirebbe con una build di release.',
    idle: 'Nessun controllo eseguito da quando hai aperto l’app.',
    checking: 'Controllo in corso…',
    upToDate: 'Hai già la versione più recente.',
    downloading: 'Download della {version} in corso…',
    ready: 'La {version} è pronta da installare.',
    // Non è sfortuna come le altre: quello che è arrivato non è quello che dice di essere.
    failedRejected:
      'Aggiornamento rifiutato: la firma non corrisponde, e non è stato installato nulla. Scarica l’installer dalla pagina delle release invece di riprovare da qui.',
    failedOffline:
      'Impossibile raggiungere GitHub. Riprova quando sei di nuovo online.',
    failedNotPublished: 'Non ci sono ancora versioni pubblicate da scaricare.',
    failedInstall:
      'L’aggiornamento è stato scaricato, ma l’installazione non è partita. Il file è ancora disponibile: puoi riprovare.',
    failedUnknown: 'L’aggiornamento non è andato a buon fine.',
    failedTitle: 'Installazione non riuscita',
    notesTitle: 'Novità',
    check: 'Controlla ora',
    install: 'Riavvia e installa',
    installHint: 'Chiude l’app, installa la nuova versione e la riapre.',
    saveFailedTitle: 'L’impostazione non è stata salvata',
    saveFailed:
      'La modifica è già attiva, ma non siamo riusciti a salvarla: al prossimo avvio tornerà come prima.',
  },
  tabsSettings: {
    intro: 'Cosa ritrovi quando riapri IsaacDome.',
    resumeTitle: 'Riapri le tab dell’ultima sessione',
    resumeHint:
      'All’avvio ritrovi le finestre e le tab che avevi aperto. Se disattivata, l’app riparte dalla schermata iniziale e la sessione salvata viene cancellata subito.',
    keptTitle: 'Cosa viene salvato',
    keptWindows:
      'Le finestre che avevi aperto, con la loro posizione e dimensione.',
    keptTabs:
      'Le tab di ogni finestra, nel loro ordine, e quella in primo piano.',
    keptReading:
      'Lo stato di ogni tab: i filtri, l’ordinamento, la riga selezionata e il punto in cui eri arrivato.',
    keptSidebar: 'La larghezza della barra laterale.',
    stoppedTitle: 'La sessione non viene più salvata',
    stopped:
      'Ci sono troppe tab aperte per salvarle tutte: restano dove sono, ma non verranno riaperte al prossimo avvio. Chiudine qualcuna e poi aprine una qualsiasi: il salvataggio riprenderà.',
    saveFailedTitle: 'L’impostazione non è stata salvata',
    saveFailed:
      'La modifica è già attiva, ma non siamo riusciti a salvarla: al prossimo avvio tornerà come prima.',
  },
  appearance: {
    intro:
      'Scegli la dimensione dell’interfaccia: testo, icone, righe e finestra cambiano insieme, e la scelta resta tra un avvio e l’altro.',
    scaleTitle: 'Dimensione',
    scaleLabel: 'Dimensione dell’interfaccia',
    preview: 'Anteprima',
    previewHint:
      'Resta visibile mentre scorri: è l’app alla dimensione scelta.',
    shortcut: 'Da qualsiasi schermata:',
    shortcutReset: 'torna al 100%',
    saveFailedTitle: 'La dimensione non è stata salvata',
    saveFailed:
      'La dimensione è già attiva, ma non siamo riusciti a salvarla: al prossimo avvio tornerà come prima.',
    sample: {
      kpi: 'Achievement fatti',
      item: 'The Sad Onion',
      itemHint: 'oggetto passivo · qualità 2',
      button: 'Un pulsante',
      badge: 'sbloccabile ora',
    },
  },
  about: {
    fanMade:
      'IsaacDome è un progetto realizzato dai fan e non è affiliato, approvato o sponsorizzato da Nicalis né da Edmund McMillen.',
    version: 'Versione',
    versionUnknown: 'server di sviluppo',
    promisesTitle: 'Le nostre promesse',
    promises: {
      readOnlyTitle: 'I tuoi salvataggi non vengono mai modificati',
      readOnly:
        'IsaacDome legge i salvataggi e basta: il codice che apre i file .dat non contiene alcuna funzione di scrittura.',
      offlineTitle: 'Nessun account, server o telemetria',
      offline:
        'IsaacDome funziona offline, senza account, server o telemetria. La rete serve solo per gli aggiornamenti, e puoi disattivarli.',
      oneFileTitle: 'Scrive solo nelle sue cartelle',
      oneFile:
        'IsaacDome salva le impostazioni e il suo database, isaacdome.db, nelle proprie cartelle. Il gioco, i salvataggi e il resto del disco non vengono mai toccati.',
    },
    creditsTitle: 'Crediti e licenze',
    wikiText:
      'Il testo della wiki è distribuito con licenza CC BY-SA 4.0 e proviene da bindingofisaacrebirth.wiki.gg.',
    assets:
      'Le immagini del gioco non sono incluse nell’app: vengono estratte dalla tua installazione del gioco.',
    font: 'Il carattere Determination Mono è distribuito con licenza CC BY 3.0.',
  },

  plan: {
    opens: 'sblocca {count}',
    opensNothing: 'non sblocca altro',
    detail: 'Mostra i dettagli',
    queueCount: 'La tua coda',
    addPane: 'Obiettivi consigliati',
    hint: {
      idle: 'trascina per riordinare: ogni spostamento è consentito',
      dragging: 'rilascia dove vuoi: la coda si riordina da sola',
      stoppedUnder: 'si è fermata sotto «{name}», che è un suo prerequisito',
    },
    row: {
      move: 'Sposta la riga (Alt e freccia su o giù)',
      wanted: 'aggiunto da te',
      serves: 'serve per «{name}»',
      outsideQueue: 'prerequisiti non in coda',
    },
    achievementNumbered: 'achievement {id}',
    empty: 'La coda è vuota.',
    emptyHint: 'Aggiungi un obiettivo dai consigli qui accanto o da Unlock.',
    completed: {
      closed: 'completati giocando: {count}',
      closedWanted:
        'completati giocando: {count} · tra quelli aggiunti da te: {names}',
    },
    unresolved: 'achievement {id}: non esiste più nel gioco',
    alerts: {
      storeUnavailableTitle: 'Il piano non è disponibile',
      unreadableTitle: 'Questa versione non riesce a leggere la coda salvata',
      unreadable:
        'La coda resta salvata così com’è e non viene sovrascritta: una versione più recente dell’app potrà leggerla.',
      noCatalogTitle: 'Serve il gioco installato',
      noCatalog:
        'Senza i file del gioco non possiamo sapere a quale achievement corrisponde ogni riga, né cosa gli manca. La coda resta salvata e torna com’era appena il gioco è installato.',
      goalsPendingTitle: 'Obiettivi salvati da importare: {count}',
      goalsPending:
        'Obiettivi salvati con una versione precedente dell’app: importali per aggiungerli alla coda.',
      import: 'Importa nella coda',
    },
  },
  completion: {
    intro:
      'Tutti i personaggi e tutti i loro marchi di completamento, a colpo d’occhio.',
    headline: 'marchi presi in hard',
    kpi: {
      columns: 'boss completi',
      columnsExplain:
        'Boss battuti in hard da tutti i personaggi (Greed in Ultra Greedier), contando solo le celle leggibili. È lo stesso conto dei «personaggi completi», letto per colonna invece che per riga, ed è quello che disegna la carta qui accanto: un simbolo appare nella versione hard solo quando la sua colonna è piena.',
      normal: 'marchi in normale',
      normalExplain:
        'Celle con almeno un marchio, tra quelle leggibili. Un marchio preso in hard conta anche qui. Le celle non leggibili sono escluse dal totale.',
      hard: 'marchi in hard',
      hardExplain:
        'Celle con il marchio di secondo livello, sullo stesso totale: hard per i boss, Ultra Greedier per Greed. Non supera mai il numero accanto, perché ogni marchio hard vale anche come normale.',
      complete: 'personaggi completi',
      completeExplain:
        'Personaggi con tutte le celle leggibili in hard (Greed in Ultra Greedier): è la riga piena del gioco. Per chi ha celle non leggibili contano solo le altre.',
    },
    card: {
      title: 'Matrice dei marchi',
    },
    legend: {
      empty: 'mai fatto',
      normal: 'normale',
      hard: 'hard',
      online: 'online',
      unknown: 'non leggibile',
    },
    grid: {
      character: 'Personaggio',
      normal: 'normale',
      hard: 'hard',
      unreadable: '{n} non leggibili',
      columnTotals: 'Personaggi con il marchio',
      columnTotalsHard: 'Di cui in hard',
    },
    groups: {
      base: 'Personaggi base',
      tainted: 'Tainted',
    },
    cell: {
      empty: 'mai fatto',
      normal: 'normale',
      hard: 'hard',
      ultraGreedier: 'Ultra Greedier',
      unknown: 'non leggibile per questo personaggio',
      unexpected: 'valore inatteso:',
    },
    nothingReadable:
      'Non riusciamo a leggere nessun marchio da questo salvataggio: la parte che li contiene manca o è incompleta.',
  },
  // La schermata di atterraggio: cosa ottieni, come, perché conviene, e l'azione. L'intro
  // dice *cos'è* la lista, non come è stata calcolata: il calcolo sta nelle promesse della
  // finestra Informazioni, dove chi vuole lo trova (B32 §2).
  goals: {
    intro:
      'Cosa sbloccare e in che ordine: scegli dai consigli o cerca quello che vuoi, e aggiungilo alla coda.',
    fanOut: 'Sbloccano di più',
    closeness: 'Ci sei quasi',
    seeAll: 'Vedi tutti',
    noCatalogTitle: 'Non troviamo il gioco',
    noCatalog:
      'IsaacDome legge i file di The Binding of Isaac per sapere cosa sblocca ogni achievement, ma su questo computer non li trova. Installa il gioco da Steam e riapri l’app: qui troverai cosa conviene giocare stasera.',
    nothingNow:
      'Al momento non c’è niente da sbloccare: hai già fatto tutto, oppure ogni obiettivo richiede prima qualcos’altro.',
  },
  // B37: la stessa schermata, chiesta dall'altro capo. Dici cosa vuoi e la risposta è la
  // serie da giocare, nell'ordine in cui il Piano la giocherebbe.
  want: {
    placeholder:
      'Voglio… (un oggetto, un personaggio, una sfida, un achievement)',
    clear: 'Annulla',
    chain: 'Cosa devi giocare',
    availableNow: 'Puoi giocarlo adesso',
    done: 'Ce l’hai già',
    noProfile: 'Non sappiamo a che punto sei',
    wayOf: 'Percorso {index} di {total}',
    unknown:
      'Più {count} requisiti che non riusciamo a interpretare: il percorso potrebbe essere più lungo.',
    addAll: 'Aggiungi tutto al Piano',
    diagnostics: {
      noCatalog:
        'Senza i file del gioco non sappiamo cosa sblocca ogni achievement. Installa The Binding of Isaac e riapri l’app.',
      noProfile:
        'Nessun salvataggio scelto: possiamo dirti come si ottiene, ma non a che punto sei.',
      nothingUnlocks:
        'Nessun achievement lo sblocca: o è disponibile da subito, o si ottiene giocando.',
      notUnlockable: 'Questo non si sblocca.',
    },
  },
  // Il blocco che la pagina wiki di un achievement guadagna quando c'è un profilo attivo.
  // Dice soltanto dove sei tu: cos'è e cosa chiede lo dice già l'infobox sotto.
  profileBlock: {
    title: 'I tuoi progressi',
    missing: 'Cosa ti manca',
    unlocks: 'Cosa ottieni',
    opens: 'Sbloccarlo apre altri {count} contenuti.',
    // Un nodo già fatto non "aprirà": ha aperto. La stessa frase al futuro, sotto un
    // "Già fatto", si legge come se ci fosse ancora qualcosa da fare.
    opened: 'Ha sbloccato altri {count} contenuti.',
    opensNothing: 'Non sblocca nient’altro.',
    openedNothing: 'Non ha sbloccato nient’altro.',
    stepsMissing: 'Prima servono ancora {count} sblocchi.',
    done: 'Già fatto.',
  },
  graph: {
    state: {
      done: 'fatto',
      now: 'sbloccabile ora',
      blocked: 'requisiti mancanti: {count}',
      // Non "grafo parziale": chi legge non sa cosa sia un grafo, e la cosa che deve
      // sapere è che la risposta non c'è — non come mai. E deve dire *di cosa* non c'è:
      // "non sappiamo dirlo", da solo su un badge, non diceva cosa non sapevamo.
      partial: 'requisiti incerti',
    },
    why: {
      title: 'Cosa gli manca',
      character: 'Personaggi',
      boss: 'Boss',
      challenge: 'Sfide',
      item: 'Oggetti',
      gate: 'Condizioni',
      mark: 'Marchi di completamento',
      counter: 'Boss da battere',
      threshold: 'Trasformazioni',
      unknown: 'Condizioni che non riusciamo a interpretare',
    },
    kinds: {
      passive: 'oggetto passivo',
      active: 'oggetto attivo',
      familiar: 'famiglio',
      trinket: 'trinket',
      character: 'personaggio',
      boss: 'boss',
      challenge: 'sfida',
      nothing: 'nessuno sblocco indicato dal gioco',
    },
    stateName: {
      done: 'fatto',
      now: 'sbloccabile ora',
      blocked: 'bloccato',
      partial: 'requisiti incerti',
    },
    // Una cella della matrice: il boss e il personaggio con cui va battuto.
    markName: '{boss} con {character}',
    thresholdName: '{name} — {current} di {atLeast}',
    originNone: 'non indicata',
    unknownAchievement: 'Achievement sconosciuto',
    unknownAchievementInSlot: 'Achievement sconosciuto · slot {slot}',
    slot: 'slot {slot}',
    // La forma Tainted di un personaggio: il gioco scrive lo stesso nome per le due forme.
    // "Tainted" resta in inglese come ogni nome del gioco (DESIGN-BRIEF.md §12).
    taintedName: 'Tainted {name}',
  },
  unlock: {
    intro:
      'Tutti gli achievement del gioco, filtrabili come vuoi. Il filtro più utile è «sbloccabile ora».',
    rows: 'righe',
    search: 'cerca nome, condizione o cosa sblocca',
    sortBy: 'ordina per',
    sort: {
      fanOut: 'quanti ne sblocca',
      steps: 'passi mancanti',
      name: 'nome',
    },
    empty: 'Nessun achievement da mostrare.',
    noResults: 'Nessuna riga con questi filtri.',
    facet: {
      state: 'Stato',
      unlocks: 'Cosa sblocca',
      origin: 'DLC di origine',
      character: 'Personaggio richiesto',
    },
    columns: {
      achievement: 'Achievement',
      unlocks: 'Cosa sblocca',
      condition: 'Condizione',
      state: 'Stato',
      fanOut: 'Ne sblocca',
    },
    unlocksNothing: 'nessuno sblocco indicato dal gioco',
    noCondition: 'condizione non indicata dal gioco',
    diagnostics: {
      noCatalogTitle: 'Non troviamo il gioco',
      noCatalog:
        'Senza i file del gioco gli achievement non hanno nome né condizione: puoi vedere solo quali hai già fatto.',
      noAchievementSectionTitle: 'Non sappiamo cosa hai sbloccato',
      noAchievementSection:
        'Non riusciamo a leggere la parte del salvataggio con gli achievement: se nessuno risulta fatto, non significa che non ne hai.',
      slotsBeyondCatalog:
        '{count} achievement presenti nel salvataggio ma non nel gioco installato: probabilmente vengono da una versione più recente',
      catalogBeyondSlots:
        '{count} achievement del gioco non compaiono in questo salvataggio e sono esclusi dalla lista',
    },
  },
  // La prima cosa che si incontra, quando nessun salvataggio è ancora scelto. Ha preso il
  // posto delle chiavi `gate.*`: la scelta non vive più dentro Progressi.
  welcome: {
    title: 'Con quale salvataggio giochi?',
    subtitle:
      'Tutti i dati dell’app vengono dal salvataggio che scegli qui. Puoi cambiarlo quando vuoi.',
    savedGone:
      'Il salvataggio che usavi non si trova più dov’era. Non ne abbiamo scelto un altro al posto tuo: scegli tu quale leggere.',
    hint: 'Ti suggeriamo il più recente, ma la scelta è tua.',
    use: 'Entra',
    cancel: 'Annulla',
    card: {
      slot: 'slot {slot}',
      achievements: 'achievement',
      items: 'oggetti',
      marks: 'marchi',
      unread: 'non leggibile',
      suggested: 'più recente',
      never: 'data sconosciuta',
      unreadableCells: '{count} celle non leggibili',
      unreadable: 'Non riusciamo a leggere questo file',
    },
    nothing: {
      title: 'Nessun salvataggio trovato',
      retry: 'Cerca di nuovo',
      diagnostics: 'Dettagli — dove abbiamo cercato',
      chooseGame: 'Scegli la cartella del gioco',
      chooseSaves: 'Scegli la cartella dei salvataggi',
    },
    failed: {
      title: 'Lettura non riuscita',
      retry: 'Riprova',
    },
  },
  indicator: {
    noProfile: 'Nessun salvataggio scelto',
    notFound: 'Salvataggi non trovati',
    slot: 'slot {slot}',
  },
  profile: {
    title: 'Profilo di gioco',
    intro:
      'Il salvataggio che stai usando, dove l’abbiamo trovato e cosa siamo riusciti a leggere. Per cambiarlo, usa l’indicatore in alto.',
    chain: {
      title: 'Cosa serve',
      summary: 'Steam, il gioco e i salvataggi',
      steam: 'Steam',
      game: 'Gioco',
      saves: 'Salvataggi',
      found: 'trovato',
      missing: 'non trovato',
      yourChoice: 'scegli tu',
      several: 'più di uno',
      chosen: 'scelto',
      candidates: '{n} file trovati',
    },
    none: {
      steamNotFound:
        'Non troviamo Steam su questo computer: è da lì che partiamo per trovare il gioco e i salvataggi.',
      gameNotFound:
        'Steam è installato, ma il gioco non è in nessuna delle sue librerie: senza la cartella del gioco non possiamo trovare i salvataggi.',
      noSaves:
        'Il gioco è installato, ma non abbiamo trovato salvataggi nelle posizioni abituali.',
      noSavesInChosenFolder:
        'Nella cartella che hai indicato non c’è nessun salvataggio. Il gioco li chiama `rep_persistentgamedata1.dat` o `rep+persistentgamedata1.dat`: se non sono lì, la cartella giusta è un’altra.',
    },
    diagnostics: {
      steamNotFound: 'Steam: nessuna installazione trovata',
      gameNotFound: 'Gioco: non presente nelle librerie di Steam',
      noSavesFound: 'Salvataggi: nessun file nelle posizioni abituali',
      noSavesInChosenFolder:
        'Salvataggi: nessun file nella cartella che hai indicato',
      unreadablePath: 'Percorso non leggibile · {name} · {reason}',
      malformedManifest: 'Manifest di Steam non leggibile · {name}',
    },
    sources: {
      steamCloud: 'Steam Cloud',
      documents: 'Documenti',
      manual: 'Scelto a mano',
    },
    active: {
      title: 'Salvataggio attivo',
      autoSelected: 'scelto in automatico · era l’unico',
      modified: 'Modificato',
      size: 'Dimensione',
      dlcs: 'DLC',
      foundIn: 'Trovato in {source}',
      change: 'Cambia salvataggio',
      reload: 'Rileggi il file',
      unknownDate: 'sconosciuto',
      bytes: '{count} byte',
    },
    read: {
      title: 'Cosa abbiamo letto',
      sections: 'sezioni',
      note: 'Alcune parti del salvataggio non sono ancora decifrate, e le quantità cambiano con le patch del gioco. I numeri che vedi qui e nel resto dell’app vengono sempre dal tuo file, mai da stime nostre.',
      diagnostics: 'Il file contiene dati inattesi',
    },
    saveDiagnostics: {
      unexpectedKind: 'Una sezione non è quella attesa ({expected}, {found})',
      sectionOverrun: 'Una sezione va oltre la fine del file ({section})',
      trailingBytes: 'Ci sono byte in più alla fine del file',
    },
    sections: {
      achievements: 'Achievement e segreti',
      counters: 'Contatori e marchi',
      levelCounters: 'Contatori per piano',
      items: 'Collezione oggetti',
      bosses: 'Boss incontrati',
      challenges: 'Sfide',
      bestiary: 'Bestiario',
      unknown: 'Da identificare',
    },
    errors: {
      title: 'Non riusciamo a leggere il salvataggio',
      retry: 'Riprova',
    },
  },
  queue: {
    inQueue: 'in coda',
    inPlan: 'già nella coda del Piano',
    add: 'Aggiungi alla coda',
    remove: 'Rimuovi dalla coda',
    removeShort: 'Rimuovi dalla coda',
    errorTitle: 'La coda non è cambiata',
  },
  // Perché un comando non ha potuto rispondere. Da N2 sono varianti sul filo, non una frase
  // costruita in Rust: i numeri arrivano come numeri e le parole stanno qui.
  ipcReasons: {
    ioNotFound: 'Il file non esiste.',
    ioPermissionDenied: 'Windows non ci permette di aprirlo.',
    ioOther: 'Il sistema non è riuscito ad aprirlo.',
    saveTooShort: 'È troppo corto per essere un salvataggio.',
    saveBadMagic: 'Non sembra un salvataggio di Isaac.',
    settingsConfigDirUnknown:
      'Windows non indica dove salvare le impostazioni.',
    settingsEncoding: 'Non siamo riusciti a scrivere le impostazioni.',
    storeDataDirUnknown: 'Windows non indica dove salvare i dati dell’app.',
    storeDataDirNotCreatable: 'Non riusciamo a creare la cartella dell’app.',
    storeUnreadable: 'Il file non si apre, oppure non è un database.',
    storeNewerSchema:
      'Viene da una versione più recente dell’app ({found} contro {supported}).',
    storeQueueUnparseable: 'Non riusciamo a leggere il piano salvato.',
    autostartWriteRefused:
      'Windows ha rifiutato la voce: potrebbe dipendere da un criterio di sistema o da un antivirus. All’accesso IsaacDome non partirà.',
    autostartWriteIgnored:
      'La voce è stata scritta, ma Windows continua a bloccarla. Controlla in Gestione attività, alla scheda App di avvio: se IsaacDome è disattivata lì, va riattivata da lì.',
  },
  // One sentence per IpcError, for every screen that has to say why a command failed.
  ipcErrors: {
    noBackend: 'L’app non ha risposto.',
    noActiveProfile: 'Nessun salvataggio scelto.',
    unknownProfile: 'Il salvataggio scelto non esiste più.',
    unreadableSave: 'Non riusciamo a leggere il salvataggio.',
    settingsNotWritable: 'Non siamo riusciti a salvare la scelta.',
    unknownTarget: 'Quello che cerchi non esiste.',
    catalogUnavailable: 'Il catalogo del gioco non è disponibile.',
    storeUnavailable: 'Il database dell’app non è disponibile.',
    wikiUnavailable: 'La wiki non è disponibile.',
    sessionTooLarge:
      'La sessione delle tab è troppo grande per essere salvata.',
    autostartNotWritable:
      'Windows non ha accettato l’avvio automatico all’accesso.',
    updateNotReady: 'Non c’è nessun aggiornamento scaricato da installare.',
    folderNotOpenable: 'Esplora file non si è aperto su questa cartella.',
  },
}

export type MessageSchema = typeof it
