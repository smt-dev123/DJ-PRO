<script setup lang="ts">
const props = defineProps<{
  color: "cyan" | "rose";
}>();

const emit = defineEmits<{
  (e: "trigger", pad: number): void;
}>();

const PADS = [1, 2, 3, 4];

function triggerPad(pad: number) {
  emit("trigger", pad);
}
</script>

<template>
  <div class="grid grid-cols-2 gap-1.5 flex-1">
    <button
      v-for="pad in PADS"
      :key="'cue-' + pad"
      @click="triggerPad(pad)"
      class="rounded-lg border text-[9px] font-black active:scale-95 transition-all cursor-pointer flex flex-col items-center justify-center p-1 shadow-xs group"
      :class="color === 'cyan'
        ? 'bg-cyan-950/20 border-cyan-500/30 text-cyan-300 hover:bg-cyan-500/25 hover:border-cyan-400'
        : 'bg-rose-950/20 border-rose-500/30 text-rose-300 hover:bg-rose-500/25 hover:border-rose-400'"
    >
      <span>CUE {{ pad }}</span>
      <span class="text-[6.5px] text-zinc-500 group-hover:text-zinc-300 font-mono">
        0{{ pad }}:0{{ pad * 8 }}
      </span>
    </button>
  </div>
</template>