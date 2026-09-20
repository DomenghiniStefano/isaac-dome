<script setup lang="ts">
import { Trash2Icon } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
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
import { useMessages } from '@/i18n'

// Emptying the grid is the one thing on this screen that cannot be undone: the painted floor
// lives in memory and nowhere else, so a stray click is a floor redrawn from scratch. Hence
// the red and the bin — the app's warning colour says "this one does not come back" — and
// hence a question before it happens.

const emit = defineEmits<{ clear: [] }>()
const { t } = useMessages()
</script>

<template>
  <Dialog>
    <DialogTrigger as-child>
      <Button :variant="ButtonVariant.Destructive" :size="ButtonSize.Compact">
        <Trash2Icon />
        {{ t('floor.clear') }}
      </Button>
    </DialogTrigger>
    <DialogContent>
      <DialogHeader>
        <DialogTitle>{{ t('floor.clearConfirm.title') }}</DialogTitle>
      </DialogHeader>
      <DialogDescription>{{ t('floor.clearConfirm.body') }}</DialogDescription>
      <DialogFooter>
        <DialogClose as-child>
          <Button :variant="ButtonVariant.Outline">{{
            t('floor.clearConfirm.cancel')
          }}</Button>
        </DialogClose>
        <DialogClose as-child>
          <Button :variant="ButtonVariant.Destructive" @click="emit('clear')">
            <Trash2Icon />
            {{ t('floor.clearConfirm.confirm') }}
          </Button>
        </DialogClose>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
