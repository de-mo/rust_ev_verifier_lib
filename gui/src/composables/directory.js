import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { createSharedComposable } from "@vueuse/core"
import { CMD_CHOOSE_DIRECTORY } from "../tauri_commands"

function useDirectory() {
    const directory = ref("");
    const directoryError = ref("");
    const isTally = ref(false);
    const directoryChange = ref(false);
    const hasDirectory = computed(() => !hasError.value && !directory.value == "")
    const hasError = computed(() => !directoryError.value == "")

    const chooseDirectory = () => {
        invoke(CMD_CHOOSE_DIRECTORY)
            .then((data) => {
                if (data) {
                    directory.value = data.path,
                    isTally.value = data.is_tally,
                    directoryError.value = ""
                    directoryChange.value = true
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
        chooseDirectory,
        directoryChange
    }
}

export const useSharedDirectory = createSharedComposable(useDirectory)