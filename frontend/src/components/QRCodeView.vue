<template>
  <div class="flex flex-col h-full">
    <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center gap-3">
      <button @click="$emit('back')" class="p-2 rounded-full hover:bg-gray-100 dark:hover:bg-gray-700 transition">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-gray-600 dark:text-gray-300" fill="none"
          viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
      <h2 class="text-lg font-bold text-gray-800 dark:text-white">QR Code</h2>
    </div>

    <div class="flex-1 flex flex-col items-center justify-center p-6 gap-4">
      <img v-if="qrCodeUrl" :src="qrCodeUrl" class="w-48 h-48 rounded-lg shadow-lg bg-white p-2" alt="Server QR Code" />
      <div class="text-center">
        <p class="text-sm text-gray-600 dark:text-gray-300 font-mono">{{ displayUrl }}</p>
        <p class="text-xs text-gray-400 mt-2">Scan to connect</p>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  qrCodeUrl: String,
  displayUrl: String
});

defineEmits(['back']);
</script>
