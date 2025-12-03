<template>
  <div class="border-t border-gray-200 dark:border-gray-700 shrink-0">
    <div class="p-4 space-y-3">
      <h3 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-3">Settings</h3>

      <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
        <div class="flex items-center gap-3">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-gray-600 dark:text-gray-300" fill="none"
            viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
          </svg>
          <span class="text-sm text-gray-800 dark:text-white">Service Discovery</span>
        </div>
        <button @click="$emit('toggle-discovery')"
          class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
          :class="discoveryEnabled ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'">
          <span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
            :class="discoveryEnabled ? 'translate-x-6' : 'translate-x-1'"></span>
        </button>
      </div>

      <div class="space-y-2">
        <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
          <div class="flex items-center gap-3">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-gray-600 dark:text-gray-300" fill="none"
              viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
            <span class="text-sm text-gray-800 dark:text-white">Room Code</span>
          </div>
          <button @click="$emit('toggle-room-code')"
            class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
            :class="roomCodeEnabled ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'">
            <span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
              :class="roomCodeEnabled ? 'translate-x-6' : 'translate-x-1'"></span>
          </button>
        </div>

        <div v-if="roomCodeEnabled" class="pl-3">
          <div class="flex items-center gap-2">
            <div class="relative flex-1">
              <input v-model="localRoomCode" :type="showRoomCode || isEditingRoomCode ? 'text' : 'password'" :readonly="!isEditingRoomCode"
                maxlength="6" @input="e => localRoomCode = e.target.value.replace(/\D/g, '')" @blur="handleBlur"
                class="w-full px-2 py-1 pr-16 text-sm border rounded focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                :class="isEditingRoomCode ? 'border-blue-400' : 'border-gray-300 dark:border-gray-600'"
                placeholder="6位数字" />
              <div class="absolute right-1 top-1/2 -translate-y-1/2 flex gap-1">
                <button @click="showRoomCode = !showRoomCode"
                  class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition">
                  <svg v-if="showRoomCode" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none"
                    viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                  </svg>
                  <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                  </svg>
                </button>
                <button @click="toggleEdit"
                  class="p-1 text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <button v-if="!hideQrcodeButton" @click="$emit('show-qrcode')"
        class="w-full p-3 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition flex items-center justify-between">
        <div class="flex items-center gap-3">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-gray-600 dark:text-gray-300" fill="none"
            viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M12 4v1m6 11h2m-6 0h-2v4m0-11v3m0 0h.01M12 12h4.01M16 20h4M4 12h4m12 0h.01M5 8h2a1 1 0 001-1V5a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1zm12 0h2a1 1 0 001-1V5a1 1 0 00-1-1h-2a1 1 0 00-1 1v2a1 1 0 001 1zM5 20h2a1 1 0 001-1v-2a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1z" />
          </svg>
          <span class="text-sm text-gray-800 dark:text-white">Show QR Code</span>
        </div>
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24"
          stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue';

const props = defineProps({
  discoveryEnabled: Boolean,
  roomCodeEnabled: Boolean,
  roomCode: String,
  hideQrcodeButton: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits([
  'toggle-discovery',
  'toggle-room-code',
  'update-room-code',
  'show-qrcode'
]);

const showRoomCode = ref(false);
const isEditingRoomCode = ref(false);
const localRoomCode = ref(props.roomCode || '');

watch(() => props.roomCode, (newVal) => {
  localRoomCode.value = newVal || '';
});

const toggleEdit = () => {
  isEditingRoomCode.value = !isEditingRoomCode.value;
  if (isEditingRoomCode.value) {
    setTimeout(() => {
      const input = document.querySelector('input[maxlength="6"]');
      if (input) input.focus();
    }, 50);
  }
};

const handleBlur = () => {
  if (isEditingRoomCode.value) {
    emit('update-room-code', localRoomCode.value);
    isEditingRoomCode.value = false;
  }
};
</script>
