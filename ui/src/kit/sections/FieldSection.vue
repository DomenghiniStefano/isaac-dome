<script setup lang="ts">
import {
  Field,
  FieldDescription,
  FieldError,
  FieldLabel,
} from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Input · Label · Field">
    <Field>
      <FieldLabel for="kit-game-folder">Cartella del gioco</FieldLabel>
      <Input id="kit-game-folder" placeholder="Scegli una cartella…" />
      <FieldDescription
        >Trovata automaticamente, se Steam c'è.</FieldDescription
      >
    </Field>
    <Field>
      <FieldLabel for="kit-save-folder">Cartella dei salvataggi</FieldLabel>
      <Input
        id="kit-save-folder"
        aria-invalid="true"
        default-value="D:\Giochi"
      />
      <FieldError :errors="['Nessun salvataggio in questa cartella.']" />
    </Field>
    <Field>
      <FieldLabel for="kit-disabled">Disabilitato</FieldLabel>
      <Input id="kit-disabled" disabled default-value="—" />
    </Field>
  </KitSection>
</template>
