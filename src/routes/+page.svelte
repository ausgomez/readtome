<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface AppConfig {
    hotkey: string;
    provider: string;
    voice_id: string;
    playback_speed: number;
  }

  interface VoiceInfo {
    id: string;
    name: string;
  }

  let config = $state<AppConfig>({
    hotkey: "CmdOrCtrl+Shift+R",
    provider: "local",
    voice_id: "",
    playback_speed: 1.0,
  });

  let voices = $state<VoiceInfo[]>([]);
  let status = $state("");
  let saving = $state(false);

  onMount(async () => {
    try {
      config = await invoke<AppConfig>("get_config");
      voices = await invoke<VoiceInfo[]>("get_voices");
    } catch (e) {
      status = `Failed to load: ${e}`;
    }
  });

  async function save() {
    saving = true;
    status = "";
    try {
      await invoke("save_config", { newConfig: config });
      status = "Saved";
      setTimeout(() => { if (status === "Saved") status = ""; }, 2000);
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      saving = false;
    }
  }

  async function testSpeak() {
    try {
      await invoke("test_speak");
    } catch (e) {
      status = `TTS error: ${e}`;
    }
  }
</script>

<main>
  <h1>ReadToMe</h1>
  <p class="subtitle">Settings</p>

  <div class="field">
    <label for="hotkey">Hotkey</label>
    <input id="hotkey" type="text" bind:value={config.hotkey} placeholder="CmdOrCtrl+Shift+R" />
    <span class="hint">e.g. CmdOrCtrl+Shift+R, Alt+Space</span>
  </div>

  <div class="field">
    <label for="provider">TTS Provider</label>
    <select id="provider" bind:value={config.provider}>
      <option value="local">Local (System Voices)</option>
      <option value="elevenlabs" disabled>ElevenLabs (coming soon)</option>
      <option value="google" disabled>Google Cloud TTS (coming soon)</option>
    </select>
  </div>

  <div class="field">
    <label for="voice">Voice</label>
    <select id="voice" bind:value={config.voice_id}>
      <option value="">System Default</option>
      {#each voices as voice}
        <option value={voice.id}>{voice.name}</option>
      {/each}
    </select>
  </div>

  <div class="field">
    <label for="speed">Speed: {config.playback_speed.toFixed(1)}x</label>
    <input
      id="speed"
      type="range"
      min="0.5"
      max="3.0"
      step="0.1"
      bind:value={config.playback_speed}
    />
    <div class="range-labels">
      <span>0.5x</span>
      <span>1.0x</span>
      <span>2.0x</span>
      <span>3.0x</span>
    </div>
  </div>

  <div class="actions">
    <button onclick={save} disabled={saving}>
      {saving ? "Saving..." : "Save"}
    </button>
    <button class="secondary" onclick={testSpeak}>Test Voice</button>
  </div>

  {#if status}
    <p class="status" class:error={status.startsWith("Error") || status.startsWith("TTS error") || status.startsWith("Failed")}>{status}</p>
  {/if}
</main>

<style>
  :root {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 14px;
    line-height: 1.5;
    color: #1a1a1a;
    background-color: #fafafa;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #e0e0e0;
      background-color: #1e1e1e;
    }
    input, select {
      background-color: #2a2a2a;
      color: #e0e0e0;
      border-color: #444;
    }
    input:focus, select:focus {
      border-color: #5b9bd5;
    }
    button {
      background-color: #3a7bd5;
    }
    button:hover {
      background-color: #2d6bc4;
    }
    button.secondary {
      background-color: #333;
      color: #e0e0e0;
      border-color: #555;
    }
    button.secondary:hover {
      background-color: #444;
    }
    .hint {
      color: #888;
    }
    .range-labels span {
      color: #888;
    }
  }

  main {
    padding: 24px;
    max-width: 440px;
    margin: 0 auto;
  }

  h1 {
    font-size: 20px;
    font-weight: 600;
    margin: 0 0 2px 0;
  }

  .subtitle {
    margin: 0 0 24px 0;
    color: #888;
    font-size: 13px;
  }

  .field {
    margin-bottom: 20px;
  }

  label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 6px;
  }

  input[type="text"], select {
    width: 100%;
    padding: 8px 10px;
    border: 1px solid #ccc;
    border-radius: 6px;
    font-size: 14px;
    box-sizing: border-box;
  }

  input[type="text"]:focus, select:focus {
    outline: none;
    border-color: #3a7bd5;
    box-shadow: 0 0 0 2px rgba(58, 123, 213, 0.15);
  }

  input[type="range"] {
    width: 100%;
    margin: 4px 0;
  }

  .range-labels {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: #999;
    margin-top: 2px;
  }

  .hint {
    display: block;
    font-size: 11px;
    color: #999;
    margin-top: 4px;
  }

  .actions {
    display: flex;
    gap: 10px;
    margin-top: 28px;
  }

  button {
    padding: 8px 20px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    background-color: #3a7bd5;
    color: white;
  }

  button:hover {
    background-color: #2d6bc4;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  button.secondary {
    background-color: #f0f0f0;
    color: #333;
    border: 1px solid #ccc;
  }

  button.secondary:hover {
    background-color: #e4e4e4;
  }

  .status {
    margin-top: 12px;
    font-size: 13px;
    color: #3a7bd5;
  }

  .status.error {
    color: #d9534f;
  }
</style>
