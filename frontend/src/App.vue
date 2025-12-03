<script setup>
import { ref, onMounted, onUnmounted, computed, nextTick } from 'vue';
import { useSocket } from './composables/useSocket';
import { useChat } from './composables/useChat';
import { useFileTransfer } from './composables/useFileTransfer';
import { useQRCode } from './composables/useQRCode';
import { useTheme } from './composables/useTheme';
import { useUI } from './composables/useUI';
import { useMessageActions } from './composables/useMessageActions';
import { useUserActions } from './composables/useUserActions';
import { copyText as copyToClipboard } from './utils/textUtils';
import MobileUserModal from './components/MobileUserModal.vue';
import ChatMessages from './components/ChatMessages.vue';
import ZipProgressModal from './components/ZipProgressModal.vue';
import AppHeader from './components/AppHeader.vue';
import MessageInput from './components/MessageInput.vue';
import UserPanel from './components/UserPanel.vue';


const { isDarkMode, toggleDarkMode } = useTheme();

const {
  showMobileUsers, rightPanelView, inputText, isMultiLine, isDragging,
  editNameInput, copiedMessageId, windowWidth, placeholderText,
  inputContainerClass, resetInput, autoResize, setupWindowResize
} = useUI();

const {
  users, currentUser, serverUrl, isEditingName,
  connect, disconnect, emit, requestNameChange
} = useSocket();

const { messages, loadMoreMessages, addMessage, loadChatHistory } = useChat();

// Room code refs need to be defined before useQRCode
const roomCodeEnabled = ref(localStorage.getItem('zher_room_code_enabled') === 'true');
const roomCode = ref(localStorage.getItem('zher_room_code') || '');

const { qrCodeUrl, displayUrl, generateQRCode } = useQRCode(serverUrl, roomCode, roomCodeEnabled);

const messageInputRef = ref(null);
const chatContainer = ref(null);

const scrollToBottom = () => {
  nextTick(() => {
    if (chatContainer.value) {
      chatContainer.value.scrollTop = chatContainer.value.scrollHeight;
    }
  });
};

const sendMessageFn = ref(null);

const discoveryEnabled = ref(localStorage.getItem('zher_discovery_enabled') !== 'false');

// Get room code from URL query parameter
const getUrlRoomCode = () => {
  const params = new URLSearchParams(window.location.search);
  return params.get('code') || '';
};

const urlRoomCode = ref(getUrlRoomCode());

const {
  selectedFile, isZipping, zipProgress, currentZipName, currentZipFile,
  handleFileChange, handleDrop, handlePaste, downloadFile,
  handleStartUpload, addSharedFile
} = useFileTransfer(() => {
  if (sendMessageFn.value) {
    sendMessageFn.value();
  }
});

const { startEditName, saveName } = useUserActions(
  currentUser,
  isEditingName,
  requestNameChange,
  editNameInput
);

const { sendMessage: performSendMessage, handleNewMessage } = useMessageActions(
  emit,
  addMessage,
  addSharedFile,
  currentUser,
  scrollToBottom,
  chatContainer
);

const canSend = computed(() => inputText.value.trim() || selectedFile.value);
const onlineCount = computed(() => users.value.length);

const bgClasses = computed(() => ({
  root: 'bg-gray-100 dark:bg-gray-900',
  panel: 'bg-white dark:bg-gray-800'
}));

const sendMessage = () => {
  performSendMessage(inputText, selectedFile, () => {
    const fileInputRef = messageInputRef.value?.fileInputRef;
    resetInput({ value: fileInputRef?.value });
    selectedFile.value = null;
  });
};

sendMessageFn.value = sendMessage;

const triggerFileSelect = () => {
  messageInputRef.value?.fileInputRef?.click();
};

const onDrop = (e) => {
  isDragging.value = false;
  handleDrop(e);
};

const onDragOver = () => {
  isDragging.value = true;
};

const onDragLeave = () => {
  isDragging.value = false;
};

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

const toggleDiscovery = async () => {
  const newValue = !discoveryEnabled.value;
  try {
    const response = await fetch('/api/discovery', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ enabled: newValue })
    });

    if (response.ok) {
      discoveryEnabled.value = newValue;
      localStorage.setItem('zher_discovery_enabled', newValue);
    } else {
      alert('Failed to toggle service discovery');
    }
  } catch (err) {
    console.error('Toggle discovery error:', err);
    alert('Failed to toggle service discovery');
  }
};

const toggleRoomCode = async () => {
  const newValue = !roomCodeEnabled.value;
  try {
    const response = await fetch('/api/roomcode/toggle', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ enabled: newValue })
    });

    if (response.ok) {
      roomCodeEnabled.value = newValue;
      localStorage.setItem('zher_room_code_enabled', newValue);
    } else {
      alert('Failed to toggle room code');
    }
  } catch (err) {
    console.error('Toggle room code error:', err);
    alert('Failed to toggle room code');
  }
};

const updateRoomCode = async (newCode) => {
  if (newCode.length === 6 && /^\d{6}$/.test(newCode)) {
    try {
      const response = await fetch('/api/roomcode', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code: newCode })
      });

      if (response.ok) {
        roomCode.value = newCode;
        urlRoomCode.value = newCode;
        localStorage.setItem('zher_room_code', newCode);

        // Reconnect WebSocket with new room code
        disconnect();
        setTimeout(() => {
          connect({
            onMessage: (msg) => {
              if (msg.senderId !== currentUser.value.id) {
                addMessage(msg);
                scrollToBottom();
              }
            },
            onWelcome: (data) => {
              editNameInput.value = data.user.name;
            },
            onStartUpload: handleStartUpload,
            roomCode: newCode
          });
        }, 100);
      } else {
        const data = await response.json();
        alert(data.error || 'Failed to update room code');
      }
    } catch (err) {
      console.error('Update room code error:', err);
      alert('Failed to update room code');
    }
  }
};

const loadRoomCodeSettings = async () => {
  try {
    const response = await fetch('/api/roomcode');
    if (response.ok) {
      const data = await response.json();
      roomCodeEnabled.value = data.enabled;
      if (data.code) {
        roomCode.value = data.code;
      }
      // Sync to localStorage
      localStorage.setItem('zher_room_code_enabled', data.enabled);
      if (data.code) {
        localStorage.setItem('zher_room_code', data.code);
      }
    }
  } catch (err) {
    console.error('Failed to load room code settings:', err);
  }
};

onMounted(() => {
  loadChatHistory();
  generateQRCode();
  loadRoomCodeSettings();

  connect({
    onMessage: (msg) => {
      if (msg.senderId !== currentUser.value.id) {
        addMessage(msg);
        scrollToBottom();
      }
    },
    onWelcome: (data) => {
      editNameInput.value = data.user.name;
    },
    onStartUpload: handleStartUpload,
    roomCode: urlRoomCode.value
  });

  const cleanupResize = setupWindowResize();

  onUnmounted(() => {
    disconnect();
    cleanupResize();
  });
});
</script>

<template>
  <div class="flex h-[100dvh] font-sans overflow-hidden relative text-gray-800 dark:text-gray-100"
    :class="bgClasses.root">

    <ZipProgressModal :show="isZipping" :progress="zipProgress" :zip-name="currentZipName"
      :current-file="currentZipFile" />

    <MobileUserModal :show="showMobileUsers" :users="users" :current-user="currentUser" :is-editing-name="isEditingName"
      :edit-name-input="editNameInput" :server-url="serverUrl" :qr-code-url="qrCodeUrl" :display-url="displayUrl"
      :discovery-enabled="discoveryEnabled" :room-code-enabled="roomCodeEnabled" :room-code="roomCode"
      @update:edit-name-input="editNameInput = $event" @close="showMobileUsers = false" @start-edit-name="startEditName"
      @save-name="saveName" @toggle-discovery="toggleDiscovery" @toggle-room-code="toggleRoomCode"
      @update-room-code="updateRoomCode" />

    <div class="flex-1 flex flex-col min-w-0 h-full">
      <AppHeader :is-dark-mode="isDarkMode" :online-count="onlineCount" :bg-class="bgClasses.panel"
        @toggle-theme="toggleDarkMode" @show-users="showMobileUsers = true" />

      <div ref="chatContainer" @scroll="handleScroll" class="flex-1 overflow-y-auto p-4 space-y-4 scroll-smooth">
        <ChatMessages :messages="messages" :current-user="currentUser" :copied-message-id="copiedMessageId"
          @copy-text="copyText" @download-file="downloadFile" />
      </div>

      <MessageInput ref="messageInputRef" v-model="inputText" :placeholder="placeholderText"
        :container-class="inputContainerClass" :bg-class="bgClasses.panel" :can-send="canSend"
        :is-multi-line="isMultiLine" @send="sendMessage" @file-select="triggerFileSelect"
        @file-change="handleFileChange" @paste="handlePaste" @drop="onDrop" @drag-over="onDragOver"
        @drag-leave="onDragLeave" @resize="autoResize" />
    </div>

    <UserPanel :users="users" :current-user="currentUser" :is-editing-name="isEditingName"
      :edit-name-input="editNameInput" :right-panel-view="rightPanelView" :qr-code-url="qrCodeUrl"
      :display-url="displayUrl" :server-url="serverUrl" :bg-class="bgClasses.panel"
      :discovery-enabled="discoveryEnabled" :room-code-enabled="roomCodeEnabled" :room-code="roomCode"
      @update:edit-name-input="editNameInput = $event" @start-edit-name="startEditName" @save-name="saveName"
      @show-qrcode="rightPanelView = 'qrcode'" @back-to-users="rightPanelView = 'users'"
      @toggle-discovery="toggleDiscovery" @toggle-room-code="toggleRoomCode" @update-room-code="updateRoomCode" />
  </div>
</template>
