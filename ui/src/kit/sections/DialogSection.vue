<script setup lang="ts">
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Dialog">
    <Dialog>
      <DialogTrigger as-child>
        <Button class="w-fit">Apri dialog</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Rileggi il salvataggio</DialogTitle>
        </DialogHeader>
        <DialogDescription>
          Le schermate mostreranno quello che il file dichiara adesso.
        </DialogDescription>
        <DialogFooter>
          <DialogClose as-child>
            <Button :variant="ButtonVariant.Outline">Annulla</Button>
          </DialogClose>
          <DialogClose as-child>
            <Button>Rileggi</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </KitSection>
</template>
