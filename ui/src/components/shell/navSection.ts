import type { Component } from 'vue'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { TabOrigin } from './tabs'
import { tabOriginIcon } from './tabOriginIcon'

// The navbar's two sections (DESIGN-BRIEF.md §4): two preconditions, two places.
export const NavSection = { Wiki: 'wiki', Progress: 'progress' } as const
export type NavSection = (typeof NavSection)[keyof typeof NavSection]

// A section wears the icon of the tabs that come from it: the choice is made once, in
// tabOriginIcon.
export const navSectionIcon: Record<NavSection, Component> = {
  [NavSection.Wiki]: tabOriginIcon[TabOrigin.Wiki],
  [NavSection.Progress]: tabOriginIcon[TabOrigin.Progress],
}

export const navSectionLabel: Record<NavSection, MessageKey<MessageSchema>> = {
  [NavSection.Wiki]: 'shell.sections.wiki',
  [NavSection.Progress]: 'shell.sections.progress',
}
