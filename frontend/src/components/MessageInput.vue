<template>
  <div class="p-3 border-t border-gray-200 dark:border-gray-700 shrink-0 pb-safe transition-colors" :class="bgClass">
    <div class="flex gap-2 items-end">
      <div
        class="relative flex-1 rounded-2xl focus-within:bg-white dark:focus-within:bg-gray-600 focus-within:ring-2 focus-within:ring-blue-500/20 transition-all border focus-within:border-blue-500 dark:focus-within:border-blue-500"
        :class="containerClass" @drop.prevent="$emit('drop', $event)" @dragover.prevent="$emit('drag-over')"
        @dragleave.prevent="$emit('drag-leave')">

        <textarea :value="modelValue" @input="handleInput" rows="1"
          @keydown.enter.prevent.exact="$emit('send')" @paste="$emit('paste', $event)" type="text"
          :placeholder="placeholder"
          class="w-full pl-4 pr-20 py-3 bg-transparent border-none focus:ring-0 outline-none text-sm text-gray-800 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 resize-none block max-h-[120px] overflow-y-auto"
          style="min-height: 44px;"></textarea>

        <button @click="$emit('file-select')" v-if="!isMultiLine"
          class="absolute right-1 top-1/2 -translate-y-1/2 p-2 text-gray-400 hover:text-blue-600 transition-colors rounded-full hover:bg-gray-100 dark:hover:bg-gray-600">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
          </svg>
        </button>

        <input type="file" ref="fileInputRef" @change="$emit('file-change', $event)" class="hidden" multiple>
      </div>

      <button @click="$emit('send')" :disabled="!canSend"
        class="p-3 bg-blue-600 text-white rounded-full shadow-lg hover:bg-blue-700 disabled:opacity-50 disabled:shadow-none disabled:cursor-not-allowed transition-all active:scale-95 shrink-0">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path
            d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';

const props = defineProps({
  modelValue: String,
  placeholder: String,
  containerClass: String,
  bgClass: String,
  canSend: Boolean,
  isMultiLine: Boolean
});

const emit = defineEmits([
  'update:modelValue',
  'send',
  'file-select',
  'file-change',
  'paste',
  'drop',
  'drag-over',
  'drag-leave',
  'resize'
]);

const fileInputRef = ref(null);

const handleInput = (e) => {
  emit('update:modelValue', e.target.value);
  emit('resize', e);
};

defineExpose({
  fileInputRef
});
</script>

<style scoped>
.pb-safe {
  padding-bottom: env(safe-area-inset-bottom, 12px);
}
</style>
