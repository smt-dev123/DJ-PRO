<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const driveInfo = ref<{
  path: string;
  hasMetadata: boolean;
  playlistsCount: number;
  tracksCount: number;
} | null>(null);

const cacheStats = ref<{ entries: number; hits: number }>({ entries: 0, hits: 0 });

async function refresh() {
  try {
    const stats = await invoke<[number, number]>("cache_stats");
    cacheStats.value = { entries: stats[0], hits: stats[1] };
  } catch {}

  try {
    const drives = await invoke<any>("get_system_drives");
    // Scan សម្រាប់ metadata
    const paths = drives.map((d: any) => d.path);
    const found = await invoke<any[]>("scan_for_metadata_folders", { searchPaths: paths });
    if (found.length > 0) {
      driveInfo.value = {
        path: found[0].drive_root,
        hasMetadata: found[0].has_metadata,
        playlistsCount: found[0].playlists_count,
        tracksCount: found[0].tracks_count,
      };
    }
  } catch {}
}

async function clearCache() {
  await invoke("cache_clear");
  await refresh();
}

onMounted(refresh);
</script>

<template>
  <div class="bg-[#0a0c14] border border-zinc-800 rounded-xl p-3 space-y-3">
    <h3 class="text-xs font-black text-zinc-200 uppercase">⚙ Storage Settings</h3>

    <!-- Drive Info -->
    <div v-if="driveInfo" class="bg-[#050609] border border-zinc-800 rounded-lg p-2">
      <div class="text-[9px] font-black text-cyan-400 uppercase mb-1">Active Drive</div>
      <div class="text-[10px] font-mono text-zinc-300 truncate">{{ driveInfo.path }}</div>
      <div class="flex items-center gap-3 mt-2 text-[9px]">
        <span class="text-zinc-500">Playlists: <span class="text-cyan-400 font-bold">{{ driveInfo.playlistsCount }}</span></span>
        <span class="text-zinc-500">Tracks: <span class="text-cyan-400 font-bold">{{ driveInfo.tracksCount }}</span></span>
      </div>
    </div>

    <!-- Cache Stats -->
    <div class="bg-[#050609] border border-zinc-800 rounded-lg p-2">
      <div class="text-[9px] font-black text-purple-400 uppercase mb-1">RAM Cache</div>
      <div class="flex items-center gap-3 text-[9px]">
        <span class="text-zinc-500">Entries: <span class="text-purple-400 font-bold">{{ cacheStats.entries }}</span></span>
        <span class="text-zinc-500">Hits: <span class="text-purple-400 font-bold">{{ cacheStats.hits }}</span></span>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex gap-2">
      <button
        @click="refresh"
        class="flex-1 py-1.5 bg-zinc-900 hover:bg-zinc-800 border border-zinc-700 rounded text-[9px] font-black text-zinc-300 transition cursor-pointer"
      >
        🔄 REFRESH
      </button>
      <button
        @click="clearCache"
        class="flex-1 py-1.5 bg-red-950/40 hover:bg-red-900/40 border border-red-500/40 rounded text-[9px] font-black text-red-300 transition cursor-pointer"
      >
        🗑 CLEAR CACHE
      </button>
    </div>
  </div>
</template>