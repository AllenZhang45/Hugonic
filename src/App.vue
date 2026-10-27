<script setup lang="ts">
import ActivityBar from "./components/ActivityBar.vue";
import AppTitleBar from "./components/AppTitleBar.vue";
import BuildModal from "./components/BuildModal.vue";
import DeployModal from "./components/DeployModal.vue";
import EditorPane from "./components/EditorPane.vue";
import ExplorerPanel from "./components/ExplorerPanel.vue";
import PreviewPane from "./components/PreviewPane.vue";
import StatusBar from "./components/StatusBar.vue";
</script>

<template>
  <div class="app">
    <AppTitleBar />
    <div class="workspace">
      <ActivityBar />
      <ExplorerPanel />
      <EditorPane />
      <PreviewPane />
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
</style>
