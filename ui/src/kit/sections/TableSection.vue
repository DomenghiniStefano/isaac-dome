<script setup lang="ts">
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Table,
  TableBody,
  TableCell,
  TableDensity,
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import KitSection from '../KitSection.vue'

const rows = [
  {
    name: 'Godhead',
    condition: 'Sfida 33',
    status: BadgeVariant.Now,
    label: 'Ora',
    quality: '4',
  },
  {
    name: 'Sacred Orb',
    condition: 'The Lost, Mother (Hard)',
    status: BadgeVariant.Blocked,
    label: 'Da 2',
    quality: '4',
  },
  {
    name: "Dad's Note",
    condition: 'Colonna non ancora localizzata',
    status: BadgeVariant.Unknown,
    label: '',
    quality: '—',
  },
  {
    name: 'Brimstone',
    condition: "10 uccisioni di Mom's Heart",
    status: BadgeVariant.Done,
    label: 'Fatto',
    quality: '4',
  },
]
</script>

<template>
  <KitSection title="Table" class="col-span-2">
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead class="w-9"><Checkbox /></TableHead>
          <TableHead class="w-9" />
          <TableHead>Sblocca</TableHead>
          <TableHead>Condizione</TableHead>
          <TableHead>Stato</TableHead>
          <TableHead class="text-right">Q</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow v-for="row in rows" :key="row.name" tabindex="0">
          <TableCell><Checkbox /></TableCell>
          <TableCell>
            <div class="size-5 border border-border hatch-placeholder" />
          </TableCell>
          <TableCell>{{ row.name }}</TableCell>
          <TableCell class="text-foreground-soft">{{
            row.condition
          }}</TableCell>
          <TableCell>
            <Badge :variant="row.status">{{ row.label }}</Badge>
          </TableCell>
          <TableCell class="text-right">{{ row.quality }}</TableCell>
        </TableRow>
      </TableBody>
      <TableFooter>
        <TableRow>
          <TableCell :colspan="6" class="py-2">0 di 4 selezionate</TableCell>
        </TableRow>
      </TableFooter>
    </Table>
    <Table :density="TableDensity.Compact">
      <TableBody>
        <TableRow v-for="row in rows" :key="row.name">
          <TableCell>{{ row.name }}</TableCell>
          <TableCell class="text-right">{{ row.quality }}</TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </KitSection>
</template>
