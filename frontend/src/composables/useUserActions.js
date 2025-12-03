import { nextTick } from 'vue';

export function useUserActions(currentUser, isEditingName, requestNameChange, editNameInput) {
    const startEditName = () => {
        editNameInput.value = currentUser.value.name;
        isEditingName.value = true;
        nextTick(() => {
            const inputs = document.querySelectorAll('.name-edit-input');
            if (inputs.length > 0) inputs[0].focus();
        });
    };

    const saveName = () => {
        const newName = editNameInput.value.trim();
        if (!newName || newName === currentUser.value.name) {
            isEditingName.value = false;
            editNameInput.value = currentUser.value.name;
            return;
        }
        requestNameChange(newName);
    };

    return {
        startEditName,
        saveName
    };
}
