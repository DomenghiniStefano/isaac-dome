import type { ClassValue } from 'clsx'
import { clsx } from 'clsx'
import { extendTailwindMerge } from 'tailwind-merge'
import motion from '@/assets/theme/motion.css?raw'
import radius from '@/assets/theme/radius.css?raw'
import spacing from '@/assets/theme/spacing.css?raw'
import typography from '@/assets/theme/typography.css?raw'
import { ThemeNamespace, themeKeys } from '@/lib/design/themeKeys'

// shadcn's cn() is twMerge(clsx(...)) with no configuration, and with our tokens that
// drops classes silently: tailwind-merge only knows t-shirt font sizes, reads `text-body`
// as a colour, and cn('text-body', 'text-foreground') keeps only the second. The names
// come from the theme files themselves, so a token is still declared in one place.
// tailwind-merge has no theme key for durations: they extend the class group instead.
const merge = extendTailwindMerge({
  extend: {
    theme: {
      text: themeKeys(typography, ThemeNamespace.Text),
      font: themeKeys(typography, ThemeNamespace.Font),
      spacing: themeKeys(spacing, ThemeNamespace.Spacing),
      radius: themeKeys(radius, ThemeNamespace.Radius),
      ease: themeKeys(motion, ThemeNamespace.Ease),
      animate: themeKeys(motion, ThemeNamespace.Animate),
    },
    classGroups: {
      duration: [
        { duration: themeKeys(motion, ThemeNamespace.TransitionDuration) },
      ],
    },
  },
})

export const cn = (...inputs: ClassValue[]) => merge(clsx(inputs))
