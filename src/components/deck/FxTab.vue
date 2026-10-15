<script setup lang="ts">
import { ref, computed } from "vue";
import FxSettings from "./FxSettings.vue";
import LandingSettings from "./LandingSettings.vue";

type FxType = "echo" | "reverb" | "delay" | "flanger" | "filter" | "roll";

const props = defineProps<{
  fxType: string;
  fxAmount: number;
  fxActive: boolean;
  params: any;
  landingSettings: any;  // ✅ Landing settings
}>();

const emit = defineEmits<{
  (e: "fxType", fxType: FxType): void;
  (e: "fxAmount", amount: number): void;
  (e: "fxActive", active: boolean): void;
  (e: "fxOut", fxType: "echo" | "reverb" | "delay" | "roll", duration: number): void;
  (e: "cancelFxOut"): void;
  (e: "updateParams", params: any): void;
  (e: "updateLandingSettings", fxType: "echo" | "reverb" | "delay" | "roll", settings: any): void;
}>();

const showFxSettings = ref(false);
const showLandingSettings = ref(false);
const landingSettingsFx = ref<"echo" | "reverb" | "delay" | "roll">("echo");

// ✅ FX List — តែអក្សរ
const FX_LIST: {
  id: FxType;
  label: string;
  activeClass: string;
}[] = [
  {
    id: "echo",
    label: "ECHO",
    activeClass: "bg-cyan-500 border-cyan-300 text-black shadow-[0_0_10px_rgba(6,182,212,0.7)]",
  },
  {
    id: "reverb",
    label: "REVERB",
    activeClass: "bg-blue-500 border-blue-300 text-white shadow-[0_0_10px_rgba(59,130,246,0.7)]",
  },
  {
    id: "delay",
    label: "DELAY",
    activeClass: "bg-amber-500 border-amber-300 text-black shadow-[0_0_10px_rgba(245,158,11,0.7)]",
  },
  {
    id: "flanger",
    label: "FLANGER",
    activeClass: "bg-purple-500 border-purple-300 text-white shadow-[0_0_10px_rgba(168,85,247,0.7)]",
  },
  {
    id: "filter",
    label: "FILTER",
    activeClass: "bg-pink-500 border-pink-300 text-white shadow-[0_0_10px_rgba(236,72,153,0.7)]",
  },
  {
    id: "roll",
    label: "ROLL",
    activeClass: "bg-green-500 border-green-300 text-white shadow-[0_0_10px_rgba(34,197,94,0.7)]",
  },
];

function selectFx(fx: FxType) {
  if (props.fxType === fx && props.fxActive) {
    emit("fxActive", false);
    emit("cancelFxOut");
  } else {
    emit("fxType", fx);
    emit("fxAmount", props.fxAmount > 0 ? props.fxAmount : 0.8);
    if (!props.fxActive) {
      emit("fxActive", true);
    }
  }
}

function openFxSettings() {
  showFxSettings.value = true;
}

function openLandingSettings(fx: "echo" | "reverb" | "delay" | "roll") {
  landingSettingsFx.value = fx;
  showLandingSettings.value = true;
}

// ✅ Landing triggers — ប្រើ settings ពី landingSettings ដោយសុវត្ថិភាព
function triggerEchoLanding() {
  const duration = props.landingSettings?.echo?.duration ?? 2.5;
  emit("fxOut", "echo", duration);
}

function triggerReverbLanding() {
  const duration = props.landingSettings?.reverb?.duration ?? 3.0;
  emit("fxOut", "reverb", duration);
}

function triggerDelayLanding() {
  const duration = props.landingSettings?.delay?.duration ?? 2.5;
  emit("fxOut", "delay", duration);
}

function triggerRollLanding() {
  const duration = props.landingSettings?.roll?.duration ?? 2.0;
  const beats = props.landingSettings?.roll?.beats ?? 0.25;
  console.log(`[Roll Out] Duration: ${duration}s, Beats: ${beats}`);
  emit("fxOut", "roll", duration);
}

function handleUpdateParams(params: any) {
  emit("updateParams", params);
}

function handleUpdateLandingSettings(
  fxType: "echo" | "reverb" | "delay" | "roll",
  settings: any
) {
  emit("updateLandingSettings", fxType, settings);
}
</script>

<template>
  <div class="flex flex-col gap-1.5 flex-1 relative">

    <!-- ================= FX TYPE GRID (2 × 3) ================= -->
    <div class="grid grid-cols-3 gap-1">
      <div
        v-for="fx in FX_LIST"
        :key="fx.id"
        class="relative group"
      >
        <button
          @click="selectFx(fx.id)"
          class="w-full py-2 rounded-md border text-[8.5px] font-black uppercase transition-all cursor-pointer flex items-center justify-center pr-3"
          :class="fxType === fx.id && fxActive
            ? fx.activeClass
            : 'bg-zinc-900/60 border-zinc-800 text-zinc-500 hover:text-zinc-200 hover:border-zinc-600'"
        >
          {{ fx.label }}
        </button>

        <!-- ✅ Settings ⚙ -->
        <button
          @click.stop="openFxSettings"
          class="absolute top-0.5 right-0.5 w-3.5 h-3.5 rounded flex items-center justify-center transition-all cursor-pointer"
          :class="fxType === fx.id && fxActive
            ? 'bg-black/40 text-white hover:bg-black/70'
            : 'bg-zinc-800/80 text-zinc-500 hover:text-zinc-200 hover:bg-zinc-700'"
          :title="`${fx.label} Settings`"
        >
          <span class="text-[8px] leading-none">⚙</span>
        </button>
      </div>
    </div>

    <!-- ================= LANDING OUT ================= -->
    <div class="flex flex-col gap-0.5">
      <div class="text-[6.5px] font-black text-amber-400 uppercase tracking-wider px-1 flex items-center gap-1">
        <span>⚡</span> LANDING OUT
      </div>

      <!-- Row 1: ECHO / REVERB -->
      <div class="grid grid-cols-2 gap-1">
        <div class="relative group">
          <button
            @click="triggerEchoLanding"
            class="w-full py-1.5 rounded-md border border-cyan-500/40 bg-cyan-950/30 text-cyan-300 text-[7.5px] font-black hover:bg-cyan-500/30 transition-all cursor-pointer active:scale-95 uppercase pr-3"
          >
            ECHO OUT
          </button>
          <button
            @click.stop="openLandingSettings('echo')"
            class="absolute top-0.5 right-0.5 w-3 h-3 rounded bg-black/40 text-cyan-300 hover:text-white hover:bg-cyan-500/40 flex items-center justify-center transition-all cursor-pointer"
            title="Echo Landing Settings"
          >
            <span class="text-[7px] leading-none">⚙</span>
          </button>
        </div>

        <div class="relative group">
          <button
            @click="triggerReverbLanding"
            class="w-full py-1.5 rounded-md border border-blue-500/40 bg-blue-950/30 text-blue-300 text-[7.5px] font-black hover:bg-blue-500/30 transition-all cursor-pointer active:scale-95 uppercase pr-3"
          >
            REVERB OUT
          </button>
          <button
            @click.stop="openLandingSettings('reverb')"
            class="absolute top-0.5 right-0.5 w-3 h-3 rounded bg-black/40 text-blue-300 hover:text-white hover:bg-blue-500/40 flex items-center justify-center transition-all cursor-pointer"
            title="Reverb Landing Settings"
          >
            <span class="text-[7px] leading-none">⚙</span>
          </button>
        </div>
      </div>

      <!-- Row 2: DELAY / ROLL -->
      <div class="grid grid-cols-2 gap-1">
        <div class="relative group">
          <button
            @click="triggerDelayLanding"
            class="w-full py-1.5 rounded-md border border-amber-500/40 bg-amber-950/30 text-amber-300 text-[7.5px] font-black hover:bg-amber-500/30 transition-all cursor-pointer active:scale-95 uppercase pr-3"
          >
            DELAY OUT
          </button>
          <button
            @click.stop="openLandingSettings('delay')"
            class="absolute top-0.5 right-0.5 w-3 h-3 rounded bg-black/40 text-amber-300 hover:text-white hover:bg-amber-500/40 flex items-center justify-center transition-all cursor-pointer"
            title="Delay Landing Settings"
          >
            <span class="text-[7px] leading-none">⚙</span>
          </button>
        </div>

        <div class="relative group">
          <button
            @click="triggerRollLanding"
            class="w-full py-1.5 rounded-md border border-green-500/40 bg-green-950/30 text-green-300 text-[7.5px] font-black hover:bg-green-500/30 transition-all cursor-pointer active:scale-95 uppercase pr-3"
          >
            ROLL OUT
          </button>
          <button
            @click.stop="openLandingSettings('roll')"
            class="absolute top-0.5 right-0.5 w-3 h-3 rounded bg-black/40 text-green-300 hover:text-white hover:bg-green-500/40 flex items-center justify-center transition-all cursor-pointer"
            title="Roll Landing Settings"
          >
            <span class="text-[7px] leading-none">⚙</span>
          </button>
        </div>
      </div>

    </div>

    <!-- ================= FX SETTINGS MODAL ================= -->
    <Teleport to="body">
      <div
        v-if="showFxSettings"
        class="fixed inset-0 z-[9999] flex items-center justify-center p-4"
        @click.self="showFxSettings = false"
      >
        <div class="absolute inset-0 bg-black/75 backdrop-blur-sm"></div>
        <div class="relative z-10 w-[440px] max-h-[85vh] overflow-y-auto">
          <FxSettings
            :fx-type="fxType"
            :params="params"
            :fx-amount="fxAmount"
            @update="handleUpdateParams"
            @update-amount="(v) => emit('fxAmount', v)"
            @close="showFxSettings = false"
          />
        </div>
      </div>
    </Teleport>

    <!-- ================= LANDING SETTINGS MODAL ================= -->
    <Teleport to="body">
      <div
        v-if="showLandingSettings"
        class="fixed inset-0 z-[9999] flex items-center justify-center p-4"
        @click.self="showLandingSettings = false"
      >
        <div class="absolute inset-0 bg-black/75 backdrop-blur-sm"></div>
        <div class="relative z-10 w-[400px] max-h-[85vh] overflow-y-auto">
          <LandingSettings
            v-if="landingSettings"
            :fx-type="landingSettingsFx"
            :settings="landingSettings[landingSettingsFx] || { duration: 2.5, amount: 1.0, beats: 0.25 }"
            @update="(settings) => handleUpdateLandingSettings(landingSettingsFx, settings)"
            @close="showLandingSettings = false"
          />
        </div>
      </div>
    </Teleport>

  </div>
</template>