<script setup lang="ts">
import { ref } from 'vue'
import NavBar from '@/components/shell/NavBar.vue'
import { NavSection } from '@/components/shell/navSection'
import KitSection from '../../KitSection.vue'

const section = ref<NavSection>(NavSection.Wiki)
</script>

<template>
  <KitSection title="NavBar" class="col-span-3">
    <NavBar v-model:section="section" :focused="true" />
    <NavBar :section="NavSection.Progress" :focused="false" />
  </KitSection>
</template>
