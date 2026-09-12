// A route query value is a string, an array of them, or nothing: only one string is a query.
// The Wiki, Unlock, the Collection and the Search screen all read their location the same way.
export const singleQuery = (value: unknown): string | null =>
  typeof value === 'string' ? value : null
