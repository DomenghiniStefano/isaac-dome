import { onScopeDispose, ref } from 'vue'
import type { Ref } from 'vue'

/** From `now` to the next local midnight. Built from the calendar, not by adding a day's
 * milliseconds, so a day that daylight saving makes 23 or 25 hours long still ends at 00:00. */
export const msUntilTomorrow = (now: Date): number =>
  new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1).getTime() -
  now.getTime()

/**
 * The current date, moved on at every midnight. A `computed` that calls `new Date()` reads
 * the clock once and never again, and the app lives in the tray: "today" read at launch was
 * still "today" the morning after (card #80, P10).
 */
export const useToday = (): Ref<Date> => {
  const today = ref(new Date())
  let timer: ReturnType<typeof setTimeout> | undefined
  const arm = (): void => {
    timer = setTimeout(() => {
      today.value = new Date()
      arm()
    }, msUntilTomorrow(today.value))
  }
  arm()
  onScopeDispose(() => clearTimeout(timer))
  return today
}
