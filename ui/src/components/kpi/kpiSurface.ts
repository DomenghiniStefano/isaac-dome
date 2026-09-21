// What a KPI tile sits on. Two, and the reason is the surface underneath it rather than the
// number on it: a `Panel` draws its own bordered sheet, which is right on the page's own
// background and wrong on a band that is already a lit surface — three bordered boxes on a
// gradient read as three windows cut into it (card #58).
//
// It says nothing about the number. The tone does that, and the two are chosen apart.
export const KpiSurface = { Panel: 'panel', Bare: 'bare' } as const
export type KpiSurface = (typeof KpiSurface)[keyof typeof KpiSurface]
