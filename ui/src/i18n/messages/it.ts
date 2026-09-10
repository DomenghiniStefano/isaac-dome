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
}

export type MessageSchema = typeof it
