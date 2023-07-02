import { ref, computed, watch } from 'vue'
import { createSharedComposable } from "@vueuse/core"
import { useSharedDirectory } from "./directory"

function useApplication() {
    const { hasDirectory, isTally: isDirTally } = useSharedDirectory()
    const period = ref("setup")
    const isTally = computed(() => period.value == "tally")

    watch(isDirTally, (newB) => {
        if (!newB) {
            period.value = "setup"
        }
    })

    return {
        hasDirectory, 
        isDirTally,
        isTally,
        period,
    }
}

export const useSharedApplication = createSharedComposable(useApplication)