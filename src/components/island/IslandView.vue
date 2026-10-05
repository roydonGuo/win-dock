<script setup lang="ts">
import { computed, shallowRef, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { CloudSun, Search, Settings2, Plus, ChevronDown, Command, X, ArrowUpRight } from 'lucide-vue-next'
import { useDesktopState } from '@/composables/useDesktopState'
import { useWeather, weatherLabel } from '@/composables/useWeather'
import { command, native, onEvent } from '@/services/desktop'
import { filterApps } from '@/domain/apps'
import type { DockEntry, IslandMode } from '@/types'
import DockApp from './DockApp.vue'
import WindowPicker from './WindowPicker.vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

const desktop = useDesktopState()
const { config, apps, error, loading } = desktop
const settings = computed(() => config.value.settings)
const { weather, weatherError, fetching } = useWeather(settings)
const mode = shallowRef<IslandMode>('compact')
const query = shallowRef('')
const searchOpen = shallowRef(false)
const selectedId = shallowRef<string | null>(null)
const context = shallowRef(false)
const dragging = shallowRef('')
const now = shallowRef(new Date())
const hover = shallowRef(false)
const time = computed(() => now.value.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', hour12: false }))
const date = computed(() => now.value.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric', weekday: 'short' }))
const visible = computed(() => filterApps(apps.value, query.value))
const pinned = computed(() => visible.value.filter(a => a.pinned))
const running = computed(() => visible.value.filter(a => !a.pinned))
const selected = computed(() => apps.value.find(a => a.id === selectedId.value) ?? null)
const dockWidth = computed(() => Math.min(1100, Math.max(480, apps.value.length * 62 + 250)))
let interval: ReturnType<typeof setInterval>
let collapseTimer: ReturnType<typeof setTimeout> | undefined
let resizeObserver: ResizeObserver | undefined
let stopMode: (() => void) | undefined
let stopDrop: (() => void) | undefined
let stopGeometry: (() => void) | undefined
let disposed = false
let regionUpdatePending = false
let lastRegions = ''
async function updateHitRegions() {
 if (!native || regionUpdatePending || disposed) return
 regionUpdatePending = true
 await nextTick()
 const regions = Array.from(document.querySelectorAll<HTMLElement>('[data-hit-region]')).map(el => { const r = el.getBoundingClientRect(); return [r.x, r.y, r.width, r.height] })
 const serialized = JSON.stringify(regions)
 try { if (regions.length && serialized !== lastRegions) { await command('set_hit_regions', { regions }); lastRegions = serialized } }
 catch (e) { if (!error.value) error.value = String(e) }
 finally { regionUpdatePending = false }
}
async function setMode(next: IslandMode) {
 selectedId.value = null; searchOpen.value = false; query.value = ''; mode.value = next
 await desktop.run(() => command('set_island_mode', { expanded: next === 'dock', width: dockWidth.value }))
 await updateHitRegions()
}
async function selectApp(app: DockEntry) {
 if (app.windows.length > 1) { selectedId.value = selectedId.value === app.id ? null : app.id; context.value = false; searchOpen.value = false; return }
 if (app.windows.length === 1) await desktop.windowAction(app.windows[0]!.handle, app.windows[0]!.active ? 'minimize' : 'activate')
 else await desktop.launch(app.id)
 if (!error.value && settings.value.autoCollapse) await setMode('compact')
}
function showContext(app: DockEntry) { selectedId.value = app.id; context.value = true; searchOpen.value = false }
async function action(handle: string, action: 'activate' | 'minimize' | 'close') { await desktop.windowAction(handle, action); if (action === 'activate' && !error.value) await setMode('compact') }
async function changePin() { const app = selected.value; if (!app) return; if (app.pinned) await desktop.unpin(app.id); else await desktop.pin(app.id); selectedId.value = null }
function enter() { hover.value = true; clearTimeout(collapseTimer) }
function leave() { hover.value = false; if (mode.value === 'dock' && settings.value.autoCollapse && !selected.value && !searchOpen.value && !error.value) collapseTimer = setTimeout(() => setMode('compact'), 1800) }
function keydown(event: KeyboardEvent) {
 if (event.key === 'Escape') { if (selected.value) selectedId.value = null; else void setMode('compact') }
 if (!native && event.ctrlKey && event.altKey && event.code === 'Space') { event.preventDefault(); void setMode(mode.value === 'compact' ? 'dock' : 'compact') }
 if (event.key === 'Enter' && searchOpen.value && visible.value[0]) void selectApp(visible.value[0])
}
async function openSearch() { searchOpen.value = !searchOpen.value; selectedId.value = null; await nextTick(); document.querySelector<HTMLInputElement>('.dock-search input')?.focus() }
watch([mode, selected, context, searchOpen, hover, error], async () => {
 await nextTick()
 resizeObserver?.disconnect()
 document.querySelectorAll('[data-hit-region]').forEach(el => resizeObserver?.observe(el))
 await updateHitRegions()
 setTimeout(() => { if (!disposed) void updateHitRegions() }, 380)
})
watch(() => apps.value.length, () => { if (mode.value === 'dock') void desktop.run(() => command('set_island_mode', { expanded: true, width: dockWidth.value })) })
onMounted(async () => {
 interval = setInterval(() => { now.value = new Date() }, 1000)
 window.addEventListener('keydown', keydown)
 stopMode = await onEvent<IslandMode>('island-mode', value => { mode.value = value; if (value === 'compact') { query.value = ''; searchOpen.value = false; selectedId.value = null } })
 if (disposed) stopMode()
 stopGeometry = await onEvent('island-geometry', () => {
  lastRegions = ''; void updateHitRegions()
  setTimeout(() => { if (!disposed) { lastRegions = ''; void updateHitRegions() } }, 100)
 })
 if (disposed) stopGeometry()
 if (native) {
  const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
  stopDrop = await getCurrentWebviewWindow().onDragDropEvent(event => { if (event.payload.type === 'drop') event.payload.paths.forEach(path => void desktop.addApp(path)) })
  if (disposed) stopDrop()
 }
 resizeObserver = new ResizeObserver(() => void updateHitRegions()); document.querySelectorAll('[data-hit-region]').forEach(el => resizeObserver?.observe(el))
 await updateHitRegions()
})
onUnmounted(() => { disposed = true; clearInterval(interval); clearTimeout(collapseTimer); resizeObserver?.disconnect(); stopMode?.(); stopDrop?.(); stopGeometry?.(); window.removeEventListener('keydown', keydown) })
</script>

<template>
 <main class="island-root" :class="[{ 'browser-preview': !native, 'reduce-motion': settings.reducedMotion }, settings.theme]" @mouseenter="enter" @mouseleave="leave">
  <div v-if="!native" class="preview-heading"><div class="preview-brand"><span class="brand-mark" /> DESKTOP ISLAND</div><h1>让桌面，轻一点。</h1><p>平时是一座信息岛，需要时是你的应用中心。</p><span class="preview-tag">界面预览 · 系统操作请使用桌面应用</span></div>
  <div class="island-anchor">
   <div v-if="error" class="island-error glass" data-hit-region><span>{{ error }}</span><button aria-label="关闭提示" @click="error = ''"><X :size="14" /></button></div>
   <Transition name="popup"><WindowPicker v-if="selected" :app="selected" :context="context" @action="action" @pin="changePin" @launch="desktop.launch(selected.id)" @dismiss="selectedId = null" /></Transition>
   <Transition name="popup"><div v-if="searchOpen && mode === 'dock'" class="dock-search glass" data-hit-region><Search :size="16" /><Input v-model="query" placeholder="搜索应用或窗口…" aria-label="搜索应用或窗口" class="border-none bg-transparent shadow-none focus-visible:ring-0" /><span class="text-[10px] text-muted-foreground">ENTER ↵</span></div></Transition>
   <section class="island-surface glass" :class="{ expanded: mode === 'dock', hovered: hover && mode === 'compact' }" :style="{ '--dock-width': `${dockWidth}px` }" data-hit-region @contextmenu.prevent="desktop.run(() => command('open_settings'))">
    <template v-if="mode === 'compact'">
     <button class="compact-main" aria-label="展开 Dock" @click="setMode('dock')"><span class="status-light" /><span class="compact-date">{{ date }}</span><span class="compact-time">{{ time }}</span><span class="compact-divider" /><CloudSun :size="17" class="weather-icon" /><span class="compact-weather">{{ settings.weatherEnabled ? (weather ? `${weather.temperature}°` : fetching ? '…' : '—') : '你好' }}</span></button>
     <span class="compact-hint">{{ weather ? `${weather.city} · ${weatherLabel(weather.code)}` : weatherError || `${settings.shortcut} 唤醒` }}</span>
    </template>
    <template v-else>
     <button class="dock-clock" aria-label="收起 Dock" @click="setMode('compact')"><span>{{ time }}</span><span class="text-[10px] text-muted-foreground">{{ date }}</span></button><span class="dock-separator" />
     <div class="dock-apps">
      <DockApp v-for="app in pinned" :key="app.id" :app="app" @select="selectApp" @context="showContext" @drag="dragging = $event" @drop="desktop.reorder(dragging, $event)" />
      <span v-if="pinned.length && running.length" class="dock-separator" />
      <DockApp v-for="app in running" :key="app.id" :app="app" @select="selectApp" @context="showContext" @drag="dragging = $event" @drop="desktop.reorder(dragging, $event)" />
      <button v-if="!visible.length" class="empty-dock" @click="desktop.chooseApp()"><Plus :size="20" /><span>{{ loading ? '正在连接桌面…' : query ? '没有匹配的应用' : '添加你的第一个应用' }}</span></button>
     </div><span class="dock-separator" />
     <div class="dock-actions"><Button variant="ghost" size="icon" aria-label="搜索" :class="{ 'bg-white/10': searchOpen }" @click="openSearch"><Search :size="18" /></Button><Button variant="ghost" size="icon" aria-label="添加应用" @click="desktop.chooseApp()"><Plus :size="18" /></Button><Button variant="ghost" size="icon" aria-label="打开设置" @click="desktop.run(() => command('open_settings'))"><Settings2 :size="18" /></Button><Button variant="ghost" size="icon" aria-label="收起" @click="setMode('compact')"><ChevronDown :size="17" /></Button></div>
    </template>
   </section>
  </div>
  <div v-if="!native" class="preview-controls"><Button variant="outline" @click="setMode(mode === 'compact' ? 'dock' : 'compact')"><Command :size="15" />{{ mode === 'compact' ? '展开 Dock' : '收起信息岛' }}<ArrowUpRight :size="14" /></Button><Button variant="ghost" @click="desktop.run(() => command('open_settings'))">打开设置<Settings2 :size="14" /></Button><p>点击信息岛，或按 Ctrl + Alt + Space</p></div>
 </main>
</template>
