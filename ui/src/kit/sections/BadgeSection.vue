<script setup lang="ts">
import { Badge, BadgeVariant } from '@/components/ui/badge'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Badge">
    <div class="flex flex-wrap items-center gap-2">
      <Badge :variant="BadgeVariant.Done">Fatto</Badge>
      <Badge :variant="BadgeVariant.Now">Ora</Badge>
      <Badge :variant="BadgeVariant.Blocked">Da 2</Badge>
      <Badge :variant="BadgeVariant.Unknown">Sconosciuto</Badge>
      <Badge :variant="BadgeVariant.Unexpected">Anomalo</Badge>
      <Badge :variant="BadgeVariant.Tag">Oggetto</Badge>
      <Badge :variant="BadgeVariant.Challenge">Sfida</Badge>
    </div>
  </KitSection>
</template>
