// The dotted path of every string in a message tree: { ui: { close } } gives 'ui.close'.
export type MessageKey<T> = {
  [K in keyof T & string]: T[K] extends string ? K : `${K}.${MessageKey<T[K]>}`
}[keyof T & string]
