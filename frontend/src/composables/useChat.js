import { ref, nextTick } from 'vue';
import { storageService } from '../services/storageService';

const PAGE_SIZE = 20;

export function useChat() {
    const messages = ref([]);
    const allMessages = ref([]);

    const saveChatHistory = () => {
        const now = Date.now();
        const tenMinutesAgo = now - 10 * 60 * 1000;
        allMessages.value = allMessages.value.filter(m => m.id > tenMinutesAgo);
        try {
            storageService.setItem('zher_chat_history', JSON.stringify(allMessages.value));
        } catch (e) {
            const twoMinutesMore = tenMinutesAgo + 2 * 60 * 1000;
            allMessages.value = allMessages.value.filter(m => m.id > twoMinutesMore);
            try {
                storageService.setItem('zher_chat_history', JSON.stringify(allMessages.value));
            } catch (e2) {
                console.error("Storage full", e2);
            }
        }
    };

    const loadChatHistory = () => {
        const stored = storageService.getItem('zher_chat_history');
        if (stored) {
            try {
                const parsed = JSON.parse(stored);
                const now = Date.now();
                const tenMinutesAgo = now - 10 * 60 * 1000;
                allMessages.value = parsed.filter(m => m.id > tenMinutesAgo);
                messages.value = allMessages.value.slice(-PAGE_SIZE);
            } catch (e) { }
        }
    };

    const loadMoreMessages = (container) => {
        if (messages.value.length >= allMessages.value.length) return;
        const currentCount = messages.value.length;
        const remaining = allMessages.value.length - currentCount;
        const toLoad = Math.min(remaining, PAGE_SIZE);
        const startIndex = allMessages.value.length - currentCount - toLoad;
        const newBatch = allMessages.value.slice(startIndex, startIndex + toLoad);

        if (container) {
            const oldHeight = container.scrollHeight;
            const oldScrollTop = container.scrollTop;
            messages.value = [...newBatch, ...messages.value];
            nextTick(() => {
                const newHeight = container.scrollHeight;
                container.scrollTop = newHeight - oldHeight + oldScrollTop;
            });
        }
    };

    const addMessage = (msg) => {
        allMessages.value.push(msg);
        messages.value.push(msg);
        saveChatHistory();
    };

    return {
        messages,
        allMessages,
        saveChatHistory,
        loadChatHistory,
        loadMoreMessages,
        addMessage
    };
}
