// HTML's default for a `<button>` with no `type` is `submit`: put one inside a `<form>` and
// clicking it submits the form, which is never what a Button here means. The default belongs
// in the primitive rather than in every caller — and only where we are the ones rendering the
// element: `as-child` renders the caller's own node, and `as` may be an anchor or a component,
// where `type` is an attribute that means nothing.
export const buttonType = (
  as: unknown,
  asChild: boolean | undefined,
): 'button' | undefined => (as === 'button' && !asChild ? 'button' : undefined)
