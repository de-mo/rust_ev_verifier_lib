import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

const notImplementedText = "Not Implemented";

export function useVerifications() {
    const verifications = ref([]);
    const exclusion_ids = ref([]);

    const verification = computed(() => {
        return (id) => verifications.value.find((v) => v.id == id)
    })

    const notImplemented = computed(() => {
        return (id) => verification.value(id).status == notImplementedText
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

    const changeChecked = (id) => {
        console.log("changeChecked", id, notImplemented.value(id));
        if (!notImplemented.value(id)) {
            console.log("should change", isExcluded.value(id))
            if (isExcluded.value(id)) {
                exclusion_ids.value.splice(exclusion_ids.value.indexOf(id), 1);
            } else {
                exclusion_ids.value.push(id)
            }
            console.log("exclusions", exclusion_ids.value)
        } else {
            console.log("no change")
        }
    }

    const getVerifications = (isTally) => {
        console.log("getVerifications")
        //let p = isTally ? "tally" : "setup"
        //console.log("period", p)
        invoke('plugin:verifications|get_verifications', { is_tally: isTally })
            .then((data) => {
                console.log("data", data)
                verifications.value=data
            })
            .catch((error) => {console.log("Error", error)})
    }

    return {
        verifications,
        exclusion_ids,
        notImplemented,
        checked_deactivated,
        checked,
        getVerifications,
        changeChecked
    }
}