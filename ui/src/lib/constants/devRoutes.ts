// Hash routes that only exist under `pnpm ui:dev`: main.ts checks import.meta.env.DEV
// before honouring them, and the production build drops what they import.
export const DevRoute = { Kit: '#kit' } as const
export type DevRoute = (typeof DevRoute)[keyof typeof DevRoute]
