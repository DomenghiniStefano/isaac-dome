<script setup lang="ts">
import {
  Field,
  FieldDescription,
  FieldError,
  FieldLabel,
} from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { HelpTip } from '@/components/ui/tooltip'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Input · Label · Field">
    <Field>
      <div class="flex items-center gap-2">
        <FieldLabel for="kit-game-folder">Cartella del gioco</FieldLabel>
        <HelpTip
          >Trovata automaticamente, se Steam c'è. Questo è il posto in cui va
          una spiegazione: fuori dalla pagina, e una passata di mouse più in
          là.</HelpTip
        >
      </div>
      <Input id="kit-game-folder" placeholder="Scegli una cartella…" />
      <FieldDescription
        >Una FieldDescription resta per quello che è stato, non per quello che
        fa: una riga che compare solo in un certo stato.</FieldDescription
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
