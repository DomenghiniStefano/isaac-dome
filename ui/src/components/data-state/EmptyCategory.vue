<template>
  <!-- A category with no pages: a dashed frame, never a fallback to another page, or the tab
       would say one thing and the content another. -->
  <div
    class="border border-dashed border-border px-3 py-2.5 text-caption text-subtle-foreground"
  >
    <slot />
  </div>
</template>
