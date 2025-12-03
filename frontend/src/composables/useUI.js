import { ref, computed } from 'vue';

export function useUI() {

    const showMobileUsers = ref(false);
    const rightPanelView = ref('users'); // 'users' | 'settings' | 'qrcode'

    const inputText = ref('');
    const isMultiLine = ref(false);
    const isDragging = ref(false);

    const editNameInput = ref('');
    const copiedMessageId = ref(null);

    const windowWidth = ref(window.innerWidth);

    const placeholderText = computed(() => {
        if (isDragging.value) {
            return '松开鼠标以发送文件...';
        }
        if (windowWidth.value < 768) {
            return '发送消息...';
        }
        return '发送消息//粘贴/拖拽文件或文件夹到此处';
    });

    const inputContainerClass = computed(() => {
        if (isDragging.value) {
            return 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 ring-2 ring-blue-500/20 border-dashed';
        }
        return 'bg-gray-100 dark:bg-gray-700 border-transparent';
    });

    const resetInput = (fileInput) => {
        inputText.value = '';
        if (fileInput?.value) {
            fileInput.value.value = '';
        }
        const textarea = document.querySelector('textarea');
        if (textarea) {
            textarea.style.height = 'auto';
        }
        isMultiLine.value = false;
    };

    const autoResize = (e) => {
        const target = e.target;
        target.style.height = 'auto';
        const newHeight = Math.min(target.scrollHeight, 120);
        target.style.height = newHeight + 'px';
        isMultiLine.value = newHeight > 50;
    };

    const setupWindowResize = () => {
        const handler = () => {
            windowWidth.value = window.innerWidth;
        };
        window.addEventListener('resize', handler);
        return () => window.removeEventListener('resize', handler);
    };

    return {

        showMobileUsers,
        rightPanelView,
        inputText,
        isMultiLine,
        isDragging,
        editNameInput,
        copiedMessageId,
        windowWidth,

        placeholderText,
        inputContainerClass,

        resetInput,
        autoResize,
        setupWindowResize
    };
}
