<template>
  <div v-if="show" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 backdrop-blur-sm">
    <div
      class="bg-white dark:bg-gray-800 rounded-2xl p-6 w-64 shadow-xl flex flex-col items-center gap-4 animate-fade-in">
      <div class="text-center w-full">
        <h3 class="font-bold text-gray-800 dark:text-white mb-1">正在打包...</h3>
        <p class="text-xs text-gray-500 dark:text-gray-400 mb-3 truncate max-w-[200px] mx-auto">
          {{ zipName }}
        </p>
        <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-4 overflow-hidden relative">
          <div class="bg-blue-500 h-4 rounded-full transition-all duration-200 ease-out relative overflow-hidden"
            :style="{ width: progress + '%' }">
            <div
              class="absolute inset-0 bg-white/30 w-full h-full animate-[shimmer_1.5s_infinite] -skew-x-12 origin-left">
            </div>
          </div>
        </div>
        <div class="flex justify-between items-center mt-2 px-1">
          <p class="text-xs text-gray-400 dark:text-gray-500 font-mono truncate max-w-[140px]">
            {{ currentFile }}
          </p>
          <p class="text-sm text-gray-500 dark:text-gray-400 font-mono">{{ progress.toFixed(0) }}%</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  show: Boolean,
  progress: Number,
  zipName: String,
  currentFile: String
});
</script>
