// A file size in the largest unit that keeps it at or above one. Binary steps (1024),
// because that is what Explorer shows beside the same file.
export const SizeUnit = {
  Byte: 'byte',
  Kilobyte: 'kilobyte',
  Megabyte: 'megabyte',
} as const
export type SizeUnit = (typeof SizeUnit)[keyof typeof SizeUnit]

const STEP = 1024

export const sizeUnit = (bytes: number): { value: number; unit: SizeUnit } => {
  if (bytes < STEP) return { value: bytes, unit: SizeUnit.Byte }
  if (bytes < STEP * STEP)
    return { value: bytes / STEP, unit: SizeUnit.Kilobyte }
  return { value: bytes / (STEP * STEP), unit: SizeUnit.Megabyte }
}

export const formatSize = (bytes: number, locale: string): string => {
  const { value, unit } = sizeUnit(bytes)
  return new Intl.NumberFormat(locale, {
    style: 'unit',
    unit,
    unitDisplay: 'short',
    maximumFractionDigits: 1,
  }).format(value)
}
