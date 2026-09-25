// A field that is there only when it has a value: `{ box }` when there is a box, `{}` when there
// is none — never `{ box: undefined }`, which is a key every reader of a stored document, a
// message or a comparison has to ask about, and which `JSON.stringify` and `toEqual` read two
// different ways. Spread into the object being built.
export const withOptional = <K extends string, V>(
  key: K,
  value: V | undefined,
): Partial<Record<K, V>> =>
  value === undefined ? {} : ({ [key]: value } as Record<K, V>)

// A flag that is `true` or absent, never `false`: the value `withOptional` keeps for a switch
// whose off position is the default.
export const whenTrue = (flag: boolean): true | undefined =>
  flag ? true : undefined
