<script setup lang="ts">
import { computed } from "vue";
import type { DeckState } from "../types/dj";

const props = defineProps<{
  deck: DeckState;
  color: "cyan" | "rose";
}>();

const glowColor = computed(() => {
  return props.color === "cyan" ? "#06b6d4" : "#f43f5e";
});
</script>

<template>
  <div 
    class="relative w-36 h-36 sm:w-38 sm:h-38 rounded-full p-1 bg-gradient-to-tr from-[#16171d] via-[#242630] to-[#121318] border border-zinc-700/80 shadow-[0_8px_24px_rgba(0,0,0,0.85)] flex items-center justify-center select-none group"
  >
    <!-- Outer Bezel Knurling / Tactile Grip Ring -->
    <div class="absolute inset-0 rounded-full border border-zinc-600/40 pointer-events-none"></div>
    <div class="absolute inset-[2px] rounded-full border border-zinc-800/80 pointer-events-none"></div>

    <!-- Outer Vinyl Platter (Spins when playing) -->
    <div 
      class="absolute inset-1.5 rounded-full bg-[#0b0c10] overflow-hidden border border-zinc-800 shadow-inner flex items-center justify-center"
      :class="{ 'spin-vinyl': deck.isPlaying }"
    >
      <!-- Vinyl Grooves -->
      <div class="absolute inset-1 rounded-full border border-zinc-800/60 pointer-events-none"></div>
      <div class="absolute inset-2.5 rounded-full border border-zinc-800/40 pointer-events-none"></div>
      <div class="absolute inset-4 rounded-full border border-zinc-800/30 pointer-events-none"></div>
      <div class="absolute inset-5.5 rounded-full border border-zinc-800/20 pointer-events-none"></div>

      <!-- Platter Radial Shimmer Reflection -->
      <div 
        class="absolute inset-0 opacity-15 pointer-events-none bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-white via-transparent to-transparent"
      ></div>

      <!-- Needle White/Neon Platter Marker at outer edge -->
      <div 
        class="absolute top-0.5 left-1/2 -translate-x-1/2 w-1.5 h-3 rounded-full shadow-[0_0_8px]"
        :class="color === 'cyan' ? 'bg-cyan-400 shadow-cyan-400' : 'bg-rose-400 shadow-rose-400'"
      ></div>

      <!-- Opposite subtle marker -->
      <div class="absolute bottom-0.5 left-1/2 -translate-x-1/2 w-1 h-1.5 rounded-full bg-zinc-600/60"></div>
    </div>

    <!-- Center On-Jog LCD Display (Stationary Center Screen) -->
    <div 
      class="relative z-10 w-24 h-24 rounded-full bg-gradient-to-b from-[#090a0d] to-[#040406] border-2 flex flex-col items-center justify-between p-1.5 font-mono shadow-[inset_0_2px_8px_rgba(0,0,0,0.95)]"
      :class="color === 'cyan' ? 'border-cyan-500/30' : 'border-rose-500/30'"
    >
      <!-- Top Info: Deck Status & Pitch % -->
      <div class="flex items-center justify-between w-full px-1 text-[7.5px] font-bold border-b border-zinc-800/80 pb-0.5">
        <span 
          class="px-1 rounded text-[7px]"
          :class="deck.isPlaying ? (color === 'cyan' ? 'bg-cyan-950/60 text-cyan-400 border border-cyan-500/40' : 'bg-rose-950/60 text-rose-400 border border-rose-500/40') : 'text-zinc-500'"
        >
          {{ deck.isPlaying ? "PLAY" : "STOP" }}
        </span>

        <span 
          class="font-black"
          :class="deck.pitch === 0 ? 'text-zinc-400' : (color === 'cyan' ? 'text-cyan-400' : 'text-rose-400')"
        >
          {{ (deck.pitch >= 0 ? "+" : "") + deck.pitch.toFixed(1) }}%
        </span>
      </div>

      <!-- Center Info: Big Crisp BPM & Key -->
      <div class="text-center my-auto flex flex-col items-center justify-center">
        <div class="flex items-baseline gap-1">
          <span class="text-sm sm:text-base font-black tracking-tight text-white leading-none">
            {{ deck.bpm.toFixed(1) }}
          </span>
          <span class="text-[7px] text-zinc-500 font-bold uppercase">BPM</span>
        </div>

        <div class="flex items-center gap-1.5 mt-0.5">
          <span class="text-[7.5px] font-bold text-amber-400 bg-amber-950/40 px-1 rounded border border-amber-500/30">
            {{ deck.key || "8A" }}
          </span>
          <span 
            v-if="deck.keyLock" 
            class="text-[6.5px] font-black text-cyan-300 bg-cyan-950/40 px-1 rounded border border-cyan-500/30"
            title="Master Tempo / Key Lock Enabled"
          >
            MT
          </span>
        </div>
      </div>

      <!-- Bottom Info: Time Elapsed/Remaining -->
      <div class="w-full text-center border-t border-zinc-800/80 pt-0.5 flex items-center justify-center gap-1">
        <span class="text-[8px] font-black text-zinc-300 tracking-wider">
          -{{ deck.currentTime }}
        </span>
      </div>

      <!-- Glowing Active Platter Rim Accent -->
      <div 
        v-if="deck.isPlaying"
        class="absolute -inset-0.5 rounded-full pointer-events-none opacity-40 blur-[1px]"
        :style="{ border: `1px solid ${glowColor}` }"
      ></div>
    </div>

  </div>
</template>