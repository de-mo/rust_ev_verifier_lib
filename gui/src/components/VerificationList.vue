<script setup>
    import { watch } from "vue"
    import CollapseElement from "./utils/CollapseElement.vue";
    import VerificationItem from "./VerificationItem.vue";
    import { useSharedApplication } from "../composables/application";
    import { useSharedDirectory } from "../composables/directory";
    import { useVerifications } from "../composables/verifications"
    const { isTally, isNotStarted } = useSharedApplication()
    const { directory } = useSharedDirectory()
    const { 
        verifications, 
        getVerifications,
        notImplemented,
        checked_deactivated,
        checked, 
        changeChecked,
        runAll,
        selectAll,
        deselectAll
    } = useVerifications()

    const checkedChanged = (id) => {
        changeChecked(id)
    }
    getVerifications(isTally.value)
    watch(isTally, (newV) => { getVerifications(newV) })
</script>

<template>
    <CollapseElement>
        <template #title>Verifications ({{ isTally ? 'Tally' : 'Setup' }})</template>
        <button class="button-verifier ele_inline" role="button" @click="selectAll" :disabled="!isNotStarted">Select All</button>
        <button class="button-verifier ele_inline" role="button" @click="deselectAll" :disabled="!isNotStarted">Deselect All</button>
        <button class="button-verifier ele_inline" role="button" @click="runAll(directory, isTally)" :disabled="!isNotStarted">Start</button>
        <div style="margin: 0.5em;">
            <div class="verif-grid" v-for="v in verifications" :key="v.id">
                <VerificationItem 
                    :verification="v" 
                    :checked="checked(v.id)" 
                    :deactivated="checked_deactivated(v.id)" 
                    :notImplemented="notImplemented(v.id)"
                    @checked-changed="(id) => checkedChanged(id)">
                </VerificationItem>
            </div>
        </div>
    </CollapseElement>
</template>

<style scoped>
.verif-grid {
    display: grid;
    grid-template-columns: 1fr 2fr 5fr 3fr 2fr;
    grid-row-gap: 1ch;
    padding: 1em;
    border: 1px solid #555555;
    border-radius: 4px;
    box-sizing: border-box;
}
</style>