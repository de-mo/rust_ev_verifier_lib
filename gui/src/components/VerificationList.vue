<script setup>
    import { watch } from "vue"
    import CollapseElement from "./utils/CollapseElement.vue";
    import VerificationItem from "./VerificationItem.vue";
    import { useSharedApplication } from "../composables/application";
    import { useVerifications } from "../composables/verifications"
    const { isTally } = useSharedApplication()
    const { verifications, getVerifications } = useVerifications()
    console.log("verifications", verifications)
    console.log("isTally", isTally)
    getVerifications(isTally.value)
    watch(isTally, (newV) => { getVerifications(newV) })
</script>

<template>
    <CollapseElement>
        <template #title>Verifications ({{ isTally ? 'Tally' : 'Setup' }})</template>
        <div style="margin: 0.5em;">
            <div class="verif-grid" v-for="v in verifications" :key="v.id">
                <VerificationItem :verification="v"></VerificationItem>
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