<script setup lang="ts">
// The filter the `crayon` utility names: fractal noise pushes every pixel of what it is put on
// by up to `scale` pixels, so a straight edge comes out drawn by hand. Mounted once per page
// root (`App.vue`, `KitPage.vue`): an id is unique per document, and every window is one.
</script>

<template>
  <svg aria-hidden="true" class="absolute size-0">
    <filter id="crayon" x="-10%" y="-10%" width="120%" height="120%">
      <feTurbulence
        type="fractalNoise"
        baseFrequency="0.09"
        numOctaves="2"
        seed="4"
        result="noise"
      />
      <feDisplacementMap
        in="SourceGraphic"
        in2="noise"
        scale="4"
        xChannelSelector="R"
        yChannelSelector="G"
      />
    </filter>
  </svg>
</template>
