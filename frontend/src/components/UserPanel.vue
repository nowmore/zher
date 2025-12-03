<template>
  <div class="hidden md:flex w-80 border-l border-gray-200 dark:border-gray-700 flex-col shrink-0 transition-colors"
    :class="bgClass">

    <template v-if="rightPanelView === 'users'">
      <div class="flex-1 flex flex-col min-h-0">
        <div class="p-6 border-b border-gray-100 dark:border-gray-700 shrink-0">
          <h2 class="text-lg font-bold text-gray-800 dark:text-white">在线用户 ({{ users.length }})</h2>
        </div>

        <UserListView :users="users" :current-user="currentUser" :is-editing-name="isEditingName"
          :edit-name-input="editNameInput" @update:edit-name-input="$emit('update:editNameInput', $event)"
          @start-edit-name="$emit('start-edit-name')" @save-name="$emit('save-name')" />
      </div>

      <SettingsView :discovery-enabled="discoveryEnabled" :room-code-enabled="roomCodeEnabled" :room-code="roomCode"
        @toggle-discovery="$emit('toggle-discovery')" @toggle-room-code="$emit('toggle-room-code')"
        @update-room-code="$emit('update-room-code', $event)" @show-qrcode="$emit('show-qrcode')" />
    </template>

    <QRCodeView v-else-if="rightPanelView === 'qrcode'" :qr-code-url="qrCodeUrl" :server-url="serverUrl"
      :display-url="displayUrl" @back="$emit('back-to-users')" />
  </div>
</template>

<script setup>
import UserListView from './UserListView.vue';
import SettingsView from './SettingsView.vue';
import QRCodeView from './QRCodeView.vue';

const props = defineProps({
  users: Array,
  currentUser: Object,
  isEditingName: Boolean,
  editNameInput: String,
  rightPanelView: String,
  qrCodeUrl: String,
  serverUrl: String,
  displayUrl: String,
  bgClass: String,
  discoveryEnabled: Boolean,
  roomCodeEnabled: Boolean,
  roomCode: String
});

defineEmits([
  'update:editNameInput',
  'start-edit-name',
  'save-name',
  'show-qrcode',
  'back-to-users',
  'toggle-discovery',
  'toggle-room-code',
  'update-room-code'
]);
</script>

<style scoped>
.device-icon {
  stroke-width: 3px;
  paint-order: stroke fill;
}
</style>
