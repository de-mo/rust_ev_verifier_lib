<script setup>
    import { ref } from "vue";
    const open = ref(false);

    const toggle = () => {
        open.value = !open.value;
    }
</script>

<template>
    <div>
        <div class="lbl-toggle" @click="toggle()">
            <span class="chevron open" v-if="open"></span>
            <span class="chevron" v-else></span>
            <slot class="collapse" name="title"></slot>
        </div>
        <div v-show="open">
            <slot></slot>         
        </div>
    </div>
</template>


<style scoped>
    .lbl-toggle {
        display: block;

        font-family: "SF Pro Text", "SF Pro Icons", "AOS Icons", "Helvetica Neue", Helvetica, Arial, sans-serif;
        font-size: 17px;
        font-weight: 400;
        letter-spacing: -.022em;

        padding: 1rem;

        color: #000000;
        background: #c4c4c4;

        cursor: pointer;

        border-radius: 7px;
        transition: all 0.25s ease-out;
    }

    .lbl-toggle:hover {
        text-decoration: underline;
    }

    .chevron::before {
        content: ' ';
        display: inline-block;

        border-top: 5px solid transparent;
        border-bottom: 5px solid transparent;
        border-left: 5px solid currentColor;

        vertical-align: middle;
        margin-right: .7rem;
        transform: translateY(-2px);

        transition: transform .2s ease-out;
    }
    .chevron.open:before {
        transform: rotate(90deg) translateX(-3px);
    }
</style>