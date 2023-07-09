import { ref, computed, watch } from 'vue'
import { createSharedComposable } from "@vueuse/core"
import { useSharedDirectory } from "./directory"
import { PERIOD_SETUP, PERIOD_TALLY, APP_STATUS_NOT_STARTED, APP_STATUS_RUNNING, APP_STATUS_FINISHED } from "../constants"

function useApplication() {
    const { directoryChange, hasDirectory, isTally: isDirTally } = useSharedDirectory()
    const period = ref(PERIOD_SETUP)
    const isTally = computed(() => period.value == PERIOD_TALLY)
    const status = ref(APP_STATUS_NOT_STARTED)

    watch(isDirTally, (newB) => {
        console.log("watch isDirTally", newB)
        if (!newB) {
            period.value = PERIOD_SETUP
        }
    })

    watch(directoryChange, (newB) => {
        console.log("watch directoryChange", newB)
        if (newB) {
            setStatus(APP_STATUS_NOT_STARTED)
            directoryChange.value = false
        }
    })

    const setStatus = (val) => {
        status.value = val
    }

    const isNotStarted = computed(() => status.value == APP_STATUS_NOT_STARTED)
    const isRunning = computed(() => status.value == APP_STATUS_RUNNING)
    const isFinished = computed(() => status.value == APP_STATUS_FINISHED)

    return {
        hasDirectory, 
        isDirTally,
        isTally,
        period,
        status,
        setStatus,
        isNotStarted,
        isRunning,
        isFinished
    }
}

export const useSharedApplication = createSharedComposable(useApplication)