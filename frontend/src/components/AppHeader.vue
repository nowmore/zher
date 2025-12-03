<template>
  <header class="shadow-sm px-4 py-3 flex justify-between items-center z-10 shrink-0 transition-colors"
    :class="bgClass">
    <h1 class="text-lg font-bold text-gray-800 dark:text-white">
      这儿 <span class="text-sm font-normal text-gray-400 ml-2">zhe'r</span>
    </h1>
    <div class="flex items-center gap-3">
      <button @click="$emit('toggle-theme')"
        class="p-2 rounded-full hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 dark:text-gray-400 transition-colors">
        <svg v-if="isDarkMode" xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24"
          stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
        </svg>
        <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24"
          stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
        </svg>
      </button>
      <button @click="$emit('show-users')"
        class="md:hidden px-3 py-1.5 bg-gray-100 dark:bg-gray-700 rounded-full text-sm text-blue-600 dark:text-blue-400 font-medium active:bg-gray-200 dark:active:bg-gray-600 transition-colors flex items-center gap-1">
        <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
        在线: {{ onlineCount }}
      </button>
    </div>
  </header>
</template>

<script setup>
defineProps({
  isDarkMode: Boolean,
  onlineCount: Number,
  bgClass: String
});

defineEmits(['toggle-theme', 'show-users']);
</script>
