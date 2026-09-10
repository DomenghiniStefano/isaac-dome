<script setup lang="ts">
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import KitSection from '../KitSection.vue'

const Tab = {
  Items: 'items',
  Challenges: 'challenges',
  Secrets: 'secrets',
} as const
</script>

<template>
  <KitSection title="Tabs">
    <Tabs :default-value="Tab.Items">
      <TabsList>
        <TabsTrigger :value="Tab.Items">Oggetti</TabsTrigger>
        <TabsTrigger :value="Tab.Challenges">Sfide</TabsTrigger>
        <TabsTrigger :value="Tab.Secrets">Segreti</TabsTrigger>
      </TabsList>
      <TabsContent :value="Tab.Items">118 oggetti mancanti.</TabsContent>
      <TabsContent :value="Tab.Challenges">9 sfide ancora aperte.</TabsContent>
      <TabsContent :value="Tab.Secrets">22 segreti da trovare.</TabsContent>
    </Tabs>
  </KitSection>
</template>
