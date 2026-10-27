import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface CredentialStatus {
  available: boolean;
  hasToken: boolean;
}

export interface NetlifyAccount {
  name: string;
  email: string | null;
}

export interface DeployDone {
  ok: boolean;
  url: string | null;
  siteName: string;
  deployId: string | null;
  error: string | null;
}

export type DeployStatus = "idle" | "running" | "done" | "error";

export const useDeployStore = defineStore("deploy", () => {
  const credentialAvailable = ref(true);
  const hasToken = ref(false);
  const account = ref<NetlifyAccount | null>(null);
  const status = ref<DeployStatus>("idle");
  const logs = ref<string[]>([]);
  const url = ref<string | null>(null);
  const error = ref<string | null>(null);
  const siteName = ref("");
  const modalOpen = ref(false);
  const busy = ref(false);

  let listening = false;
  const unlisteners: UnlistenFn[] = [];

  async function ensureListeners() {
    if (listening) {
      return;
    }
    listening = true;

    unlisteners.push(
      await listen<{ line: string }>("deploy://log", (event) => {
        logs.value.push(event.payload.line);
      }),
      await listen<DeployDone>("deploy://done", (event) => {
        const payload = event.payload;
        status.value = payload.ok ? "done" : "error";
        url.value = payload.url;
        error.value = payload.error;
        siteName.value = payload.siteName;
      }),
    );
  }

  async function refreshCredentials() {
    try {
      const state = await invoke<CredentialStatus>("netlify_credential_status");
      credentialAvailable.value = state.available;
      hasToken.value = state.hasToken;
      if (!state.hasToken) {
        account.value = null;
      }
      return state;
    } catch (cause) {
      credentialAvailable.value = false;
      hasToken.value = false;
      error.value = String(cause);
      return null;
    }
  }

  async function saveToken(token: string) {
    busy.value = true;
    error.value = null;
    try {
      await invoke("save_netlify_token", { token });
      await refreshCredentials();
      if (hasToken.value) {
        await testConnection();
      }
    } catch (cause) {
      error.value = String(cause);
    } finally {
      busy.value = false;
    }
  }

  async function clearToken() {
    try {
      await invoke("clear_netlify_token");
    } finally {
      hasToken.value = false;
      account.value = null;
    }
  }

  async function testConnection() {
    busy.value = true;
    try {
      account.value = await invoke<NetlifyAccount>("test_netlify_connection");
      error.value = null;
      return true;
    } catch (cause) {
      account.value = null;
      error.value = String(cause);
      return false;
    } finally {
      busy.value = false;
    }
  }

  async function start(name: string) {
    if (status.value === "running") {
      return;
    }

    const trimmed = name.trim();
    if (!trimmed) {
      error.value = "站点名不能为空";
      return;
    }

    await ensureListeners();

    logs.value = [];
    url.value = null;
    error.value = null;
    status.value = "running";
    siteName.value = trimmed;

    try {
      await invoke("start_deploy", { siteName: trimmed });
    } catch (cause) {
      status.value = "error";
      error.value = String(cause);
    }
  }

  function openModal() {
    modalOpen.value = true;
    void refreshCredentials();
  }

  function closeModal() {
    modalOpen.value = false;
  }

  return {
    credentialAvailable,
    hasToken,
    account,
    status,
    logs,
    url,
    error,
    siteName,
    modalOpen,
    busy,
    refreshCredentials,
    saveToken,
    clearToken,
    testConnection,
    start,
    openModal,
    closeModal,
  };
});
