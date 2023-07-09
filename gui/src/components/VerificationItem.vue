<script setup>
    import { computed } from "vue"
    import { useSharedApplication } from "../composables/application"
    import { V_STATUS_EXCLUDED } from "../constants"
    const { isNotStarted } = useSharedApplication()

    const props = defineProps({
        verification: {type: Object, required: true},
        checked: {type: Boolean, required: true},
        deactivated: {type: Boolean, required: true},
        notImplemented: { type: Boolean, required: true }
    })
    defineEmits(['checked-changed'])

    const capitalizeFirstLetter = (value) => {
        return value.charAt(0).toUpperCase() + value.slice(1);
    }

    const isLineGrey = computed(() => props.notImplemented || !props.checked)
    const status = computed(() => {
        if (!props.checked) {
            return V_STATUS_EXCLUDED
        } else {
            return props.verification.status
        }
    })
</script>

<template>
    <!-- eslint-disable vue/no-multiple-template-root -->
    <div class="verif-check">
        <input  v-show="!(deactivated || !isNotStarted)" type="checkbox" :checked="checked" @click.stop="$emit('checked-changed', verification.id)"/>
    </div>
    <div class="verif-id text" :class="{'text-grey': isLineGrey }">
        {{ verification.id }}
    </div>
    <div class="verif-name text" :class="{ 'text-grey': isLineGrey }">
        {{ verification.name }}
    </div>
    <div class="verif-categor text" :class="{ 'text-grey': isLineGrey }">
        {{ capitalizeFirstLetter(verification.category) }}
    </div>
    <div class="verif-status text" :class="{ 'text-grey': isLineGrey }">
        {{ status }}
    </div>
</template>

<style scoped>
.verif-check {
    grid-column: 1;
    justify-self: center;
}

.verif-id {
    grid-column: 2;
    justify-self: center;
}
.verif-name {
    grid-column: 3;
}
.verif-category {
    grid-column: 4;
    justify-self: center;
}
.verif-status {
    grid-column: 5;
    justify-self: center;
}
</style>