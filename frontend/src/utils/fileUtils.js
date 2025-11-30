export const formatFileSize = (bytes) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

export const getZipName = () => {
    const date = new Date();
    const yyyy = date.getFullYear();
    const mm = String(date.getMonth() + 1).padStart(2, '0');
    const dd = String(date.getDate()).padStart(2, '0');
    const HH = String(date.getHours()).padStart(2, '0');
    const MM = String(date.getMinutes()).padStart(2, '0');
    const SS = String(date.getSeconds()).padStart(2, '0');
    return `${yyyy}${mm}${dd}${HH}${MM}${SS}.zip`;
};

export const traverseFileTree = async (item, zipFolder) => {
    if (item.isFile) {
        const file = await new Promise(resolve => item.file(resolve));
        zipFolder.file(item.name, file);
    } else if (item.isDirectory) {
        const dirReader = item.createReader();
        const entries = await new Promise(resolve => {
            const result = [];
            const read = () => {
                dirReader.readEntries(batch => {
                    if (batch.length > 0) {
                        result.push(...batch);
                        read();
                    } else {
                        resolve(result);
                    }
                });
            };
            read();
        });
        const newZipFolder = zipFolder.folder(item.name);
        for (const entry of entries) {
            await traverseFileTree(entry, newZipFolder);
        }
    }
};
