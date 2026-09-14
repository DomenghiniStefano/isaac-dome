import { uniq } from 'lodash-es'
import { describe, expect, it } from 'vitest'
import { createFaceting } from './faceting'

// A row that belongs to neither screen, on purpose. Tested through `UnlockNode` the engine
// would be proven to work for Unlock and nothing would be said about whether it is generic —
// which is the whole claim, and the one `docs/BACKLOG.md` B3 is going to lean on.
interface Pet {
  name: string
  species: string
  colours: string[]
}

const PetFacet = {
  Species: 'species',
  Colour: 'colour',
} as const
type PetFacet = (typeof PetFacet)[keyof typeof PetFacet]

// Colour is the facet that holds several values for one row: the case a facet engine has to
// get right and a `===` comparison never does.
const pets = createFaceting<Pet, PetFacet>({
  order: [PetFacet.Species, PetFacet.Colour],
  values: (pet, facet) =>
    facet === PetFacet.Species ? [pet.species] : pet.colours,
  text: (pet) => pet.name,
  // The two shapes a real screen has: a fixed list that offers a value no row holds (fish),
  // and one read off the rows themselves.
  options: (rows, facet) =>
    facet === PetFacet.Species
      ? ['cat', 'dog', 'fish']
      : uniq(rows.flatMap((pet) => pet.colours)),
})

const mittens: Pet = {
  name: 'Mittens',
  species: 'cat',
  colours: ['black', 'white'],
}

const filter = (picks: Partial<Record<PetFacet, string[]>>, query = '') => ({
  query,
  picks: {
    [PetFacet.Species]: [],
    [PetFacet.Colour]: [],
    ...picks,
  },
})

const rex: Pet = { name: 'Rex', species: 'dog', colours: ['black'] }
const whiskers: Pet = {
  name: 'Whiskers',
  species: 'cat',
  colours: ['ginger'],
}
const all = [mittens, rex, whiskers]

describe('matches', () => {
  it('lets every row through a filter that picks nothing', () => {
    expect(pets.matches(mittens, filter({}))).toBe(true)
  })

  it("takes a row holding any one of a facet's picked values", () => {
    const picked = filter({ [PetFacet.Colour]: ['white', 'ginger'] })
    expect(pets.matches(mittens, picked)).toBe(true)
    expect(pets.matches(rex, picked)).toBe(false)
  })

  it('searches the text the spec names, whatever the case', () => {
    expect(pets.matches(mittens, filter({}, 'MITT'))).toBe(true)
    expect(pets.matches(rex, filter({}, 'MITT'))).toBe(false)
  })

  it('reads a query of blanks as no query at all', () => {
    expect(pets.matches(rex, filter({}, '   '))).toBe(true)
  })

  // Within a facet any value will do, across facets all of them must hold. The two rules read
  // alike and a single-facet test cannot tell them apart: `rex` is black, and is out anyway.
  it('asks every facet at once, not any of them', () => {
    const blackDogsOnly = filter({
      [PetFacet.Species]: ['cat'],
      [PetFacet.Colour]: ['black'],
    })
    expect(pets.matches(mittens, blackDogsOnly)).toBe(true)
    expect(pets.matches(rex, blackDogsOnly)).toBe(false)
  })
})

describe('counts', () => {
  // Picking "cat" and then reading "dog: 0" would be a list that argues with itself: the tally
  // of a facet is over the rows the *other* facets leave, so it says what picking would give.
  it('leaves its own facet out', () => {
    const cats = filter({ [PetFacet.Species]: ['cat'] })
    expect(pets.counts(all, cats, PetFacet.Species)).toEqual(
      new Map([
        ['cat', 2],
        ['dog', 1],
      ]),
    )
  })

  it('applies every other facet', () => {
    const cats = filter({ [PetFacet.Species]: ['cat'] })
    expect(pets.counts(all, cats, PetFacet.Colour)).toEqual(
      new Map([
        ['black', 1],
        ['white', 1],
        ['ginger', 1],
      ]),
    )
  })

  it('applies the search as well', () => {
    expect(pets.counts(all, filter({}, 'whisk'), PetFacet.Species)).toEqual(
      new Map([['cat', 1]]),
    )
  })
})

describe('the filter a screen starts from', () => {
  it('picks nothing in any facet', () => {
    expect(pets.empty()).toEqual({
      query: '',
      picks: { [PetFacet.Species]: [], [PetFacet.Colour]: [] },
    })
  })

  // A shared empty object would carry one screen's picks into the next list opened.
  it('is a new one every time', () => {
    pets.empty().picks[PetFacet.Species].push('cat')
    expect(pets.empty().picks[PetFacet.Species]).toEqual([])
  })
})

describe('activeCount', () => {
  it('counts the values picked across the facets, and not the search', () => {
    const picked = filter(
      { [PetFacet.Species]: ['cat'], [PetFacet.Colour]: ['black', 'white'] },
      'mitt',
    )
    expect(pets.activeCount(picked)).toBe(3)
  })
})

describe('options', () => {
  it('offers what the spec offers, rows and all', () => {
    expect(pets.options(all, PetFacet.Species)).toEqual(['cat', 'dog', 'fish'])
    expect(pets.options(all, PetFacet.Colour)).toEqual([
      'black',
      'white',
      'ginger',
    ])
  })
})
