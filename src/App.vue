<script setup lang="ts">
import ActivityBar from "./components/ActivityBar.vue";
import AnalyticsView from "./components/AnalyticsView.vue";
import AppTitleBar from "./components/AppTitleBar.vue";
import BuildModal from "./components/BuildModal.vue";
import DashboardView from "./components/DashboardView.vue";
import DeployModal from "./components/DeployModal.vue";
import EditorPane from "./components/EditorPane.vue";
import ExplorerPanel from "./components/ExplorerPanel.vue";
import MediaView from "./components/MediaView.vue";
import PreviewPane from "./components/PreviewPane.vue";
import StatusBar from "./components/StatusBar.vue";
import ThemesView from "./components/ThemesView.vue";
import { useUiStore } from "./stores/ui";

const ui = useUiStore();
</script>

<template>
  <div class="app">
    <AppTitleBar />
    <div class="workspace">
      <ActivityBar />

      <div v-show="ui.activeView === 'content'" class="content-workspace">
        <ExplorerPanel />
        <EditorPane />
        <PreviewPane />
      </div>

      <DashboardView v-if="ui.activeView === 'dashboard'" />
      <AnalyticsView v-else-if="ui.activeView === 'analytics'" />
      <MediaView v-else-if="ui.activeView === 'media'" />
      <ThemesView v-else-if="ui.activeView === 'themes'" />
    </div>
    <StatusBar />
    <BuildModal />
    <DeployModal />
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-color);
}

.workspace {
  display: flex;
  flex: 1;
  min-height: 0;
}

.content-workspace {
  display: flex;
  flex: 1;
  min-width: 0;
  min-height: 0;
}
</style>
