<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{
  fxType: "echo" | "reverb" | "delay" | "roll";
  settings: {
    duration: number;
    amount: number;
    beats?: number;
  };
}>();

const emit = defineEmits<{
  (e: "update", settings: any): void;
  (e: "close"): void;
}>();

const local = ref({
  duration: props.settings?.duration ?? 2.5,
  amount: props.settings?.amount ?? 1.0,
  beats: props.settings?.beats ?? 0.25,
  ...props.settings,
});

watch(() => props.settings, (newVal) => {
  if (newVal) {
    local.value = {
      duration: newVal.duration ?? 2.5,
      amount: newVal.amount ?? 1.0,
      beats: newVal.beats ?? 0.25,
      ...newVal,
    };
  }
}, { deep: true });

function update(key: string, value: any) {
  (local.value as any)[key] = value;
  emit("update", { ...local.value });
}

// ✅ Duration presets
const DURATIONS = [
  { label: "1s", value: 1.0 },
  { label: "2s", value: 2.0 },
  { label: "2.5s", value: 2.5 },
  { label: "3s", value: 3.0 },
  { label: "5s", value: 5.0 },
  { label: "8s", value: 8.0 },
];

// ✅ Roll beats
const ROLL_BEATS = [
  { label: "1/16", value: 0.0625 },
  { label: "1/8", value: 0.125 },
  { label: "1/4", value: 0.25 },
  { label: "1/2", value: 0.5 },
  { label: "1", value: 1.0 },
];

const COLORS: Record<string, { bg: string; border: string; text: string }> = {
  echo: { bg: "bg-cyan-500", border: "border-cyan-500/50", text: "text-cyan-300" },
  reverb: { bg: "bg-blue-500", border: "border-blue-500/50", text: "text-blue-300" },
  delay: { bg: "bg-amber-500", border: "border-amber-500/50", text: "text-amber-300" },
  roll: { bg: "bg-green-500", border: "border-green-500/50", text: "text-green-300" },
};

const color = COLORS[props.fxType] || { bg: "bg-purple-500", border: "border-purple-500/50", text: "text-purple-300" };
</script>

<template>
  <div
    class="bg-[#0a0c14] border-2 rounded-xl p-3 shadow-2xl"
    :class="color.border"
  >
    <!-- Header -->
    <div class="flex items-center justify-between mb-3 pb-2 border-b" :class="color.border">
      <div class="flex items-center gap-2">
        <span class="text-[11px] font-black uppercase tracking-wider" :class="color.text">
          🎯 {{ fxType.toUpperCase() }} LANDING SETTINGS
        </span>
        <span class="px-1.5 py-0.5 rounded text-[7.5px] font-black uppercase bg-emerald-500/20 text-emerald-400 border border-emerald-500/40 flex items-center gap-1">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
          AUTO-SAVED
        </span>
      </div>
      <button
        @click="$emit('close')"
        class="w-6 h-6 rounded bg-zinc-900 hover:bg-red-500/40 text-zinc-400 hover:text-white text-[12px] font-black cursor-pointer"
      >
        ✕
      </button>
    </div>

    <!-- Duration -->
    <div class="mb-3">
      <div class="flex items-center justify-between mb-1">
        <label class="text-[9px] font-black text-zinc-300 uppercase">DURATION</label>
        <span class="text-[10px] font-mono font-bold" :class="color.text">
          {{ local.duration.toFixed(1) }}s
        </span>
      </div>
      <div class="grid grid-cols-6 gap-1 mb-2">
        <button
          v-for="d in DURATIONS"
          :key="d.value"
          @click="update('duration', d.value)"
          class="py-1.5 rounded text-[8px] font-black border transition-all cursor-pointer"
          :class="local.duration === d.value
            ? `${color.bg} border-white text-black`
            : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
        >
          {{ d.label }}
        </button>
      </div>
      <input
        type="range" min="0.5" max="10" step="0.1"
        :value="local.duration"
        @input="update('duration', parseFloat(($event.target as HTMLInputElement).value))"
        class="w-full h-1.5 rounded-full appearance-none cursor-pointer"
        :style="{
          background: `linear-gradient(to right, #a855f7 ${((local.duration - 0.5) / 9.5) * 100}%, #3f3f46 ${((local.duration - 0.5) / 9.5) * 100}%)`
        }"
      />
    </div>

    <!-- Amount -->
    <div class="mb-3">
      <div class="flex items-center justify-between mb-1">
        <label class="text-[9px] font-black text-zinc-300 uppercase">MAX AMOUNT</label>
        <span class="text-[10px] font-mono font-bold" :class="color.text">
          {{ Math.round(local.amount * 100) }}%
        </span>
      </div>
      <input
        type="range" min="0" max="1" step="0.01"
        :value="local.amount"
        @input="update('amount', parseFloat(($event.target as HTMLInputElement).value))"
        class="w-full h-1.5 rounded-full appearance-none cursor-pointer"
        :style="{
          background: `linear-gradient(to right, #a855f7 ${local.amount * 100}%, #3f3f46 ${local.amount * 100}%)`
        }"
      />
    </div>

    <!-- Roll Beats (សម្រាប់ Roll តែប៉ុណ្ណោះ) -->
    <div v-if="fxType === 'roll'" class="mb-3">
      <label class="text-[9px] font-black text-zinc-300 uppercase mb-1 block">ROLL BEATS</label>
      <div class="grid grid-cols-5 gap-1">
        <button
          v-for="b in ROLL_BEATS"
          :key="b.value"
          @click="update('beats', b.value)"
          class="py-2 rounded text-[9px] font-black border transition-all cursor-pointer"
          :class="local.beats === b.value
            ? 'bg-green-500 border-green-300 text-white'
            : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-300'"
        >
          {{ b.label }}
        </button>
      </div>
    </div>

    <!-- Footer -->
    <div class="mt-3 pt-2 border-t border-zinc-800 flex justify-end">
      <button
        @click="$emit('close')"
        class="px-3 py-1.5 text-black text-[10px] font-black rounded-lg cursor-pointer"
        :class="color.bg"
      >
        CLOSE
      </button>
    </div>
  </div>
</template>