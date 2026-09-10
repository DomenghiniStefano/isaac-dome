<script setup lang="ts">
import MarkCell from '@/components/marks/MarkCell.vue'
</script>

<template>
  <!-- Not "never done": outside the denominator of every percentage. The cell is decoration
       here, the text beside it says what it is. -->
  <span class="inline-flex items-center gap-2.5">
    <MarkCell :cell="{ kind: 'unknown' }" :art="null" />
    <span class="text-caption text-muted-foreground"><slot /></span>
  </span>
</template>
