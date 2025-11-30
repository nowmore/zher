export const copyText = async (text, onSuccess, onError) => {
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
        if (onSuccess) onSuccess();
    } else {
        if (onError) onError();
    }
};

export const parseMessage = (text) => {
    if (!text) return [];
    const urlRegex = /(https?:\/\/[^\s]+)/g;
    return text.split(urlRegex).map(part => {
        if (part.match(urlRegex)) {
            return { type: 'link', content: part, url: part };
        }
        return { type: 'text', content: part };
    });
};
