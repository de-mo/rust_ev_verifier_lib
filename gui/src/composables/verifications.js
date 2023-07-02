import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export function useVerifications() {
    const verifications = ref([]);

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
        status,
        getVerifications
    }
}