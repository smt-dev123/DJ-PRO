import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface DeckSettings {
  volume: number;
  eq_high: number;
  eq_mid: number;
  eq_low: number;
  filter: number;
  gain: number;
  key_lock: boolean;
  fx_type: string;
  fx_amount: number;
  fx_params: any;
}

export interface AppSettings {
  crossfader: number;
  master_volume: number;
  deck_a: DeckSettings;
  deck_b: DeckSettings;
  recent_tracks: string[];
  version: number;
}

export function useSettingsStorage() {
  const settings = ref<AppSettings | null>(null);
  const activeDrive = ref<string>("");

  async function loadSettings(driveRoot: string) {
    activeDrive.value = driveRoot;
    try {
      const s = await invoke<AppSettings>("load_settings", { driveRoot });
      settings.value = s;
      console.log("[Settings] ✅ Loaded");
      return s;
    } catch (err) {
      console.error("[Settings] ❌ Load failed:", err);
      return null;
    }
  }

  async function saveSettings(s: AppSettings) {
    if (!activeDrive.value) return;
    try {
      await invoke("save_settings", {
        driveRoot: activeDrive.value,
        settings: s,
      });
      settings.value = s;
    } catch (err) {
      console.error("[Settings] ❌ Save failed:", err);
    }
  }

  return {
    settings,
    activeDrive,
    loadSettings,
    saveSettings,
  };
}