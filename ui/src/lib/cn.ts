import type { ClassValue } from 'clsx'
import { clsx } from 'clsx'
import { extendTailwindMerge } from 'tailwind-merge'
import motion from '@/assets/theme/motion.css?raw'
import opacity from '@/assets/theme/opacity.css?raw'
import radius from '@/assets/theme/radius.css?raw'
import spacing from '@/assets/theme/spacing.css?raw'
import typography from '@/assets/theme/typography.css?raw'
import { ThemeNamespace, themeKeys } from '@/lib/design/themeKeys'

// shadcn's cn() is twMerge(clsx(...)) with no configuration, and with our tokens that
// drops classes silently: tailwind-merge only knows t-shirt font sizes, reads `text-body`
// as a colour, and cn('text-body', 'text-foreground') keeps only the second. The names
// come from the theme files themselves, so a token is still declared in one place.
// tailwind-merge has no theme key for durations, nor for opacity: they extend the class
// group instead. An unknown class is kept rather than dropped, so before this
// `cn('opacity-muted', 'opacity-disabled')` came out as both — and with both on the element
// the winner is the one written later in the stylesheet, not the one the caller asked for
// last. Nothing in `src/` composes two of them today: this is the guard, not a repair.
const merge = extendTailwindMerge({
  extend: {
    theme: {
      text: themeKeys(typography, ThemeNamespace.Text),
      font: themeKeys(typography, ThemeNamespace.Font),
      tracking: themeKeys(typography, ThemeNamespace.Tracking),
      spacing: themeKeys(spacing, ThemeNamespace.Spacing),
      radius: themeKeys(radius, ThemeNamespace.Radius),
      ease: themeKeys(motion, ThemeNamespace.Ease),
      animate: themeKeys(motion, ThemeNamespace.Animate),
    },
    classGroups: {
      duration: [
        { duration: themeKeys(motion, ThemeNamespace.TransitionDuration) },
      ],
      // The names join the group the numbers are already in, rather than replacing it:
      // `opacity-0` still wins over `opacity-muted`, in either order.
      opacity: [{ opacity: themeKeys(opacity, ThemeNamespace.Opacity) }],
    },
  },
})

export const cn = (...inputs: ClassValue[]) => merge(clsx(inputs))
