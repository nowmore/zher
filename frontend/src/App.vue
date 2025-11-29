<script setup>
import { ref, onMounted, onUnmounted, nextTick, computed } from 'vue';
import { io } from 'socket.io-client';
import QRCode from 'qrcode';
import JSZip from 'jszip';

const socket = ref(null);
const currentUser = ref({});
const users = ref([]);
const messages = ref([]);
const inputText = ref('');
const fileInput = ref(null);
const selectedFile = ref(null);
const chatContainer = ref(null);
const showMobileUsers = ref(false);
const isDragging = ref(false);

const isEditingName = ref(false);
const editNameInput = ref('');
const nameInputRef = ref(null);

const copiedMessageId = ref(null);
const qrCodeUrl = ref('');
const serverUrl = ref('');

const onlineCount = computed(() => users.value.length);
const canSend = computed(() => inputText.value.trim() || selectedFile.value);

const sharedFiles = new Map(); // fileId -> File

const getSessionId = () => {
  let id = localStorage.getItem('zher_uid');
  if (!id) {
    id = Math.random().toString(36).substring(2) + Date.now().toString(36);
    localStorage.setItem('zher_uid', id);
  }
  return id;
};

const formatFileSize = (bytes) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

onMounted(() => {
  socket.value = io({
    auth: {
      sessionId: getSessionId()
    },
    transports: ['websocket']
  });

  socket.value.on('welcome', (data) => {
    currentUser.value = data.user;
    users.value = data.allUsers;
    editNameInput.value = data.user.name;
    if (data.serverUrl) {
      serverUrl.value = data.serverUrl;
      generateQRCode();
    }
  });

  socket.value.on('user-joined', (user) => {
    if (!users.value.some(u => u.id === user.id)) {
      users.value.push(user);
    }
  });

  socket.value.on('user-left', (id) => {
    users.value = users.value.filter(u => u.id !== id);
  });

  socket.value.on('update-user-list', (allUsers) => {
    if (Array.isArray(allUsers)) {
      users.value = allUsers;
      const me = allUsers.find(u => u.id === currentUser.value.id);
      if (me) currentUser.value = me;
    } else {
      console.error('Received invalid user list:', allUsers);
    }
  });

  socket.value.on('name-change-success', (newName) => {
    currentUser.value.name = newName;
    isEditingName.value = false;
  });

  socket.value.on('name-change-fail', (msg) => {
    alert(msg);
    nextTick(() => {
      if (nameInputRef.value) nameInputRef.value.focus();
    });
  });

  socket.value.on('message', (msg) => {
    messages.value.push(msg);
    nextTick(() => {
      if (chatContainer.value) {
        chatContainer.value.scrollTop = chatContainer.value.scrollHeight;
      }
    });
  });

  socket.value.on('start-upload', async ({ fileId, transferId }) => {
    const file = sharedFiles.get(fileId);
    if (file) {
      try {
        await fetch(`/api/upload/${transferId}`, {
          method: 'POST',
          body: file, // Browser handles streaming
        });
      } catch (err) {
        console.error("Upload failed", err);
      }
    }
  });
});

onUnmounted(() => {
  if (socket.value) {
    socket.value.disconnect();
  }
});

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
  socket.value.emit('request-name-change', newName);
};

const triggerFileSelect = () => {
  fileInput.value.click();
};

const getZipName = () => {
  const date = new Date();
  const yyyy = date.getFullYear();
  const mm = String(date.getMonth() + 1).padStart(2, '0');
  const dd = String(date.getDate()).padStart(2, '0');
  const HH = String(date.getHours()).padStart(2, '0');
  const MM = String(date.getMinutes()).padStart(2, '0');
  const SS = String(date.getSeconds()).padStart(2, '0');
  return `${yyyy}${mm}${dd}${HH}${MM}${SS}.zip`;
};

const handleFileChange = async (e) => {
  const files = Array.from(e.target.files);
  if (files.length === 0) return;

  if (files.length === 1) {
    selectedFile.value = files[0];
    sendMessage();
  } else {
    const zip = new JSZip();
    files.forEach(f => zip.file(f.name, f));
    const content = await zip.generateAsync({ type: "blob" });
    selectedFile.value = new File([content], getZipName(), { type: "application/zip" });
    sendMessage();
  }
  // Reset input
  e.target.value = '';
};

const traverseFileTree = async (item, zipFolder) => {
  if (item.isFile) {
    const file = await new Promise(resolve => item.file(resolve));
    zipFolder.file(item.name, file);
  } else if (item.isDirectory) {
    const dirReader = item.createReader();
    const entries = await new Promise(resolve => {
      const result = [];
      const read = () => {
        dirReader.readEntries(batch => {
          if (batch.length > 0) {
            result.push(...batch);
            read();
          } else {
            resolve(result);
          }
        });
      };
      read();
    });
    const newZipFolder = zipFolder.folder(item.name);
    for (const entry of entries) {
      await traverseFileTree(entry, newZipFolder);
    }
  }
};

const handleDrop = async (e) => {
  isDragging.value = false;
  const items = Array.from(e.dataTransfer.items);

  if (items.length === 0) return;

  // Check if it's a single file (not directory)
  const firstEntry = items[0].webkitGetAsEntry ? items[0].webkitGetAsEntry() : null;
  if (items.length === 1 && firstEntry && firstEntry.isFile) {
    firstEntry.file(file => {
      selectedFile.value = file;
      sendMessage();
    });
    return;
  }

  // Multiple files or directories
  const zip = new JSZip();
  const promises = items.map(item => {
    const entry = item.webkitGetAsEntry ? item.webkitGetAsEntry() : null;
    if (entry) {
      return traverseFileTree(entry, zip);
    }
    // Fallback for non-entry items (unlikely for files)
    const file = item.getAsFile();
    if (file) {
      zip.file(file.name, file);
    }
    return Promise.resolve();
  });

  await Promise.all(promises);
  const content = await zip.generateAsync({ type: "blob" });
  selectedFile.value = new File([content], getZipName(), { type: "application/zip" });
  sendMessage();
};

const handlePaste = async (e) => {
  if (e.clipboardData && e.clipboardData.files.length > 0) {
    e.preventDefault();
    const files = Array.from(e.clipboardData.files);

    if (files.length === 1) {
      selectedFile.value = files[0];
      sendMessage();
    } else {
      const zip = new JSZip();
      files.forEach(f => zip.file(f.name, f));
      const content = await zip.generateAsync({ type: "blob" });
      selectedFile.value = new File([content], getZipName(), { type: "application/zip" });
      sendMessage();
    }
  }
};

const copyText = async (text, msgId) => {
  let success = false;
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text);
      success = true;
    } else {
      throw new Error('Secure context required');
    }
  } catch (err) {
    const textArea = document.createElement("textarea");
    textArea.value = text;
    textArea.style.position = "fixed";
    textArea.style.left = "-9999px";
    textArea.style.top = "0";
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    try {
      success = document.execCommand('copy');
    } catch (e) { }
    document.body.removeChild(textArea);
  }

  if (success) {
    copiedMessageId.value = msgId;
    setTimeout(() => {
      if (copiedMessageId.value === msgId) {
        copiedMessageId.value = null;
      }
    }, 2000);
  } else {
    alert('复制失败，请手动长按文本复制');
  }
};

const parseMessage = (text) => {
  if (!text) return [];
  const urlRegex = /(https?:\/\/[^\s]+)/g;
  return text.split(urlRegex).map(part => {
    if (part.match(urlRegex)) {
      return { type: 'link', content: part, url: part };
    }
    return { type: 'text', content: part };
  });
};

const sendMessage = () => {
  if (selectedFile.value) {
    const file = selectedFile.value;
    const fileId = Math.random().toString(36).substr(2, 9);
    sharedFiles.set(fileId, file);

    socket.value.emit('file-meta', {
      fileId,
      fileName: file.name,
      fileSize: file.size,
      fileType: file.type
    });
    resetInput();
  } else if (inputText.value.trim()) {
    socket.value.emit('text-message', inputText.value);
    resetInput();
  }
};

const resetInput = () => {
  inputText.value = '';
  selectedFile.value = null;
  if (fileInput.value) fileInput.value.value = '';
};

const downloadFile = (fileId, fileName) => {
  const link = document.createElement('a');
  link.href = `/api/download/${fileId}`;
  link.download = fileName;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
};

const generateQRCode = async () => {
  if (serverUrl.value) {
    try {
      qrCodeUrl.value = await QRCode.toDataURL(serverUrl.value, { margin: 2, width: 200 });
    } catch (err) {
      console.error(err);
    }
  }
};
</script>

<template>
  <div class="flex h-[100dvh] bg-gray-100 font-sans overflow-hidden relative">

    <div v-if="showMobileUsers"
      class="absolute inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 md:hidden"
      @click.self="showMobileUsers = false">
      <div
        class="bg-white rounded-2xl w-full max-w-sm max-h-[90%] flex flex-col shadow-2xl overflow-hidden animate-fade-in">
        <div class="p-4 border-b border-gray-100 flex justify-between items-center bg-gray-50">
          <h3 class="font-bold text-gray-800">在线用户列表</h3>
          <button @click="showMobileUsers = false" class="p-1 rounded-full hover:bg-gray-200">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-gray-500" fill="none" viewBox="0 0 24 24"
              stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="flex-1 overflow-y-auto p-4 space-y-3">
          <div v-for="user in users" :key="user.id"
            class="flex items-center gap-3 p-2 rounded-lg hover:bg-gray-50 transition">
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
                  class="name-edit-input w-full px-2 py-1 text-sm border border-blue-400 rounded focus:outline-none focus:ring-2 focus:ring-blue-200"
                  placeholder="输入新昵称">
              </div>
              <div v-else class="flex items-center justify-between w-full">
                <div class="flex flex-col min-w-0">
                  <span class="font-medium text-gray-700 truncate">{{ user.name }}</span>
                  <span class="text-xs text-gray-400" v-if="user.id === currentUser.id">我</span>
                </div>
                <button v-if="user.id === currentUser.id" @click="startEditName"
                  class="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-full transition">
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
      <header class="bg-white shadow-sm px-4 py-3 flex justify-between items-center z-10 shrink-0">
        <h1 class="text-lg font-bold text-gray-800">这儿 <span class="text-sm font-normal text-gray-400 ml-2">zhe'r</span>
        </h1>
        <button @click="showMobileUsers = true"
          class="md:hidden px-3 py-1.5 bg-gray-100 rounded-full text-sm text-blue-600 font-medium active:bg-gray-200 transition-colors flex items-center gap-1">
          <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
          在线: {{ onlineCount }}
        </button>
      </header>

      <div ref="chatContainer" class="flex-1 overflow-y-auto p-4 space-y-4 scroll-smooth">
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
            <div class="px-4 py-2.5 rounded-2xl shadow-sm text-sm break-words w-full"
              :class="msg.senderId === currentUser.id ? 'bg-blue-500 text-white rounded-tr-none' : 'bg-white text-gray-800 rounded-tl-none'">

              <div v-if="msg.text" class="flex gap-3 items-end min-w-[60px]">
                <div class="whitespace-pre-wrap leading-relaxed flex-1 break-all min-w-0">
                  <template v-for="(part, index) in parseMessage(msg.text)" :key="index">
                    <a v-if="part.type === 'link'" :href="part.url" target="_blank" rel="noopener noreferrer"
                      class="underline hover:opacity-80"
                      :class="msg.senderId === currentUser.id ? 'text-white' : 'text-blue-600'" @click.stop>{{
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
                class="mt-1 p-2 bg-white rounded-xl border border-gray-200 flex items-center justify-between gap-3 min-w-[200px] shadow-sm text-gray-800">
                <div class="flex items-center gap-3 min-w-0 overflow-hidden">
                  <div class="w-10 h-10 bg-gray-100 rounded-lg flex items-center justify-center text-gray-500 shrink-0">
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

      <div class="p-3 bg-white border-t border-gray-200 shrink-0 pb-safe">
        <div class="flex gap-2 items-end">
          <div
            class="relative flex-1 bg-gray-100 rounded-2xl focus-within:bg-white focus-within:ring-2 focus-within:ring-blue-500/20 transition-all border focus-within:border-blue-500"
            :class="isDragging ? 'border-blue-500 bg-blue-50 ring-2 ring-blue-500/20 border-dashed' : 'border-transparent'"
            @drop.prevent="handleDrop" @dragover.prevent="isDragging = true" @dragleave.prevent="isDragging = false">
            <input v-model="inputText" @keyup.enter="sendMessage" @paste="handlePaste" type="text"
              :placeholder="isDragging ? '松开鼠标上传文件' : '发送消息...'"
              class="w-full pl-4 pr-10 py-3 bg-transparent border-none focus:ring-0 outline-none text-sm">
            <button @click="triggerFileSelect"
              class="absolute right-1 top-1/2 -translate-y-1/2 p-2 text-gray-400 hover:text-blue-600 transition-colors rounded-full hover:bg-gray-100">
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
            <!-- <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 translate-x-0.5" viewBox="0 0 20 20" -->
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
              <path
                d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <div class="hidden md:flex w-80 bg-white border-l border-gray-200 flex-col shrink-0">
      <div class="p-6 border-b border-gray-100">
        <h2 class="text-lg font-bold text-gray-800">在线用户 ({{ onlineCount }})</h2>
      </div>

      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        <div v-for="user in users" :key="user.id"
          class="flex items-center gap-3 p-2 rounded-lg hover:bg-gray-50 transition">
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
                class="name-edit-input w-full px-2 py-1 text-sm border border-blue-400 rounded focus:outline-none focus:ring-2 focus:ring-blue-200"
                placeholder="输入新昵称">
            </div>
            <div v-else class="flex items-center justify-between w-full">
              <div class="flex flex-col min-w-0">
                <span class="font-medium text-gray-700 truncate">{{ user.name }}</span>
                <span class="text-xs text-gray-400" v-if="user.id === currentUser.id">我</span>
              </div>
              <button v-if="user.id === currentUser.id" @click="startEditName"
                class="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-full transition">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path
                    d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>

      <div v-if="serverUrl" class="p-4 border-t border-gray-200 bg-gray-50 flex flex-col items-center gap-3 shrink-0">
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
