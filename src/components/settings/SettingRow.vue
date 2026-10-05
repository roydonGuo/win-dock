<script setup lang="ts">
defineProps<{ title: string; description: string }>()
</script>
<template><div class="setting-row"><div class="setting-row-copy"><div class="setting-row-title">{{ title }}</div><p>{{ description }}</p></div><div class="setting-row-control"><slot /></div></div></template>
