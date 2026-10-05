import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { defaultSettings, type Config, type Snapshot } from '@/types'
export const native = isTauri()
const configKey = 'desktop-island-preview-config'
let previewConfig: Config = (() => { try { return JSON.parse(localStorage.getItem(configKey) ?? 'null') ?? { settings: { ...defaultSettings }, pinned: [] } } catch { return { settings: { ...defaultSettings }, pinned: [] } } })()
previewConfig.settings = { ...defaultSettings, ...previewConfig.settings }
const emptySnapshot: Snapshot = { apps: [], fullscreen: false, monitors: [{ id: 'preview', name: '浏览器预览', primary: true, scale: 1 }] }
const previewEvents = new EventTarget()
export async function command<T>(name: string, args: Record<string, unknown> = {}): Promise<T> {
 if (native) return invoke<T>(name, args)
 switch (name) {
  case 'get_config': return structuredClone(previewConfig) as T
  case 'get_snapshot': return structuredClone(emptySnapshot) as T
  case 'save_config': previewConfig = structuredClone(args.config as Config); localStorage.setItem(configKey, JSON.stringify(previewConfig)); previewEvents.dispatchEvent(new CustomEvent('config-changed', { detail: previewConfig })); return structuredClone(previewConfig) as T
  case 'set_island_mode': previewEvents.dispatchEvent(new CustomEvent('island-mode', { detail: args.expanded ? 'dock' : 'compact' })); return undefined as T
  case 'set_hit_regions': return undefined as T
  case 'open_settings': window.open('/?view=settings', 'island-settings'); return undefined as T
  default: throw new Error('此操作需要在 Windows 桌面应用中使用')
 }
}
export async function onEvent<T>(name: string, callback: (payload: T) => void): Promise<UnlistenFn> {
 if (native) return listen<T>(name, event => callback(event.payload))
 const handler = (event: Event) => callback((event as CustomEvent<T>).detail)
 previewEvents.addEventListener(name, handler)
 return () => previewEvents.removeEventListener(name, handler)
}
