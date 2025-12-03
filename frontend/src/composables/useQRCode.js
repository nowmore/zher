import { ref, watch } from 'vue';
import QRCode from 'qrcode';


export function useQRCode(serverUrl, roomCode, roomCodeEnabled) {
    const qrCodeUrl = ref('');
    const displayUrl = ref('');

    const generateQRCode = async () => {
        if (serverUrl.value) {
            try {
                let url = serverUrl.value;
                if (roomCodeEnabled && roomCodeEnabled.value && roomCode && roomCode.value) {
                    const separator = url.includes('?') ? '&' : '?';
                    url = `${url}${separator}code=${roomCode.value}`;
                }
                displayUrl.value = url
                qrCodeUrl.value = await QRCode.toDataURL(url, {
                    margin: 2,
                    width: 200
                });
            } catch (err) {
                console.error('QR Code generation failed:', err);
            }
        }
    };

    watch(serverUrl, generateQRCode);
    if (roomCode) {
        watch(roomCode, generateQRCode);
    }
    if (roomCodeEnabled) {
        watch(roomCodeEnabled, generateQRCode);
    }

    return {
        qrCodeUrl,
        displayUrl,
        generateQRCode
    };
}
