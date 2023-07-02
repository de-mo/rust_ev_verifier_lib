<script setup>
    defineProps({
        verification: {type: Object, required:true}
    })

    const capitalizeFirstLetter = (string) => {
        return string.charAt(0).toUpperCase() + string.slice(1);
    }
</script>

<template>
    <!-- eslint-disable vue/no-multiple-template-root -->
    <div class="verif-check">
        <input  type="checkbox" checked/>
    </div>
    <div class="verif-id text">
        {{ verification.id }}
    </div>
    <div class="verif-name text">
        {{ verification.name }}
    </div>
    <div class="verif-categor text">
        {{ capitalizeFirstLetter(verification.category) }}
    </div>
    <div class="verif-status text">
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