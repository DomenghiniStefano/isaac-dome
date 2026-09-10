<script setup lang="ts">
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  Card,
  CardAction,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Card">
    <Card>
      <CardHeader>
        <CardTitle>01 · Priorità</CardTitle>
        <CardAction>sblocca 14</CardAction>
      </CardHeader>
      <CardContent class="flex gap-3">
        <div class="size-10 shrink-0 border border-input hatch-placeholder" />
        <div class="flex flex-col gap-1">
          <span class="text-heading">The Lost</span>
          <span class="text-caption text-muted-foreground">Mother · Hard</span>
        </div>
      </CardContent>
      <CardFooter>
        <Button class="flex-1">Segna come obiettivo</Button>
        <Button :variant="ButtonVariant.Outline">Dettagli</Button>
      </CardFooter>
    </Card>
  </KitSection>
</template>
