<script setup lang="ts">
import { X, Minus, SquareArrowOutUpRight } from 'lucide-vue-next'
import type { DockEntry } from '@/types'
import AppIcon from './AppIcon.vue'
import { Button } from '@/components/ui/button'
defineProps<{ app: DockEntry; context: boolean }>()
defineEmits<{ action: [handle: string, action: 'activate' | 'minimize' | 'close']; pin: []; launch: []; dismiss: [] }>()
</script>
<template>
 <section class="window-picker glass" data-hit-region>
  <header class="flex items-center gap-3 px-3 pb-3"><span class="h-8 w-8"><AppIcon :icon="app.icon" :name="app.name" /></span><div class="min-w-0 flex-1"><div class="truncate text-sm font-medium">{{ app.name }}</div><div class="text-[10px] text-muted-foreground">{{ app.windows.length }} 个窗口</div></div><Button variant="ghost" size="icon" class="h-7 w-7" aria-label="关闭窗口列表" @click="$emit('dismiss')"><X :size="14" /></Button></header>
  <div class="max-h-32 overflow-auto">
   <div v-for="window in app.windows" :key="window.handle" class="window-row"><button class="min-w-0 flex-1 truncate text-left text-xs" @click="$emit('action', window.handle, 'activate')"><span v-if="window.active" class="mr-2 text-emerald-300">●</span>{{ window.title }}</button><Button variant="ghost" size="icon" class="h-6 w-6" aria-label="最小化窗口" @click="$emit('action', window.handle, 'minimize')"><Minus :size="12" /></Button><Button variant="ghost" size="icon" class="h-6 w-6" aria-label="关闭应用窗口" @click="$emit('action', window.handle, 'close')"><X :size="12" /></Button></div>
  </div>
  <div v-if="context" class="mt-2 flex gap-1 border-t border-white/10 pt-2"><Button variant="ghost" size="sm" class="flex-1" @click="$emit('pin')">{{ app.pinned ? '取消固定' : '固定到 Dock' }}</Button><Button v-if="app.pinned" variant="ghost" size="sm" @click="$emit('launch')"><SquareArrowOutUpRight :size="13" />新开应用</Button></div>
 </section>
</template>
