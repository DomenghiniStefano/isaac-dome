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
    nextSteps: 'Prossimi passi',
    completion: 'Completamento',
    unlock: 'Unlock',
    plan: 'Piano',
    collection: 'Collezione',
    runs: 'Run',
    live: 'Live',
    wiki: 'Wiki',
    profile: 'Profilo di gioco',
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
    settingsTitle: 'Impostazioni',
    settingsHint: "Da qui l'app trova gioco e salvataggi.",
  },
  placeholder: {
    collection: 'Arriva con la schermata Collezione.',
    runArchive: "Arriva con l'archivio delle run (M4).",
    wiki: 'Arriva con le pagine wiki nelle tab e la ricerca.',
    settings: 'Arriva con Impostazioni e Informazioni.',
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
      hint: 'indizio del gioco:',
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
      goalsPendingTitle: 'Obiettivi salvati da importare',
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
    originNone: 'non indicata',
    unknownAchievement: 'Achievement sconosciuto',
    slot: 'slot',
  },
  nextSteps: {
    intro:
      'Al massimo cinque righe, tutte sbloccabili adesso: le cinque che aprono più cose a valle. Un nodo che il grafo sa dire solo parziale non è un passo, perché non possiamo garantirlo.',
    unlocks: 'sblocca',
    noCatalogTitle: 'Nessun passo: manca il catalogo',
    noCatalog:
      'Senza il gioco installato non si può sapere cosa sblocca cosa: la lista è vuota di proposito, non cinque righe indovinate.',
    nothingNow:
      "Niente è sbloccabile adesso: è tutto fatto, oppure tutto aspetta qualcos'altro.",
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
    facets: 'Faccette',
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
        'slot del salvataggio oltre il catalogo, mostrati come achievement sconosciuti',
      catalogBeyondSlots:
        'achievement del catalogo oltre il salvataggio, che non compaiono',
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
    eyebrow: 'Schermata 0',
    title: 'Profilo di gioco',
    intro:
      "Non è un passaggio da attraversare una volta: è lo stato che decide ogni numero dell'app. Resta consultabile e modificabile per sempre.",
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
