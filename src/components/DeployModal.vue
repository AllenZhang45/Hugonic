<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import Icon from "./Icon.vue";
import { useDeployStore } from "../stores/deploy";
import { useProjectStore } from "../stores/project";

const deploy = useDeployStore();
const project = useProjectStore();

const tokenInput = ref("");
const siteInput = ref("");

const running = computed(() => deploy.status === "running");

function slug(input: string) {
  const name = input
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 50);
  return name || "hugonic-site";
}

watch(
  () => deploy.modalOpen,
  (open) => {
    if (open && !siteInput.value) {
      siteInput.value = project.info ? slug(project.info.name) : "";
    }
  },
);

async function submitToken() {
  await deploy.saveToken(tokenInput.value);
  if (deploy.hasToken) {
    tokenInput.value = "";
  }
}

async function submitDeploy() {
  await deploy.start(siteInput.value);
}

async function copyUrl() {
  if (deploy.url) {
    await navigator.clipboard.writeText(deploy.url).catch(() => undefined);
  }
}

function openResult() {
  if (deploy.url) {
    void openUrl(deploy.url);
  }
}
</script>

<template>
  <div v-if="deploy.modalOpen" class="overlay" @click.self="deploy.closeModal()">
    <div class="modal" role="dialog" aria-label="部署" aria-modal="true">
      <div class="head">
        <Icon name="upload" class="head-icon" />
        <span class="title">部署</span>
        <span class="target">Netlify</span>
        <span v-if="running" class="status">部署中…</span>
        <button class="close" type="button" title="关闭" @click="deploy.closeModal()">×</button>
      </div>

      <div class="body">
        <p v-if="!deploy.credentialAvailable" class="warning">
          系统钥匙串不可用，无法安全保存 Netlify token。
        </p>

        <template v-if="!deploy.hasToken">
          <p class="label">Netlify personal access token</p>
          <div class="token-row">
            <input
              v-model="tokenInput"
              class="input"
              type="password"
              autocomplete="off"
              spellcheck="false"
              placeholder="粘贴 token（只保存在系统钥匙串）"
            />
            <button
              class="btn primary"
              type="button"
              :disabled="deploy.busy || !tokenInput.trim()"
              @click="submitToken"
            >
              {{ deploy.busy ? "保存中…" : "保存并测试" }}
            </button>
          </div>
          <p class="hint">
            在 Netlify：User settings → Applications → Personal access tokens 创建。
            账号级凭据，不会写入项目或日志。
          </p>
        </template>

        <template v-else>
          <div class="account">
            <span class="account-name">{{ deploy.account?.name ?? "已配置 token" }}</span>
            <span v-if="deploy.account?.email" class="account-email">{{
              deploy.account.email
            }}</span>
            <button
              class="link"
              type="button"
              :disabled="deploy.busy"
              @click="deploy.testConnection()"
            >
              测试连接
            </button>
            <button
              class="link danger"
              type="button"
              :disabled="deploy.busy"
              @click="deploy.clearToken()"
            >
              更换 token
            </button>
          </div>

          <p class="label">站点名（Netlify 子域名，全局唯一）</p>
          <div class="token-row">
            <input
              v-model="siteInput"
              class="input"
              type="text"
              spellcheck="false"
              placeholder="demo-site"
              :disabled="running"
            />
            <button
              class="btn primary"
              type="button"
              :disabled="running || !siteInput.trim()"
              @click="submitDeploy"
            >
              {{ running ? "部署中…" : "部署到 Netlify" }}
            </button>
          </div>
          <p class="hint">站点不存在时会自动创建；重名会提示换一个。</p>
        </template>

        <p v-if="deploy.error" class="error">{{ deploy.error }}</p>

        <div v-if="deploy.url" class="result">
          <span class="result-url">{{ deploy.url }}</span>
          <button class="btn" type="button" @click="copyUrl()">复制</button>
          <button class="btn" type="button" @click="openResult()">在浏览器打开</button>
        </div>

        <template v-if="deploy.logs.length">
          <p class="label">部署日志</p>
          <pre class="log">{{ deploy.logs.join("\n") }}</pre>
        </template>
      </div>

      <div class="actions">
        <span class="spacer" />
        <button class="btn" type="button" @click="deploy.closeModal()">关闭</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 30;
  display: grid;
  place-items: center;
  background: rgb(0 0 0 / 55%);
}

.modal {
  display: flex;
  flex-direction: column;
  width: min(680px, calc(100vw - 80px));
  max-height: calc(100vh - 80px);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  background: var(--bg-secondary);
  box-shadow: 0 24px 64px rgb(0 0 0 / 45%);
}

.head {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 44px;
  flex-shrink: 0;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-color);
}

.head-icon {
  width: 16px;
  height: 16px;
  color: var(--theme-color);
}

.title {
  font-size: 13px;
  font-weight: 600;
}

.target {
  padding: 1px 7px;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  color: var(--text-tertiary);
  font-size: 10px;
}

.status {
  margin-left: 4px;
  color: var(--theme-color);
  font-size: 11px;
}

.close {
  margin-left: auto;
  width: 24px;
  height: 24px;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
  font-size: 16px;
}

.close:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.body {
  padding: 14px 14px 4px;
  overflow: auto;
}

.label {
  margin: 10px 0 6px;
  color: var(--text-tertiary);
  font-size: 11px;
}

.token-row {
  display: flex;
  gap: 6px;
}

.input {
  flex: 1;
  min-width: 0;
  height: 30px;
  padding: 0 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-color);
  color: var(--text-primary);
  font: inherit;
  font-size: 12px;
}

.input:focus {
  border-color: var(--theme-color);
  outline: none;
}

.hint {
  margin: 6px 0 0;
  color: var(--text-tertiary);
  font-size: 11px;
  line-height: 1.6;
}

.warning {
  margin: 0 0 8px;
  padding: 6px 8px;
  border: 1px solid var(--warning-color);
  border-radius: var(--radius-sm);
  color: var(--warning-color);
  font-size: 11px;
}

.account {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-color);
}

.account-name {
  font-size: 12px;
  font-weight: 600;
}

.account-email {
  color: var(--text-tertiary);
  font-size: 11px;
}

.link {
  margin-left: auto;
  border: 0;
  background: transparent;
  color: var(--theme-color);
  font-size: 11px;
}

.link.danger {
  color: var(--text-tertiary);
}

.error {
  margin: 10px 0 0;
  padding: 8px 10px;
  border: 1px solid var(--error-color);
  border-radius: var(--radius-sm);
  background: rgb(239 68 68 / 10%);
  color: var(--error-color);
  font-size: 11px;
  user-select: text;
}

.result {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 10px;
  padding: 8px 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-color);
}

.result-url {
  flex: 1;
  overflow: hidden;
  color: var(--success-color);
  font-size: 12px;
  white-space: nowrap;
  text-overflow: ellipsis;
  user-select: text;
}

.log {
  max-height: 200px;
  margin: 0;
  padding: 8px 10px;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-color);
  color: var(--text-secondary);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  line-height: 1.55;
  white-space: pre-wrap;
  word-break: break-word;
  user-select: text;
}

.actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  padding: 10px 14px;
  border-top: 1px solid var(--border-color);
}

.spacer {
  flex: 1;
}

.btn {
  height: 28px;
  padding: 0 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
}

.btn:hover:not(:disabled) {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.btn:disabled {
  opacity: 0.45;
}

.btn.primary {
  border-color: var(--theme-color);
  background: var(--theme-color);
  color: #fff;
}
</style>
