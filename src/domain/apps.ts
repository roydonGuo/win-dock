import type { DockEntry, PinnedApp, RunningApp } from '@/types'
export function mergeApps(pinned: PinnedApp[], running: RunningApp[]): DockEntry[] {
 const active = new Map(running.map(a => [a.id, a]))
 const normalize = (path: string) => path.replaceAll('/', '\\').toLocaleLowerCase()
 const result: DockEntry[] = pinned.map(pin => {
  const matches = [...active.values()].filter(app => app.id === pin.id || ((!pin.id.startsWith('aumid:') || !app.id.startsWith('aumid:')) && normalize(app.path) === normalize(pin.matchPath ?? pin.path)))
  matches.forEach(app => active.delete(app.id))
  return { ...pin, icon: pin.icon ?? matches[0]?.icon ?? null, pinned: true, windows: matches.flatMap(app => app.windows) }
 })
 for (const app of active.values()) result.push({ ...app, args: '', pinned: false })
 return result
}
export function filterApps(apps: DockEntry[], query: string): DockEntry[] {
 const search = query.trim().toLocaleLowerCase()
 return search ? apps.filter(a => [a.name, a.path, ...a.windows.map(w => w.title)].some(s => s.toLocaleLowerCase().includes(search))) : apps
}
export function movePinned(pinned: PinnedApp[], from: string, to: string): PinnedApp[] {
 const source = pinned.findIndex(a => a.id === from); const target = pinned.findIndex(a => a.id === to)
 if (source < 0 || target < 0 || source === target) return [...pinned]
 const next = [...pinned]; const [item] = next.splice(source, 1); next.splice(target, 0, item!); return next
}
