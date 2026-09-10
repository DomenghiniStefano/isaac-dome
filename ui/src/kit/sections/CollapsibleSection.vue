<script setup lang="ts">
import { ChevronRightIcon } from '@lucide/vue'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible'
import KitSection from '../KitSection.vue'

const blocking = ['Sacred Orb', 'Death Certificate', "Dad's Note"]
</script>

<template>
  <KitSection title="Collapsible">
    <Collapsible>
      <CollapsibleTrigger
        class="group flex cursor-pointer items-center gap-2 text-row text-foreground"
      >
        <ChevronRightIcon
          class="size-3.5 text-muted-foreground transition-transform duration-panel ease-panel group-data-[state=open]:rotate-90"
        />
        3 voci bloccanti
      </CollapsibleTrigger>
      <CollapsibleContent class="mt-2 border border-border bg-data">
        <p
          v-for="name in blocking"
          :key="name"
          class="border-b border-hairline px-2.5 py-1.5 text-row last:border-b-0"
        >
          {{ name }}
        </p>
      </CollapsibleContent>
    </Collapsible>
  </KitSection>
</template>
