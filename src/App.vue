<script setup lang="ts">
import IslandView from '@/components/island/IslandView.vue'
import SettingsView from '@/components/settings/SettingsView.vue'
const settings = new URLSearchParams(location.search).get('view') === 'settings'
</script>
<template><SettingsView v-if="settings" /><IslandView v-else /></template>
