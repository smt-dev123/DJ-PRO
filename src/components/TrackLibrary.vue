<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { LibraryTrack } from "../types/dj";

const emit = defineEmits<{
  (e: "loadToDeck", deckId: "A" | "B", track: LibraryTrack): void;
}>();

interface DriveNode {
  name: string;
  path: string;
  kind: string;
}

interface VirtualFolder {
  id: string;
  name: string;
  icon?: string;
  tracks: LibraryTrack[];
}

// System Drives & Directories
const systemDrives = ref<DriveNode[]>([]);
const currentPath = ref("");
const currentFolderTitle = ref("🔥 Peak Time Festival");

// Default Curated Virtual Folders
const virtualFolders = ref<VirtualFolder[]>([
  {
    id: "f1",
    name: "🔥 Peak Time Festival",
    icon: "🔥",
    tracks: [
      { id: "1", title: "Midnight City (Club Rework)", artist: "M83", bpm: 126.0, key: "8A", duration: "04:12", path: "C:/Music/Midnight_City.mp3" },
      { id: "2", title: "Titanium (Festival VIP)", artist: "David Guetta ft. Sia", bpm: 128.0, key: "4B", duration: "05:04", path: "C:/Music/Titanium_VIP.mp3" },
      { id: "3", title: "Levels (Extended Original)", artist: "Avicii", bpm: 126.0, key: "4A", duration: "05:38", path: "C:/Music/Levels.mp3" },
      { id: "4", title: "One More Time (Zedd Remix)", artist: "Daft Punk", bpm: 128.0, key: "7B", duration: "04:45", path: "C:/Music/One_More_Time.mp3" },
      { id: "5", title: "Animals (Oliver Heldens Edit)", artist: "Martin Garrix", bpm: 126.0, key: "1A", duration: "04:30", path: "C:/Music/Animals.mp3" },
    ],
  },
  {
    id: "f2",
    name: "⚡ Cyber Bass & House",
    icon: "⚡",
    tracks: [
      { id: "6", title: "Satisfaction (Hardwell Extended)", artist: "Benny Benassi", bpm: 130.0, key: "2A", duration: "04:10", path: "C:/Music/Satisfaction.mp3" },
      { id: "7", title: "Do It To It (Sub Focus Flip)", artist: "Acraze", bpm: 125.0, key: "11A", duration: "03:55", path: "C:/Music/Do_It_To_It.mp3" },
      { id: "8", title: "Ray Of Solar (Mau P Remix)", artist: "Swedish House Mafia", bpm: 127.0, key: "6A", duration: "06:12", path: "C:/Music/Ray_Of_Solar.mp3" },
    ],
  },
  {
    id: "f3",
    name: "🌴 Deep Melodic Sunset",
    icon: "🌴",
    tracks: [
      { id: "9", title: "Innerbloom (Lane 8 Remix)", artist: "RÜFÜS DU SOL", bpm: 122.0, key: "9A", duration: "07:15", path: "C:/Music/Innerbloom.mp3" },
      { id: "10", title: "Opus (Four Tet Extended)", artist: "Eric Prydz", bpm: 126.0, key: "5B", duration: "09:58", path: "C:/Music/Opus.mp3" },
    ],
  },
]);

const activeVirtualFolderId = ref<string | null>("f1");
const loadedTracks = ref<LibraryTrack[]>([]);
const searchQuery = ref("");
const selectedFilter = ref<"ALL" | "120-126" | "126-130">("ALL");
const newFolderName = ref("");
const showCreateFolderModal = ref(false);

// Filtered & Searched Tracks
const filteredTracks = computed(() => {
  let list = activeVirtualFolderId.value 
    ? (virtualFolders.value.find(f => f.id === activeVirtualFolderId.value)?.tracks || [])
    : loadedTracks.value;

  if (selectedFilter.value === "120-126") {
    list = list.filter(t => t.bpm >= 120 && t.bpm <= 126);
  } else if (selectedFilter.value === "126-130") {
    list = list.filter(t => t.bpm > 126 && t.bpm <= 132);
  }

  if (!searchQuery.value.trim()) return list;
  const q = searchQuery.value.toLowerCase();
  return list.filter(
    t => t.title.toLowerCase().includes(q) || 
         t.artist.toLowerCase().includes(q) ||
         t.key.toLowerCase().includes(q)
  );
});

// Load Drives on mount
onMounted(async () => {
  try {
    const drives = await invoke<DriveNode[]>("get_system_drives");
    systemDrives.value = drives;
  } catch (err) {
    console.error("Error fetching system drives:", err);
  }
});

async function handleSelectDrive(drive: DriveNode) {
  activeVirtualFolderId.value = null;
  currentPath.value = drive.path;
  currentFolderTitle.value = drive.name;

  try {
    const scanned = await invoke<LibraryTrack[]>("scan_folder_tracks", { folderPath: drive.path });
    loadedTracks.value = scanned;
  } catch (err) {
    console.error("Failed to scan directory:", err);
    loadedTracks.value = [];
  }
}

function handleSelectVirtualFolder(f: VirtualFolder) {
  activeVirtualFolderId.value = f.id;
  currentPath.value = "";
  currentFolderTitle.value = f.name;
}

function handleCreateFolder() {
  if (!newFolderName.value.trim()) return;
  const newF: VirtualFolder = {
    id: "vf_" + Date.now(),
    name: "📁 " + newFolderName.value.trim(),
    icon: "📁",
    tracks: [],
  };
  virtualFolders.value.push(newF);
  activeVirtualFolderId.value = newF.id;
  currentFolderTitle.value = newF.name;
  newFolderName.value = "";
  showCreateFolderModal.value = false;
}

// ================= DRAG & DROP SUPPORT =================
const isLibraryDragOver = ref(false);

function handleTrackDragStart(e: DragEvent, track: LibraryTrack) {
  if (e.dataTransfer) {
    e.dataTransfer.setData("application/json", JSON.stringify(track));
    e.dataTransfer.setData("text/plain", track.path);
    e.dataTransfer.effectAllowed = "copyMove";
  }
}

function handleLibraryDragOver(e: DragEvent) {
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = "copy";
  }
  isLibraryDragOver.value = true;
}

function handleLibraryDragLeave(e: DragEvent) {
  const currentTarget = e.currentTarget as HTMLElement;
  const relatedTarget = e.relatedTarget as HTMLElement;
  if (!currentTarget || !relatedTarget || !currentTarget.contains(relatedTarget)) {
    isLibraryDragOver.value = false;
  }
}

function handleLibraryDrop(e: DragEvent) {
  isLibraryDragOver.value = false;
  if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
    addAudioFiles(e.dataTransfer.files);
  }
}

function addAudioFiles(files: FileList | File[] | string[]) {
  const audioExts = ["mp3", "wav", "flac", "aac", "m4a", "ogg"];
  const newTracks: LibraryTrack[] = [];

  for (let i = 0; i < files.length; i++) {
    const item = files[i];
    let path = "";
    let name = "";

    if (typeof item === "string") {
      path = item;
      name = item.split(/[\\/]/).pop() || "Audio Track";
    } else {
      path = (item as any).path || item.name;
      name = item.name;
    }

    const ext = name.split(".").pop()?.toLowerCase() || "";
    if (typeof item !== "string" && !audioExts.includes(ext)) {
      continue;
    }

    const title = name.replace(/\.[^/.]+$/, "");
    newTracks.push({
      id: "trk_" + Date.now() + "_" + i,
      title,
      artist: "Imported Track",
      bpm: 128.0,
      key: "8A",
      duration: "03:45",
      path,
    });
  }

  if (newTracks.length > 0) {
    if (activeVirtualFolderId.value) {
      const folder = virtualFolders.value.find(f => f.id === activeVirtualFolderId.value);
      if (folder) {
        folder.tracks.unshift(...newTracks);
      }
    } else {
      loadedTracks.value.unshift(...newTracks);
    }
  }
}

defineExpose({
  addAudioFiles,
});
</script>

<template>
  <div class="w-full h-full bg-[#0a0b0f] border border-zinc-800/90 rounded-xl flex overflow-hidden shadow-2xl font-mono select-none">
    
    <!-- ================= LEFT: FOLDER & CRATE EXPLORER ================= -->
    <aside class="w-60 bg-[#0c0d12] border-r border-zinc-800/80 flex flex-col justify-between shrink-0">
      
      <!-- Explorer Title & New Crate Action -->
      <div class="p-2.5 bg-[#101117] border-b border-zinc-800 flex items-center justify-between">
        <div class="flex items-center gap-1.5">
          <span class="text-xs">📁</span>
          <span class="text-[10px] font-black text-zinc-300 tracking-wider uppercase">BROWSER</span>
        </div>
        <button 
          @click="showCreateFolderModal = true"
          class="px-2 py-0.5 bg-zinc-800 hover:bg-cyan-500 hover:text-black border border-zinc-700 rounded text-[8.5px] font-black text-zinc-200 transition-all active:scale-95 cursor-pointer flex items-center gap-1"
          title="Create Virtual Playlist Folder"
        >
          <span>➕</span> CRATE
        </button>
      </div>

      <!-- Navigation Tree Items -->
      <div class="flex-1 overflow-y-auto p-2 space-y-3">
        
        <!-- SECTION 1: VIRTUAL PLAYLISTS / CRATES -->
        <div>
          <span class="text-[7.5px] font-black text-cyan-400/90 tracking-widest uppercase px-2 block mb-1">
            CRATES & PLAYLISTS
          </span>
          <div class="space-y-0.5">
            <div 
              v-for="folder in virtualFolders" :key="folder.id"
              @click="handleSelectVirtualFolder(folder)"
              class="px-2.5 py-1.5 rounded-lg text-xs font-semibold flex items-center justify-between cursor-pointer transition-all"
              :class="activeVirtualFolderId === folder.id 
                ? 'bg-cyan-950/40 text-cyan-300 border-l-2 border-cyan-400 shadow-sm font-bold' 
                : 'text-zinc-400 hover:bg-zinc-900/60 hover:text-zinc-200'"
            >
              <span class="truncate">{{ folder.name }}</span>
              <span class="text-[8.5px] text-zinc-500 font-bold px-1.5 py-0.2 rounded-full bg-black/50 border border-zinc-800/80">
                {{ folder.tracks.length }}
              </span>
            </div>
          </div>
        </div>

        <!-- SECTION 2: SYSTEM DRIVES & VOLUMES -->
        <div>
          <span class="text-[7.5px] font-black text-amber-400/90 tracking-widest uppercase px-2 block mb-1">
            LOCAL STORAGE & DRIVES
          </span>
          <div class="space-y-0.5">
            <div 
              v-for="drive in systemDrives" :key="drive.path"
              @click="handleSelectDrive(drive)"
              class="px-2.5 py-1.5 rounded-lg text-xs font-semibold flex items-center justify-between cursor-pointer transition-all"
              :class="currentPath === drive.path && !activeVirtualFolderId 
                ? 'bg-amber-950/40 text-amber-300 border-l-2 border-amber-400 shadow-sm font-bold' 
                : 'text-zinc-400 hover:bg-zinc-900/60 hover:text-zinc-200'"
            >
              <div class="flex items-center gap-1.5 truncate">
                <span class="text-[11px]">{{ drive.kind === 'drive' ? '💽' : '📁' }}</span>
                <span class="truncate">{{ drive.name }}</span>
              </div>
              <span class="text-[7.5px] text-zinc-500 uppercase font-mono">{{ drive.kind }}</span>
            </div>
          </div>
        </div>

      </div>

      <!-- Footer Info -->
      <div class="p-2 bg-[#08090d] border-t border-zinc-800/80 text-[7.5px] text-zinc-500 truncate flex items-center justify-between">
        <span class="truncate">PATH: <span class="text-zinc-300">{{ currentPath || 'VIRTUAL STORAGE' }}</span></span>
        <span class="text-emerald-400 font-bold shrink-0">READY</span>
      </div>
    </aside>

    <!-- ================= RIGHT: MUSIC LIBRARY TABLE ================= -->
    <main 
      class="relative flex-1 flex flex-col min-w-0 bg-[#090a0f] overflow-hidden"
      @dragover="handleLibraryDragOver"
      @dragleave="handleLibraryDragLeave"
      @drop="handleLibraryDrop"
    >
      <!-- Drag & Drop Visual Dropzone Overlay -->
      <div 
        v-if="isLibraryDragOver"
        class="absolute inset-0 z-50 rounded-xl bg-cyan-950/85 backdrop-blur-xs flex flex-col items-center justify-center p-6 border-2 border-dashed border-cyan-400 text-cyan-200 pointer-events-none transition-all animate-pulse"
      >
        <div class="w-14 h-14 rounded-full bg-cyan-500/20 text-cyan-300 border border-cyan-400 flex items-center justify-center text-2xl mb-3 shadow-lg">
          📥
        </div>
        <span class="text-sm font-black tracking-widest uppercase">
          DROP AUDIO FILES TO ADD TO PLAYLIST
        </span>
        <span class="text-xs text-zinc-400 mt-1 font-sans">
          Add directly into "{{ currentFolderTitle }}" (.mp3, .wav, .flac, .aac, .m4a, .ogg)
        </span>
      </div>
      
      <!-- Top Search & Filter Bar -->
      <div class="p-2 bg-[#0e0f15] border-b border-zinc-800/80 flex items-center justify-between gap-3">
        <!-- Title & Count -->
        <div class="flex items-center gap-2 min-w-0">
          <span class="text-cyan-400 font-black text-xs uppercase truncate">{{ currentFolderTitle }}</span>
          <span class="text-[9px] text-zinc-400 font-bold bg-zinc-900 px-2 py-0.5 rounded-full border border-zinc-800">
            {{ filteredTracks.length }} Tracks
          </span>
        </div>

        <!-- Filter Chips & Search Input -->
        <div class="flex items-center gap-2 shrink-0">
          <!-- BPM Filter Chips -->
          <div class="flex items-center bg-black/60 p-0.5 rounded-lg border border-zinc-800 text-[8px] font-black">
            <button 
              @click="selectedFilter = 'ALL'"
              class="px-2 py-1 rounded transition-colors cursor-pointer"
              :class="selectedFilter === 'ALL' ? 'bg-zinc-700 text-white' : 'text-zinc-400 hover:text-white'"
            >
              ALL
            </button>
            <button 
              @click="selectedFilter = '120-126'"
              class="px-2 py-1 rounded transition-colors cursor-pointer"
              :class="selectedFilter === '120-126' ? 'bg-zinc-700 text-white' : 'text-zinc-400 hover:text-white'"
            >
              120-126 BPM
            </button>
            <button 
              @click="selectedFilter = '126-130'"
              class="px-2 py-1 rounded transition-colors cursor-pointer"
              :class="selectedFilter === '126-130' ? 'bg-zinc-700 text-white' : 'text-zinc-400 hover:text-white'"
            >
              126-130 BPM
            </button>
          </div>

          <!-- Search Box -->
          <div class="relative flex items-center w-56 sm:w-64">
            <span class="absolute left-2.5 text-[10px] text-zinc-500 pointer-events-none">🔍</span>
            <input 
              v-model="searchQuery"
              type="text" 
              placeholder="Search title, artist, key..." 
              class="w-full bg-black/70 border border-zinc-800 rounded-lg pl-7 pr-7 py-1 text-xs text-zinc-200 placeholder:text-zinc-600 focus:outline-none focus:border-cyan-400 font-sans"
            />
            <button 
              v-if="searchQuery" 
              @click="searchQuery = ''" 
              class="absolute right-2 text-zinc-500 hover:text-zinc-300 text-xs cursor-pointer"
            >
              ✕
            </button>
          </div>
        </div>
      </div>

      <!-- Track Data Table -->
      <div class="flex-1 overflow-y-auto">
        <table class="w-full text-left border-collapse text-xs">
          <thead class="bg-[#0b0c11] text-[8px] text-zinc-400 uppercase tracking-wider sticky top-0 border-b border-zinc-800 z-10 font-bold">
            <tr>
              <th class="py-2 px-3 w-10 text-center">#</th>
              <th class="py-2 px-3">TITLE</th>
              <th class="py-2 px-3">ARTIST</th>
              <th class="py-2 px-3 w-18 text-center">BPM</th>
              <th class="py-2 px-3 w-14 text-center">KEY</th>
              <th class="py-2 px-3 w-16 text-center">TIME</th>
              <th class="py-2 px-3 w-28 text-center hidden md:table-cell">SPECTRUM</th>
              <th class="py-2 px-3 w-28 text-center">LOAD TO</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-zinc-900">
            <tr 
              v-for="(track, index) in filteredTracks" :key="track.id"
              draggable="true"
              @dragstart="handleTrackDragStart($event, track)"
              @dblclick="$emit('loadToDeck', 'A', track)"
              class="hover:bg-zinc-900/60 transition-colors group cursor-grab active:cursor-grabbing select-none"
              title="Drag and drop onto DECK A or DECK B, or double click to load"
            >
              <!-- Index Number / Drag Grip Icon on Hover -->
              <td class="py-2 px-3 text-center text-zinc-500 group-hover:text-cyan-400 font-bold w-10">
                <span class="hidden group-hover:inline-block cursor-grab text-[11px] leading-none">⠿</span>
                <span class="group-hover:hidden">{{ index + 1 }}</span>
              </td>

              <!-- Title -->
              <td class="py-2 px-3 font-bold text-zinc-100 truncate max-w-[220px]">
                <div class="flex items-center gap-2">
                  <span class="text-zinc-600 group-hover:text-cyan-400 transition-colors text-[10px]">🎵</span>
                  <span class="truncate">{{ track.title }}</span>
                </div>
              </td>

              <!-- Artist -->
              <td class="py-2 px-3 text-zinc-400 truncate max-w-[160px] font-sans">
                {{ track.artist }}
              </td>

              <!-- BPM -->
              <td class="py-2 px-3 text-center font-bold text-cyan-400 font-mono">
                {{ track.bpm.toFixed(1) }}
              </td>

              <!-- Key -->
              <td class="py-2 px-3 text-center">
                <span class="px-1.5 py-0.5 rounded text-[8px] font-bold text-amber-400 bg-amber-950/40 border border-amber-500/30">
                  {{ track.key }}
                </span>
              </td>

              <!-- Time -->
              <td class="py-2 px-3 text-center text-zinc-400 font-mono text-[10px]">
                {{ track.duration }}
              </td>

              <!-- Mini Spectrum Preview -->
              <td class="py-2 px-3 text-center hidden md:table-cell">
                <div class="h-3 w-20 mx-auto flex items-center justify-between gap-[1px] opacity-70 group-hover:opacity-100 transition-opacity">
                  <div 
                    v-for="s in 14" :key="'s-' + s"
                    class="w-[2px] rounded-full"
                    :class="s % 3 === 0 ? 'bg-cyan-400' : (s % 2 === 0 ? 'bg-indigo-400' : 'bg-rose-400')"
                    :style="{ height: `${((s * 17) % 70) + 30}%` }"
                  ></div>
                </div>
              </td>

              <!-- Load Action Buttons -->
              <td class="py-2 px-3 text-center">
                <div class="flex items-center justify-center gap-1.5">
                  <button 
                    @click.stop="$emit('loadToDeck', 'A', track)"
                    class="px-2.5 py-1 rounded bg-cyan-950/40 hover:bg-cyan-500 hover:text-black border border-cyan-500/40 text-cyan-300 font-black text-[8.5px] transition-all active:scale-90 cursor-pointer shadow-xs"
                    title="Load track to Deck A"
                  >
                    LOAD A
                  </button>
                  <button 
                    @click.stop="$emit('loadToDeck', 'B', track)"
                    class="px-2.5 py-1 rounded bg-rose-950/40 hover:bg-rose-500 hover:text-white border border-rose-500/40 text-rose-300 font-black text-[8.5px] transition-all active:scale-90 cursor-pointer shadow-xs"
                    title="Load track to Deck B"
                  >
                    LOAD B
                  </button>
                </div>
              </td>
            </tr>

            <!-- Empty State -->
            <tr v-if="filteredTracks.length === 0">
              <td colspan="8" class="py-14 text-center text-zinc-500 text-xs">
                <div class="flex flex-col items-center gap-2">
                  <span class="text-2xl">🔍</span>
                  <span>No audio tracks found matching criteria.</span>
                  <span class="text-[10px] text-zinc-600">Try searching another title or select a different folder.</span>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

    </main>

    <!-- Modal Popup for New Folder -->
    <div 
      v-if="showCreateFolderModal" 
      class="fixed inset-0 bg-black/75 backdrop-blur-xs flex items-center justify-center z-50 p-4"
    >
      <div class="bg-[#12141c] border border-zinc-700 rounded-xl p-4 w-84 shadow-2xl">
        <h3 class="text-xs font-black text-zinc-100 uppercase mb-3 flex items-center gap-2">
          <span>📁</span> New Playlist Crate
        </h3>
        <input 
          v-model="newFolderName"
          @keyup.enter="handleCreateFolder"
          placeholder="Crate name (e.g. Club Hits 2026)" 
          class="w-full bg-black/60 border border-zinc-700 rounded-lg px-3 py-1.5 text-xs text-zinc-100 placeholder:text-zinc-600 focus:outline-none focus:border-cyan-400 mb-4 font-sans"
          autofocus
        />
        <div class="flex justify-end gap-2">
          <button 
            @click="showCreateFolderModal = false"
            class="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-xs font-bold rounded-lg text-zinc-400 cursor-pointer transition-colors"
          >
            Cancel
          </button>
          <button 
            @click="handleCreateFolder"
            class="px-3 py-1.5 bg-cyan-500 hover:bg-cyan-400 text-black text-xs font-bold rounded-lg cursor-pointer transition-colors"
          >
            Create Crate
          </button>
        </div>
      </div>
    </div>

  </div>
</template>