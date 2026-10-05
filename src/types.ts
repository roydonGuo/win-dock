export interface AppWindow { handle: string; title: string; active: boolean; minimized: boolean }
export interface PinnedApp { id: string; name: string; path: string; matchPath?: string | null; args: string; icon: string | null }
export interface RunningApp { id: string; name: string; path: string; launchPath?: string; icon: string | null; windows: AppWindow[] }
export interface DockEntry extends PinnedApp { pinned: boolean; windows: AppWindow[] }
export type IslandMode = 'compact' | 'dock'
export interface Settings { theme: 'dark' | 'light'; monitor: string; bottomOffset: number; autoCollapse: boolean; hideFullscreen: boolean; doubleWin: boolean; shortcut: string; taskbar: 'system' | 'autoHide' | 'hidden'; desktopIcons: boolean; manageDesktopIcons: boolean; weatherEnabled: boolean; city: string; latitude: number; longitude: number; reducedMotion: boolean }
export interface Config { settings: Settings; pinned: PinnedApp[] }
export interface MonitorInfo { id: string; name: string; primary: boolean; scale: number; width?: number; height?: number; work_area?: number[] }
export interface Snapshot { apps: RunningApp[]; fullscreen: boolean; monitors: MonitorInfo[]; desktopIconsVisible?: boolean | null }
export interface Weather { temperature: number; code: number; city: string }
export const defaultSettings: Settings = { theme: 'dark', monitor: '', bottomOffset: 18, autoCollapse: true, hideFullscreen: true, doubleWin: false, shortcut: 'Ctrl+Alt+Space', taskbar: 'system', desktopIcons: true, manageDesktopIcons: false, weatherEnabled: false, city: '北京', latitude: 39.9042, longitude: 116.4074, reducedMotion: false }
