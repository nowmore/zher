import { ref, watch } from 'vue';


export function useTheme() {
    const isDarkMode = ref(localStorage.getItem('zher_dark_mode') === 'true');

    watch(isDarkMode, (val) => {
        if (val) {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
    }, { immediate: true });

    const toggleDarkMode = () => {
        isDarkMode.value = !isDarkMode.value;
        localStorage.setItem('zher_dark_mode', isDarkMode.value);
    };

    return {
        isDarkMode,
        toggleDarkMode
    };
}
