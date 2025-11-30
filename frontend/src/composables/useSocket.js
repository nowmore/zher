import { ref } from 'vue';
import { io } from 'socket.io-client';
import { storageService } from '../services/storageService';

export function useSocket() {
    const socket = ref(null);
    const users = ref([]);
    const currentUser = ref({});
    const serverUrl = ref('');
    const isEditingName = ref(false);

    const getSessionId = () => {
        let id = storageService.getItem('zher_uid');
        if (!id) {
            id = Math.random().toString(36).substring(2) + Date.now().toString(36);
            storageService.setItem('zher_uid', id);
        }
        return id;
    };

    const connect = ({ onMessage, onWelcome, onStartUpload }) => {
        socket.value = io({
            auth: {
                sessionId: getSessionId()
            },
            transports: ['websocket']
        });

        socket.value.on('welcome', (data) => {
            currentUser.value = data.user;
            users.value = data.allUsers;
            if (data.serverUrl) serverUrl.value = data.serverUrl;
            if (onWelcome) onWelcome(data);
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
            let list = allUsers;
            if (Array.isArray(allUsers) && Array.isArray(allUsers[0]) && allUsers.length === 1) {
                list = allUsers[0];
            }

            if (Array.isArray(list)) {
                users.value = list;
                const me = list.find(u => u.id === currentUser.value.id);
                if (me) currentUser.value = me;
            }
        });

        socket.value.on('message', (msg) => {
            if (onMessage) onMessage(msg);
        });

        socket.value.on('start-upload', (data) => {
            if (onStartUpload) onStartUpload(data);
        });

        socket.value.on('name-change-success', (newName) => {
            currentUser.value.name = newName;
            isEditingName.value = false;
        });

        socket.value.on('name-change-fail', (msg) => {
            alert(msg);
        });
    };

    const disconnect = () => {
        if (socket.value) socket.value.disconnect();
    };

    const emit = (event, data) => {
        if (socket.value) socket.value.emit(event, data);
    };

    const requestNameChange = (newName) => {
        emit('request-name-change', newName);
    };

    return {
        socket,
        users,
        currentUser,
        serverUrl,
        isEditingName,
        connect,
        disconnect,
        emit,
        requestNameChange
    };
}
