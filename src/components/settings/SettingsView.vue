<script setup lang="ts">
import { computed, reactive, shallowRef, watch, onMounted } from 'vue'
import { Waves, SlidersHorizontal, Layers3, AppWindow, Keyboard, Monitor, CloudSun, ShieldCheck, Plus, X, Check, ArrowUpRight, RotateCcw, Power, Search, Sun, Moon, GripVertical, ChevronRight } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import SettingRow from './SettingRow.vue'
import AppIcon from '@/components/island/AppIcon.vue'
import { useDesktopState } from '@/composables/useDesktopState'
import { command, native } from '@/services/desktop'
import { defaultSettings } from '@/types'
const desktop = useDesktopState()
const { config, snapshot, error, busy } = desktop
const tab = shallowRef('appearance')
const draft = reactive({ ...defaultSettings })
const saved = shallowRef(false)
const autostart = shallowRef(false)
const autoBusy = shallowRef(false)
const cityQuery = shallowRef('')
const cityBusy = shallowRef(false)
const cities = shallowRef<{ id: number; name: string; country: string; admin1?: string; latitude: number; longitude: number }[]>([])
const dragging = shallowRef('')
const changed = computed(() => JSON.stringify(draft) !== JSON.stringify(config.value.settings))
const nav = [
 { id: 'appearance', label: '外观与交互', icon: SlidersHorizontal },
 { id: 'apps', label: '应用管理', icon: AppWindow },
 { id: 'shortcuts', label: '键盘快捷键', icon: Keyboard },
 { id: 'system', label: '桌面与任务栏', icon: Layers3 },
 { id: 'monitors', label: '显示器', icon: Monitor },
 { id: 'weather', label: '天气', icon: CloudSun },
 { id: 'general', label: '通用与恢复', icon: ShieldCheck },
]
const currentNav = computed(() => nav.find(n => n.id === tab.value)!)
watch(() => JSON.stringify(config.value.settings), () => { Object.assign(draft, config.value.settings); saved.value = false }, { immediate: true })
watch(() => draft.theme, theme => {
 document.documentElement.classList.toggle('light', theme === 'light')
 document.documentElement.classList.toggle('dark', theme === 'dark')
}, { immediate: true })
async function save() { saved.value = await desktop.updateSettings({ ...draft }) }
async function toggleAutostart(value: boolean) {
 autoBusy.value = true
 await desktop.run(async () => {
  if (!native) throw new Error('开机启动需要桌面应用')
  const plugin = await import('@tauri-apps/plugin-autostart'); await (value ? plugin.enable() : plugin.disable()); autostart.value = await plugin.isEnabled()
 })
 autoBusy.value = false
}
async function searchCity() {
 if (!cityQuery.value.trim()) return
 cityBusy.value = true
 await desktop.run(async () => {
  const result = await fetch(`https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(cityQuery.value.trim())}&count=5&language=zh&format=json`, { signal: AbortSignal.timeout(12000) })
  if (!result.ok) throw new Error('城市搜索暂不可用')
  cities.value = (await result.json()).results ?? []
  if (!cities.value.length) throw new Error('没有找到城市，请尝试拼音或英文名称')
 })
 cityBusy.value = false
}
function selectCity(city: typeof cities.value[number]) { draft.city = city.name; draft.latitude = city.latitude; draft.longitude = city.longitude; cities.value = []; cityQuery.value = '' }
onMounted(async () => { if (native) await desktop.run(async () => { autostart.value = await (await import('@tauri-apps/plugin-autostart')).isEnabled() }) })
</script>

<template>
 <div class="settings-shell" :class="draft.theme">
  <aside class="settings-sidebar">
   <div class="settings-brand"><span class="settings-logo"><Waves :size="23" /></span><div>Desktop Island<span>你的桌面，恰到好处。</span></div></div>
   <div class="sidebar-caption">偏好设置</div>
   <nav aria-label="设置分类"><button v-for="item in nav" :key="item.id" :class="{ selected: tab === item.id }" @click="tab = item.id"><component :is="item.icon" :size="17" />{{ item.label }}<span v-if="tab === item.id" class="nav-dot" /></button></nav>
   <div class="sidebar-bottom"><div class="flex items-center gap-2"><span class="status-light" /><span>{{ native ? '桌面服务已连接' : '浏览器界面预览' }}</span></div><div class="mt-3 text-[10px] text-muted-foreground">DESKTOP ISLAND <span class="float-right">v0.1.0</span></div></div>
  </aside>
  <main class="settings-main">
   <header class="settings-topbar"><span>偏好设置 <ChevronRight :size="12" /> {{ currentNav.label }}</span><Button variant="ghost" size="sm" @click="desktop.run(() => command('set_island_mode', { expanded: true }))">唤醒信息岛<ArrowUpRight :size="13" /></Button></header>
   <div class="settings-content">
    <div class="settings-page-heading"><div class="section-eyebrow">MAKE IT YOURS</div><h1>{{ currentNav.label }}</h1><p>{{ tab === 'appearance' ? '一点透明，一点灵动。让信息岛自然融入你的桌面。' : tab === 'apps' ? '常用的留在这里，运行的随时可见。' : tab === 'system' ? '选择你喜欢的桌面方式，所有改动都可以恢复。' : tab === 'weather' ? '窗外的天气，在桌面上轻轻一瞥。' : '调整信息岛，让每一次操作都更顺手。' }}</p></div>

    <template v-if="tab === 'appearance'">
     <div class="appearance-preview"><div class="preview-caption">信息岛 · 收起状态<span>LIVE PREVIEW</span></div><div class="sample-island glass"><span class="status-light" /><span class="text-[11px] text-muted-foreground">10月6日 周二</span><span class="text-xl font-medium tabular-nums">09:41</span><span class="h-4 w-px bg-white/15" /><CloudSun :size="18" /><span class="text-xs">{{ draft.weatherEnabled ? '天气' : '你好' }}</span></div><div class="preview-description">安静地陪伴，随时准备展开。</div></div>
     <h2 class="settings-section-title">视觉风格</h2>
     <div class="theme-options"><button :class="{ chosen: draft.theme === 'dark' }" @click="draft.theme = 'dark'"><span class="theme-swatch dark-swatch"><span /><Moon :size="17" /></span><span>深色玻璃<Check v-if="draft.theme === 'dark'" :size="14" /></span></button><button :class="{ chosen: draft.theme === 'light' }" @click="draft.theme = 'light'"><span class="theme-swatch light-swatch"><span /><Sun :size="17" /></span><span>浅色玻璃<Check v-if="draft.theme === 'light'" :size="14" /></span></button></div>
     <div class="settings-card"><SettingRow title="底部间距" description="信息岛与屏幕工作区底部的距离"><div class="flex items-center gap-3"><Input v-model.number="draft.bottomOffset" type="number" min="0" max="160" class="w-20 text-center" aria-label="底部间距" /><span class="text-xs text-muted-foreground">px</span></div></SettingRow><SettingRow title="自动收起" description="鼠标离开 Dock 后，回到安静的信息岛"><Switch v-model="draft.autoCollapse" aria-label="自动收起" /></SettingRow><SettingRow title="全屏时隐藏" description="观看视频或进入游戏时，让内容占据整个屏幕"><Switch v-model="draft.hideFullscreen" aria-label="全屏时隐藏" /></SettingRow><SettingRow title="减少动态效果" description="保留状态变化，减少缩放与过渡动画"><Switch v-model="draft.reducedMotion" aria-label="减少动态效果" /></SettingRow></div>
    </template>

    <template v-else-if="tab === 'apps'">
     <div class="flex items-center justify-between pb-4"><h2 class="settings-section-title !mb-0 !mt-0">固定应用 <span class="ml-2 text-muted-foreground">{{ config.pinned.length }}</span></h2><Button size="sm" @click="desktop.chooseApp()"><Plus :size="15" />添加应用</Button></div>
     <div class="settings-card"><div v-if="!config.pinned.length" class="apps-empty"><AppWindow :size="32" /><h3>把常用应用留在岛上</h3><p>添加程序或快捷方式，也可以直接拖入 Dock。</p><Button variant="outline" size="sm" @click="desktop.chooseApp()"><Plus :size="14" />选择应用</Button></div><div v-for="app in config.pinned" :key="app.id" class="pinned-row" draggable="true" @dragstart="dragging = app.id" @dragover.prevent @drop.prevent="desktop.reorder(dragging, app.id)"><GripVertical :size="15" class="text-muted-foreground" /><span class="h-9 w-9"><AppIcon :icon="app.icon" :name="app.name" /></span><div class="min-w-0 flex-1"><div class="text-sm">{{ app.name }}</div><div class="mt-1 truncate text-[10px] text-muted-foreground">{{ app.path }}</div></div><Button variant="ghost" size="icon" :aria-label="`取消固定 ${app.name}`" @click="desktop.unpin(app.id)"><X :size="15" /></Button></div></div>
     <h2 class="settings-section-title">正在运行</h2><div class="settings-card"><div v-if="!snapshot.apps.length" class="p-6 text-center text-xs text-muted-foreground">{{ native ? '当前没有可切换的应用窗口' : '桌面应用会在这里显示真实运行窗口' }}</div><div v-for="app in snapshot.apps" :key="app.id" class="pinned-row"><span class="h-9 w-9"><AppIcon :icon="app.icon" :name="app.name" /></span><div class="min-w-0 flex-1"><div class="text-sm">{{ app.name }}</div><div class="mt-1 text-[10px] text-muted-foreground">{{ app.windows.length }} 个窗口</div></div><Button variant="outline" size="sm" :disabled="config.pinned.some(p => p.id === app.id)" @click="desktop.pin(app.id)">{{ config.pinned.some(p => p.id === app.id) ? '已固定' : '固定' }}</Button></div></div>
    </template>

    <template v-else-if="tab === 'shortcuts'">
     <div class="settings-card"><SettingRow title="唤醒信息岛" description="注册系统快捷键；被其他应用占用时会提示失败"><Input v-model="draft.shortcut" class="w-48 text-center font-mono text-xs" aria-label="唤醒快捷键" /></SettingRow><SettingRow title="双击 Win 键" description="实验功能：保留系统 Win 行为，第一次按键可能打开开始菜单"><Switch v-model="draft.doubleWin" aria-label="双击 Win 键" /></SettingRow></div>
     <div class="help-note"><Keyboard :size="18" /><div><h3>保持熟悉的 Windows 操作</h3><p>Win + E、Win + R 和 Win + D 保持系统行为。Dock 中按 Esc 收起，搜索后按 Enter 打开第一个结果。</p></div></div>
    </template>

    <template v-else-if="tab === 'system'">
     <div class="settings-card"><SettingRow title="Windows 任务栏" description="推荐使用自动隐藏；完全隐藏为 Explorer 兼容功能"><Select v-model="draft.taskbar"><SelectTrigger class="w-36" aria-label="任务栏模式"><SelectValue /></SelectTrigger><SelectContent><SelectItem value="system">跟随系统</SelectItem><SelectItem value="autoHide">自动隐藏</SelectItem><SelectItem value="hidden">完全隐藏</SelectItem></SelectContent></Select></SettingRow><SettingRow title="显示桌面图标" :description="draft.manageDesktopIcons ? '由信息岛管理，退出时恢复修改前状态；不删除文件' : '目前跟随系统。切换后将由信息岛管理显隐'"><div class="flex items-center gap-3"><Button v-if="draft.manageDesktopIcons" size="sm" variant="ghost" class="text-[10px]" @click="draft.manageDesktopIcons = false">跟随系统</Button><Switch :model-value="draft.manageDesktopIcons ? draft.desktopIcons : snapshot.desktopIconsVisible ?? draft.desktopIcons" aria-label="显示桌面图标" @update:model-value="value => { draft.desktopIcons = value; draft.manageDesktopIcons = true }" /></div></SettingRow></div>
     <div class="help-note"><ShieldCheck :size="20" /><div><h3>系统状态可以恢复</h3><p>每次修改前记录原始状态。正常退出或恢复进程检测到异常退出时，尝试恢复仍由本应用控制的设置。</p><Button variant="outline" size="sm" class="mt-4" @click="desktop.restore()"><RotateCcw :size="14" />立即恢复系统状态</Button></div></div>
    </template>

    <template v-else-if="tab === 'monitors'">
     <div class="settings-card"><SettingRow title="显示信息岛的屏幕" description="显示器断开时自动回到主屏，支持不同 DPI"><Select v-model="draft.monitor"><SelectTrigger class="w-44" aria-label="显示器"><SelectValue placeholder="主显示器" /></SelectTrigger><SelectContent><SelectItem value="primary">主显示器（自动）</SelectItem><SelectItem v-for="monitor in snapshot.monitors" :key="monitor.id" :value="monitor.id">{{ monitor.name }}</SelectItem></SelectContent></Select></SettingRow></div>
     <div class="monitor-list"><button v-for="monitor in snapshot.monitors" :key="monitor.id" class="monitor-card" :class="{ chosen: draft.monitor === monitor.id || ((!draft.monitor || draft.monitor === 'primary') && monitor.primary) }" @click="draft.monitor = monitor.id"><Monitor :size="46" :stroke-width="1" /><span>{{ monitor.name }}</span><small>{{ monitor.primary ? '主屏幕 · ' : '' }}{{ Math.round(monitor.scale * 100) }}% 缩放</small></button></div>
    </template>

    <template v-else-if="tab === 'weather'">
     <div class="settings-card"><SettingRow title="显示天气" description="使用 Open-Meteo 实时天气，约每 30 分钟更新"><Switch v-model="draft.weatherEnabled" aria-label="显示天气" /></SettingRow><SettingRow title="所在城市" description="手动选择城市，不读取设备定位"><span class="text-sm">{{ draft.city }}</span></SettingRow></div>
     <h2 class="settings-section-title">选择城市</h2><form class="flex gap-3" @submit.prevent="searchCity"><Input v-model="cityQuery" placeholder="搜索城市，如 Beijing、Shanghai…" aria-label="搜索城市" /><Button :disabled="cityBusy || !cityQuery.trim()"><Search :size="14" />{{ cityBusy ? '搜索中' : '搜索' }}</Button></form><div v-if="cities.length" class="settings-card mt-3"><button v-for="city in cities" :key="city.id" class="city-result" @click="selectCity(city)"><span>{{ city.name }}</span><span>{{ city.admin1 }} · {{ city.country }}</span><Plus :size="15" /></button></div><p class="mt-4 text-[11px] leading-6 text-muted-foreground">天气请求仅发送所选城市的经纬度。服务不可用时显示“—”，不会展示示例天气。数据来源：Open-Meteo。</p>
    </template>

    <template v-else>
     <div class="settings-card"><SettingRow title="开机启动" description="登录 Windows 后，自动唤醒信息岛"><Switch :model-value="autostart" :disabled="autoBusy || !native" aria-label="开机启动" @update:model-value="toggleAutostart" /></SettingRow><SettingRow title="恢复桌面状态" description="撤销本应用对任务栏和桌面图标的设置"><Button variant="outline" size="sm" @click="desktop.restore()"><RotateCcw :size="14" />恢复</Button></SettingRow><SettingRow title="退出 Desktop Island" description="恢复系统设置后退出，后台恢复进程也将结束"><Button variant="outline" size="sm" @click="desktop.run(() => command('quit_app'))"><Power :size="14" />退出应用</Button></SettingRow></div><div class="about-card"><Waves :size="28" /><h3>Desktop Island</h3><p>一个安静、轻盈、随时响应的桌面入口。</p><small>Tauri 2 · Vue 3 · shadcn-vue · Tailwind CSS</small></div>
    </template>
    <div v-if="error" class="settings-error" role="alert">{{ error }}<button aria-label="关闭错误" @click="error = ''"><X :size="14" /></button></div>
   </div>
   <footer class="settings-footer"><span v-if="saved && !changed" class="flex items-center gap-2 text-primary"><Check :size="14" />设置已保存</span><span v-else>{{ changed ? '有未保存的更改' : '设置将在保存后应用到信息岛' }}</span><div class="flex gap-2"><Button variant="ghost" size="sm" :disabled="!changed || busy" @click="Object.assign(draft, config.settings)">撤销</Button><Button size="sm" :disabled="!changed || busy" @click="save">{{ busy ? '正在保存…' : '保存更改' }}</Button></div></footer>
  </main>
 </div>
</template>

<style scoped>
.settings-shell { display: flex; height: 100vh; overflow: hidden; background: var(--background); color: var(--foreground); }
.settings-sidebar { width: 228px; flex-shrink: 0; border-right: 1px solid var(--border); padding: 30px 16px 22px; display: flex; flex-direction: column; background: color-mix(in srgb, var(--card) 55%, var(--background)); }
.settings-brand { display: flex; gap: 11px; align-items: center; padding: 0 8px; font-size: 14px; font-weight: 600; }
.settings-logo { width: 37px; height: 37px; border: 1px solid #a7d5bc25; color: var(--primary); background: #a7d5bc0b; border-radius: 12px; display: grid; place-items: center; }
.settings-brand div span { display: block; font-size: 9px; font-weight: 400; color: var(--muted-foreground); margin-top: 5px; letter-spacing: .5px; }
.sidebar-caption { color: var(--muted-foreground); font-size: 9px; margin: 41px 12px 13px; letter-spacing: 1px; }
nav { display: flex; flex-direction: column; gap: 5px; } nav button { display: flex; align-items: center; gap: 11px; padding: 12px 13px; border-radius: 8px; font-size: 12px; text-align: left; color: var(--muted-foreground); background: transparent; border: 0; } nav button:hover { background: var(--secondary); } nav button.selected { background: #a7d5bc0e; color: var(--primary); } .nav-dot { margin-left: auto; width: 4px; height: 4px; border-radius: 50%; background: var(--primary); }
.sidebar-bottom { margin-top: auto; border-top: 1px solid var(--border); padding: 19px 10px 0; font-size: 10px; color: var(--muted-foreground); }
.settings-main { min-width: 0; flex: 1; display: flex; flex-direction: column; }
.settings-topbar { height: 61px; border-bottom: 1px solid var(--border); display: flex; align-items: center; justify-content: space-between; padding: 0 36px; flex-shrink: 0; }
.settings-topbar > span { display: flex; gap: 10px; align-items: center; font-size: 10px; color: var(--muted-foreground); }
.settings-topbar button { font-size: 10px; color: var(--muted-foreground); }
.settings-content { padding: 34px 38px 36px; overflow-y: auto; flex: 1; }
.section-eyebrow { font-size: 8px; letter-spacing: 2.3px; color: var(--primary); }
.settings-page-heading h1 { font-size: 26px; font-weight: 500; letter-spacing: -.8px; margin: 11px 0 10px; }
.settings-page-heading p { font-size: 11px; color: var(--muted-foreground); margin-bottom: 28px; line-height: 1.8; }
.appearance-preview { height: 187px; background: radial-gradient(ellipse at 30% 110%, #486a544a, transparent 65%), radial-gradient(ellipse at 80% 0%, #4b5c723a, transparent 65%), var(--card); border: 1px solid var(--border); border-radius: 15px; position: relative; display: flex; align-items: center; justify-content: center; }
.preview-caption { position: absolute; top: 16px; left: 18px; right: 18px; color: var(--muted-foreground); font-size: 9px; }
.preview-caption span { float: right; font-size: 7px; letter-spacing: 1.3px; opacity: .6; }
.sample-island { display: flex; align-items: center; gap: 13px; border-radius: 30px; padding: 19px 25px; }
.preview-description { position: absolute; bottom: 18px; font-size: 9px; color: var(--muted-foreground); }
.settings-section-title { font-size: 11px; font-weight: 500; margin: 25px 0 13px; }
.theme-options { display: flex; gap: 13px; margin-bottom: 22px; }
.theme-options > button { width: 50%; border: 1px solid var(--border); padding: 7px; border-radius: 12px; background: var(--card); }
.theme-options > button.chosen { border-color: #a7d5bc70; }
.theme-swatch { height: 56px; border-radius: 7px; display: flex; align-items: center; justify-content: center; gap: 16px; }
.dark-swatch { background: linear-gradient(130deg, #263331, #1a2331); color: #9dab9f; }
.light-swatch { background: linear-gradient(130deg, #e3eae5, #cbd6dd); color: #708779; }
.theme-swatch > span { height: 18px; width: 80px; background: #ffffff15; border: 1px solid #ffffff30; border-radius: 20px; }
.light-swatch > span { background: #ffffff75; border-color: #ffffff; }
.theme-options > button > span:last-child { display: flex; align-items: center; justify-content: space-between; padding: 10px 8px 3px; font-size: 10px; }
.settings-card { border: 1px solid var(--border); background: var(--card); border-radius: 12px; overflow: hidden; }
:deep(.setting-row) { display: flex; align-items: center; justify-content: space-between; gap: 20px; padding: 18px 20px; border-bottom: 1px solid var(--border); }
:deep(.setting-row:last-child) { border-bottom: 0; }
:deep(.setting-row-title) { font-size: 11px; font-weight: 500; }
:deep(.setting-row-copy p) { font-size: 9px; color: var(--muted-foreground); margin: 6px 0 0; line-height: 1.6; max-width: 330px; }
:deep(.setting-row-control) { flex-shrink: 0; }
.settings-footer { height: 66px; padding: 0 38px; border-top: 1px solid var(--border); display: flex; align-items: center; justify-content: space-between; flex-shrink: 0; }
.settings-footer > span { font-size: 9px; color: var(--muted-foreground); }
.settings-footer button { font-size: 10px; }
.apps-empty { padding: 37px 20px; display: flex; flex-direction: column; align-items: center; color: var(--muted-foreground); }
.apps-empty h3 { font-size: 13px; margin-top: 17px; color: var(--foreground); }.apps-empty p { font-size: 10px; margin: 8px 0 18px; }
.pinned-row { display: flex; align-items: center; gap: 14px; padding: 15px 18px; border-bottom: 1px solid var(--border); }.pinned-row:last-child { border: 0; }
.help-note { display: flex; gap: 14px; background: #a7d5bc06; border: 1px solid #a7d5bc15; border-radius: 12px; padding: 21px; margin-top: 22px; color: var(--primary); }.help-note > svg { flex-shrink: 0; }.help-note h3 { font-size: 12px; }.help-note p { margin-top: 7px; font-size: 10px; line-height: 1.9; color: var(--muted-foreground); }
.monitor-list { display: flex; flex-wrap: wrap; gap: 16px; margin-top: 24px; }.monitor-card { padding: 25px; display: flex; flex-direction: column; align-items: center; gap: 14px; background: var(--card); border: 1px solid var(--border); border-radius: 14px; min-width: 180px; }.monitor-card.chosen { border-color: var(--primary); }.monitor-card span { font-size: 12px; }.monitor-card small { font-size: 10px; color: var(--muted-foreground); }
.city-result { display: flex; align-items: center; gap: 12px; width: 100%; padding: 16px; font-size: 12px; text-align: left; }.city-result:hover { background: var(--secondary); }.city-result span:nth-child(2) { margin-left: auto; color: var(--muted-foreground); font-size: 10px; }
.settings-error { display: flex; justify-content: space-between; gap: 10px; color: #e9acae; background: #e775790c; border: 1px solid #e7757920; padding: 14px; border-radius: 10px; font-size: 11px; margin-top: 20px; }
.about-card { text-align: center; color: var(--muted-foreground); padding-top: 50px; }.about-card svg { margin: 0 auto; color: var(--primary); }.about-card h3 { margin: 15px 0 8px; font-size: 16px; color: var(--foreground); }.about-card p { font-size: 11px; }.about-card small { display: block; font-size: 9px; margin-top: 18px; }
@media(max-width: 850px) { .settings-sidebar { width: 190px; }.settings-content { padding: 24px; }.settings-topbar { padding: 0 24px; }.settings-footer { padding: 0 24px; } }
</style>
