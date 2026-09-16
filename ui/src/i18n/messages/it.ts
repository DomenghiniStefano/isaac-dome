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
    back: 'Indietro',
    forward: 'Avanti',
    minimize: 'Riduci a icona',
    maximize: 'Ingrandisci',
    closeWindow: 'Chiudi finestra',
    search: 'Cerca in tutto',
    settings: 'Impostazioni',
    about: 'Informazioni',
    resizeSidebar: 'Ridimensiona la barra laterale',
    sections: {
      progress: 'Progressi',
      tool: 'Tool',
      wiki: 'Wiki',
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
    floor: 'Piano di gioco',
    wiki: 'Wiki',
    profile: 'Profilo di gioco',
    appearance: 'Aspetto',
    background: 'Background',
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
    transformations: 'Trasformazioni',
  },
  sidebar: {
    progressTitle: 'Progressi',
    progressHint: 'Ogni voce si legge sul profilo attivo.',
    toolTitle: 'Tool',
    toolHint:
      'Funzionano senza salvataggio: leggono il log, o quello che disegni tu.',
    wikiTitle: 'Wiki',
    wikiHint: 'La Wiki funziona senza gioco né salvataggio.',
    wikiOverview: 'Panoramica',
    settingsTitle: 'Impostazioni',
    settingsHint: 'Il salvataggio che stai giocando, e come apre le tab.',
  },
  live: {
    intro:
      'Quello che stai giocando adesso, e cosa aprirebbe se lo finisci. L’app deve essere aperta mentre giochi: il gioco riscrive il log a ogni avvio.',
    run: 'La run in corso',
    floors: 'piani',
    heldActive: 'in mano',
    collected: 'raccolti',
    wouldOpen: 'Se finisci questa run',
    missing: 'da prendere',
    secondLevel: 'secondo livello',
    noCondition: 'il gioco non lo scrive',
    column: {
      achievement: 'Achievement',
      cell: 'Serve',
      condition: 'Come si ottiene',
      opens: 'Apre',
    },
    marks: 'I marchi di chi stai giocando',
    items: 'Cosa hai in mano',
    startingItems: 'Di partenza',
    collectedItems: 'Raccolti',
    unlockedHere: 'Sbloccati in questa run',
    opens: 'apre {count}',
    opensNothingMore: 'non apre altro',
    beat: 'Batti {column} con {character}',
    nothing:
      'Niente che questa run possa aprire da sola: quello che manca agli achievement non è un marchio di questo personaggio.',
    diagnostic: {
      noRun: 'Nessuna run in corso: l’app sta guardando, e non stai giocando.',
      characterNotNamed:
        'Non sappiamo ancora con chi stai giocando: il nome compare quando raccogli il primo oggetto.',
      unknownCharacter: 'Il personaggio «{name}» non è nel catalogo del gioco.',
      ambiguousCharacter:
        'Il log scrive «{name}», e il gioco chiama così {forms} personaggi: la forma base e quella Tainted. Sotto trovi entrambe, perché indovinare quale sia sarebbe un’ipotesi.',
      noGraph: 'Senza il gioco installato non sappiamo cosa aprirebbe.',
      noProfile: 'Senza un salvataggio scelto non sappiamo cosa ti manca.',
    },
  },
  floor: {
    intro:
      'Disegna la mappa del piano come la vedi sulla minimappa: le celle vuote si accendono dove le regole del gioco permettono una stanza segreta. Ogni cella accesa dice quale regola l’ha accesa e da dove è stata letta.',
    erase: 'Cancella',
    clear: 'Svuota la griglia',
    painted: 'stanze disegnate',
    neighbours: 'stanze adiacenti',
    source: 'Fonte',
    unresolved: 'Quello che la griglia non può giudicare',
    none: 'Nessuna cella permessa dalle regole, con quello che hai disegnato finora.',
    failed:
      'Non è stato possibile calcolare i candidati. Quello che hai disegnato resta.',
    target: {
      secret: 'Stanza segreta',
      superSecret: 'Super segreta',
      ultraSecret: 'Ultra segreta',
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
      gridEmpty:
        'Disegna almeno una stanza: senza, non c’è niente da giudicare.',
      noStartRoom:
        'Segna la stanza di partenza. Senza, la regola sui passi dalla partenza non si può leggere, e la Super segreta resta giudicata solo a metà.',
      rulesUnreadable:
        'Le regole di piazzamento non si sono caricate: {reason}.',
      gridMalformed:
        'La griglia arrivata non è un piano: {cells} celle invece di 169.',
    },
  },
  runs: {
    intro:
      'Le run che l’app ha letto: quelle di questa sessione e quelle delle partite online che il gioco ha già registrato. Il log non ha un orologio, quindi l’ordine è per sessione e non per ora.',
    rows: 'run',
    facets: 'Filtri',
    activeFilters: 'filtri attivi',
    noFilters: 'nessun filtro',
    reset: 'Azzera i filtri',
    search: 'cerca un seed o un personaggio',
    facet: {
      outcome: 'Esito',
      character: 'Personaggio',
      online: 'Con chi',
      source: 'Da dove',
    },
    outcome: {
      won: 'vinta',
      died: 'morta',
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
      runs: 'run lette',
      won: 'vinte',
      died: 'morte',
      abandoned: 'abbandonate',
      open: 'in corso',
    },
    column: {
      character: 'Personaggio',
      outcome: 'Esito',
      floors: 'Piani',
      seed: 'Seed',
      source: 'Da dove',
    },
    noCharacter: 'personaggio non nominato',
    noItems: 'nessuno',
    openPage: 'Clic per la pagina, Ctrl+clic per aprirla di fianco',
    itemId: 'oggetto {id}',
    floors: 'piani',
    killedBy: 'uccisa da {killer}',
    endedWith: 'finale {ending}',
    startingItems: 'Oggetti iniziali',
    collected: 'Raccolti',
    heldActive: 'Attivo in mano',
    unnamedItem: 'oggetto {id}',
    empty:
      'Nessuna run nell’archivio: non è ancora stata giocata una partita da quando l’app è installata.',
    noMatch: 'Nessuna run con questi filtri.',
    diagnostic: {
      noLogFolder:
        'Non troviamo la cartella dove il gioco scrive i log, quindi non c’è niente da leggere.',
      storeUnavailable:
        'Il database dell’app non si apre, quindi l’archivio non si può leggere.',
      unreadableEvents:
        '{count} righe del log non sono state capite: le run che le contengono possono essere incomplete.',
      noCatalog:
        'Il gioco non è installato: gli oggetti hanno il loro numero e non il nome.',
    },
  },
  collection: {
    intro:
      'Gli oggetti che questo salvataggio non ha ancora, con la loro qualità e i pool in cui compaiono. I trinket non ci sono: il gioco non tiene traccia di quali hai trovato.',
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
    empty: 'Nessun oggetto da mostrare.',
    noResults: 'Nessun oggetto con questi filtri.',
    resetFilters: 'Azzera i filtri',
    noFilters: 'nessun filtro',
    activeFilters: 'filtri attivi',
    reset: 'Azzera',
    diagnostics: {
      noCatalogTitle: 'Non troviamo il gioco',
      noCatalog:
        'Senza i file del gioco gli oggetti non hanno nome né qualità: sappiamo quanti ne hai, non quali. Installa The Binding of Isaac da Steam e riapri l’app.',
      noCollectionSectionTitle: 'Il salvataggio non dice cosa hai raccolto',
      noCollectionSection:
        'Ogni oggetto risulta non leggibile. Non vuol dire che non l’hai mai trovato: vuol dire che quella parte del salvataggio non si è aperta.',
      noAchievementSectionTitle: 'Il salvataggio non dice cosa hai sbloccato',
      noAchievementSection:
        'Non sappiamo dire quali oggetti siano ancora bloccati: quelli che arrivano da un achievement restano incerti.',
      itemsBeyondSlots:
        '{count} oggetti che il gioco conosce e questo salvataggio non nomina: li mostriamo come non leggibili',
    },
  },
  wiki: {
    intro:
      'Una copia della wiki di Isaac dentro l’app: oggetti, personaggi, boss, sfide, achievement e trasformazioni. Funziona anche senza il gioco installato e senza aver scelto un salvataggio.',
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
      'Senza il gioco installato le pagine non hanno immagine: le figure vengono dalla tua copia di Isaac, non dall’app.',
    search: 'cerca una pagina',
    noResults: 'Nessuna pagina con questo nome.',
    emptyCategory: 'Questa categoria non ha pagine.',
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
      transformation: 'Trasformazione',
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
      requires: 'Ne servono',
      contributors: 'Contano',
      target: 'Agisce su',
      none: 'nessuno',
    },
    states: {
      unknownTitle: 'Pagina sconosciuta',
      unknown: 'Questa pagina non è nella nostra copia della wiki.',
      unknownHint:
        'Può essere stata aggiunta dopo: la copia risale alla data qui sopra e si aggiorna con l’app.',
      noSections: 'Di questa pagina abbiamo solo la scheda, senza testo.',
      failedTitle: 'La Wiki non ha risposto',
      missingTitle: 'La Wiki non si è aperta',
      missing:
        'La copia della wiki dentro l’app non si legge, quindi non c’è nessuna pagina da mostrare. Riavviare l’app di solito basta; se continua, è un problema da segnalare.',
    },
  },
  search: {
    intro:
      'Una ricerca sola su tutto: nomi del gioco, condizioni degli achievement, titoli e testo delle pagine della wiki.',
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
      noWikiTitle: 'La Wiki non si è aperta',
      noAchievementSectionTitle: 'Il salvataggio non dice cosa hai sbloccato',
      noCollectionSectionTitle: 'Il salvataggio non dice cosa hai raccolto',
      noProfile:
        'I risultati non dicono cosa hai già fatto: scegli un salvataggio dalle impostazioni.',
      noCatalog:
        'Si cerca solo nei titoli e nel testo della wiki, senza immagini, e nessun risultato apre Unlock o la Collezione.',
      noWiki:
        'Si cerca solo nei nomi del gioco, e nessun risultato apre una pagina.',
      noAchievementSection:
        'Accanto agli achievement non compare "fatto" o "da fare". Non vuol dire che non li hai: vuol dire che non lo sappiamo.',
      noCollectionSection:
        'Accanto agli oggetti non compare se ce l’hai già. Non vuol dire di no: vuol dire che non lo sappiamo.',
    },
  },
  background: {
    intro: "Quando parte l'app, e cosa fa quando chiudi l'ultima finestra.",
    startTitle: 'Avvia con Windows',
    startHint:
      "IsaacDome deve essere aperta mentre giochi: il gioco riscrive il suo log a ogni avvio. Una sessione giocata prima di aprire l'app si recupera ancora — finché il gioco non viene riavviato, e da lì è persa.",
    startDev:
      'Non disponibile in questa build. Una build di sviluppo metterebbe nel tuo accesso un percorso temporaneo, che a ogni avvio fallirebbe senza dirlo.',
    startWithoutBackground:
      "Con «resta aperta in background» spenta, all'accesso trovi l'icona accanto all'orologio e nessuna finestra; la prima finestra che apri e chiudi chiude l'app, e la lettura si ferma con lei.",
    startFailedTitle: 'Windows non ha accettato la voce',
    stayTitle: 'Resta aperta in background',
    stayHint:
      "Chiudendo l'ultima finestra l'app resta accanto all'orologio, nell'area di notifica: un clic sull'icona riapre la finestra. Spenta, chiudere l'ultima finestra chiude l'app.",
    saveFailedTitle: "L'impostazione non è stata salvata",
    saveFailed:
      "L'app si comporta già così, ma non siamo riusciti a scriverlo: al prossimo avvio torna com'era.",
  },
  tabsSettings: {
    intro: "Cosa ritrovi quando riapri l'app.",
    resumeTitle: "Riapri le tab dell'ultima sessione",
    resumeHint:
      "All'avvio ritrovi le finestre che avevi, con dentro le stesse tab. Spenta, l'app riparte dalla schermata iniziale e quello che era salvato viene cancellato subito.",
    keptTitle: 'Cosa viene salvato',
    keptWindows:
      'Le finestre che avevi aperte, la loro posizione e la loro dimensione.',
    keptTabs:
      'Le tab di ogni finestra, nel loro ordine, e quale era in primo piano.',
    keptReading:
      "Come stavi leggendo ogni tab: i filtri, l'ordinamento, la riga selezionata e il punto in cui eri arrivato.",
    keptSidebar: 'La larghezza della barra laterale.',
    stoppedTitle: 'La sessione non viene più salvata',
    stopped:
      'Le tab aperte restano dove sono, ma non verranno riaperte al prossimo avvio: quello che le finestre stanno mostrando supera lo spazio riservato alla sessione. Chiudere qualche tab e riaprirne una qualsiasi riprende il salvataggio.',
    saveFailedTitle: "L'impostazione non è stata salvata",
    saveFailed:
      "L'app si comporta già così, ma non siamo riusciti a scriverlo: al prossimo avvio torna com'era.",
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
    tabs: 'Arriva con le tab che sopravvivono alla chiusura.',
  },
  plan: {
    intro:
      'L’ordine in cui vuoi fare le cose: quelle che hai chiesto, più quello che serve prima. Trascina una riga dove vuoi — se le manca un prerequisito si ferma appena sotto, invece di rifiutare lo spostamento.',
    summary: {
      rows: 'righe',
      wanted: 'chieste',
      pulledIn: 'tirate dentro',
    },
    queueTitle: 'La coda',
    hint: {
      idle: 'trascina per riordinare — nessuno spostamento viene rifiutato',
      dragging: 'rilascia dove vuoi: la coda si sistema da sola',
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
    unresolved: 'il gioco non lo conosce più',
    alerts: {
      storeUnavailableTitle: 'Il piano non è disponibile',
      unreadableTitle: 'La coda salvata non si legge con questa versione',
      unreadable:
        'Resta com’è nel file e non viene sovrascritta: una versione più recente dell’app potrebbe saperla leggere.',
      noCatalogTitle: 'Serve il gioco installato',
      noCatalog:
        'Senza i file del gioco non sappiamo che achievement sia ogni riga né cosa le manchi. La coda resta salvata: torna com’era appena il gioco c’è.',
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
        'Caselle che nel salvataggio non sappiamo ancora dove stiano. Non vuol dire "mai fatto": vuol dire che non lo sappiamo dire.',
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
      unknown: 'non lo sappiamo dire per questo personaggio',
      unexpected: 'un valore che non ci aspettavamo:',
    },
    nothingReadable:
      'Di questo salvataggio non riusciamo a leggere nessun marchio: la parte che li contiene manca o si interrompe a metà.',
  },
  // La schermata di atterraggio: cosa ottieni, come, perché conviene, e l'azione. L'intro
  // dice *cos'è* la lista, non come è stata calcolata: il calcolo sta nelle promesse della
  // finestra Informazioni, dove chi vuole lo trova (B32 §2).
  goals: {
    intro:
      'Cose che puoi sbloccare adesso. In alto quelle che aprono di più, sotto quelle a cui sei più vicino.',
    fanOut: 'Aprono di più',
    closeness: 'Ci sei quasi',
    inPlan: 'Nel tuo Piano',
    openPlan: 'Apri il Piano',
    seeAll: 'Vedile tutte',
    opens: 'Apre altre {count} cose',
    opensNothing: 'Non apre altro',
    noCatalogTitle: 'Non troviamo il gioco',
    noCatalog:
      'IsaacDome legge i file di The Binding of Isaac per sapere quale achievement sblocca cosa, e sul computer non li trova. Installa il gioco da Steam e riapri l’app: qui troverai cosa conviene giocare stasera.',
    nothingNow:
      'Non c’è niente da sbloccare adesso: o è tutto fatto, o tutto aspetta qualcos’altro.',
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
    wayOf: 'Strada {index} di {total}',
    unknown:
      'Più {count} requisiti che non riusciamo a leggere: la serie potrebbe essere più lunga.',
    addAll: 'Metti tutto nel Piano',
    diagnostics: {
      noCatalog:
        'Senza i file del gioco non sappiamo quale achievement sblocchi cosa. Installa The Binding of Isaac e riapri l’app.',
      noProfile:
        'Nessun profilo letto: possiamo dirti come si ottiene, non a che punto sei.',
      nothingUnlocks:
        'Niente lo sblocca: o ce l’hai da sempre, o si ottiene giocando e non da un achievement.',
      notUnlockable: 'Questo non è qualcosa che si sblocca.',
    },
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
      // Non "grafo parziale": chi legge non sa cosa sia un grafo, e la cosa che deve
      // sapere è che la risposta non c'è — non come mai.
      partial: 'non sappiamo dirlo',
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
      unknown: 'Condizioni che non sappiamo leggere',
    },
    kinds: {
      passive: 'oggetto passivo',
      active: 'oggetto attivo',
      familiar: 'famiglio',
      trinket: 'trinket',
      character: 'personaggio',
      boss: 'boss',
      challenge: 'sfida',
      nothing: 'niente che il gioco nomini',
    },
    stateName: {
      done: 'fatto',
      now: 'sbloccabile ora',
      blocked: 'bloccato',
      partial: 'non sappiamo dirlo',
    },
    // Una cella della matrice: il boss e il personaggio con cui va battuto.
    markName: '{boss} con {character}',
    thresholdName: '{name} — {current} di {atLeast}',
    originNone: 'non indicata',
    unknownAchievement: 'Achievement sconosciuto',
    slot: 'slot',
    // La forma Tainted di un personaggio: il gioco scrive lo stesso nome per le due forme.
    // "Tainted" resta in inglese come ogni nome del gioco (DESIGN-BRIEF.md §12).
    taintedName: 'Tainted {name}',
  },
  unlock: {
    intro:
      'Tutti gli achievement del gioco, da filtrare come vuoi. Il filtro che conta più degli altri è "sbloccabile ora".',
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
    empty: 'Nessun obiettivo da mostrare.',
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
    unlocksNothing: 'niente che il gioco nomini',
    noCondition: 'non lo sappiamo',
    diagnostics: {
      noCatalogTitle: 'Non troviamo il gioco',
      noCatalog:
        'Senza i file del gioco gli achievement non hanno nome né condizione: le righe dicono soltanto quali hai già fatto.',
      noAchievementSectionTitle: 'Il salvataggio non dice cosa hai sbloccato',
      noAchievementSection:
        'Zero righe fatte non vuol dire zero achievement: vuol dire che quella parte del salvataggio non si è aperta.',
      slotsBeyondCatalog:
        '{count} achievement che il salvataggio conosce e la tua copia del gioco no: probabilmente sono di una versione più recente',
      catalogBeyondSlots:
        '{count} achievement che il gioco ha e questo salvataggio non nomina: non compaiono nella lista',
    },
  },
  gate: {
    needsProfile:
      'Progressi dipende dal profilo attivo: scegline uno per vedere questa schermata.',
    blocked: {
      title: 'Progressi ha bisogno di un salvataggio',
      settings: 'Apri le impostazioni del profilo',
    },
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
      title: 'Le tre cose che servono',
      summary: 'può mancarne una sola',
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
        'Non troviamo Steam su questo computer, e partiamo da lì per arrivare alla cartella del gioco e poi ai salvataggi.',
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
      note: 'Di alcune parti del salvataggio non sappiamo ancora cosa contengano, e le quantità cambiano a ogni patch del gioco. Quello che vedi qui e nel resto dell’app è sempre quanto dichiara il tuo file, mai un numero deciso da noi.',
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
    autostartWriteRefused:
      "Windows ha rifiutato la voce: può esserci di mezzo un criterio di sistema o un antivirus. All'accesso non partirà niente.",
    autostartWriteIgnored:
      'La voce è stata scritta e Windows continua a dire di no. Guarda nel Task Manager, in App di avvio: se lì è disattivata, IsaacDome resta spenta qualunque cosa si scriva da qui.',
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
    sessionTooLarge:
      'La sessione delle tab è troppo grande per essere salvata.',
    autostartNotWritable:
      "Windows non ha accettato la voce di avvio all'accesso.",
  },
}

export type MessageSchema = typeof it
