<script setup>
import { ref, onMounted, onUnmounted, nextTick, computed, watch } from 'vue';
import QRCode from 'qrcode';
import { useSocket } from './composables/useSocket';
import { useChat } from './composables/useChat';
import { useFileTransfer } from './composables/useFileTransfer';
import { formatFileSize } from './utils/fileUtils';
import { parseMessage, copyText as copyToClipboard } from './utils/textUtils';

// UI State
const showMobileUsers = ref(false);
const editNameInput = ref('');
const copiedMessageId = ref(null);
const inputText = ref('');
const fileInput = ref(null);
const chatContainer = ref(null);
const windowWidth = ref(window.innerWidth);
const isMultiLine = ref(false);
const isDragging = ref(false);
const qrCodeUrl = ref('');

const isDarkMode = ref(localStorage.getItem('zher_dark_mode') === 'true');

// Sync theme class
watch(isDarkMode, (val) => {
  if (val) document.documentElement.classList.add('dark');
  else document.documentElement.classList.remove('dark');
}, { immediate: true });

const bgClasses = computed(() => ({
  root: 'bg-gray-100 dark:bg-gray-900',
  panel: 'bg-white dark:bg-gray-800'
}));

const inputContainerClass = computed(() => {
  if (isDragging.value) {
    return 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 ring-2 ring-blue-500/20 border-dashed';
  }
  return 'bg-gray-100 dark:bg-gray-700 border-transparent';
});

const toggleDarkMode = () => {
  isDarkMode.value = !isDarkMode.value;
  localStorage.setItem('zher_dark_mode', isDarkMode.value);
};

// Composables
const {
  users, currentUser, serverUrl, isEditingName,
  connect, disconnect, emit, requestNameChange
} = useSocket();

const { messages, loadMoreMessages, addMessage, loadChatHistory } = useChat();

// Send Message Logic
const resetInput = () => {
  inputText.value = '';
  selectedFile.value = null;
  if (fileInput.value) fileInput.value.value = '';
  const textarea = document.querySelector('textarea');
  if (textarea) textarea.style.height = 'auto';
  isMultiLine.value = false;
};

const realSendMessage = () => {
  if (selectedFile.value) {
    const file = selectedFile.value;
    const fileId = Math.random().toString(36).substr(2, 9);
    addSharedFile(fileId, file);

    emit('file-meta', {
      fileId,
      fileName: file.name,
      fileSize: file.size,
      fileType: file.type
    });
    resetInput();
  } else if (inputText.value.trim()) {
    emit('text-message', inputText.value);
    resetInput();
  }
};

const sendMessage = realSendMessage;

const {
  selectedFile, isZipping, zipProgress, currentZipName, currentZipFile,
  handleFileChange, handleDrop, handlePaste, downloadFile,
  handleStartUpload, addSharedFile
} = useFileTransfer(sendMessage);

const canSend = computed(() => inputText.value.trim() || selectedFile.value);
const onlineCount = computed(() => users.value.length);

const sortedUsers = computed(() => {
  const me = users.value.find(u => u.id === currentUser.value.id);
  const others = users.value.filter(u => u.id !== currentUser.value.id);
  return me ? [me, ...others] : others;
});

// QR Code
const generateQRCode = async () => {
  if (serverUrl.value) {
    try {
      qrCodeUrl.value = await QRCode.toDataURL(serverUrl.value, { margin: 2, width: 200 });
    } catch (err) {
      console.error(err);
    }
  }
};
watch(serverUrl, generateQRCode);

// Lifecycle
onMounted(() => {
  loadChatHistory();

  connect({
    onMessage: (msg) => {
      addMessage(msg);
      nextTick(() => {
        if (chatContainer.value) chatContainer.value.scrollTop = chatContainer.value.scrollHeight;
      });
    },
    onWelcome: (data) => {
      editNameInput.value = data.user.name;
      nextTick(() => {
        if (chatContainer.value) chatContainer.value.scrollTop = chatContainer.value.scrollHeight;
      });
    },
    onStartUpload: handleStartUpload
  });

  window.addEventListener('resize', () => {
    windowWidth.value = window.innerWidth;
  });
});

onUnmounted(() => {
  disconnect();
  window.removeEventListener('resize', () => {
    windowWidth.value = window.innerWidth;
  });
});

// UI Actions
const startEditName = () => {
  editNameInput.value = currentUser.value.name;
  isEditingName.value = true;
  nextTick(() => {
    const inputs = document.querySelectorAll('.name-edit-input');
    if (inputs.length > 0) inputs[0].focus();
  });
};

const saveName = () => {
  const newName = editNameInput.value.trim();
  if (!newName || newName === currentUser.value.name) {
    isEditingName.value = false;
    editNameInput.value = currentUser.value.name;
    return;
  }
  requestNameChange(newName);
};

const triggerFileSelect = () => {
  fileInput.value.click();
};

const onDrop = (e) => {
  isDragging.value = false;
  handleDrop(e);
};

const autoResize = (e) => {
  const target = e.target;
  target.style.height = 'auto';
  const newHeight = Math.min(target.scrollHeight, 120);
  target.style.height = newHeight + 'px';
  isMultiLine.value = newHeight > 50;
};

const placeholderText = computed(() => {
  if (isDragging.value) {
    return '松开鼠标以发送文件...';
  }
  if (windowWidth.value < 768) {
    return '发送消息...';
  }
  return '发送消息//粘贴/拖拽文件或文件夹到此处';
});

const handleScroll = (e) => {
  if (e.target.scrollTop <= 10) {
    loadMoreMessages(e.target);
  }
};

const copyText = (text, msgId) => {
  copyToClipboard(text, () => {
    copiedMessageId.value = msgId;
    setTimeout(() => {
      if (copiedMessageId.value === msgId) {
        copiedMessageId.value = null;
      }
    }, 2000);
  }, () => {
    alert('复制失败，请手动长按文本复制');
  });
};
</script>

<template>
  <div class="flex h-[100dvh] font-sans overflow-hidden relative text-gray-800 dark:text-gray-100"
    :class="bgClasses.root">

    <div v-if="isZipping" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div
        class="bg-white dark:bg-gray-800 rounded-2xl p-6 w-64 shadow-xl flex flex-col items-center gap-4 animate-fade-in">
        <div class="text-center w-full">
          <h3 class="font-bold text-gray-800 dark:text-white mb-1">正在打包...</h3>
          <p class="text-xs text-gray-500 dark:text-gray-400 mb-3 truncate max-w-[200px] mx-auto">{{ currentZipName }}
          </p>
          <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-4 overflow-hidden relative">
            <div class="bg-blue-500 h-4 rounded-full transition-all duration-200 ease-out relative overflow-hidden"
              :style="{ width: zipProgress + '%' }">
              <div
                class="absolute inset-0 bg-white/30 w-full h-full animate-[shimmer_1.5s_infinite] -skew-x-12 origin-left">
              </div>
            </div>
          </div>
          <div class="flex justify-between items-center mt-2 px-1">
            <p class="text-xs text-gray-400 dark:text-gray-500 font-mono truncate max-w-[140px]">{{ currentZipFile }}
            </p>
            <p class="text-sm text-gray-500 dark:text-gray-400 font-mono">{{ zipProgress.toFixed(0) }}%</p>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showMobileUsers"
      class="absolute inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 md:hidden"
      @click.self="showMobileUsers = false">
      <div
        class="bg-white dark:bg-gray-800 rounded-2xl w-full max-w-sm max-h-[90%] flex flex-col shadow-2xl overflow-hidden animate-fade-in">
        <div
          class="p-4 border-b border-gray-100 dark:border-gray-700 flex justify-between items-center bg-gray-50 dark:bg-gray-800">
          <h3 class="font-bold text-gray-800 dark:text-white">在线用户列表</h3>
          <button @click="showMobileUsers = false" class="p-1 rounded-full hover:bg-gray-200 dark:hover:bg-gray-700">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-gray-500 dark:text-gray-400" fill="none"
              viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="flex-1 overflow-y-auto p-4 space-y-3">
          <div v-for="user in sortedUsers" :key="user.id"
            class="flex items-center gap-3 p-2 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition">
            <div class="relative shrink-0">
              <div class="w-10 h-10 rounded-full flex items-center justify-center shadow-sm"
                :style="{ backgroundColor: user.color }">
                <span class="text-white text-sm font-bold leading-none">{{ user.name.substring(0, 1) }}</span>
              </div>
              <div class="absolute -bottom-1 -right-1 z-10">
                <svg v-if="user.device === 'mobile'" xmlns="http://www.w3.org/2000/svg"
                  class="h-4 w-4 text-white device-icon" viewBox="0 0 20 20" fill="currentColor"
                  :style="{ stroke: user.color }">
                  <path fill-rule="evenodd"
                    d="M7 2a2 2 0 00-2 2v12a2 2 0 002 2h6a2 2 0 002-2V4a2 2 0 00-2-2H7zm3 14a1 1 0 100-2 1 1 0 000 2z"
                    clip-rule="evenodd" />
                </svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-white device-icon"
                  viewBox="0 0 20 20" fill="currentColor" :style="{ stroke: user.color }">
                  <path fill-rule="evenodd"
                    d="M3 5a2 2 0 012-2h10a2 2 0 012 2v8a2 2 0 01-2 2h-2.22l.123.489.804.804A1 1 0 0113 18H7a1 1 0 01-.707-1.707l.804-.804L7.22 15H5a2 2 0 01-2-2V5zm6 11a1 1 0 100-2 1 1 0 000 2z"
                    clip-rule="evenodd" />
                </svg>
              </div>
            </div>

            <div class="flex flex-col flex-1 min-w-0">
              <div v-if="user.id === currentUser.id && isEditingName" class="flex items-center">
                <input v-model="editNameInput" @blur="saveName" @keyup.enter="$event.target.blur()" type="text"
                  class="name-edit-input w-full px-2 py-1 text-sm border border-blue-400 rounded focus:outline-none focus:ring-2 focus:ring-blue-200 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                  placeholder="输入新昵称">
              </div>
              <div v-else class="flex items-center justify-between w-full">
                <div class="flex flex-col min-w-0">
                  <span class="font-medium text-gray-700 dark:text-gray-200 truncate">{{ user.name }}</span>
                  <span class="text-xs text-gray-400" v-if="user.id === currentUser.id">我</span>
                </div>
                <button v-if="user.id === currentUser.id" @click="startEditName"
                  class="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 dark:hover:bg-gray-700 rounded-full transition">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path
                      d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                  </svg>
                </button>
              </div>
            </div>
          </div>

          <div v-if="serverUrl" class="pt-4 flex flex-col items-center gap-3 pb-4">
            <img v-if="qrCodeUrl" :src="qrCodeUrl" class="w-32 h-32 rounded-lg shadow-sm p-1 border border-gray-100"
              alt="Server QR Code" />
            <div class="text-center w-full">
              <p class="text-xs text-gray-400 mt-1">扫码打开👆</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="flex-1 flex flex-col min-w-0 h-full">
      <header class="shadow-sm px-4 py-3 flex justify-between items-center z-10 shrink-0 transition-colors"
        :class="bgClasses.panel">
        <h1 class="text-lg font-bold text-gray-800 dark:text-white">这儿 <span
            class="text-sm font-normal text-gray-400 ml-2">zhe'r</span>
        </h1>
        <div class="flex items-center gap-3">
          <button @click="toggleDarkMode"
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
          <button @click="showMobileUsers = true"
            class="md:hidden px-3 py-1.5 bg-gray-100 dark:bg-gray-700 rounded-full text-sm text-blue-600 dark:text-blue-400 font-medium active:bg-gray-200 dark:active:bg-gray-600 transition-colors flex items-center gap-1">
            <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
            在线: {{ onlineCount }}
          </button>
        </div>
      </header>

      <div ref="chatContainer" @scroll="handleScroll" class="flex-1 overflow-y-auto p-4 space-y-4 scroll-smooth">
        <div v-for="msg in messages" :key="msg.id" class="flex items-start gap-2 max-w-full"
          :class="{ 'flex-row-reverse': msg.senderId === currentUser.id }">

          <div class="relative shrink-0">
            <div class="w-8 h-8 rounded-full flex items-center justify-center shadow-sm"
              :style="{ backgroundColor: msg.senderColor }">
              <span class="text-white text-xs font-bold -translate-y-0.5">{{ msg.senderName.substring(0, 1) }}</span>
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
                      @click.stop>{{
                        part.content }}</a>
                    <span v-else>{{ part.content }}</span>
                  </template>
                </div>
                <button @click="copyText(msg.text, msg.id)"
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

                <button @click.stop="downloadFile(msg.fileId, msg.fileName)"
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
      </div>

      <div class="p-3 border-t border-gray-200 dark:border-gray-700 shrink-0 pb-safe transition-colors"
        :class="bgClasses.panel">
        <div class="flex gap-2 items-end">
          <div
            class="relative flex-1 rounded-2xl focus-within:bg-white dark:focus-within:bg-gray-600 focus-within:ring-2 focus-within:ring-blue-500/20 transition-all border focus-within:border-blue-500 dark:focus-within:border-blue-500"
            :class="inputContainerClass" @drop.prevent="onDrop" @dragover.prevent="isDragging = true"
            @dragleave.prevent="isDragging = false">
            <textarea v-model="inputText" rows="1" @input="autoResize" @keydown.enter.prevent.exact="sendMessage"
              @paste="handlePaste" type="text" :placeholder="placeholderText"
              class="w-full pl-4 pr-20 py-3 bg-transparent border-none focus:ring-0 outline-none text-sm text-gray-800 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 resize-none block max-h-[120px] overflow-y-auto"
              style="min-height: 44px;"></textarea>
            <button @click="triggerFileSelect" v-if="!isMultiLine"
              class="absolute right-1 top-1/2 -translate-y-1/2 p-2 text-gray-400 hover:text-blue-600 transition-colors rounded-full hover:bg-gray-100 dark:hover:bg-gray-600">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
              </svg>
            </button>
            <input type="file" ref="fileInput" @change="handleFileChange" class="hidden" multiple>
          </div>

          <button @click="sendMessage" :disabled="!canSend"
            class="p-3 bg-blue-600 text-white rounded-full shadow-lg hover:bg-blue-700 disabled:opacity-50 disabled:shadow-none disabled:cursor-not-allowed transition-all active:scale-95 shrink-0">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
              <path
                d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <div class="hidden md:flex w-80 border-l border-gray-200 dark:border-gray-700 flex-col shrink-0 transition-colors"
      :class="bgClasses.panel">
      <div class="p-6 border-b border-gray-100 dark:border-gray-700">
        <h2 class="text-lg font-bold text-gray-800 dark:text-white">在线用户 ({{ onlineCount }})</h2>
      </div>

      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        <div v-for="user in sortedUsers" :key="user.id"
          class="flex items-center gap-3 p-2 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition">
          <div class="relative shrink-0">
            <div class="w-10 h-10 rounded-full flex items-center justify-center shadow-sm"
              :style="{ backgroundColor: user.color }">
              <span class="text-white text-sm font-bold -translate-y-0.5">{{ user.name.substring(0, 1) }}</span>
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
              <input v-model="editNameInput" @blur="saveName" @keyup.enter="$event.target.blur()" type="text"
                class="name-edit-input w-full px-2 py-1 text-sm border border-blue-400 rounded focus:outline-none focus:ring-2 focus:ring-blue-200 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                placeholder="输入新昵称">
            </div>
            <div v-else class="flex items-center justify-between w-full">
              <div class="flex flex-col min-w-0">
                <span class="font-medium text-gray-700 dark:text-gray-200 truncate">{{ user.name }}</span>
                <span class="text-xs text-gray-400" v-if="user.id === currentUser.id">我</span>
              </div>
              <button v-if="user.id === currentUser.id" @click="startEditName"
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

      <div v-if="serverUrl"
        class="p-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 flex flex-col items-center gap-3 shrink-0">
        <img v-if="qrCodeUrl" :src="qrCodeUrl" class="w-32 h-32 rounded-lg shadow-sm bg-white p-1"
          alt="Server QR Code" />
        <div class="text-center w-full px-2">
          <p class="text-xs text-gray-400 mt-1">扫码打开👆</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pb-safe {
  padding-bottom: env(safe-area-inset-bottom, 12px);
}

.device-icon {
  stroke-width: 3px;
  paint-order: stroke fill;
}
</style>
