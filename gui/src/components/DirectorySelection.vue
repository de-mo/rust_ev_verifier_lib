<script setup>
  import { useSharedDirectory } from "../composables/directory"
  import { useSharedApplication } from "../composables/application"
  const { directory, hasDirectory, directoryError, hasError, chooseDirectory } = useSharedDirectory()
  const { isRunning } = useSharedApplication()
</script>

<template>
  <div v-if="hasError">
    <span class="ele_inline text" style="color:red">{{ directoryError }}</span>
    <button  class="button-verifier ele_inline" role="button" @click="chooseDirectory" :disabled="isRunning">Change Directory</button>
  </div>
  <div v-else-if="hasDirectory">
      <span class="ele_inline text"><b>Chosen directory: </b>{{ directory }}</span>
      <button class="button-verifier ele_inline" role="button" @click="chooseDirectory" :disabled="isRunning">Change Directory</button>
  </div>
  <button  v-else class="button-verifier" role="button" @click="chooseDirectory">Choose a Directory</button>
</template>

<style scoped>
.input2 {
    display: inline-block;
    font-family: "SF Pro Text", "SF Pro Icons", "AOS Icons", "Helvetica Neue", Helvetica, Arial, sans-serif;
    font-size: 17px;
    font-weight: 400;
    letter-spacing: -.022em;
    line-height: 1.47059;
    width: 100%;
    border: 1px solid #0077CC;
    border-radius: 4px;
    color: #8a8a8a;
}
</style>