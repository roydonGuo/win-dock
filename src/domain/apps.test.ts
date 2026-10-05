import { describe, expect, it } from 'vitest'
import { mergeApps, filterApps, movePinned } from './apps'
import type { PinnedApp, RunningApp } from '@/types'
const pinned: PinnedApp[] = [{ id: 'editor', name: 'Editor', path: 'C:\\Editor.exe', args: '', icon: null }]
const running: RunningApp[] = [{ id: 'editor', name: 'Editor', path: 'C:\\Editor.exe', icon: null, windows: [{ handle: '123', title: 'file.ts', active: true, minimized: false }] }]
describe('Dock application model', () => {
 it('matches an executable pin against a running window with a Shell identity', () => {
  const entries = mergeApps(pinned, [{ ...running[0]!, id: 'aumid:editor' }])
  expect(entries).toHaveLength(1); expect(entries[0]!.windows).toHaveLength(1)
 })
 it('matches shortcuts by their resolved target while preserving their launch path', () => {
  const entries = mergeApps([{ ...pinned[0]!, id: 'shortcut', path: 'C:\\Desktop\\Editor.lnk', matchPath: 'C:\\Editor.exe' }], running)
  expect(entries).toHaveLength(1); expect(entries[0]!.path).toBe('C:\\Desktop\\Editor.lnk')
 })
 it('keeps separate Shell application identities separate even when their executables match', () => {
  const entries = mergeApps([{ ...pinned[0]!, id: 'aumid:web-app' }], [{ ...running[0]!, id: 'aumid:browser' }])
  expect(entries).toHaveLength(2); expect(entries[0]!.windows).toHaveLength(0)
 })
 it('merges a running pinned app into one entry with its windows', () => {
  expect(mergeApps(pinned, running)).toMatchObject([{ id: 'editor', pinned: true, windows: [{ handle: '123' }] }])
  expect(mergeApps(pinned, running)).toHaveLength(1)
 })
 it('retains pinned order and appends unpinned running apps', () => {
  expect(mergeApps(pinned, [...running, { ...running[0]!, id: 'browser', name: 'Browser', path: 'C:\\Browser.exe' }]).map(a => a.id)).toEqual(['editor', 'browser'])
 })
 it('searches application and window names without case sensitivity', () => {
  expect(filterApps(mergeApps(pinned, running), 'FILE')).toHaveLength(1)
  expect(filterApps(mergeApps(pinned, running), 'missing')).toHaveLength(0)
 })
 it('reorders pinned apps without mutating input or losing entries', () => {
  const input = [...pinned, { ...pinned[0]!, id: 'browser' }]
  expect(movePinned(input, 'browser', 'editor').map(a => a.id)).toEqual(['browser', 'editor'])
  expect(input[0]!.id).toBe('editor')
 })
})
