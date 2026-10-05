import { shallowRef, computed, onMounted, onUnmounted } from 'vue'
import { command, onEvent, native } from '@/services/desktop'
import { defaultSettings, type Config, type Snapshot, type Settings } from '@/types'
import { mergeApps, movePinned } from '@/domain/apps'
export function useDesktopState() {
 const config = shallowRef<Config>({ settings: { ...defaultSettings }, pinned: [] })
 const snapshot = shallowRef<Snapshot>({ apps: [], fullscreen: false, monitors: [] })
 const error = shallowRef('')
 const loading = shallowRef(true)
 const busy = shallowRef(false)
 const apps = computed(() => mergeApps(config.value.pinned, snapshot.value.apps))
 const cleanups: (() => void)[] = []
 let disposed = false
 async function run<T>(action: () => Promise<T>): Promise<T | undefined> {
  error.value = ''; try { return await action() } catch (e) { error.value = String(e instanceof Error ? e.message : e); return undefined }
 }
 onMounted(async () => {
  await run(async () => {
   for (const [name, handler] of [
    ['config-changed', (value: Config) => { config.value = value }],
    ['desktop-snapshot', (value: Snapshot) => { snapshot.value = value }],
    ['native-error', (value: string) => { error.value = value }],
   ] as const) {
    const off = await onEvent(name, handler as (value: unknown) => void); if (disposed) off(); else cleanups.push(off)
   }
   const [c, s] = await Promise.all([command<Config>('get_config'), command<Snapshot>('get_snapshot')]); config.value = c; snapshot.value = s
  })
  loading.value = false
 })
 onUnmounted(() => { disposed = true; cleanups.forEach(off => off()) })
 async function save(next: Config) {
  if (busy.value) return false
  busy.value = true
  const result = await run(() => command<Config>('save_config', { config: next }))
  if (result) config.value = result
  busy.value = false
  return !!result
 }
 async function updateSettings(settings: Settings) { return save({ ...config.value, settings: { ...settings } }) }
 async function unpin(id: string) { return save({ ...config.value, pinned: config.value.pinned.filter(a => a.id !== id) }) }
 async function reorder(from: string, to: string) { return save({ ...config.value, pinned: movePinned(config.value.pinned, from, to) }) }
 async function addApp(path: string) { const result = await run(() => command<Config>('add_app', { path })); if (result) config.value = result }
 async function chooseApp() {
  await run(async () => {
   if (!native) throw new Error('请在桌面应用中添加程序；浏览器仅用于界面预览')
   const { open } = await import('@tauri-apps/plugin-dialog')
   const path = await open({ multiple: false, directory: false, filters: [{ name: 'Windows 应用', extensions: ['exe', 'lnk', 'url'] }] })
   if (typeof path === 'string') await addApp(path)
  })
 }
 async function pin(id: string) { const result = await run(() => command<Config>('pin_running', { id })); if (result) config.value = result }
 async function windowAction(handle: string, action: 'activate' | 'minimize' | 'close') { return run(() => command('window_action', { handle, action })) }
 async function launch(id: string) { return run(() => command('launch_app', { id })) }
 async function restore() { const result = await run(() => command<Config>('restore_system')); if (result) config.value = result }
 return { config, snapshot, apps, error, loading, busy, run, save, updateSettings, unpin, reorder, addApp, chooseApp, pin, windowAction, launch, restore }
}
