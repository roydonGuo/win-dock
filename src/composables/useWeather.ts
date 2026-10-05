import { shallowRef, watch, onUnmounted, type Ref } from 'vue'
import type { Settings, Weather } from '@/types'
export const weatherLabel = (code: number) => code === 0 ? '晴' : code <= 3 ? '多云' : code <= 48 ? '雾' : code <= 67 ? '雨' : code <= 77 ? '雪' : code <= 82 ? '阵雨' : code <= 86 ? '阵雪' : '雷雨'
export function useWeather(settings: Ref<Settings>) {
 const weather = shallowRef<Weather | null>(null)
 const weatherError = shallowRef('')
 const fetching = shallowRef(false)
 let timer: ReturnType<typeof setInterval> | undefined
 const stop = watch(() => [settings.value.weatherEnabled, settings.value.latitude, settings.value.longitude, settings.value.city] as const, async ([enabled, latitude, longitude, city], _, cleanup) => {
  clearInterval(timer); weather.value = null; weatherError.value = ''
  if (!enabled) return
  const lifecycle = new AbortController(); cleanup(() => { lifecycle.abort(); clearInterval(timer) })
  async function fetchWeather() {
   if (lifecycle.signal.aborted) return
   fetching.value = true
   const controller = new AbortController()
   const abort = () => controller.abort()
   lifecycle.signal.addEventListener('abort', abort, { once: true })
   const timeout = setTimeout(() => controller.abort(), 12000)
   try {
    const res = await fetch(`https://api.open-meteo.com/v1/forecast?latitude=${latitude}&longitude=${longitude}&current=temperature_2m,weather_code`, { signal: controller.signal })
    if (!res.ok) throw new Error('天气服务暂不可用')
    const data = await res.json()
    if (!Number.isFinite(data.current?.temperature_2m) || !Number.isFinite(data.current?.weather_code)) throw new Error('天气数据不可用')
    if (!lifecycle.signal.aborted) { weather.value = { temperature: Math.round(data.current.temperature_2m), code: data.current.weather_code, city }; weatherError.value = '' }
   } catch (e) { if (!lifecycle.signal.aborted) weatherError.value = controller.signal.aborted ? '天气请求超时' : String(e instanceof Error ? e.message : e) }
   finally { clearTimeout(timeout); lifecycle.signal.removeEventListener('abort', abort); if (!lifecycle.signal.aborted) fetching.value = false }
  }
  await fetchWeather(); if (!lifecycle.signal.aborted) timer = setInterval(fetchWeather, 30 * 60 * 1000)
 }, { immediate: true })
 onUnmounted(() => { stop(); clearInterval(timer) })
 return { weather, weatherError, fetching }
}
