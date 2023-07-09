<script setup>
  import DirectorySelection from "./DirectorySelection.vue";
  import { useSharedApplication } from "../composables/application"
  const { hasDirectory, isDirTally, period, isNotStarted } = useSharedApplication()
</script>

<template>
  <div>
    <DirectorySelection></DirectorySelection>
    <div v-show="isNotStarted">
      <div v-if="hasDirectory">
        <div v-if="isDirTally">
          <span class="ele_inline text">Select a period:</span>
          <div class="ele_inline">
            <input type="radio" id="setup" name="setup" v-model="period" value="setup">
            <label for="setup" class="text">Setup</label>
          </div>
          <div class="ele_inline">
            <input type="radio" id="tally" name="tally" v-model="period" value="tally">
            <label for="tally" class="text">Tally</label>
          </div>
        </div>
        <span v-else class="text">The chosen directory contains only setup data</span>
      </div>
    </div>
  </div>
</template>
