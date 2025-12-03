
export const VIEW_TYPES = {
    USERS: 'users',
    SETTINGS: 'settings',
    QRCODE: 'qrcode'
};


export const DEVICE_TYPES = {
    MOBILE: 'mobile',
    DESKTOP: 'desktop'
};


export const MESSAGE_TYPES = {
    TEXT: 'text',
    FILE_META: 'file-meta'
};


export const SOCKET_EVENTS = {

    TEXT_MESSAGE: 'text-message',
    FILE_META: 'file-meta',
    REQUEST_NAME_CHANGE: 'request-name-change',


    WELCOME: 'welcome',
    MESSAGE: 'message',
    USER_JOINED: 'user-joined',
    USER_LEFT: 'user-left',
    UPDATE_USER_LIST: 'update-user-list',
    START_UPLOAD: 'start-upload',
    NAME_CHANGE_SUCCESS: 'name-change-success',
    NAME_CHANGE_FAIL: 'name-change-fail'
};

export const STORAGE_KEYS = {
    USER_ID: 'zher_uid',
    CHAT_HISTORY: 'zher_chat_history',
    DARK_MODE: 'zher_dark_mode'
};

export const UI_CONFIG = {
    MOBILE_BREAKPOINT: 768,
    MAX_TEXTAREA_HEIGHT: 120,
    MIN_TEXTAREA_HEIGHT: 44,
    MULTILINE_THRESHOLD: 50,
    CHAT_PAGE_SIZE: 20,
    SCROLL_LOAD_THRESHOLD: 10,
    COPY_FEEDBACK_DURATION: 2000
};

export const FILE_CONFIG = {
    QR_CODE_WIDTH: 200,
    QR_CODE_MARGIN: 2
};
