// aria-current's value for the item that is the page being shown.
export const AriaCurrent = { Page: 'page' } as const
export type AriaCurrent = (typeof AriaCurrent)[keyof typeof AriaCurrent]
