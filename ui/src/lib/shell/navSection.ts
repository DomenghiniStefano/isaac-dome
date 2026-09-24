import type { Component } from 'vue'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { TabOrigin } from '@/lib/shell/tabs'
import { tabOriginIcon } from '@/lib/shell/tabOriginIcon'

// The navbar's three sections (DESIGN-BRIEF.md §4): three preconditions, three places.
// Progress reads the save, Tool reads the log or what you painted, the Wiki reads neither.
// The order of these keys is the order on screen — NavBar.vue draws `Object.values` — so
// moving a section is moving its line, and there is no index to keep in step with it.
export const NavSection = {
  Progress: 'progress',
  Tool: 'tool',
  Wiki: 'wiki',
} as const
export type NavSection = (typeof NavSection)[keyof typeof NavSection]

// A section wears the icon of the tabs that come from it: the choice is made once, in
// tabOriginIcon.
export const navSectionIcon: Record<NavSection, Component> = {
  [NavSection.Progress]: tabOriginIcon[TabOrigin.Progress],
  [NavSection.Tool]: tabOriginIcon[TabOrigin.Tool],
  [NavSection.Wiki]: tabOriginIcon[TabOrigin.Wiki],
}

export const navSectionLabel: Record<NavSection, MessageKey<MessageSchema>> = {
  [NavSection.Progress]: 'shell.sections.progress',
  [NavSection.Tool]: 'shell.sections.tool',
  [NavSection.Wiki]: 'shell.sections.wiki',
}
