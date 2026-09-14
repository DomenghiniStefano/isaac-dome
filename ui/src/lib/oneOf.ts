// The value of a const-object set that a string equals, or `undefined`.
//
// A filter's picks and a facet's values travel as plain strings — that is what a `Record` of
// picks can hold — and turning one back into a typed value is a narrowing every label does.
// It reads the set's *values*, never its keys: `OriginValue.Rebirth` is `'rebirth'`, and the
// string that reaches here is the value.
export const oneOf = <T extends string>(
  values: Record<string, T>,
  value: string,
): T | undefined => Object.values(values).find((v) => v === value)
