<script setup>
    defineProps({
        verification: {type: Object, required: true},
        checked: {type: Boolean, required: true},
        deactivated: {type: Boolean, required: true},
        notImplemented: { type: Boolean, required: true }
    })
    defineEmits(['checked-changed'])

    const capitalizeFirstLetter = (value) => {
        return value.charAt(0).toUpperCase() + value.slice(1);
    }
</script>

<template>
    <!-- eslint-disable vue/no-multiple-template-root -->
    <div class="verif-check">
        <input  type="checkbox" :checked="checked" :disabled="deactivated" @click="$emit('checked-changed', verification.id)"/>
    </div>
    <div class="verif-id text" :class="{'text-grey': notImplemented}">
        {{ verification.id }}
    </div>
    <div class="verif-name text" :class="{ 'text-grey': notImplemented }">
        {{ verification.name }}
    </div>
    <div class="verif-categor text" :class="{ 'text-grey': notImplemented }">
        {{ capitalizeFirstLetter(verification.category) }}
    </div>
    <div class="verif-status text" :class="{ 'text-grey': notImplemented }">
        {{ verification.status }}
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