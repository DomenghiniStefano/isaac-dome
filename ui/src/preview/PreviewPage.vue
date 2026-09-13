<script setup lang="ts">
// The tab's label, in the query: a title is not a path, an offset or a profile, and nothing
// else travels with it. This page is not part of the app's routing and never becomes one —
// which is also why it has no `@tauri-apps/api` import: the window that owns it moves it.
const label = new URLSearchParams(window.location.search).get('label') ?? ''
</script>

<template>
  <div
    class="flex h-screen w-screen items-center gap-1.5 overflow-hidden border-2 border-primary bg-sheet px-2 text-label text-foreground select-none"
  >
    <span class="min-w-0 flex-1 truncate">{{ label }}</span>
  </div>
</template>
