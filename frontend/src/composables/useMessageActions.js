export function useMessageActions(emit, addMessage, addSharedFile, currentUser, scrollToBottom, chatContainer) {
    const sendTextMessage = (inputText, resetInput) => {
        if (inputText.value.trim() && currentUser.value) {
            const text = inputText.value;
            emit('text-message', text);

            addMessage({
                id: Date.now(),
                text: text,
                senderId: currentUser.value.id,
                senderName: currentUser.value.name || 'Unknown',
                senderColor: currentUser.value.color || '#999',
                senderDevice: currentUser.value.device || 'desktop'
            });
            
            scrollToBottom(chatContainer);
            resetInput();
        }
    };

    const sendFileMessage = (selectedFile, resetInput) => {
        if (selectedFile.value && currentUser.value) {
            const file = selectedFile.value;
            const fileId = Math.random().toString(36).substring(2, 11);
            addSharedFile(fileId, file);

            const fileMetaData = {
                fileId,
                fileName: file.name,
                fileSize: file.size,
                fileType: file.type
            };
            
            emit('file-meta', fileMetaData);
            
            addMessage({
                id: Date.now(),
                type: 'file-meta',
                ...fileMetaData,
                senderId: currentUser.value.id,
                senderName: currentUser.value.name || 'Unknown',
                senderColor: currentUser.value.color || '#999',
                senderDevice: currentUser.value.device || 'desktop'
            });
            
            scrollToBottom(chatContainer);
            resetInput();
        }
    };

    const sendMessage = (inputText, selectedFile, resetInput) => {
        if (selectedFile.value) {
            sendFileMessage(selectedFile, resetInput);
        } else if (inputText.value.trim()) {
            sendTextMessage(inputText, resetInput);
        }
    };

    const handleNewMessage = (msg, currentUserId) => {
        if (msg.senderId === currentUserId) {
            return;
        }
        addMessage(msg);
        scrollToBottom(chatContainer);
    };

    return {
        sendMessage,
        sendTextMessage,
        sendFileMessage,
        handleNewMessage
    };
}
