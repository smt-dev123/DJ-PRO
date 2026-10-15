<script setup lang="ts">
import { ref, computed } from "vue";
import FxSettings from "./FxSettings.vue";

type FxType = "echo" | "reverb" | "delay" | "flanger" | "filter" | "roll";

const props = defineProps<{
  fxType: string;
  fxAmount: number;
  fxActive: boolean;
  params: any;  // ✅ FX parameters
}>();

const emit = defineEmits<{
  (e: "fxType", fxType: FxType): void;
  (e: "fxAmount", amount: number): void;
  (e: "fxActive", active: boolean): void;
  (e: "fxOut", fxType: "echo" | "reverb", duration: number): void;
  (e: "cancelFxOut"): void;
  (e: "updateParams", params: any): void;  // ✅ ថ្មី
}>();

const showSettings = ref(false);

const FX_ROW1: FxType[] = ["echo", "reverb", "delay"];
const FX_ROW2: FxType[] = ["flanger", "filter", "roll"];

function selectFx(fx: FxType) {
  emit("fxType", fx);
  if (!props.fxActive) emit("fxActive", true);
}

function setFxAmount(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value);
  emit("fxAmount", val);
}

function triggerEchoOut() {
  emit("fxOut", "echo", 2.5);
}

function triggerReverbOut() {
  emit("fxOut", "reverb", 3.0);
}

function killFx() {
  if (props.fxActive) {
    emit("fxActive", false);
    emit("fxAmount", 0);
    emit("cancelFxOut");
  }
}

function handleUpdateParams(params: any) {
  emit("updateParams", params);
}

const amountPercent = computed(() => Math.round(props.fxAmount * 100));
</script>

<template>
  <div class="flex flex-col gap-1 flex-1 relative">
    <!-- FX Type Row 1 -->
    <div class="grid grid-cols-3 gap-1">
      <button
        v-for="fx in FX_ROW1"
        :key="'fx1-' + fx"
        @click="selectFx(fx)"
        class="py-1 rounded border text-[7px] font-black uppercase transition-all cursor-pointer"
        :class="fxType === fx
          ? 'bg-purple-500 border-purple-300 text-white shadow-[0_0_8px_#a855f7]'
          : 'bg-purple-950/20 border-purple-500/30 text-purple-300 hover:bg-purple-500/20'"
      >
        {{ fx }}
      </button>
    </div>

    <!-- FX Type Row 2 -->
    <div class="grid grid-cols-3 gap-1">
      <button
        v-for="fx in FX_ROW2"
        :key="'fx2-' + fx"
        @click="selectFx(fx)"
        class="py-1 rounded border text-[7px] font-black uppercase transition-all cursor-pointer"
        :class="fxType === fx
          ? 'bg-purple-500 border-purple-300 text-white shadow-[0_0_8px_#a855f7]'
          : 'bg-purple-950/20 border-purple-500/30 text-purple-300 hover:bg-purple-500/20'"
      >
        {{ fx }}
      </button>
    </div>

    <!-- FX Amount + Settings Button -->
    <div class="flex items-center gap-1.5">
      <span class="text-[7px] font-black text-purple-400 w-8">AMT</span>
      <input
        type="range" min="0" max="1" step="0.01"
        :value="fxAmount"
        @input="setFxAmount"
        class="flex-1 h-1 rounded-full appearance-none cursor-pointer"
        :style="{
          background: `linear-gradient(to right, #a855f7 ${fxAmount * 100}%, #3f3f46 ${fxAmount * 100}%)`
        }"
      />
      <span class="text-[7px] font-mono text-purple-400 w-6 text-right">
        {{ amountPercent }}
      </span>
      <!-- ✅ Settings Button -->
      <button
        @click="showSettings = !showSettings"
        class="px-1 py-0.5 rounded border border-purple-500/40 bg-purple-950/30 text-purple-300 text-[8px] font-black hover:bg-purple-500/30 transition-all cursor-pointer active:scale-90"
        title="FX Settings"
      >
        ⚙
      </button>
    </div>

    <!-- FX Out Buttons -->
    <div class="grid grid-cols-2 gap-1">
      <button
        @click="triggerEchoOut"
        class="py-1.5 rounded border border-cyan-500/40 bg-cyan-950/30 text-cyan-300 text-[8px] font-black hover:bg-cyan-500/30 transition-all cursor-pointer active:scale-95"
      >
        🔊 ECHO OUT
      </button>
      <button
        @click="triggerReverbOut"
        class="py-1.5 rounded border border-blue-500/40 bg-blue-950/30 text-blue-300 text-[8px] font-black hover:bg-blue-500/30 transition-all cursor-pointer active:scale-95"
      >
        🌊 REVERB OUT
      </button>
    </div>

    <!-- Kill FX -->
    <button
      @click="killFx"
      class="w-full py-1 rounded border text-[8px] font-black transition-all cursor-pointer active:scale-95"
      :class="fxActive
        ? 'bg-red-500 border-red-300 text-white shadow-[0_0_8px_#ef4444] animate-pulse'
        : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
    >
      {{ fxActive ? '⏹ KILL FX' : '○ FX OFF' }}
    </button>

    <!-- ✅ Settings Popup (overlay) -->
    <div
      v-if="showSettings"
      class="absolute top-full left-0 right-0 mt-1 z-50"
    >
      <FxSettings
        :fx-type="fxType"
        :params="params"
        @update="handleUpdateParams"
        @close="showSettings = false"
      />
    </div>
  </div>
</template>