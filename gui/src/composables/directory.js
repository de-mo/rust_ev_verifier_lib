import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { createSharedComposable } from "@vueuse/core"

function useDirectory() {
    const directory = ref("");
    const directoryError = ref("");
    const isTally = ref(false);
    const hasDirectory = computed(() => !hasError.value && !directory.value == "")
    const hasError = computed(() => !directoryError.value == "")

    const chooseDirectory = () => {
        invoke('plugin:directory|choose_directory')
            .then((data) => {
                if (data) {
                    directory.value = data.path,
                    isTally.value = data.is_tally,
                    directoryError.value = ""
                }
            })
            .catch((error) => {
                directoryError.value = error
            })  
    }

    return {
        directory,
        hasDirectory,
        directoryError,
        isTally,
        hasError,
        chooseDirectory
    }
}

export const useSharedDirectory = createSharedComposable(useDirectory)