<script setup lang="ts">
import { computed } from 'vue'
import type { DockEntry } from '@/types'
import AppIcon from './AppIcon.vue'
const props = defineProps<{ app: DockEntry }>()
defineEmits<{ select: [app: DockEntry]; context: [app: DockEntry]; drag: [id: string]; drop: [id: string] }>()
const active = computed(() => props.app.windows.some(w => w.active))
</script>
<template>
 <button class="dock-app group" :class="{ active }" :aria-label="`${app.name}${app.windows.length ? `，${app.windows.length} 个窗口` : ''}`" :draggable="app.pinned" @click="$emit('select', app)" @contextmenu.prevent="$emit('context', app)" @dragstart="$emit('drag', app.id)" @dragover.prevent @drop.prevent="$emit('drop', app.id)">
  <span class="dock-tooltip">{{ app.name }}</span><span class="dock-icon"><AppIcon :icon="app.icon" :name="app.name" /></span>
  <span class="running-mark" :class="{ running: app.windows.length, active }" />
  <span v-if="app.windows.length > 1" class="window-count">{{ app.windows.length }}</span>
 </button>
</template>
