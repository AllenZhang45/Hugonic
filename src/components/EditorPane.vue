<template>
  <section class="editor" aria-label="编辑器">
    <div class="tabs">
      <span class="tab-empty">未打开文件</span>
    </div>
    <div class="empty">
      <p class="empty-title">没有打开的文件</p>
      <p class="empty-hint">编辑器将在 M2 接入（Monaco Editor）</p>
    </div>
  </section>
</template>

<style scoped>
.editor {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  background: var(--bg-color);
}

.tabs {
  display: flex;
  align-items: center;
  height: 34px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.tab-empty {
  padding: 0 12px;
  color: var(--text-tertiary);
  font-size: 12px;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  margin: auto;
  text-align: center;
}

.empty-title {
  margin: 0;
  color: var(--text-secondary);
  font-size: 13px;
}

.empty-hint {
  margin: 0;
  color: var(--text-tertiary);
  font-size: 11px;
}
</style>
