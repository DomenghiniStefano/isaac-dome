<script setup lang="ts">
import { ref } from 'vue'
import NavBar from '@/components/shell/NavBar.vue'
import { NavSection } from '@/lib/shell/navSection'
import KitSection from '../../KitSection.vue'

const section = ref<NavSection>(NavSection.Wiki)
</script>

<template>
  <KitSection title="NavBar" class="col-span-3">
    <!-- The three bars also carry the three states of the history arrows: something behind
         only, nothing either side, both ways open. -->
    <NavBar
      v-model:section="section"
      :settings-active="false"
      :focused="true"
      :can-back="true"
      :can-forward="false"
    />
    <NavBar
      :section="NavSection.Progress"
      :settings-active="false"
      :focused="false"
      :can-back="false"
      :can-forward="false"
    />
    <NavBar
      :section="null"
      :settings-active="true"
      :focused="true"
      :can-back="true"
      :can-forward="true"
    />
  </KitSection>
</template>
