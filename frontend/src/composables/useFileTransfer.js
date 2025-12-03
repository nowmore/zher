import { ref } from 'vue';
import JSZip from 'jszip';
import { getZipName, traverseFileTree } from '../utils/fileUtils';

export function useFileTransfer(onFileReady) {
    const selectedFile = ref(null);
    const isZipping = ref(false);
    const zipProgress = ref(0);
    const currentZipName = ref('');
    const currentZipFile = ref('');
    const sharedFiles = new Map();

    const addSharedFile = (fileId, file) => {
        sharedFiles.set(fileId, file);
    };

    const handleStartUpload = async ({ fileId, transferId, offset = 0, end }) => {
        const file = sharedFiles.get(fileId);
        if (file) {
            try {
                let body = file;
                if (offset > 0 || (typeof end === 'number' && end < file.size - 1)) {
                    const sliceEnd = (typeof end === 'number') ? end + 1 : file.size;
                    body = file.slice(offset, sliceEnd);
                }

                await fetch(`/api/upload/${transferId}`, {
                    method: 'POST',
                    body: body, // Browser handles streaming
                });
            } catch (err) {
                console.error("Upload failed", err);
            }
        }
    };

    const processAndSendFiles = async (items, isEntries) => {
        if (items.length === 0) return;

        // Check for single file
        if (items.length === 1) {
            if (isEntries) {
                if (items[0].isFile) {
                    items[0].file(file => {
                        selectedFile.value = file;
                        if (onFileReady) onFileReady();
                    });
                    return;
                }
            } else {
                // If file object
                if (!items[0].webkitRelativePath) {
                    selectedFile.value = items[0];
                    if (onFileReady) onFileReady();
                    return;
                }
            }
        }

        const zip = new JSZip();
        let zipName = getZipName();

        // Naming Logic
        if (isEntries) {
            if (items.length === 1 && items[0].isDirectory) {
                zipName = items[0].name + ".zip";
            }
        } else {
            // Files
            if (items[0].webkitRelativePath) {
                const parts = items[0].webkitRelativePath.split('/');
                if (parts.length > 1) {
                    zipName = parts[parts.length - 2] + ".zip";
                }
            }
        }

        const promises = items.map(item => {
            if (isEntries) {
                if (item.isDirectory) return traverseFileTree(item, zip);
                // File entry
                return new Promise(resolve => item.file(f => { zip.file(item.name, f); resolve(); }));
            } else {
                const path = item.webkitRelativePath || item.name;
                zip.file(path, item);
                return Promise.resolve();
            }
        });

        await Promise.all(promises);

        if (Object.keys(zip.files).length === 0) return;

        isZipping.value = true;
        zipProgress.value = 0;
        currentZipName.value = zipName;

        try {
            const content = await zip.generateAsync({ type: "blob", compression: "STORE" }, (metadata) => {
                zipProgress.value = metadata.percent;
                currentZipFile.value = metadata.currentFile || '';
            });
            selectedFile.value = new File([content], zipName, { type: "application/zip" });
            if (onFileReady) onFileReady();
        } catch (err) {
            console.error("Zip failed", err);
        } finally {
            isZipping.value = false;
        }
    };

    const handleFileChange = async (e) => {
        const files = Array.from(e.target.files);
        await processAndSendFiles(files, false);
        e.target.value = '';
    };

    const handleDrop = async (e) => {
        const items = Array.from(e.dataTransfer.items).map(i => i.webkitGetAsEntry()).filter(i => i);
        await processAndSendFiles(items, true);
    };

    const handlePaste = async (e) => {
        const items = e.clipboardData && e.clipboardData.items ? Array.from(e.clipboardData.items) : [];
        const entries = items.filter(i => i.kind === 'file').map(i => i.webkitGetAsEntry()).filter(i => i);
        if (entries.length > 0) {
            e.preventDefault();
            await processAndSendFiles(entries, true);
        }
    };

    const downloadFile = (fileId, fileName) => {
        const link = document.createElement('a');
        link.href = `/api/download/${fileId}`;
        link.download = fileName;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
    };

    return {
        selectedFile,
        isZipping,
        zipProgress,
        currentZipName,
        currentZipFile,
        handleFileChange,
        handleDrop,
        handlePaste,
        downloadFile,
        handleStartUpload,
        addSharedFile
    };
}
