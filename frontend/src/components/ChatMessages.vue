<template>
  <div v-for="msg in messages" :key="msg.id" class="flex items-start gap-2 max-w-full"
    :class="{ 'flex-row-reverse': msg.senderId === currentUser.id }">

    <div class="relative shrink-0">
      <div class="w-8 h-8 rounded-full flex items-center justify-center shadow-sm"
        :style="{ backgroundColor: msg.senderColor }">
        <span class="text-white text-xs font-bold -translate-y-0.5">{{ msg.senderName?.substring(0, 1) || '?' }}</span>
      </div>
      <div class="absolute -bottom-1 -right-1 z-10">
        <svg v-if="msg.senderDevice === 'mobile'" xmlns="http://www.w3.org/2000/svg"
          class="h-3.5 w-3.5 text-white device-icon" viewBox="0 0 20 20" fill="currentColor"
          :style="{ stroke: msg.senderColor }">
          <path fill-rule="evenodd"
            d="M7 2a2 2 0 00-2 2v12a2 2 0 002 2h6a2 2 0 002-2V4a2 2 0 00-2-2H7zm3 14a1 1 0 100-2 1 1 0 000 2z"
            clip-rule="evenodd" />
        </svg>
        <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-white device-icon"
          viewBox="0 0 20 20" fill="currentColor" :style="{ stroke: msg.senderColor }">
          <path fill-rule="evenodd"
            d="M3 5a2 2 0 012-2h10a2 2 0 012 2v8a2 2 0 01-2 2h-2.22l.123.489.804.804A1 1 0 0113 18H7a1 1 0 01-.707-1.707l.804-.804L7.22 15H5a2 2 0 01-2-2V5zm6 11a1 1 0 100-2 1 1 0 000 2z"
            clip-rule="evenodd" />
        </svg>
      </div>
    </div>

    <div class="flex flex-col max-w-[75%]" :class="{ 'items-end': msg.senderId === currentUser.id }">
      <span class="text-[10px] text-gray-400 mb-1 px-1">{{ msg.senderName }}</span>
      <div class="px-4 py-2.5 rounded-2xl shadow-sm text-sm break-words w-full transition-colors"
        :class="msg.senderId === currentUser.id ? 'bg-blue-500 text-white rounded-tr-none' : 'bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-100 rounded-tl-none'">

        <div v-if="msg.text" class="flex gap-3 items-end min-w-[60px]">
          <div class="whitespace-pre-wrap leading-relaxed flex-1 break-all min-w-0">
            <template v-for="(part, index) in parseMessage(msg.text)" :key="index">
              <a v-if="part.type === 'link'" :href="part.url" target="_blank" rel="noopener noreferrer"
                class="underline hover:opacity-80"
                :class="msg.senderId === currentUser.id ? 'text-white' : 'text-blue-600 dark:text-blue-400'"
                @click.stop>{{ part.content }}</a>
              <span v-else>{{ part.content }}</span>
            </template>
          </div>
          <button @click="$emit('copy-text', msg.text, msg.id)"
            class="shrink-0 opacity-60 hover:opacity-100 transition-opacity text-current" title="复制">
            <svg v-if="copiedMessageId === msg.id" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none"
              viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
              stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          </button>
        </div>

        <div v-else-if="msg.type === 'file-meta'"
          class="mt-1 p-2 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 flex items-center justify-between gap-3 min-w-[200px] shadow-sm text-gray-800 dark:text-gray-100">
          <div class="flex items-center gap-3 min-w-0 overflow-hidden">
            <div
              class="w-10 h-10 bg-gray-100 dark:bg-gray-700 rounded-lg flex items-center justify-center text-gray-500 dark:text-gray-400 shrink-0">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            </div>
            <div class="flex flex-col min-w-0">
              <span class="text-sm font-medium truncate">{{ msg.fileName }}</span>
              <span class="text-[10px] text-gray-400">{{ formatFileSize(msg.fileSize) }}</span>
            </div>
          </div>

          <button @click.stop="$emit('download-file', msg.fileId, msg.fileName)"
            class="w-9 h-9 rounded-full bg-green-500 hover:bg-green-600 flex items-center justify-center text-white shadow-sm transition shrink-0 active:scale-95">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24"
              stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5"
                d="M19 14l-7 7m0 0l-7-7m7 7V3" />
            </svg>
          </button>
        </div>

      </div>
    </div>
  </div>
</template>

<script setup>
import { parseMessage } from '../utils/textUtils';
import { formatFileSize } from '../utils/fileUtils';

defineProps({
  messages: Array,
  currentUser: Object,
  copiedMessageId: [Number, String, null]
});

defineEmits(['copy-text', 'download-file']);
</script>

<style scoped>
.device-icon {
  stroke-width: 3px;
  paint-order: stroke fill;
}
</style>
