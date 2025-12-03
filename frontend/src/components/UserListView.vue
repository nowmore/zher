<template>
  <div class="flex-1 overflow-y-auto p-4 space-y-3">
    <div v-for="user in sortedUsers" :key="user.id"
      class="flex items-center gap-3 p-2 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition">
      <div class="relative shrink-0">
        <div class="w-10 h-10 rounded-full flex items-center justify-center shadow-sm"
          :style="{ backgroundColor: user.color }">
          <span class="text-white text-sm font-bold -translate-y-0.5">{{ user.name?.substring(0, 1) || '?' }}</span>
        </div>
        <div class="absolute -bottom-1 -right-1 z-10">
          <svg v-if="user.device === 'mobile'" xmlns="http://www.w3.org/2000/svg"
            class="h-4 w-4 text-white device-icon" viewBox="0 0 20 20" fill="currentColor"
            :style="{ stroke: user.color }">
            <path fill-rule="evenodd"
              d="M7 2a2 2 0 00-2 2v12a2 2 0 002 2h6a2 2 0 002-2V4a2 2 0 00-2-2H7zm3 14a1 1 0 100-2 1 1 0 000 2z"
              clip-rule="evenodd" />
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-white device-icon" viewBox="0 0 20 20"
            fill="currentColor" :style="{ stroke: user.color }">
            <path fill-rule="evenodd"
              d="M3 5a2 2 0 012-2h10a2 2 0 012 2v8a2 2 0 01-2 2h-2.22l.123.489.804.804A1 1 0 0113 18H7a1 1 0 01-.707-1.707l.804-.804L7.22 15H5a2 2 0 01-2-2V5zm6 11a1 1 0 100-2 1 1 0 000 2z"
              clip-rule="evenodd" />
          </svg>
        </div>
      </div>

      <div class="flex flex-col flex-1 min-w-0">
        <div v-if="user.id === currentUser.id && isEditingName" class="flex items-center">
          <input :value="editNameInput" 
            @input="$emit('update:editNameInput', $event.target.value)"
            @blur="$emit('save-name')" 
            @keyup.enter="$event.target.blur()" type="text"
            class="name-edit-input w-full px-2 py-1 text-sm border border-blue-400 rounded focus:outline-none focus:ring-2 focus:ring-blue-200 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
            placeholder="输入新昵称">
        </div>
        <div v-else class="flex items-center justify-between w-full">
          <div class="flex flex-col min-w-0">
            <span class="font-medium text-gray-700 dark:text-gray-200 truncate">{{ user.name }}</span>
            <span class="text-xs text-gray-400" v-if="user.id === currentUser.id">我</span>
          </div>
          <button v-if="user.id === currentUser.id" @click="$emit('start-edit-name')"
            class="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 dark:hover:bg-gray-700 rounded-full transition">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
              <path
                d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  users: Array,
  currentUser: Object,
  isEditingName: Boolean,
  editNameInput: String
});

defineEmits(['start-edit-name', 'save-name', 'update:editNameInput']);

const sortedUsers = computed(() => {
  const me = props.users.find(u => u.id === props.currentUser.id);
  const others = props.users.filter(u => u.id !== props.currentUser.id);
  return me ? [me, ...others] : others;
});
</script>

<style scoped>
.device-icon {
  stroke-width: 3px;
  paint-order: stroke fill;
}
</style>
