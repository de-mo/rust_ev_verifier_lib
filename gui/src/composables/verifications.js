import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { useTimer } from "./timer"
import { CMD_GET_VERFICATIONS, CMD_RUN_ALL, CMD_UPDATE_VERIFICATIONS } from "../tauri_commands"
import { V_STATUS_NOT_IMPLEMENTED, V_STATUS_WAITING, V_STATUS_RUNNING, APP_STATUS_RUNNING, APP_STATUS_FINISHED } from "../constants"
import { useSharedApplication } from "./application"

export function useVerifications() {
    const verifications = ref([]);
    const exclusion_ids = ref([]);

    const { setStatus, isRunning: appIsRunning } = useSharedApplication();

    const verification = computed(() => {
        return (id) => verifications.value.find((v) => v.id == id)
    })

    const notImplemented = computed(() => {
        return (id) => verification.value(id).status == V_STATUS_NOT_IMPLEMENTED
    })

    const isExcluded = computed(() => {
        return (id) => exclusion_ids.value.includes(id)
    })

    const checked = computed(() => {
        return (id) => (notImplemented.value(id)) ? false : !isExcluded.value(id)
    })

    const checked_deactivated = computed(() => {
        return (id) => notImplemented.value(id)
    })

    const isRunning = computed(() => 
        verifications.value.find((v) => v.status == V_STATUS_WAITING || v.status == V_STATUS_RUNNING) != undefined
    )

    const selectAll = () => {
        exclusion_ids.value = []
    }

    const deselectAll = () => {
        exclusion_ids.value = verifications.value.filter((v) => !notImplemented.value(v.id)).map((v) => v.id)
    }

    const changeChecked = (id) => {
        if (!notImplemented.value(id)) {
            if (isExcluded.value(id)) {
                exclusion_ids.value.splice(exclusion_ids.value.indexOf(id), 1);
            } else {
                exclusion_ids.value.push(id)
            }
        }
    }

    const getVerifications = (isTally) => {
        console.log("getVerifications")
        let p = isTally ? "tally" : "setup"
        console.log("period", p)
        invoke(CMD_GET_VERFICATIONS, { is_tally: isTally })
            .then((data) => {
                console.log("data", data)
                verifications.value=data
            })
            .catch((error) => {console.error("Error get verifications", error)})
    }

    const {startTimer, stopTimer} = useTimer()

    const updateVerifications = () => {
        if (!appIsRunning.value) {
            console.log("stop Timer")
            stopTimer()
            return
        }
        invoke(CMD_UPDATE_VERIFICATIONS)
            .then((data) => {
                // console.log("data", data)
                verifications.value=data
            })
    }

    const runAll = (dir, isTally) => {
        startTimer(updateVerifications, 1000)
        setStatus(APP_STATUS_RUNNING)
        invoke(CMD_RUN_ALL, {dir: dir, is_tally: isTally, exclusions: exclusion_ids.value})
    }

    watch(verifications, () => {
        if (appIsRunning.value && !isRunning.value) {
            setStatus(APP_STATUS_FINISHED)
        }
    })

    return {
        verifications,
        exclusion_ids,
        notImplemented,
        checked_deactivated,
        checked,
        isRunning,
        getVerifications,
        updateVerifications,
        runAll,
        changeChecked,
        selectAll,
        deselectAll
    }
}