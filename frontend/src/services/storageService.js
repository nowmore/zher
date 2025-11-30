class StorageService {
    constructor() {
        this.provider = window.localStorage;
    }

    setProvider(provider) {
        this.provider = provider;
    }

    getItem(key) {
        return this.provider.getItem(key);
    }

    setItem(key, value) {
        this.provider.setItem(key, value);
    }

    removeItem(key) {
        this.provider.removeItem(key);
    }
}

export const storageService = new StorageService();
