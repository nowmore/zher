<template>
  <div v-if="show"
    class="absolute inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 md:hidden"
    @click.self="$emit('close')">
    <div
      class="bg-white dark:bg-gray-800 rounded-2xl w-full max-w-sm max-h-[90%] flex flex-col shadow-2xl overflow-hidden animate-fade-in">
      <div
        class="p-4 border-b border-gray-100 dark:border-gray-700 flex justify-between items-center bg-gray-50 dark:bg-gray-800">
        <h3 class="font-bold text-gray-800 dark:text-white">在线用户列表</h3>
        <button @click="$emit('close')" class="p-1 rounded-full hover:bg-gray-200 dark:hover:bg-gray-700">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-gray-500 dark:text-gray-400" fill="none"
            viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <UserListView :users="users" :current-user="currentUser" :is-editing-name="isEditingName"
        :edit-name-input="editNameInput" @update:edit-name-input="$emit('update:editNameInput', $event)"
        @start-edit-name="$emit('start-edit-name')" @save-name="$emit('save-name')" />

      <SettingsView :discovery-enabled="discoveryEnabled" :room-code-enabled="roomCodeEnabled" :room-code="roomCode"
        :hide-qrcode-button="true" @toggle-discovery="$emit('toggle-discovery')"
        @toggle-room-code="$emit('toggle-room-code')" @update-room-code="$emit('update-room-code', $event)" />

      <div v-if="serverUrl && qrCodeUrl"
        class="p-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 flex flex-col items-center gap-3 shrink-0">
        <img :src="qrCodeUrl" class="w-32 h-32 rounded-lg shadow-sm bg-white p-1" alt="Server QR Code" />
        <div class="text-center w-full px-2">
          <p class="text-xs text-gray-400 mt-1">{{ displayUrl }}</p>
          <p class="text-xs text-gray-400 mt-1">扫码打开👆</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import UserListView from './UserListView.vue';
import SettingsView from './SettingsView.vue';

defineProps({
  show: Boolean,
  users: Array,
  currentUser: Object,
  isEditingName: Boolean,
  editNameInput: String,
  displayUrl: String,
  serverUrl: String,
  qrCodeUrl: String,
  discoveryEnabled: Boolean,
  roomCodeEnabled: Boolean,
  roomCode: String
});

defineEmits([
  'close',
  'start-edit-name',
  'save-name',
  'update:editNameInput',
  'toggle-discovery',
  'toggle-room-code',
  'update-room-code'
]);
</script>
