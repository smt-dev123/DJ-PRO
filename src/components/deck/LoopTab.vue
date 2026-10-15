<script setup lang="ts">
import { ref } from "vue";

const emit = defineEmits<{
  (e: "toggleLoop", beats: string): void;
}>();

const activeLoop = ref<string | null>(null);

const LOOPS = ["1/2", "1", "2", "4"];

function toggleLoop(beats: string) {
  if (activeLoop.value === beats) {
    activeLoop.value = null;
  } else {
    activeLoop.value = beats;
  }
  emit("toggleLoop", beats);
}

// ✅ Expose សម្រាប់ parent បើចង់ reset
function reset() {
  activeLoop.value = null;
}

defineExpose({ reset });
</script>

<template>
  <div class="grid grid-cols-2 gap-1.5 flex-1">
    <button
      v-for="l in LOOPS"
      :key="'loop-' + l"
      @click="toggleLoop(l)"
      class="rounded-lg border text-[9px] font-black active:scale-95 transition-all cursor-pointer flex items-center justify-center shadow-xs"
      :class="activeLoop === l
        ? 'bg-amber-500 border-amber-300 text-black shadow-[0_0_8px_#f59e0b]'
        : 'bg-amber-950/20 border-amber-500/30 text-amber-300 hover:bg-amber-500/20'"
    >
      {{ l }} BEAT
    </button>
  </div>
</template>