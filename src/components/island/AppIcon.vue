<script setup lang="ts">
import { AppWindow } from 'lucide-vue-next'
defineProps<{ icon: string | null; name: string }>()
</script>
<template><img v-if="icon" :src="icon" :alt="name" class="h-full w-full object-contain" draggable="false" /><span v-else class="app-fallback"><AppWindow :size="24" /><span>{{ name.slice(0, 1).toUpperCase() }}</span></span></template>
