// Italian is the schema: en.ts is typed against it, so a key missing from English is a
// compile error, not a review note. Item, character and boss names are data, not
// messages: they stay in English and never appear here.
export const it = {
  ui: {
    close: 'Chiudi',
  },
  shell: {
    newTab: 'Nuova tab',
    closeTab: 'Chiudi tab',
    minimize: 'Riduci a icona',
    maximize: 'Ingrandisci',
    closeWindow: 'Chiudi finestra',
    search: 'Cerca in tutto',
    settings: 'Impostazioni',
    about: 'Informazioni',
    resizeSidebar: 'Ridimensiona la barra laterale',
    sections: {
      wiki: 'Wiki',
      progress: 'Progressi',
    },
  },
  marks: {
    thirdLevel: 'terzo livello, significato non confermato',
  },
  routes: {
    search: 'Cerca',
    goals: 'Obiettivi consigliati',
    completion: 'Completamento',
    unlock: 'Unlock',
    plan: 'Piano',
    collection: 'Collezione',
    runs: 'Run',
    live: 'Live',
    wiki: 'Wiki',
    profile: 'Profilo di gioco',
    appearance: 'Aspetto',
    tabsSettings: 'Tab',
    about: 'Informazioni',
  },
  wikiCategories: {
    items: 'Oggetti',
    trinkets: 'Trinket',
    achievements: 'Achievement',
    bosses: 'Boss',
    challenges: 'Sfide',
    characters: 'Personaggi',
  },
  sidebar: {
    progressTitle: 'Progressi',
    progressHint: 'Ogni voce si legge sul profilo attivo.',
    wikiTitle: 'Wiki',
    wikiHint: 'La Wiki funziona senza gioco né salvataggio.',
    wikiOverview: 'Panoramica',
    settingsTitle: 'Impostazioni',
    settingsHint: 'Il salvataggio che stai giocando, e come apre le tab.',
  },
  collection: {
    intro:
      'Gli oggetti che la collezione di questo salvataggio non ha ancora, per qualità e pool: non un muro di icone. Qualità e pool vengono dai file del gioco, non indovinati dal nome. I trinket non ci sono: il salvataggio non tiene la loro collezione.',
    state: {
      inCollection: 'in collezione',
      available: 'da trovare',
      locked: 'bloccato',
      unknown: 'non leggibile',
    },
    facets: 'Filtri',
    facet: {
      state: 'Stato',
      quality: 'Qualità',
      pool: 'Pool',
      kind: 'Tipo',
      origin: 'DLC di origine',
    },
    qualityUnrated: 'non valutato',
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
    noResults: 'Nessun oggetto con questi filtri.',
    resetFilters: 'Azzera i filtri',
    noFilters: 'nessun filtro',
    activeFilters: 'filtri attivi',
    reset: 'Azzera',
    diagnostics: {
      noCatalogTitle: 'Manca il catalogo',
      noCatalog:
        'Senza il gioco installato gli oggetti non hanno nome né qualità: la collezione del salvataggio si legge, ma non si sa di cosa.',
      noCollectionSectionTitle: 'La collezione del salvataggio non si legge',
      noCollectionSection:
        "Ogni oggetto risulta non leggibile: non vuol dire che non l'hai mai trovato, vuol dire che quella parte del salvataggio non è stata letta.",
      noAchievementSectionTitle:
        'Gli achievement del salvataggio non si leggono',
      noAchievementSection:
        'Non si sa quali oggetti siano ancora bloccati: quelli che dipendono da un achievement risultano non leggibili.',
      itemsBeyondSlots:
        '{count} oggetti del catalogo oltre la collezione del salvataggio, mostrati come non leggibili',
    },
  },
  wiki: {
    intro:
      "Il dataset è compilato nel binario: funziona senza gioco installato e senza salvataggio scelto. È l'unico pezzo dell'app che non dipende da niente.",
    provenance: {
      title: 'Da dove viene',
      snapshot: 'Istantanea del',
      patch: 'Ultima patch nota',
      patchUnknown: 'sconosciuta',
      newerGame:
        'Il gioco è più recente di questa istantanea: le modifiche recenti possono mancare.',
      license: 'wiki.gg · CC BY-SA 4.0',
      licenseLong:
        'Il testo della wiki è CC BY-SA 4.0, da bindingofisaacrebirth.wiki.gg',
      unresolved: 'riferimenti non risolti',
      unknownTemplates: 'template sconosciuti',
    },
    categories: 'Categorie',
    pages: 'pagine',
    noCatalog:
      'Senza il gioco installato le pagine non hanno immagine: i nomi vengono dai titoli della wiki.',
    search: 'cerca una pagina',
    noResults: 'Nessuna pagina con questo nome.',
    resetFilters: 'Azzera la ricerca',
    back: 'Torna alla categoria',
    id: 'id',
    kind: {
      item: 'Oggetto',
      trinket: 'Trinket',
      achievement: 'Achievement',
      boss: 'Boss',
      challenge: 'Sfida',
      character: 'Personaggio',
    },
    revision: 'rev.',
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
      pool: 'Pool',
      goal: 'Obiettivo',
      items: 'Oggetti',
      trinkets: 'Trinket',
      pickups: 'Pickup',
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
      none: 'nessuno',
    },
    states: {
      unknownTitle: 'Pagina sconosciuta',
      unknown: 'Il dataset non conosce questa pagina.',
      unknownHint:
        'La chiave della tab non corrisponde a nessuna pagina del dataset incorporato: può venire da una versione diversa del dataset.',
      noSections: 'Questa pagina non ha sezioni di testo nel dataset.',
      failedTitle: 'La Wiki non ha risposto',
      missingTitle: 'Il dataset della wiki non si è caricato',
      missing:
        'Il binario contiene un dataset che non si legge: la Wiki non ha pagine da mostrare.',
    },
  },
  search: {
    intro:
      "Un indice solo su tutto quello che l'app conosce per nome o per testo: nomi del gioco, condizioni degli achievement, titoli e testo delle pagine wiki.",
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
      noProfileTitle: 'Nessun profilo scelto',
      noCatalogTitle: 'Gioco non installato',
      noWikiTitle: 'Dataset della wiki non caricato',
      noAchievementSectionTitle: 'Sezione degli achievement non letta',
      noCollectionSectionTitle: 'Sezione della collezione non letta',
      noProfile:
        'I risultati non dicono cosa hai già fatto: scegline uno dalle impostazioni.',
      noCatalog:
        'Si cerca solo nei titoli e nel testo della wiki, senza immagini, e nessun risultato apre Unlock o la Collezione.',
      noWiki:
        'Si cerca solo nei nomi del gioco, e nessun risultato apre una pagina.',
      noAchievementSection:
        'Per gli achievement non si sa cosa è fatto: non letta non è non fatta.',
      noCollectionSection:
        'Per gli oggetti non si sa cosa hai già: non letta non è non trovata.',
    },
  },
  appearance: {
    intro:
      "Scegli quanto è grande l'interfaccia. Cambia tutto insieme — testo, icone, righe, la finestra — e resta com'è tra un avvio e l'altro.",
    scaleTitle: 'Dimensione',
    scaleLabel: "Dimensione dell'interfaccia",
    preview: 'Anteprima',
    previewHint:
      "Resta qui mentre scorri: è l'app disegnata alla dimensione scelta.",
    shortcut: 'Da qualsiasi schermata:',
    shortcutReset: 'torna al 100%',
    saveFailedTitle: 'La dimensione non è stata salvata',
    saveFailed:
      "L'interfaccia è già a questa dimensione, ma non siamo riusciti a scriverla: al prossimo avvio torna com'era.",
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
    promisesTitle: 'Le tre promesse',
    promises: {
      readOnlyTitle: 'Salvataggi in sola lettura',
      readOnly:
        'I salvataggi vengono esclusivamente letti. Il modulo che apre i file .dat non contiene codice di scrittura e il checksum non viene mai ricalcolato.',
      offlineTitle: 'Nessun account, server o telemetria',
      offline:
        'IsaacDome funziona offline. Non richiede un account e non utilizza server né sistemi di telemetria. La connessione di rete viene utilizzata esclusivamente per un eventuale aggiornamento facoltativo del dataset.',
      oneFileTitle: 'Un solo file scritto',
      oneFile:
        "L'app scrive un solo file, isaacdome.db, nella cartella dati dell'app. Nessun altro file sul disco viene modificato o scritto.",
    },
    creditsTitle: 'Crediti e licenze',
    wikiText:
      'Il testo della wiki è distribuito con licenza CC BY-SA 4.0 e proviene da bindingofisaacrebirth.wiki.gg.',
    assets:
      "Le immagini del gioco non sono incluse né distribuite con l'app. Vengono estratte direttamente dalla copia del gioco dell'utente.",
    font: 'Il carattere Determination Mono è distribuito con licenza CC BY 3.0.',
  },
  placeholder: {
    runArchive: "Arriva con l'archivio delle run (M4).",
    tabs: 'Arriva con le tab che sopravvivono alla chiusura.',
  },
  plan: {
    intro:
      "Gli obiettivi sono l'insieme di ciò che vuoi, la coda è l'ordine in cui intendi farlo: le righe che hai chiesto, più i prerequisiti che si sono tirate dietro. Trascini una riga e la coda si ripara attorno al vincolo: i prerequisiti sono un muro contro cui si ferma, non un rifiuto.",
    summary: {
      rows: 'righe',
      wanted: 'chieste',
      pulledIn: 'tirate dentro',
    },
    queueTitle: 'La coda',
    hint: {
      idle: 'trascina per riordinare — una mossa ripara, non fallisce',
      dragging: 'rilascia dove vuoi: la coda si ripara',
      stoppedUnder: 'si è fermata sotto',
      prerequisite: 'è un prerequisito',
    },
    row: {
      move: 'Sposta la riga (Alt e freccia su o giù)',
      wanted: 'chiesta',
      serves: 'serve',
      unlocks: 'sblocca',
      fanOut: 'sblocca',
      outsideQueue: 'passi fuori dalla coda',
      condition: 'come si prende:',
    },
    achievement: 'achievement',
    empty: 'La coda è vuota.',
    emptyHint: 'Aggiungi una riga dalla proposta qui accanto, o da Unlock.',
    completed: {
      closed: 'righe chiuse giocando',
      wanted: 'fra quelle chieste',
    },
    unresolved: 'non è più nel catalogo',
    alerts: {
      storeUnavailableTitle: 'Il piano non è disponibile',
      unreadableTitle: 'La coda salvata non si legge con questa versione',
      unreadable:
        'Resta com’è nel file e non viene sovrascritta: una versione più recente dell’app potrebbe saperla leggere.',
      noCatalogTitle: 'Serve il gioco installato',
      noCatalog:
        'Senza catalogo non si sa quale achievement sia ogni riga né cosa le manchi: la coda resta salvata e torna appena il gioco c’è.',
      goalsPendingTitle: 'Obiettivi salvati da importare: {count}',
      goalsPending:
        'Obiettivi salvati prima che esistesse la coda: nessuno li sposta da solo.',
      import: 'Importa nella coda',
    },
    aside: {
      title: 'Prossimi passi',
      intro:
        'Righe sbloccabili adesso, ordinate per quante cose aprono. Non sono la tua coda: sono la proposta.',
      empty: 'Nessuna proposta adesso.',
    },
  },
  completion: {
    intro:
      'Ogni personaggio, ogni marchio, e quanto il salvataggio lascia leggere.',
    kpi: {
      started: 'marchi iniziati',
      startedExplain:
        'Celle con almeno un segno, normale o hard, fra quelle che il salvataggio lascia leggere. Le celle non leggibili restano fuori dal denominatore.',
      both: 'normale + hard',
      bothExplain:
        'Celle in cui sono presenti sia il segno normale sia quello hard.',
      cells: 'celle',
      complete: 'personaggi completi',
      completeExplain:
        'Personaggi che hanno iniziato tutte le loro celle leggibili: chi ha celle non leggibili risulta completo su meno colonne.',
      unknown: 'non leggibili',
      unknownExplain:
        'Celle la cui colonna non è localizzata nel salvataggio per quel personaggio. Non vuol dire "mai fatto": non lo sappiamo leggere.',
    },
    card: {
      title: 'Matrice dei marchi',
    },
    legend: {
      empty: 'mai fatto',
      normal: 'normale',
      hard: 'hard',
      third: 'terzo livello',
      unknown: 'non leggibile',
    },
    grid: {
      character: 'Personaggio',
      started: 'iniziati',
      unreadable: 'non leggibili',
      columnTotals: 'Personaggi con il marchio',
    },
    groups: {
      base: 'Personaggi base',
      tainted: 'Tainted',
    },
    cell: {
      empty: 'mai fatto',
      normal: 'normale',
      hard: 'hard',
      both: 'normale e hard',
      unknown:
        'non leggibile: la colonna non è localizzata per questo personaggio',
      unexpected: 'valore fuori da quelli previsti:',
    },
    nothingReadable:
      'Il salvataggio non lascia leggere nessun marchio: la sezione dei contatori manca o è troncata.',
  },
  // La schermata di atterraggio: cosa ottieni, come, perché conviene, e l'azione. L'intro
  // dice *cos'è* la lista, non come è stata calcolata: il calcolo sta nelle promesse della
  // finestra Informazioni, dove chi vuole lo trova (B32 §2).
  goals: {
    intro:
      'Cose che puoi sbloccare adesso. In alto quelle che aprono di più, sotto quelle a cui sei più vicino.',
    fanOut: 'Aprono di più',
    closeness: 'Ci sei quasi',
    seeAll: 'Vedile tutte',
    opens: 'Apre altre {count} cose',
    opensNothing: 'Non apre altro',
    noCatalogTitle: 'Niente da consigliare senza il gioco',
    noCatalog:
      'Senza il gioco installato non si sa cosa sblocca cosa: meglio una lista vuota che cinque righe indovinate.',
    nothingNow:
      'Non c’è niente da sbloccare adesso: o è tutto fatto, o tutto aspetta qualcos’altro.',
  },
  // Il blocco che la pagina wiki di un achievement guadagna quando c'è un profilo attivo.
  // Dice soltanto dove sei tu: cos'è e cosa chiede lo dice già l'infobox sotto.
  profileBlock: {
    title: 'Il tuo profilo',
    missing: 'Cosa ti manca',
    unlocks: 'Cosa ottieni',
    opens: 'Sbloccarlo apre altre {count} cose.',
    // Un nodo già fatto non "aprirà": ha aperto. La stessa frase al futuro, sotto un
    // "Già fatto", si legge come se ci fosse ancora qualcosa da fare.
    opened: 'Ha aperto altre {count} cose.',
    opensNothing: 'Non apre nient’altro: è una fine di ramo.',
    openedNothing: 'Non ha aperto nient’altro: è una fine di ramo.',
    stepsMissing: 'Prima servono ancora {count} sblocchi.',
    done: 'Già fatto.',
  },
  graph: {
    state: {
      done: 'fatto',
      now: 'sbloccabile ora',
      blocked: 'bloccato da',
      partial: 'grafo parziale',
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
      unknown: 'Requisiti che non sappiamo interpretare',
    },
    kinds: {
      passive: 'oggetto passivo',
      active: 'oggetto attivo',
      familiar: 'famiglio',
      trinket: 'trinket',
      character: 'personaggio',
      boss: 'boss',
      challenge: 'sfida',
      nothing: 'niente di catalogato',
    },
    stateName: {
      done: 'fatto',
      now: 'sbloccabile ora',
      blocked: 'bloccato',
      partial: 'grafo parziale',
    },
    // Una cella della matrice: il boss e il personaggio con cui va battuto.
    markName: '{boss} con {character}',
    originNone: 'non indicata',
    unknownAchievement: 'Achievement sconosciuto',
    slot: 'slot',
    // La forma Tainted di un personaggio: il gioco scrive lo stesso nome per le due forme.
    // "Tainted" resta in inglese come ogni nome del gioco (DESIGN-BRIEF.md §12).
    taintedName: 'Tainted {name}',
  },
  unlock: {
    intro:
      'Ogni nodo del grafo, filtrabile. Un filtro conta più degli altri: sbloccabile ora.',
    rows: 'righe',
    search: 'cerca nome, condizione o cosa sblocca',
    sortBy: 'ordina per',
    sort: {
      fanOut: 'sblocca',
      steps: 'passi mancanti',
      name: 'nome',
    },
    facets: 'Filtri',
    noFilters: 'nessun filtro',
    activeFilters: 'filtri attivi',
    reset: 'Azzera',
    resetFilters: 'Azzera i filtri',
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
      fanOut: 'Sblocca',
    },
    unlocksNothing: 'niente di catalogato',
    noCondition: 'nessuna condizione nel file',
    diagnostics: {
      noCatalogTitle: 'Manca il catalogo',
      noCatalog:
        'Senza il gioco installato gli achievement non hanno nome né condizione: le righe dicono solo quali slot sono fatti.',
      noAchievementSectionTitle: 'La sezione degli achievement non si legge',
      noAchievementSection:
        'Zero righe non vuol dire zero achievement fatti: vuol dire che quella parte del salvataggio non è stata letta.',
      slotsBeyondCatalog:
        '{count} slot del salvataggio oltre il catalogo, mostrati come achievement sconosciuti',
      catalogBeyondSlots:
        '{count} achievement del catalogo oltre il salvataggio, che non compaiono',
    },
  },
  gate: {
    needsProfile:
      'Progressi dipende dal profilo attivo: scegline uno per vedere questa schermata.',
  },
  indicator: {
    noProfile: 'Nessun profilo attivo',
    notFound: 'Salvataggi non trovati',
    slot: 'slot',
  },
  profile: {
    title: 'Profilo di gioco',
    intro:
      'Scegli il salvataggio con cui stai giocando: ogni numero delle altre schermate si legge su questo. Puoi cambiarlo quando vuoi.',
    chain: {
      title: 'La catena dei tre requisiti',
      summary: 'ogni livello può mancare da solo',
      steam: 'Steam',
      game: 'Gioco',
      saves: 'Salvataggi',
      found: 'trovato',
      missing: 'non trovato',
      yourChoice: 'scegli tu',
      several: 'più di uno',
      chosen: 'scelto',
      candidates: 'file candidati',
    },
    none: {
      title: 'Nessun salvataggio trovato',
      steamNotFound:
        'Steam non è nel registro di sistema: senza Steam non troviamo la cartella del gioco, e senza quella non troviamo i salvataggi.',
      gameNotFound:
        'Steam c’è, ma il gioco non è in nessuna delle sue librerie: senza la cartella del gioco non troviamo i salvataggi.',
      noSaves:
        'Il gioco c’è, ma nelle posizioni note non c’è nessun file di salvataggio.',
      retry: 'Riprova la ricerca',
      diagnostics: 'Diagnostica — cosa abbiamo provato',
    },
    diagnostics: {
      steamNotFound: 'Steam: nessuna installazione trovata',
      gameNotFound: 'Gioco: non presente nelle librerie di Steam',
      noSavesFound: 'Salvataggi: nessun file nelle posizioni note',
      unreadablePath: 'Percorso non leggibile',
      malformedManifest: 'Manifest di Steam non leggibile',
    },
    pick: {
      title: 'Serve scegliere',
      summary: 'finché non scegli, nessun profilo è attivo',
      savedGone:
        'Il profilo che usavi non esiste più dove stava. Non ne abbiamo scelto un altro al suo posto: i numeri di un profilo diverso, mostrati senza dirlo, sono l’errore che non ti accorgi di avere.',
      edition: 'Edizione',
      slot: 'Slot',
      foundIn: 'Trovato qui',
      modified: 'Modificato',
      size: 'Dimensione',
      suggested: 'più recente',
      hint: 'Il più recente è solo un suggerimento: scegli tu quale profilo leggere.',
      use: 'Usa questo profilo',
      cancel: 'Annulla',
    },
    sources: {
      steamCloud: 'Steam Cloud',
      documents: 'Documenti',
      manual: 'Scelto a mano',
    },
    active: {
      title: 'Profilo attivo',
      autoSelected: 'scelto da noi · era l’unico',
      modified: 'Modificato',
      size: 'Dimensione',
      dlcs: 'DLC',
      foundIn: 'Trovato in',
      change: 'Cambia profilo',
      reload: 'Rileggi il file',
      unknownDate: 'sconosciuto',
      bytes: 'byte',
    },
    read: {
      title: 'Cosa siamo riusciti a leggere',
      sections: 'sezioni',
      note: 'Alcune sezioni non sappiamo ancora cosa contengano, e i conteggi cambiano a ogni patch: nessun numero è cablato, né qui né nel resto dell’app. Le schermate mostrano quello che il file dichiara oggi.',
      diagnostics: 'Il file contiene qualcosa che non ci aspettavamo',
    },
    saveDiagnostics: {
      unexpectedKind: 'Una sezione non è quella attesa',
      sectionOverrun: 'Una sezione va oltre la fine del file',
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
      title: 'Non riusciamo a leggere il profilo',
      retry: 'Riprova',
    },
  },
  queue: {
    inQueue: 'in coda',
    inPlan: 'già nella coda del Piano',
    add: 'Aggiungi alla coda',
    addShort: 'Aggiungi',
    remove: 'Togli dalla coda',
    removeShort: 'Togli',
    errorTitle: 'La coda non è cambiata',
  },
  // Perché un comando non ha potuto rispondere. Da N2 sono varianti sul filo, non una frase
  // costruita in Rust: i numeri arrivano come numeri e le parole stanno qui.
  ipcReasons: {
    ioNotFound: 'Il file non c’è.',
    ioPermissionDenied: 'Windows non ci lascia aprirlo.',
    ioOther: 'Il sistema non lo ha aperto.',
    saveTooShort: 'È troppo corto per contenere un salvataggio.',
    saveBadMagic: 'Non sembra un salvataggio di Isaac.',
    settingsConfigDirUnknown: 'Windows non dice dove vanno le impostazioni.',
    settingsEncoding: 'Le impostazioni non si sono scritte.',
    storeDataDirUnknown: 'Windows non dice dove vanno i dati dell’app.',
    storeDataDirNotCreatable: 'La cartella dell’app non si crea.',
    storeUnreadable: 'Il file non si apre, o non è un database.',
    storeNewerSchema:
      'Viene da una versione più recente dell’app ({found} contro {supported}).',
    storeQueueUnparseable: 'Il piano salvato non si legge.',
  },
  // One sentence per IpcError, for every screen that has to say why a command failed.
  ipcErrors: {
    noBackend: 'Il backend non ha risposto.',
    noActiveProfile: 'Nessun profilo attivo.',
    unknownProfile: 'Il profilo scelto non esiste più.',
    unreadableSave: 'Il salvataggio non si legge.',
    settingsNotWritable: 'Non riusciamo a ricordare la scelta.',
    unknownTarget: 'Quello che cerchi non esiste.',
    catalogUnavailable: 'Il catalogo del gioco non è disponibile.',
    storeUnavailable: "Il database dell'app non è disponibile.",
    wikiUnavailable: 'Il dataset della wiki non è disponibile.',
  },
}

export type MessageSchema = typeof it
